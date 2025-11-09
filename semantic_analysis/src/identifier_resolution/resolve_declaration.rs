use crate::IdentifierResolver;
use crate::identifier_resolution::{ResolverContext, ResolverEntry};
use crate::semantic_error::ErrorType;
use parser::ast::{Block, Declaration, FunctionDecl, StorageClass, VariableDecl};
use shared_context::{Identifier, SpannedIdentifier};

impl<'src, 'ctx> IdentifierResolver<'src, 'ctx> {
    /// resolve a global declaration
    pub(super) fn resolve_global_declaration(
        &mut self,
        decl: Declaration,
        resolver_ctx: &mut ResolverContext,
    ) -> Result<Declaration, ErrorType> {
        match decl {
            Declaration::FunDecl(fun_decl) => Ok(Declaration::FunDecl(
                self.resolve_function_declaration(fun_decl, resolver_ctx)?,
            )),
            Declaration::VarDecl(var_decl) => Ok(Declaration::VarDecl(
                self.resolve_global_variable_declaration(var_decl, resolver_ctx)?,
            )),
        }
    }

    /// Resolves a local declaration (function or variable).
    ///
    /// Nested function definitions are not allowed in C, so they generate an error.
    pub(super) fn resolve_local_declaration(
        &mut self,
        decl: Declaration,
        resolver_ctx: &mut ResolverContext,
    ) -> Result<Declaration, ErrorType> {
        match decl {
            Declaration::FunDecl(fun_decl) => {
                if fun_decl.get_storage_class() == StorageClass::Static {
                    Err(ErrorType::InvalidStaticDecl(
                        fun_decl.get_span(),
                        "can't declare a static function locally",
                    ))
                } else if fun_decl.has_body() {
                    Err(ErrorType::NestedFunctionDecl(fun_decl.get_span()))
                } else {
                    Ok(Declaration::FunDecl(
                        self.resolve_function_declaration(fun_decl, resolver_ctx)?,
                    ))
                }
            }
            Declaration::VarDecl(var_decl) => Ok(Declaration::VarDecl(
                self.resolve_local_variable_declaration(var_decl, resolver_ctx)?,
            )),
        }
    }

    /// Resolve a global variable declaration
    fn resolve_global_variable_declaration(
        &mut self,
        var_decl: VariableDecl,
        resolver_ctx: &mut ResolverContext,
    ) -> Result<VariableDecl, ErrorType> {
        let sp_iden = var_decl.get_sp_identifier();
        let symbol = sp_iden.get_identifier().get_symbol();
        resolver_ctx.insert_entry(symbol, ResolverEntry::new(sp_iden, true));
        return Ok(var_decl);
    }

    /// Resolves a local variable declaration by resolving its identifier and initialization expression.
    pub(super) fn resolve_local_variable_declaration(
        &mut self,
        var_decl: VariableDecl,
        resolver_ctx: &mut ResolverContext,
    ) -> Result<VariableDecl, ErrorType> {
        let (name, var_type, mut init, storage_class, span) = var_decl.into_parts();
        let resolved_name =
            self.resolve_variable_declaration_identifier(name, storage_class, resolver_ctx)?;
        if let Some(expr) = init {
            init = Some(self.resolve_expression(expr, resolver_ctx)?);
        }
        Ok(VariableDecl::new(
            resolved_name,
            var_type,
            init,
            storage_class,
            span,
        ))
    }

    /// Resolves a variable declaration's identifier.
    ///
    /// Inserts the identifier into the current scope with linkage=false (local variable).
    fn resolve_variable_declaration_identifier(
        &mut self,
        name: SpannedIdentifier,
        storage_class: StorageClass,
        resolver_ctx: &mut ResolverContext,
    ) -> Result<SpannedIdentifier, ErrorType> {
        let (identifier, name_span) = name.into_parts();
        let symbol = identifier.get_symbol();

        // Detect duplicate variable declarations in the same scope
        if let Some(prev_entry) = resolver_ctx.search_current_scope(&symbol) {
            // the previous entry have a linkage and the current declaration is extern, throw an error
            if !(prev_entry.has_linkage() && storage_class == StorageClass::Extern) {
                return Err(ErrorType::DuplicateDefintion {
                    first: prev_entry.get_sp_identifier().get_span(),
                    second: name_span,
                });
            }
        }

        // if the current declaration is extern, add it to the table and return it
        if let StorageClass::Extern = storage_class {
            resolver_ctx.insert_entry(symbol, ResolverEntry::new(name, true));
            return Ok(name);
        }

        // Assign a unique compiler-generated ID to this variable
        let count = self.get_var_count_and_increment();
        let resolved_identifier = Identifier::new(symbol, count);
        let resolved_name = SpannedIdentifier::new(resolved_identifier, name_span);

        resolver_ctx.insert_entry(symbol, ResolverEntry::new(resolved_name, false)); // local variable
        Ok(resolved_name)
    }

    /// Resolves a single function declaration.
    ///
    /// Checks for duplicate declarations in the current scope, resolves parameters and body.
    fn resolve_function_declaration(
        &mut self,
        function: FunctionDecl,
        resolver_ctx: &mut ResolverContext,
    ) -> Result<FunctionDecl, ErrorType> {
        let (name, var_type, params, body, storage_class, span) = function.into_parts();
        let symbol = name.get_identifier().get_symbol();

        // Check if an identifier with the same name already exists in the current scope
        if let Some(prev_entry) = resolver_ctx.search_current_scope(&symbol) {
            if !prev_entry.has_linkage() {
                return Err(ErrorType::DuplicateDefintion {
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
            var_type,
            resolved_params,
            resolved_body,
            storage_class,
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
            resolved_params.push(self.resolve_variable_declaration_identifier(
                param,
                StorageClass::None,
                resolver_ctx,
            )?);
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
}
