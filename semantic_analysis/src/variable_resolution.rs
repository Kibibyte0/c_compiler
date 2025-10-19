use crate::semantic_error::{ErrorType, SemanticErr};
use crate::{ResolverContext, VariableResolver};
use parser::ast::*;
use shared_context::{CompilerContext, Identifier};

mod resolve_expressions;
mod resolve_statements;

impl<'src, 'c> VariableResolver<'src, 'c> {
    pub fn new(compiler_ctx: &'c CompilerContext<'src>) -> Self {
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
        let mut resolved_block = Vec::new();
        for item in block_items {
            let resolved_item = self.resolve_block_item(item, resolver_ctx)?;
            resolved_block.push(resolved_item);
        }

        resolver_ctx.delete_scope();

        Ok(Block::new(resolved_block, span))
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
        let (identifier, name_span) = name.into_parts();
        let symbol = identifier.get_symbol();

        if let Some(id) = resolver_ctx.search_current_scope(&symbol) {
            return Err(ErrorType::DeclaredTwice {
                first: id.get_span(),
                second: name_span,
            });
        }

        let count = self.get_var_count_and_increment();
        let resolved_identifier = Identifier::new(symbol, count);
        let resolved_name = SpannedIdentifier::new(resolved_identifier, name_span);

        resolver_ctx.insert_variable(symbol, resolved_name);

        if let Some(expr) = init {
            init = Some(self.resolve_expression(expr, resolver_ctx)?);
        }

        Ok(Declaration::new(resolved_name, init, span))
    }
}
