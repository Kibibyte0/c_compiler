use crate::IdentifierResolver;
use crate::semantic_error::{ErrorType, SemanticErr};
use parser::ast::*;
use shared_context::{SpannedIdentifier, source_map::SourceMap, symbol_interner::Symbol};
use std::collections::{HashMap, VecDeque};

mod resolve_declaration;
mod resolve_expressions;
mod resolve_statements;

/// Represents an entry in the identifier resolver.
/// Keeps track of the identifier's source info and whether it has linkage (global/static).
#[derive(Clone, Copy)]
struct ResolverEntry {
    sp_identifier: SpannedIdentifier,
    linkage: bool,
}

impl ResolverEntry {
    /// Create a new resolver entry with the given identifier and linkage flag
    pub fn new(sp_identifier: SpannedIdentifier, linkage: bool) -> Self {
        Self {
            sp_identifier,
            linkage,
        }
    }

    /// Returns true if the identifier has linkage (i.e., visible outside current scope)
    pub fn has_linkage(&self) -> bool {
        self.linkage
    }

    /// Returns the original spanned identifier
    pub fn get_sp_identifier(&self) -> SpannedIdentifier {
        self.sp_identifier
    }
}

/// Maintains a stack of scopes for identifier resolution.
/// Each scope maps a symbol to its corresponding resolver entry.
struct ResolverContext {
    scopes: VecDeque<HashMap<Symbol, ResolverEntry>>,
}

impl ResolverContext {
    /// Create a new, empty resolver context
    fn new() -> Self {
        Self {
            scopes: VecDeque::new(),
        }
    }

    /// Push a new scope on top of the stack
    fn create_scope(&mut self) {
        self.scopes.push_front(HashMap::new());
    }

    /// Remove the topmost scope from the stack
    fn delete_scope(&mut self) {
        self.scopes.pop_front();
    }

    /// Insert a new symbol-to-entry mapping in the current scope
    fn insert_entry(&mut self, key: Symbol, value: ResolverEntry) {
        self.scopes
            .front_mut()
            .expect("resolver context is empty")
            .insert(key, value);
    }

    /// Search all scopes (starting from innermost) for a symbol
    fn search_scope(&self, key: &Symbol) -> Option<ResolverEntry> {
        for scope in &self.scopes {
            if let Some(name) = scope.get(key) {
                return Some(*name);
            }
        }
        None
    }

    /// Search only the current (innermost) scope for a symbol
    fn search_current_scope(&self, key: &Symbol) -> Option<ResolverEntry> {
        self.scopes
            .front()
            .expect("resolver context is empty")
            .get(key)
            .cloned()
    }
}

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
