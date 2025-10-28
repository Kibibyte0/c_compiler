use crate::semantic_error::{ErrorType, SemanticErr};
use crate::{IdentifierResolver, ResolverContext, ResolverEntry};
use parser::ast::*;
use shared_context::source_map::SourceMap;
use shared_context::{Identifier, SpannedIdentifier};

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
        let functions = program.into_parts();
        let mut resolver_ctx = ResolverContext::new();
        resolver_ctx.create_scope(); // Create global scope

        let mut resolved_functions = Vec::new();
        for function in functions {
            resolved_functions.push(
                self.resolve_function_declaration(function, &mut resolver_ctx)
                    .map_err(|err| SemanticErr::new(err, &self.source_map))?,
            );
        }

        resolver_ctx.delete_scope(); // Clean up global scope
        Ok(Program::new(resolved_functions))
    }

    /// Resolves a single function declaration.
    ///
    /// Checks for duplicate declarations in the current scope, resolves parameters and body.
    fn resolve_function_declaration(
        &mut self,
        function: FunctionDecl,
        resolver_ctx: &mut ResolverContext,
    ) -> Result<FunctionDecl, ErrorType> {
        let (name, params, body, span) = function.into_parts();
        let symbol = name.get_identifier().get_symbol();

        // Check if a function with the same name already exists in the current scope
        if let Some(prev_entry) = resolver_ctx.search_current_scope(&symbol) {
            if !prev_entry.has_linkage() {
                return Err(ErrorType::DuplicateDeclaration {
                    first: prev_entry.get_sp_identifier().get_span(),
                    second: name.get_span(),
                });
            }
        }

        // Insert function into current scope with linkage = true (functions can be redeclared across scopes)
        resolver_ctx.insert_entry(symbol, ResolverEntry::new(name, true));

        resolver_ctx.create_scope(); // Create scope for function body

        let resolved_params = self.resolve_params(params, resolver_ctx)?;
        let resolved_body = if let Some(block) = body {
            Some(self.resolve_function_body(block, resolver_ctx)?)
        } else {
            None
        };

        resolver_ctx.delete_scope(); // Exit function body scope

        Ok(FunctionDecl::new(
            name,
            resolved_params,
            resolved_body,
            span,
        ))
    }

    /// Resolves function parameters.
    ///
    /// Each parameter is treated as a variable declaration within the function scope.
    fn resolve_params(
        &mut self,
        params: Vec<SpannedIdentifier>,
        resolver_ctx: &mut ResolverContext,
    ) -> Result<Vec<SpannedIdentifier>, ErrorType> {
        let mut resolved_params = Vec::new();
        for param in params {
            resolved_params
                .push(self.resolve_variable_declaration_identifier(param, resolver_ctx)?);
        }
        Ok(resolved_params)
    }

    /// Resolves the body of a function (block of statements/declarations).
    fn resolve_function_body(
        &mut self,
        block: Block,
        resolver_ctx: &mut ResolverContext,
    ) -> Result<Block, ErrorType> {
        let (block_items, span) = block.into_parts();
        let mut resolved_block = Vec::new();
        for item in block_items {
            let resolved_item = self.resolve_block_item(item, resolver_ctx)?;
            resolved_block.push(resolved_item);
        }
        Ok(Block::new(resolved_block, span))
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
            BlockItem::D(decl) => BlockItem::D(self.resolve_declaration(decl, resolver_ctx)?),
            BlockItem::S(stmt) => BlockItem::S(self.resolve_statement(stmt, resolver_ctx)?),
        };
        Ok(resolved_item)
    }

    /// Resolves a declaration (function or variable).
    ///
    /// Nested function definitions are not allowed in C, so they generate an error.
    fn resolve_declaration(
        &mut self,
        decl: Declaration,
        resolver_ctx: &mut ResolverContext,
    ) -> Result<Declaration, ErrorType> {
        match decl {
            Declaration::FunDecl(fun_decl) => {
                if !fun_decl.has_body() {
                    Ok(Declaration::FunDecl(
                        self.resolve_function_declaration(fun_decl, resolver_ctx)?,
                    ))
                } else {
                    Err(ErrorType::NestedFunctionDecl(fun_decl.get_span()))
                }
            }
            Declaration::VarDecl(var_decl) => Ok(Declaration::VarDecl(
                self.resolve_variable_declaration(var_decl, resolver_ctx)?,
            )),
        }
    }

    /// Resolves a variable declaration by resolving its identifier and initialization expression.
    fn resolve_variable_declaration(
        &mut self,
        var_decl: VariableDecl,
        resolver_ctx: &mut ResolverContext,
    ) -> Result<VariableDecl, ErrorType> {
        let (name, mut init, span) = var_decl.into_parts();
        let resolved_name = self.resolve_variable_declaration_identifier(name, resolver_ctx)?;
        if let Some(expr) = init {
            init = Some(self.resolve_expression(expr, resolver_ctx)?);
        }
        Ok(VariableDecl::new(resolved_name, init, span))
    }

    /// Resolves a variable declaration's identifier.
    ///
    /// Inserts the identifier into the current scope with linkage=false (local variable).
    fn resolve_variable_declaration_identifier(
        &mut self,
        name: SpannedIdentifier,
        resolver_ctx: &mut ResolverContext,
    ) -> Result<SpannedIdentifier, ErrorType> {
        let (identifier, name_span) = name.into_parts();
        let symbol = identifier.get_symbol();

        // Detect duplicate variable declarations in the same scope
        if let Some(prev_entry) = resolver_ctx.search_current_scope(&symbol) {
            return Err(ErrorType::DuplicateDeclaration {
                first: prev_entry.get_sp_identifier().get_span(),
                second: name_span,
            });
        }

        // Assign a unique compiler-generated ID to this variable
        let count = self.get_var_count_and_increment();
        let resolved_identifier = Identifier::new(symbol, count);
        let resolved_name = SpannedIdentifier::new(resolved_identifier, name_span);

        resolver_ctx.insert_entry(symbol, ResolverEntry::new(resolved_name, false)); // local variable
        Ok(resolved_name)
    }
}
