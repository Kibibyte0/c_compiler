pub mod tacky;
use parser::ast::{self};
use shared_context::{Identifier, Span, interner::Interner};

mod gen_expressions;
mod gen_statements;
mod print_ir;

pub struct IRgen<'a, 'b> {
    var_counter: usize, // counter to generate automatic variables and labels
    interner: &'b mut Interner<'a>,
}

impl<'a, 'b> IRgen<'a, 'b> {
    pub fn new(var_counter: usize, interner: &'b mut Interner<'a>) -> Self {
        Self {
            var_counter,
            interner,
        }
    }

    // generate temporary variables
    fn make_temp_var(&mut self) -> tacky::Value {
        let s = format!("tmp.{}", self.var_counter);
        self.var_counter += 1;
        let symbol = self.interner.intern(&s);
        let temp_id = Identifier::new(symbol, 0, Span::default());
        tacky::Value::Var(temp_id)
    }

    // generate labels
    fn make_label(&mut self) -> Identifier {
        let s = format!("label_{}", self.var_counter);
        self.var_counter += 1;
        let symbol = self.interner.intern(&s);
        Identifier::new(symbol, 0, Span::default())
    }

    pub fn gen_tacky(&mut self, program: ast::Program) -> tacky::Program {
        let function = program.into_parts();
        tacky::Program::new(self.gen_function_def(function))
    }

    fn gen_function_def(&mut self, function: ast::FunctionDef) -> tacky::FunctionDef {
        let (name, block, _) = function.into_parts();
        let mut instructions: Vec<tacky::Instruction> = Vec::new();
        self.gen_function_block(block, &mut instructions);
        tacky::FunctionDef::new(name, instructions)
    }

    /// gen function block will add return 0 by default
    fn gen_function_block(
        &mut self,
        block: ast::Block,
        instructions: &mut Vec<tacky::Instruction>,
    ) {
        let (block_items, _) = block.into_parts();
        for item in block_items {
            self.gen_block_item(item, instructions);
        }

        instructions.push(tacky::Instruction::Ret(tacky::Value::Constant(0)));
    }

    fn gen_block(&mut self, block: ast::Block, instructions: &mut Vec<tacky::Instruction>) {
        let (block_items, _) = block.into_parts();
        for item in block_items {
            self.gen_block_item(item, instructions);
        }
    }

    fn gen_block_item(&mut self, item: ast::BlockItem, instructions: &mut Vec<tacky::Instruction>) {
        match item {
            ast::BlockItem::D(sp_decl) => self.gen_declaration(sp_decl, instructions),
            ast::BlockItem::S(sp_stmt) => self.gen_statements(sp_stmt, instructions),
        }
    }

    fn gen_declaration(
        &mut self,
        decl: ast::Declaration,
        instructions: &mut Vec<tacky::Instruction>,
    ) {
        let (name, init, _) = decl.into_parts();
        match init {
            Some(init) => {
                let value = self.gen_expression(init, instructions);
                let instr = tacky::Instruction::Copy {
                    src: value,
                    dst: tacky::Value::Var(name),
                };
                instructions.push(instr);
            }
            None => return,
        }
    }
}
