use crate::{TypeChecker, semantic_error::ErrorType};
use parser::ast::*;
use shared_context::{
    Identifier, Span, SpannedIdentifier, symbol_table::EntryType, symbol_table::IdenAttrs,
    type_interner::TypeID,
};

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
        let (sp_iden, ty_id, params, body, storage_class, span) = function.into_parts();

        let has_body = body.is_some();
        // check if the function is external or internal
        let mut external = storage_class != StorageClass::Static;
        let mut defined = false;

        // if there exist a previous entry, update external and defined to match that of the previous entry
        match self.check_previous_function_decl(sp_iden, ty_id, storage_class, span, has_body)? {
            Some((prev_external, prev_defined)) => {
                external = prev_external;
                defined = prev_defined;
            }
            None => (),
        }

        // Register the function in the symbol table.
        self.register_function(
            sp_iden.get_identifier(),
            ty_id,
            span,
            external,
            defined || has_body,
        );

        if let Some(block) = body {
            // Function parameters are treated as variables within the function scope.
            self.register_function_params(&params, ty_id, span);
            let typechecked_body = Some(self.typecheck_block(block, ty_id)?);
            Ok(FunctionDecl::new(
                sp_iden,
                ty_id,
                params,
                typechecked_body,
                storage_class,
                span,
            ))
        } else {
            // Declaration without a body is allowed.
            Ok(FunctionDecl::new(
                sp_iden,
                ty_id,
                params,
                body,
                storage_class,
                span,
            ))
        }
    }

    /// Checks for previous declarations of the same function.
    ///
    /// # Logic
    /// - If a previous declaration exists:
    ///   - Ensure the type is compatible.
    ///   - Ensure they have the same storage class.
    ///   - Prevent redefining an already defined function.
    /// - Returns a tuple (external, defined) of the previous entry, returns None if there is no entry
    fn check_previous_function_decl(
        &self,
        sp_iden: SpannedIdentifier,
        ty_id: TypeID,
        storage_class: StorageClass,
        span: Span,
        has_body: bool,
    ) -> Result<Option<(bool, bool)>, ErrorType> {
        // if there is a previous declaration with the same identifier
        if let Some(prev_entry) = self.symbol_table.lookup(sp_iden.get_identifier()) {
            // chick if they have the same type
            if EntryType::Func(ty_id) != prev_entry.entry_type {
                return Err(ErrorType::IncompatibleDecl {
                    first: prev_entry.span,
                    second: span,
                });
            }
            let external = prev_entry.attributes.is_external();
            let defined = prev_entry.attributes.is_defined();
            // check if the previous declaration is also a definition
            if defined && has_body {
                return Err(ErrorType::DuplicateDefintion {
                    first: prev_entry.span,
                    second: sp_iden.get_span(),
                });
            }
            // check if they have compatible storage class
            if external && storage_class == StorageClass::Static {
                return Err(ErrorType::IncompatibleLinkage {
                    first: prev_entry.span,
                    second: span,
                    first_external: external,
                    second_external: storage_class != StorageClass::Static,
                });
            }
            // return the status of the previous declaration
            return Ok(Some((external, defined)));
        }
        Ok(None)
    }

    /// Registers a function in the symbol table.
    ///
    /// Marks whether it is already defined.
    fn register_function(
        &mut self,
        iden: Identifier,
        ty_id: TypeID,
        span: Span,
        external: bool,
        defined: bool,
    ) {
        let attrs = IdenAttrs::FunAttrs { defined, external };
        self.symbol_table
            .add(iden, EntryType::Func(ty_id), attrs, span);
    }

    /// Registers the parameters of a function as local variables.
    fn register_function_params(
        &mut self,
        params: &Vec<SpannedIdentifier>,
        ty_id: TypeID,
        span: Span,
    ) {
        let params_types = self.ty_interner.get(ty_id).params;
        for (sp_iden, param_type) in params.iter().zip(params_types) {
            self.symbol_table.add(
                sp_iden.get_identifier(),
                EntryType::Scalar(*param_type),
                IdenAttrs::LocalAttrs,
                span,
            );
        }
    }
}
