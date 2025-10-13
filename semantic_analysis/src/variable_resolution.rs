use crate::VariableResolver;
use crate::semantic_error::{ErrorType, SemanticErr};
use parser::ast::*;
use std::collections::HashMap;
use std::ops::Range;

mod resolve_expressions;

struct ResolverContext<'a> {
    var_map: &'a mut HashMap<Identifier, Spanned<Identifier>>,
}

impl<'a> VariableResolver<'a> {
    pub fn new(file_name: &'a str, source_code: &'a str) -> Self {
        Self {
            variable_counter: 0,
            file_name,
            source_code,
        }
    }

    pub fn get_var_count(&self) -> usize {
        self.variable_counter
    }

    /// generate a temporary variable
    pub fn make_temp(&mut self, name: &str) -> String {
        let temp = format!("{}.{}", name, self.variable_counter);
        self.variable_counter += 1;
        temp
    }

    pub fn resolve_program(
        &mut self,
        sp_program: Spanned<Program>,
    ) -> Result<Spanned<Program>, SemanticErr> {
        let (program, span) = sp_program.into_parts();
        let sp_function = program.into_parts();
        let new_sp_function = self.resolve_function(sp_function)?;
        Ok(Spanned::new(Program::new(new_sp_function), span))
    }

    fn resolve_function(
        &mut self,
        sp_function: Spanned<FunctionDef>,
    ) -> Result<Spanned<FunctionDef>, SemanticErr> {
        let (function, span) = sp_function.into_parts();
        let (sp_name, body) = function.into_parts();

        let mut var_map = HashMap::new();
        let mut ctx = ResolverContext {
            var_map: &mut var_map,
        };

        let mut new_body = Vec::new();
        for sp_item in body {
            let new_sp_item = match self.resolve_block_item(sp_item, &mut ctx) {
                Ok(new_sp_item) => new_sp_item,
                Err(err) => return Err(SemanticErr::new(err, self.file_name, self.source_code)),
            };
            new_body.push(new_sp_item);
        }

        Ok(Spanned::new(FunctionDef::new(sp_name, new_body), span))
    }

    fn resolve_block_item(
        &mut self,
        sp_item: Spanned<BlockItem>,
        ctx: &mut ResolverContext,
    ) -> Result<Spanned<BlockItem>, ErrorType> {
        let (item, span) = sp_item.into_parts();
        let new_item = match item {
            BlockItem::D(sp_decl) => BlockItem::D(self.resolve_declaration(sp_decl, ctx)?),
            BlockItem::S(sp_stmt) => BlockItem::S(self.resolve_statement(sp_stmt, ctx)?),
        };
        Ok(Spanned::new(new_item, span))
    }

    fn resolve_declaration(
        &mut self,
        sp_decl: Spanned<Declaration>,
        ctx: &mut ResolverContext,
    ) -> Result<Spanned<Declaration>, ErrorType> {
        let (decl, decl_span) = sp_decl.into_parts();
        let (sp_name, mut sp_init) = decl.into_parts();
        let (name, name_span) = sp_name.into_parts();

        if let Some(id) = ctx.var_map.get(&name) {
            return Err(ErrorType::DeclaredTwice {
                first: id.get_span_ref().clone(),
                second: name_span,
            });
        }

        let new_name = Identifier::new(self.make_temp(name.get_name_ref()));

        ctx.var_map
            .insert(name, Spanned::new(new_name.clone(), name_span.clone()));

        if let Some(expr) = sp_init {
            sp_init = Some(self.resolve_expression(expr, ctx)?);
        }

        Ok(Spanned::new(
            Declaration::new(Spanned::new(new_name, name_span), sp_init),
            decl_span,
        ))
    }

    fn resolve_statement(
        &mut self,
        sp_stmt: Spanned<Statement>,
        ctx: &mut ResolverContext,
    ) -> Result<Spanned<Statement>, ErrorType> {
        let (stmt, span) = sp_stmt.into_parts();

        match stmt {
            Statement::Return(exp) => {
                let exp = self.resolve_expression(exp, ctx)?;
                Ok(Spanned::new(Statement::Return(exp), span))
            }
            Statement::ExprStatement(exp) => {
                let exp = self.resolve_expression(exp, ctx)?;
                Ok(Spanned::new(Statement::ExprStatement(exp), span))
            }
            Statement::IfStatement {
                condition,
                if_clause,
                else_clause,
            } => self.resolve_if_statement(condition, *if_clause, else_clause, span, ctx),
            Statement::Null => Ok(Spanned::new(Statement::Null, span)),
        }
    }

    fn resolve_if_statement(
        &mut self,
        condition: Spanned<Expression>,
        if_clause: Spanned<Statement>,
        else_clause: Option<Box<Spanned<Statement>>>,
        span: Range<usize>,
        ctx: &mut ResolverContext,
    ) -> Result<Spanned<Statement>, ErrorType> {
        let condition = self.resolve_expression(condition, ctx)?;
        let if_clause = Box::new(self.resolve_statement(if_clause, ctx)?);

        let else_clause = if let Some(clause) = else_clause {
            Some(Box::new(self.resolve_statement(*clause, ctx)?))
        } else {
            None
        };

        Ok(Spanned::new(
            Statement::IfStatement {
                condition,
                if_clause,
                else_clause,
            },
            span,
        ))
    }
}
