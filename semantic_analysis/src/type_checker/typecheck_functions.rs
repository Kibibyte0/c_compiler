use crate::{TypeChecker, semantic_error::ErrorType};
use parser::ast::*;
use shared_context::{Span, SpannedIdentifier, symbol_table::Type};

impl<'src, 'ctx> TypeChecker<'src, 'ctx> {
    /// Type checks a function declaration.
    ///
    /// # Behavior
    /// - Checks if a function with the same name was previously declared.
    ///   - Ensures consistent parameter counts.
    ///   - Detects duplicate definitions.
    /// - Registers the function in the symbol table.
    /// - Type checks the function body if present.
    pub(super) fn typecheck_function_declaration(
        &mut self,
        function: FunctionDecl,
    ) -> Result<FunctionDecl, ErrorType> {
        let (sp_iden, params, body, span) = function.into_parts();

        // Currently, function type is represented by its arity (number of parameters).
        let fun_type = Type::FunType(params.len());

        let has_body = body.is_some();

        // Check for previous declarations or definitions of the same function.
        let defined = self.check_previous_function_decl(sp_iden, fun_type, span, has_body)?;

        // Register the function in the symbol table.
        self.register_function(sp_iden, fun_type, span, defined || has_body);

        if let Some(block) = body {
            // Function parameters are treated as variables within the function scope.
            self.register_function_params(&params, span);
            let typechecked_body = Some(self.typecheck_block(block)?);
            Ok(FunctionDecl::new(sp_iden, params, typechecked_body, span))
        } else {
            // Declaration without a body is allowed.
            Ok(FunctionDecl::new(sp_iden, params, body, span))
        }
    }

    /// Checks for previous declarations of the same function.
    ///
    /// # Logic
    /// - If a previous declaration exists:
    ///   - Ensure the type is compatible.
    ///   - Prevent redefining an already defined function.
    /// - Returns `true` if a previous definition exists, `false` otherwise.
    fn check_previous_function_decl(
        &self,
        sp_iden: SpannedIdentifier,
        fun_type: Type,
        span: Span,
        has_body: bool,
    ) -> Result<bool, ErrorType> {
        if let Some(prev_entry) = self.compiler_ctx.symbol_table.get(sp_iden.get_identifier()) {
            if fun_type != prev_entry.entry_type {
                return Err(ErrorType::IncompatibleDecl {
                    first: span,
                    second: prev_entry.span,
                });
            }
            if prev_entry.defined && has_body {
                return Err(ErrorType::DuplicateDeclaration {
                    first: prev_entry.sp_iden.get_span(),
                    second: sp_iden.get_span(),
                });
            }
            return Ok(prev_entry.defined);
        }
        Ok(false)
    }

    /// Registers a function in the symbol table.
    ///
    /// Marks whether it is already defined.
    fn register_function(
        &mut self,
        sp_iden: SpannedIdentifier,
        fun_type: Type,
        span: Span,
        defined: bool,
    ) {
        self.compiler_ctx
            .symbol_table
            .add(sp_iden.clone(), fun_type.clone(), span, defined);
    }

    /// Registers the parameters of a function as local variables.
    ///
    /// Currently all parameters are assumed to be `int`.
    fn register_function_params(&mut self, params: &Vec<SpannedIdentifier>, span: Span) {
        for param in params {
            self.compiler_ctx
                .symbol_table
                .add(param.clone(), Type::Int, span, false);
        }
    }
}
