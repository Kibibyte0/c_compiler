use shared_context::{Identifier, StaticVariable, interner::Interner};

use crate::tacky::{self, Value};

pub struct DebuggingPrinter<'src, 'ctx> {
    interner: &'ctx Interner<'src>,
}

// a printer for the IR, for debugging

impl<'src, 'ctx> DebuggingPrinter<'src, 'ctx> {
    pub fn new(interner: &'ctx Interner<'src>) -> Self {
        Self { interner }
    }

    pub fn print(&self, program: tacky::Program) {
        println!("Program");

        let items = program.into_parts();
        for item in items {
            match item {
                tacky::TopLevel::F(fun_def) => self.print_function(fun_def),
                tacky::TopLevel::S(var_def) => self.print_static_variable(var_def),
            }
        }
    }

    fn format_identifier(&self, identifier: Identifier) -> String {
        let (symbol, id) = identifier.into_parts();
        format!("{}.{}", self.interner.lookup(symbol), id)
    }

    fn format_value(&self, val: tacky::Value) -> String {
        match val {
            tacky::Value::Constant(int) => format!("{}", int),
            tacky::Value::Var(id) => format!("{}", self.format_identifier(id)),
        }
    }

    fn print_static_variable(&self, var_def: StaticVariable) {
        let (name, external, init) = var_def.into_parts();

        let indent = " ".repeat(2);
        println!(
            "{}StaticVariable(name: {}, linkage:{}, init: {})",
            indent,
            self.format_identifier(name),
            Self::format_lickage(external),
            init
        )
    }

    fn format_lickage(external: bool) -> &'static str {
        if external { "external" } else { "internal" }
    }

    fn print_function(&self, function: tacky::FunctionDef) {
        let (name, external, params, instructions) = function.into_parts();

        let indent = " ".repeat(2);
        println!(
            "{}FunctionDef(name: {}, linkage: {})",
            indent,
            self.format_identifier(name),
            Self::format_lickage(external)
        );
        self.print_params(params);

        for instr in instructions {
            self.print_instruction(instr);
        }
    }

    fn print_params(&self, params: Vec<Identifier>) {
        let indent = " ".repeat(2);
        println!("{}Parameters: ", indent);

        print!("{}(", indent);
        for param in params {
            print!("{} ", self.format_identifier(param))
        }
        println!(")");
    }

    fn print_instruction(&self, instr: tacky::Instruction) {
        let indent = " ".repeat(4);
        match instr {
            tacky::Instruction::Ret(val) => {
                println!("{}Ret({})", indent, self.format_value(val));
            }

            tacky::Instruction::Unary { .. } | tacky::Instruction::Binary { .. } => {
                self.print_operator(instr, indent);
            }

            tacky::Instruction::Label(_)
            | tacky::Instruction::Copy { .. }
            | tacky::Instruction::Jump(_)
            | tacky::Instruction::JumpIfZero(_, _)
            | tacky::Instruction::JumpIfNotZero(_, _) => {
                self.print_control_flow(instr, indent);
            }

            tacky::Instruction::FunCall { name, args, dst } => {
                self.print_function_call(name, args, dst, indent);
            }
        }
    }

    fn print_function_call(&self, name: Identifier, args: Vec<Value>, dst: Value, indent: String) {
        print!(
            "{}FunCall(name: {}, args: ",
            indent,
            self.format_identifier(name)
        );
        for arg in args {
            print!("{}", self.format_value(arg));
        }
        println!(", dst: {})", self.format_value(dst));
    }

    fn print_operator(&self, instr: tacky::Instruction, indent: String) {
        match instr {
            tacky::Instruction::Unary { op, src, dst } => {
                println!(
                    "{}Unary({:?} ,src: {}, dst: {})",
                    indent,
                    op,
                    self.format_value(src),
                    self.format_value(dst),
                )
            }

            tacky::Instruction::Binary {
                op,
                src1,
                src2,
                dst,
            } => {
                println!(
                    "{}Binary({:?} ,src1: {}, src2: {}, dst: {})",
                    indent,
                    op,
                    self.format_value(src1),
                    self.format_value(src2),
                    self.format_value(dst)
                );
            }
            _ => unreachable!("Only Binary and Unary expr will reach print_operator"),
        }
    }

    fn print_control_flow(&self, instr: tacky::Instruction, indent: String) {
        match instr {
            tacky::Instruction::Label(label) => {
                println!("{}Label({})", indent, self.format_identifier(label));
            }

            tacky::Instruction::Copy { src, dst } => {
                println!(
                    "{}Copy(src: {}, dst: {})",
                    indent,
                    self.format_value(src),
                    self.format_value(dst)
                );
            }

            tacky::Instruction::Jump(target) => {
                println!("{}Jump(tar: {})", indent, self.format_identifier(target));
            }

            tacky::Instruction::JumpIfZero(condition, target) => {
                println!(
                    "{}JumpIfZero(cond: {}, tar: {})",
                    indent,
                    self.format_value(condition),
                    self.format_identifier(target)
                );
            }

            tacky::Instruction::JumpIfNotZero(condition, target) => {
                println!(
                    "{}JumpIfNotZero(cond: {}, tar: {})",
                    indent,
                    self.format_value(condition),
                    self.format_identifier(target)
                );
            }

            _ => {
                println!("{}Invalid control-flow instruction", indent);
            }
        }
    }
}
