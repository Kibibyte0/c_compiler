use parser::ast::*;
use shared_context::{CompilerContext, Identifier};

use crate::{
    LoopLabeling,
    semantic_error::{ErrorType, SemanticErr},
};

impl<'src, 'c> LoopLabeling<'src, 'c> {
    pub fn new(compiler_ctx: &'c mut CompilerContext<'src>, label_counter: usize) -> Self {
        Self {
            compiler_ctx,
            label_counter,
        }
    }

    fn make_label(&mut self) -> Identifier {
        let s = format!("label_{}", self.label_counter);
        self.label_counter += 1;
        let symbol = self.compiler_ctx.interner.intern(&s);
        Identifier::new(symbol, 0)
    }

    pub fn get_label_count(&self) -> usize {
        self.label_counter
    }

    pub fn label_program(&mut self, program: Program) -> Result<Program, SemanticErr> {
        let function = program.into_parts();
        let labeled_function = self.label_function(function)?;
        Ok(Program::new(labeled_function))
    }

    fn label_function(&mut self, function: FunctionDef) -> Result<FunctionDef, SemanticErr> {
        let (name, body, span) = function.into_parts();
        let labeled_body = match self.label_block(body, None) {
            Ok(block) => block,
            Err(err) => return Err(SemanticErr::new(err, &self.compiler_ctx.source_map)),
        };
        Ok(FunctionDef::new(name, labeled_body, span))
    }

    fn label_block(
        &mut self,
        block: Block,
        curr_loop: Option<Identifier>,
    ) -> Result<Block, ErrorType> {
        let (block_items, span) = block.into_parts();
        let mut labeled_block = Vec::new();
        for item in block_items {
            let labeled_item = self.label_block_item(item, curr_loop)?;
            labeled_block.push(labeled_item);
        }
        Ok(Block::new(labeled_block, span))
    }

    fn label_block_item(
        &mut self,
        item: BlockItem,
        curr_loop: Option<Identifier>,
    ) -> Result<BlockItem, ErrorType> {
        if let BlockItem::S(stmt) = item {
            Ok(BlockItem::S(self.label_statement(stmt, curr_loop)?))
        } else {
            Ok(item)
        }
    }

    fn label_statement(
        &mut self,
        stmt: Statement,
        curr_loop: Option<Identifier>,
    ) -> Result<Statement, ErrorType> {
        let (stmt_type, span) = stmt.into_parts();
        let labeled_stmt_type = match stmt_type {
            StatementType::Break(_) => self.label_break_statement(curr_loop, span)?,
            StatementType::Continue(_) => self.label_continue_statement(curr_loop, span)?,
            StatementType::Compound(block) => {
                StatementType::Compound(self.label_block(block, curr_loop)?)
            }
            StatementType::IfStatement {
                condition,
                if_clause,
                else_clause,
            } => self.label_if_statement(condition, *if_clause, else_clause, curr_loop)?,
            StatementType::While {
                condition,
                body,
                label: _,
            } => self.label_while_statement(condition, *body)?,
            StatementType::DoWhile {
                condition,
                body,
                label: _,
            } => self.label_do_while_statement(condition, *body)?,
            StatementType::For {
                init,
                condition,
                post,
                body,
                label: _,
            } => self.label_for_statement(init, condition, post, *body)?,
            _ => stmt_type,
        };
        Ok(Statement::new(labeled_stmt_type, span))
    }

    fn label_if_statement(
        &mut self,
        condition: Expression,
        if_clause: Statement,
        else_clause: Option<Box<Statement>>,
        curr_loop: Option<Identifier>,
    ) -> Result<StatementType, ErrorType> {
        let if_clause = Box::new(self.label_statement(if_clause, curr_loop)?);
        let else_clause = if let Some(stmt) = else_clause {
            Some(Box::new(self.label_statement(*stmt, curr_loop)?))
        } else {
            None
        };
        Ok(StatementType::IfStatement {
            condition,
            if_clause,
            else_clause,
        })
    }

    fn label_while_statement(
        &mut self,
        condition: Expression,
        body: Statement,
    ) -> Result<StatementType, ErrorType> {
        let label = self.make_label();
        let curr_loop = Some(label);
        let body = Box::new(self.label_statement(body, curr_loop)?);
        Ok(StatementType::While {
            condition,
            body,
            label,
        })
    }

    fn label_do_while_statement(
        &mut self,
        condition: Expression,
        body: Statement,
    ) -> Result<StatementType, ErrorType> {
        let label = self.make_label();
        let curr_loop = Some(label);
        let body = Box::new(self.label_statement(body, curr_loop)?);
        Ok(StatementType::DoWhile {
            condition,
            body,
            label,
        })
    }

    fn label_for_statement(
        &mut self,
        init: ForInit,
        condition: Option<Expression>,
        post: Option<Expression>,
        body: Statement,
    ) -> Result<StatementType, ErrorType> {
        let label = self.make_label();
        let curr_loop = Some(label);
        let body = Box::new(self.label_statement(body, curr_loop)?);
        Ok(StatementType::For {
            init,
            condition,
            post,
            body,
            label,
        })
    }

    fn label_break_statement(
        &mut self,
        curr_loop: Option<Identifier>,
        span: Span,
    ) -> Result<StatementType, ErrorType> {
        if let Some(label) = curr_loop {
            Ok(StatementType::Break(label))
        } else {
            Err(ErrorType::BreakErr(span))
        }
    }

    fn label_continue_statement(
        &mut self,
        curr_loop: Option<Identifier>,
        span: Span,
    ) -> Result<StatementType, ErrorType> {
        if let Some(label) = curr_loop {
            Ok(StatementType::Continue(label))
        } else {
            Err(ErrorType::ContinueErr(span))
        }
    }
}
