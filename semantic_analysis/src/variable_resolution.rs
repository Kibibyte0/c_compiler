use crate::semantic_error::{ErrorType, SemanticErr};
use crate::{ResolverContext, VariableResolver};
use parser::ast::*;
use shared_context::{CompilerContext, Identifier};

mod resolve_expressions;

impl<'a, 'c> VariableResolver<'a, 'c> {
    pub fn new(compiler_ctx: &'c CompilerContext<'a>) -> Self {
        Self {
            compiler_ctx,
            variable_counter: 0,
        }
    }

    pub fn get_var_count(&self) -> usize {
        self.variable_counter
    }

    pub fn get_var_count_and_increment(&mut self) -> usize {
        let count = self.variable_counter;
        self.variable_counter += 1;
        count
    }

    pub fn resolve_program(&mut self, program: Program) -> Result<Program, SemanticErr> {
        let function = program.into_parts();
        let mut resolver_ctx = ResolverContext::new();
        let resolved_function = self.resolve_function(function, &mut resolver_ctx)?;
        Ok(Program::new(resolved_function))
    }

    fn resolve_function(
        &mut self,
        function: FunctionDef,
        resolver_ctx: &mut ResolverContext,
    ) -> Result<FunctionDef, SemanticErr> {
        let (name, block, span) = function.into_parts();
        let resolved_body = match self.resolve_block(block, resolver_ctx) {
            Ok(new_block) => new_block,
            Err(err) => {
                return Err(SemanticErr::new(err, &self.compiler_ctx.source_map));
            }
        };
        Ok(FunctionDef::new(name, resolved_body, span))
    }

    fn resolve_block(
        &mut self,
        block: Block,
        resolver_ctx: &mut ResolverContext,
    ) -> Result<Block, ErrorType> {
        resolver_ctx.create_scope();

        let (block_items, span) = block.into_parts();
        let mut resolved_body = Vec::new();
        for item in block_items {
            let resolved_item = self.resolve_block_item(item, resolver_ctx)?;
            resolved_body.push(resolved_item);
        }

        resolver_ctx.delete_scope();

        Ok(Block::new(resolved_body, span))
    }

    fn resolve_block_item(
        &mut self,
        item: BlockItem,
        resolver_ctx: &mut ResolverContext,
    ) -> Result<BlockItem, ErrorType> {
        let resolved_item = match item {
            BlockItem::D(decl) => BlockItem::D(self.resolve_declaration(decl, resolver_ctx)?),
            BlockItem::S(stmt) => BlockItem::S(self.resolve_statement(stmt, resolver_ctx)?),
        };
        Ok(resolved_item)
    }

    fn resolve_declaration(
        &mut self,
        decl: Declaration,
        resolver_ctx: &mut ResolverContext,
    ) -> Result<Declaration, ErrorType> {
        let (name, mut init, span) = decl.into_parts();
        let (symbol, _, name_span) = name.into_parts();

        if let Some(id) = resolver_ctx.search_current_scope(&symbol) {
            return Err(ErrorType::DeclaredTwice {
                first: id.get_span(),
                second: name_span,
            });
        }

        let count = self.get_var_count_and_increment();
        let resolved_name = Identifier::new(symbol, count, name_span);

        resolver_ctx.insert_variable(symbol, resolved_name);

        if let Some(expr) = init {
            init = Some(self.resolve_expression(expr, resolver_ctx)?);
        }

        Ok(Declaration::new(resolved_name, init, span))
    }

    fn resolve_statement(
        &mut self,
        stmt: Statement,
        resolver_ctx: &mut ResolverContext,
    ) -> Result<Statement, ErrorType> {
        let (stmt_type, span) = stmt.into_parts();

        let resolved_stmt_type = match stmt_type {
            StatementType::Return(expr) => {
                let expr = self.resolve_expression(expr, resolver_ctx)?;
                StatementType::Return(expr)
            }
            StatementType::ExprStatement(expr) => {
                let expr = self.resolve_expression(expr, resolver_ctx)?;
                StatementType::ExprStatement(expr)
            }
            StatementType::Compound(sp_block) => {
                StatementType::Compound(self.resolve_block(sp_block, resolver_ctx)?)
            }

            StatementType::IfStatement {
                condition,
                if_clause,
                else_clause,
            } => {
                self.resolve_if_statement_type(condition, *if_clause, else_clause, resolver_ctx)?
            }
            StatementType::Null => StatementType::Null,
        };

        Ok(Statement::new(resolved_stmt_type, span))
    }

    fn resolve_if_statement_type(
        &mut self,
        condition: Expression,
        if_clause: Statement,
        else_clause: Option<Box<Statement>>,
        resolver_ctx: &mut ResolverContext,
    ) -> Result<StatementType, ErrorType> {
        let condition = self.resolve_expression(condition, resolver_ctx)?;
        let if_clause = Box::new(self.resolve_statement(if_clause, resolver_ctx)?);

        let else_clause = if let Some(clause) = else_clause {
            Some(Box::new(self.resolve_statement(*clause, resolver_ctx)?))
        } else {
            None
        };

        Ok(StatementType::IfStatement {
            condition,
            if_clause,
            else_clause,
        })
    }
}
