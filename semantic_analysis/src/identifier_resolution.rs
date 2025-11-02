use crate::semantic_error::{ErrorType, SemanticErr};
use crate::{IdentifierResolver, ResolverContext};
use parser::ast::*;
use shared_context::source_map::SourceMap;

mod resolve_declaration;
mod resolve_expressions;
mod resolve_statements;

impl<'src, 'ctx> IdentifierResolver<'src, 'ctx> {
    /// Creates a new `IdentifierResolver` instance.
    ///
    /// # Parameters
    /// - `compiler_ctx`: Shared compiler context containing the source map, interner and symbol table.
    /// - `variable_counter`: The initial counter for generating unique variable names.
    ///
    /// # Purpose
    /// This struct handles the first pass of semantic analysis:
    /// 1. resolving identifiers (variables and functions) and detecting duplicate declarations.
    /// 2. assign all identifiers with no linkage a unqiue identifier
    pub fn new(source_map: &'ctx SourceMap<'src>) -> Self {
        Self {
            source_map,
            variable_counter: 1, // auto-generated variable counter starts at 1
        }
    }

    /// Returns the current value of the auto-generated variable counter.
    pub fn get_var_count(&self) -> usize {
        self.variable_counter
    }

    /// Returns the current auto-generated variable count and increments it.
    ///
    /// Used to assign unique identifiers to compiler-generated temporaries.
    pub fn get_var_count_and_increment(&mut self) -> usize {
        let count = self.variable_counter;
        self.variable_counter += 1;
        count
    }

    /// Resolves all functions in a `Program`, creating a global scope for declarations.
    ///
    /// This is the entry point for identifier resolution.
    pub fn resolve_program(&mut self, program: Program) -> Result<Program, SemanticErr> {
        let declarations = program.into_parts();
        let mut resolver_ctx = ResolverContext::new();
        resolver_ctx.create_scope(); // Create global scope

        let mut resolved_declarations = Vec::new();
        for decl in declarations {
            resolved_declarations.push(
                self.resolve_global_declaration(decl, &mut resolver_ctx)
                    .map_err(|err| SemanticErr::new(err, &self.source_map))?,
            );
        }

        resolver_ctx.delete_scope(); // Clean up global scope
        Ok(Program::new(resolved_declarations))
    }

    /// Resolves a generic block (used in loops, if statements, etc.) by creating a new scope.
    fn resolve_block(
        &mut self,
        block: Block,
        resolver_ctx: &mut ResolverContext,
    ) -> Result<Block, ErrorType> {
        resolver_ctx.create_scope(); // Create a new nested scope

        let (block_items, span) = block.into_parts();
        let mut resolved_block = Vec::new();
        for item in block_items {
            let resolved_item = self.resolve_block_item(item, resolver_ctx)?;
            resolved_block.push(resolved_item);
        }

        resolver_ctx.delete_scope(); // Exit nested scope

        Ok(Block::new(resolved_block, span))
    }

    /// Resolves a single block item (either a declaration or a statement).
    fn resolve_block_item(
        &mut self,
        item: BlockItem,
        resolver_ctx: &mut ResolverContext,
    ) -> Result<BlockItem, ErrorType> {
        let resolved_item = match item {
            BlockItem::D(decl) => BlockItem::D(self.resolve_local_declaration(decl, resolver_ctx)?),
            BlockItem::S(stmt) => BlockItem::S(self.resolve_statement(stmt, resolver_ctx)?),
        };
        Ok(resolved_item)
    }
}
