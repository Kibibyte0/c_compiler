pub mod tacky;
use parser::ast::{self, Spanned};

mod gen_expressions;
mod gen_statements;
mod print_ir;

pub struct IRgen {
    var_counter: usize, // counter to generate automatic variables and labels
}

impl IRgen {
    pub fn new(var_counter: usize) -> Self {
        Self { var_counter }
    }

    // generate temporary variables
    fn make_temp_var(&mut self) -> tacky::Value {
        let s = format!("tmp.{}", self.var_counter);
        self.var_counter += 1;
        tacky::Value::Var(tacky::Identifier(s))
    }

    // generate labels
    fn make_label(&mut self) -> tacky::Identifier {
        let s = format!("label_{}", self.var_counter);
        self.var_counter += 1;
        tacky::Identifier(s)
    }

    pub fn gen_tacky(&mut self, sp_program: Spanned<ast::Program>) -> tacky::Program {
        let sp_function = sp_program.discard_sp().into_parts();
        tacky::Program::new(self.gen_function_def(sp_function))
    }

    fn gen_function_def(&mut self, sp_function: Spanned<ast::FunctionDef>) -> tacky::FunctionDef {
        let (sp_name, body) = sp_function.discard_sp().into_parts();
        let mut instructions: Vec<tacky::Instruction> = Vec::new();
        self.gen_function_body(body, &mut instructions);

        tacky::FunctionDef::new(Self::convert_identifier(sp_name), instructions)
    }

    fn gen_function_body(
        &mut self,
        body: Vec<Spanned<ast::BlockItem>>,
        instructions: &mut Vec<tacky::Instruction>,
    ) {
        for sp_item in body {
            self.gen_block_item(sp_item, instructions);
        }
    }

    fn gen_block_item(
        &mut self,
        sp_item: Spanned<ast::BlockItem>,
        instructions: &mut Vec<tacky::Instruction>,
    ) {
        match sp_item.discard_sp() {
            ast::BlockItem::D(sp_decl) => self.gen_declaration(sp_decl, instructions),
            ast::BlockItem::S(sp_stmt) => self.gen_statements(sp_stmt, instructions),
        }
    }

    fn gen_declaration(
        &mut self,
        sp_decl: Spanned<ast::Declaration>,
        instructions: &mut Vec<tacky::Instruction>,
    ) {
        let decl = sp_decl.discard_sp();
        let (sp_name, sp_init) = decl.into_parts();
        match sp_init {
            Some(sp_init) => {
                let value = self.gen_expression(sp_init, instructions);
                let instr = tacky::Instruction::Copy {
                    src: value,
                    dst: tacky::Value::Var(Self::convert_identifier(sp_name)),
                };
                instructions.push(instr);
            }
            None => return,
        }
    }

    fn convert_identifier(sp_name: Spanned<ast::Identifier>) -> tacky::Identifier {
        tacky::Identifier(sp_name.discard_sp().into_parts())
    }
}
