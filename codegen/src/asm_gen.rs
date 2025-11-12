use crate::{asm, asm::Operand, asm::Operand::Reg, asm::Register};
use ir_gen::tacky;
use shared_context::type_interner::FunctionType;
use shared_context::{Const, OperandSize, SymbolRegistery, Type};
use shared_context::{Identifier, convert_type_to_operand_size, type_interner::TypeInterner};

mod gen_control_flow;
mod gen_operations;

// Responsible for generating assembly from Tacky IR.
pub(crate) struct AsmGen<'ctx, 'src> {
    ty_interner: &'ctx TypeInterner<'src>, // getting the type of each function
    symbol_reg: &'ctx SymbolRegistery,
    args_registers: Vec<asm::Register>, // predefined list of argument registers (ABI-dependent)
}

/// Implementation of the AsmGen struct, responsible for converting
/// Tacky IR into an assembly-level intermediate representation (IR).
impl<'ctx, 'src> AsmGen<'ctx, 'src> {
    /// Creates a new AsmGen instance, initializing the standard set
    /// of registers used for function arguments according to the
    /// System V AMD64 calling convention.
    pub fn new(ty_interner: &'ctx TypeInterner<'src>, symbol_reg: &'ctx SymbolRegistery) -> Self {
        let args_registers = vec![
            Register::DI,
            Register::SI,
            Register::DX,
            Register::CX,
            Register::R8,
            Register::R9,
        ];
        Self {
            ty_interner,
            symbol_reg,
            args_registers,
        }
    }

    /// returns the type of the function
    fn get_function_type<'a>(&self, iden: Identifier) -> &FunctionType<'src> {
        let fun_sy = self.symbol_reg.get_function(&iden);
        self.ty_interner.get(fun_sy.get_type_id())
    }

    // returns the type of the variable
    fn get_variable_type(&self, iden: Identifier) -> Type {
        let var_sy = self.symbol_reg.get_variable(&iden);
        var_sy.get_type()
    }

    fn get_val_type(&self, val: tacky::Value) -> Type {
        match val {
            tacky::Value::Constant(cons) => match cons {
                Const::ConstInt(_) => Type::Int,
                Const::ConstUint(_) => Type::Uint,
                Const::ConstLong(_) => Type::Long,
                Const::ConstUlong(_) => Type::Ulong,
            },
            tacky::Value::Var(iden) => self.get_variable_type(iden),
        }
    }

    fn get_val_size(&self, val: tacky::Value) -> OperandSize {
        match val {
            tacky::Value::Constant(cons) => match cons {
                Const::ConstInt(_) | Const::ConstUint(_) => OperandSize::LongWord,
                Const::ConstLong(_) | Const::ConstUlong(_) => OperandSize::QuadWord,
            },
            tacky::Value::Var(iden) => {
                let var_type = self.get_variable_type(iden);
                convert_type_to_operand_size(var_type)
            }
        }
    }

    /// Converts a Tacky value into an assembly operand.
    // the type of the immediate dosen't matter, as it will have the same bit representation
    fn convert_val(val: tacky::Value) -> Operand {
        match val {
            tacky::Value::Var(identifier) => Operand::Pseudo(identifier),
            tacky::Value::Constant(cons) => match cons {
                Const::ConstInt(int) => Operand::Immediate(int as u64),
                Const::ConstUint(uint) => Operand::Immediate(uint as u64),
                Const::ConstUlong(ulong) => Operand::Immediate(ulong),
                Const::ConstLong(long) => Operand::Immediate(long as u64),
            },
        }
    }

    /// Entry point for assembly generation.
    ///
    /// Consumes a tacky::Program and returns an asm::Program.
    /// Each function in the Tacky IR is lowered to a corresponding
    /// assembly function using `gen_function_def`.
    /// static variables remain unchanged
    pub fn gen_asm(&self, program: tacky::Program) -> asm::Program {
        let items = program.into_parts();
        let mut asm_items = Vec::new();

        for item in items {
            match item {
                tacky::TopLevel::F(fun_def) => {
                    asm_items.push(asm::TopLevel::F(self.gen_function_def(fun_def)))
                }
                tacky::TopLevel::S(var_def) => asm_items.push(asm::TopLevel::S(var_def)),
            }
        }

        asm::Program::new(asm_items)
    }

    /// Converts a single Tacky function definition into an assembly-level one.
    fn gen_function_def(&self, function: tacky::FunctionDef) -> asm::FunctionDef {
        let (name, external, params, tacky_instructions) = function.into_parts();
        let mut asm_instructions = Vec::new();

        // Placeholder stack allocation — actual size determined during
        // register allocation pass (where spills are known).
        asm_instructions.push(asm::Instruction::Binary {
            op: asm::BinaryOP::Sub,
            size: OperandSize::QuadWord,
            src: asm::Operand::Immediate(0),
            dst: Reg(Register::SP),
        });

        // Move function parameters into pseudo-registers (stack locals).
        self.push_params_into_stack(name, params, &mut asm_instructions);

        // Translate each Tacky instruction into assembly.
        self.gen_instructions(tacky_instructions, &mut asm_instructions);

        asm::FunctionDef::new(name, external, asm_instructions)
    }

    /// Moves function parameters from argument registers or stack into pseudo-registers.
    ///
    /// - The first 6 parameters go into registers (`DI`, `SI`, etc.).
    /// - Remaining ones are read from the stack, starting at offset 16.
    fn push_params_into_stack(
        &self,
        iden: Identifier,
        params: Vec<Identifier>,
        asm_instructions: &mut Vec<asm::Instruction>,
    ) {
        // it is quaranteed that the function is in the symbol table at this stage
        let fun_type = self.get_function_type(iden);
        for ((i, param), param_type) in params.iter().enumerate().zip(fun_type.params) {
            if i <= 5 {
                asm_instructions.push(asm::Instruction::Mov {
                    size: convert_type_to_operand_size(*param_type),
                    src: Reg(self.args_registers[i]),
                    dst: Operand::Pseudo(*param),
                });
            } else {
                // Stack parameters start after return address and saved base pointer.
                let stack_index = 16 + ((i as i64) - 6) * 8;
                asm_instructions.push(asm::Instruction::Mov {
                    size: convert_type_to_operand_size(*param_type),
                    src: Operand::Stack(stack_index),
                    dst: Operand::Pseudo(*param),
                });
            }
        }
    }

    /// Translates a list of Tacky instructions into assembly instructions.
    fn gen_instructions(
        &self,
        tacky_instructions: Vec<tacky::Instruction>,
        asm_instructions: &mut Vec<asm::Instruction>,
    ) {
        for tacky_instruction in tacky_instructions {
            match tacky_instruction {
                tacky::Instruction::Ret(val) => self.handle_ret(val, asm_instructions),

                tacky::Instruction::Unary { op, src, dst } => {
                    self.handle_unary(op, src, dst, asm_instructions)
                }

                tacky::Instruction::Binary {
                    op,
                    src1,
                    src2,
                    dst,
                } => self.handle_binary(op, src1, src2, dst, asm_instructions),

                tacky::Instruction::Jump(tar) => Self::handle_jump(tar, asm_instructions),

                tacky::Instruction::JumpIfZero(pred, tar) => {
                    self.handle_jump_if_zero(pred, tar, asm_instructions)
                }

                tacky::Instruction::JumpIfNotZero(pred, tar) => {
                    self.handle_jump_if_not_zero(pred, tar, asm_instructions)
                }

                tacky::Instruction::Label(tar) => Self::handle_label(tar, asm_instructions),

                tacky::Instruction::Copy { src, dst } => {
                    self.handle_copy(src, dst, asm_instructions)
                }

                tacky::Instruction::FunCall { name, args, dst } => {
                    self.handle_function_call(name, args, dst, asm_instructions);
                }
                tacky::Instruction::SignExtend { src, dst } => {
                    self.handle_sign_extention(src, dst, asm_instructions)
                }
                tacky::Instruction::Truncate { src, dst } => {
                    self.handle_truncate(src, dst, asm_instructions);
                }
                tacky::Instruction::ZeroExtend { src, dst } => {
                    self.handle_zero_extention(src, dst, asm_instructions)
                }
            }
        }
    }

    /// Handle Zero extension instruction
    fn handle_zero_extention(
        &self,
        src: tacky::Value,
        dst: tacky::Value,
        asm_instructions: &mut Vec<asm::Instruction>,
    ) {
        asm_instructions.push(asm::Instruction::Movzx {
            src: Self::convert_val(src),
            dst: Self::convert_val(dst),
        });
    }

    /// Hanlde truncate instruction with a mov instruction of operand size 32 bits
    /// this will truncate the quadword by copying only its lower half
    fn handle_truncate(
        &self,
        src: tacky::Value,
        dst: tacky::Value,
        asm_instructions: &mut Vec<asm::Instruction>,
    ) {
        asm_instructions.push(asm::Instruction::Mov {
            size: OperandSize::LongWord,
            src: Self::convert_val(src),
            dst: Self::convert_val(dst),
        });
    }

    /// Handle Sign extension instruction
    fn handle_sign_extention(
        &self,
        src: tacky::Value,
        dst: tacky::Value,
        asm_instructions: &mut Vec<asm::Instruction>,
    ) {
        asm_instructions.push(asm::Instruction::Movsx {
            src: Self::convert_val(src),
            dst: Self::convert_val(dst),
        });
    }

    /// Handles return statements by moving the result into RAX and emitting `ret`.
    fn handle_ret(&self, val: tacky::Value, asm_instructions: &mut Vec<asm::Instruction>) {
        asm_instructions.push(asm::Instruction::Mov {
            size: self.get_val_size(val),
            dst: Reg(Register::AX),
            src: Self::convert_val(val),
        });
        asm_instructions.push(asm::Instruction::Ret);
    }

    /// Handles copy (assignment) instructions by emitting a simple `mov`.
    fn handle_copy(
        &self,
        src: tacky::Value,
        dst: tacky::Value,
        asm_instructions: &mut Vec<asm::Instruction>,
    ) {
        asm_instructions.push(asm::Instruction::Mov {
            size: self.get_val_size(src),
            src: Self::convert_val(src),
            dst: Self::convert_val(dst),
        });
    }

    /// Handles function calls according to the System V AMD64 calling convention.
    ///
    /// 1. Pass up to 6 arguments via registers.
    /// 2. Push additional arguments on the stack in reverse order.
    /// 3. Maintain stack alignment (16-byte).
    /// 4. Move the return value (in RAX) into the destination pseudo-register.
    fn handle_function_call(
        &self,
        name: Identifier,
        args: Vec<tacky::Value>,
        tacky_dst: tacky::Value,
        asm_instructions: &mut Vec<asm::Instruction>,
    ) {
        // Split args into those passed in registers and those passed on the stack.
        let mid = if 6 > args.len() { args.len() } else { 6 };
        let (register_args, stack_args) = args.split_at(mid);

        // Stack must remain 16-byte aligned before a `call`.
        let stack_padding = Self::calculate_stack_padding(stack_args.len());
        if stack_padding != 0 {
            asm_instructions.push(asm::Instruction::Binary {
                op: asm::BinaryOP::Sub,
                size: OperandSize::QuadWord,
                src: asm::Operand::Immediate(stack_padding as u64),
                dst: Reg(Register::SP),
            });
        }

        // Move arguments into the appropriate registers.
        self.move_register_args(register_args, asm_instructions);

        // Push remaining arguments to stack (right-to-left).
        self.push_stack_args(stack_args, asm_instructions);

        // Emit the call instruction.
        asm_instructions.push(asm::Instruction::Call(name));

        // Clean up the stack (if we pushed arguments).
        self.cleanup_stack(stack_args.len(), stack_padding, asm_instructions);

        // Move the return value from RAX to the destination pseudo-register.
        self.move_return_value(tacky_dst, asm_instructions);
    }

    /// Calculates stack padding to maintain 16-byte alignment.
    fn calculate_stack_padding(stack_args_len: usize) -> usize {
        if stack_args_len % 2 == 0 { 0 } else { 8 }
    }

    /// Moves up to 6 function arguments into argument registers.
    fn move_register_args(
        &self,
        register_args: &[tacky::Value],
        asm_instructions: &mut Vec<asm::Instruction>,
    ) {
        for (i, tacky_arg) in register_args.iter().enumerate() {
            let asm_arg = Self::convert_val(*tacky_arg);
            let register = Reg(self.args_registers[i]);
            asm_instructions.push(asm::Instruction::Mov {
                size: self.get_val_size(*tacky_arg),
                src: asm_arg,
                dst: register,
            });
        }
    }

    /// Pushes extra (beyond six) function arguments onto the stack in reverse order.
    /// pad the with empty space if necessary to keep alignment
    fn push_stack_args(
        &self,
        stack_args: &[tacky::Value],
        asm_instructions: &mut Vec<asm::Instruction>,
    ) {
        for tacky_arg in stack_args.iter().rev() {
            let asm_arg = Self::convert_val(*tacky_arg);

            match asm_arg {
                // Immediate or register values can be pushed directly.
                asm::Operand::Immediate(_) | asm::Operand::Reg(_) => {
                    asm_instructions.push(asm::Instruction::Push(asm_arg));
                }
                // Otherwise, move into RAX first, then push (x86 requires a register source).
                _ => {
                    asm_instructions.push(asm::Instruction::Mov {
                        size: self.get_val_size(*tacky_arg),
                        src: asm_arg,
                        dst: Reg(Register::AX),
                    });
                    asm_instructions.push(asm::Instruction::Push(Reg(Register::AX)));
                }
            }
        }
    }

    /// Frees the stack space used for arguments after a function call.
    fn cleanup_stack(
        &self,
        stack_args_len: usize,
        stack_padding: usize,
        asm_instructions: &mut Vec<asm::Instruction>,
    ) {
        let bytes_to_remove = 8 * stack_args_len + stack_padding;
        if bytes_to_remove != 0 {
            asm_instructions.push(asm::Instruction::Binary {
                op: asm::BinaryOP::Add,
                size: OperandSize::QuadWord,
                src: asm::Operand::Immediate(bytes_to_remove as u64),
                dst: Reg(Register::SP),
            });
        }
    }

    /// Moves a function's return value from RAX into the destination pseudo-register.
    fn move_return_value(
        &self,
        tacky_dst: tacky::Value,
        asm_instructions: &mut Vec<asm::Instruction>,
    ) {
        let asm_dst = Self::convert_val(tacky_dst);
        asm_instructions.push(asm::Instruction::Mov {
            size: self.get_val_size(tacky_dst),
            src: Reg(Register::AX),
            dst: asm_dst,
        });
    }
}
