use crate::{TypeChecker, semantic_error::ErrorType};
use parser::ast::{Expression, ExpressionType, StorageClass, VariableDecl};
use shared_context::{
    Span, SpannedIdentifier,
    symbol_table::{IdenAttrs, InitValue, SymbolEntry, Type},
};

impl<'src, 'ctx> TypeChecker<'src, 'ctx> {
    /// Perform type checking for a global variable declaration.
    ///
    /// Handles C storage classes such as `static` and `extern`, and checks
    /// consistency with previous declarations.
    pub(crate) fn typecheck_global_variable_declaration(
        &mut self,
        var_decl: VariableDecl,
    ) -> Result<VariableDecl, ErrorType> {
        // Decompose the variable declaration into its components.
        let (name, init, storage_class, span) = var_decl.into_parts();

        // Determine the effective initialization value and linkage (external/internal)
        // based on any previous declarations.
        let (init_value, external) =
            self.check_previous_variable_declaration(name, &init, span, storage_class)?;

        // Build identifier attributes for the symbol table entry.
        let attrs = IdenAttrs::StaticAttrs {
            init_value,
            external,
        };

        // Register the variable in the symbol table with its type and attributes.
        self.symbol_table.add(name, Type::Int, attrs, span);

        Ok(VariableDecl::new(name, init, storage_class, span))
    }

    /// Check the initializer expression for a variable declaration.
    ///
    /// - If an initializer exists, ensure it’s a valid constant expression.
    /// - If there’s no initializer, return a "tentative" initialization (as in C).
    ///
    /// Returns the evaluated `InitValue` or an error if invalid.
    fn check_declaration_init(
        &self,
        init: &Option<Expression>,
        storage_class: StorageClass,
    ) -> Result<InitValue, ErrorType> {
        if let Some(expr) = init {
            // Only constant expressions are allowed for global initializers.
            match expr.get_expr_type_ref() {
                ExpressionType::Constant(int_val) => Ok(InitValue::Initial(*int_val)),
                _ => Err(ErrorType::InvalidInitializer(expr.get_span())),
            }
        } else {
            if storage_class == StorageClass::Extern {
                Ok(InitValue::NoInitializer)
            } else {
                Ok(InitValue::Tentative)
            }
        }
    }

    /// Check for previous declarations of a global variable.
    ///
    /// - Verifies compatibility between multiple declarations of the same variable.
    /// - Handles linkage rules for `extern` and `static`.
    /// - Merges initialization state according to C rules.
    ///
    /// Returns a tuple:
    /// `(resolved_init_value, is_external)`
    fn check_previous_variable_declaration(
        &self,
        name: SpannedIdentifier,
        init: &Option<Expression>,
        span: Span,
        storage_class: StorageClass,
    ) -> Result<(InitValue, bool), ErrorType> {
        let mut external = storage_class != StorageClass::Static;
        let mut init_value = self.check_declaration_init(init, storage_class)?;

        if let Some(prev_decl) = self.symbol_table.get(name.get_identifier()) {
            // 1. Ensure the previous declaration is compatible (type, kind, etc.)
            self.ensure_compatible_declaration(&prev_decl, span)?;

            // 2. Resolve linkage compatibility and update `external` if needed.
            external = self.resolve_linkage_conflict(
                prev_decl.attributes,
                storage_class,
                external,
                span,
                prev_decl.span,
            )?;

            // 3. Merge initializers according to C rules and update
            match self.merge_initial_values(
                prev_decl.attributes,
                init_value,
                span,
                prev_decl.span,
            )? {
                Some(initlizer) => init_value = initlizer,
                None => (),
            }
        }

        Ok((init_value, external))
    }

    /// Ensure the previous declaration refers to a compatible entity type.
    fn ensure_compatible_declaration(
        &self,
        prev_decl: &SymbolEntry,
        current_span: Span,
    ) -> Result<(), ErrorType> {
        match prev_decl.attributes {
            IdenAttrs::StaticAttrs { .. } => Ok(()),
            _ => Err(ErrorType::IncompatibleDecl {
                first: prev_decl.span,
                second: current_span,
            }),
        }
    }

    /// Resolve linkage compatibility between two declarations.
    /// Returns the final `external` linkage status.
    fn resolve_linkage_conflict(
        &self,
        prev_attrs: IdenAttrs,
        storage_class: StorageClass,
        external: bool,
        current_span: Span,
        prev_span: Span,
    ) -> Result<bool, ErrorType> {
        let prev_external = prev_attrs.is_external();
        if storage_class == StorageClass::Extern {
            // Extern redeclaration keeps previous linkage
            Ok(prev_external)
        } else if prev_external != external {
            // Conflict: e.g. extern followed by static
            Err(ErrorType::IncompatibleLinkage {
                first: prev_span,
                second: current_span,
                first_external: prev_external,
                second_external: external,
            })
        } else {
            Ok(external)
        }
    }

    /// Merge initializer states for redeclarations of the same variable.
    fn merge_initial_values(
        &self,
        prev_attrs: IdenAttrs,
        current_value: InitValue,
        current_span: Span,
        prev_span: Span,
    ) -> Result<Option<InitValue>, ErrorType> {
        // at this point it is gaurnteed that prev_attrs have static attributes
        // thus this method will never return None
        let prev_value = prev_attrs.get_init_value().unwrap();
        match (prev_value, current_value) {
            (InitValue::Initial(_), InitValue::Initial(_)) => Err(ErrorType::DuplicateDefintion {
                first: prev_span,
                second: current_span,
            }),
            (InitValue::Initial(v), _) if !current_value.is_constant() => {
                Ok(Some(InitValue::Initial(v)))
            }
            (InitValue::Tentative, _) if !current_value.is_constant() => {
                Ok(Some(InitValue::Tentative))
            }
            _ => Ok(None),
        }
    }

    /// Type-check a local variable declaration.
    ///
    /// Handles `extern`, `static`, and automatic (default) storage classes.
    /// Performs semantic checks such as:
    /// - Disallowing initializers on `extern` locals.
    /// - Requiring constant initializers for local `static` variables.
    /// - Type-checking expressions for automatic locals.
    /// Type-check a local variable declaration.
    ///
    /// Dispatches to specialized handlers for `extern`, `static`, or automatic storage.
    /// Performs semantic checks for each case according to C language rules.
    pub(crate) fn typecheck_local_variable_declaration(
        &mut self,
        decl: VariableDecl,
    ) -> Result<VariableDecl, ErrorType> {
        let (name, init, storage_class, span) = decl.into_parts();

        match storage_class {
            StorageClass::Extern => {
                self.handle_local_extern_declaration(name, init, span, storage_class)
            }
            StorageClass::Static => {
                self.handle_local_static_declaration(name, init, storage_class, span)
            }
            _ => self.handle_automatic_local_declaration(name, init, span, storage_class),
        }
    }

    /// Handle a local variable declared with the `extern` storage class.
    ///
    /// Rules enforced:
    /// - Local `extern` variables cannot have initializers.
    /// - If already declared, its type must match previous declarations.
    /// - Otherwise, adds a new external symbol to the table.
    fn handle_local_extern_declaration(
        &mut self,
        name: SpannedIdentifier,
        init: Option<Expression>,
        span: Span,
        storage_class: StorageClass,
    ) -> Result<VariableDecl, ErrorType> {
        // extern local: cannot have an initializer
        if init.is_some() {
            return Err(ErrorType::InvalidInitializer(span));
        }

        // Check for previous declaration
        if let Some(prev_decl) = self.symbol_table.get(name.get_identifier()) {
            if prev_decl.entry_type != Type::Int {
                // e.g. redeclaring a function as a variable
                return Err(ErrorType::IncompatibleDecl {
                    first: prev_decl.span,
                    second: span,
                });
            }
        } else {
            // Implicit extern variable (not yet declared)
            let attrs = IdenAttrs::StaticAttrs {
                init_value: InitValue::NoInitializer,
                external: true,
            };
            self.symbol_table.add(name, Type::Int, attrs, span);
        }

        Ok(VariableDecl::new(name, init, storage_class, span))
    }

    /// Handle a local variable declared with the `static` storage class.
    ///
    /// Rules enforced:
    /// - Must have a constant integer initializer or none.
    /// - If no initializer is provided, defaults to zero-initialization.
    /// - The variable is added to the symbol table as a static local.
    fn handle_local_static_declaration(
        &mut self,
        name: SpannedIdentifier,
        init: Option<Expression>,
        storage_class: StorageClass,
        span: Span,
    ) -> Result<VariableDecl, ErrorType> {
        // Local static: must have constant or no initializer
        let initial_value = if let Some(expr) = &init {
            match expr.get_expr_type_ref() {
                ExpressionType::Constant(int) => InitValue::Initial(*int),
                _ => {
                    return Err(ErrorType::InvalidInitializer(expr.get_span()));
                }
            }
        } else {
            // No initializer → default zero initialization
            InitValue::Initial(0)
        };

        let attrs = IdenAttrs::StaticAttrs {
            init_value: initial_value,
            external: false,
        };

        self.symbol_table.add(name, Type::Int, attrs, span);
        Ok(VariableDecl::new(name, init, storage_class, span))
    }

    /// Handle a local variable with automatic (default) storage.
    ///
    /// Rules enforced:
    /// - Automatic variables have no special attributes.
    /// - If an initializer is present, it is fully type-checked as an expression.
    fn handle_automatic_local_declaration(
        &mut self,
        name: SpannedIdentifier,
        init: Option<Expression>,
        span: Span,
        storage_class: StorageClass,
    ) -> Result<VariableDecl, ErrorType> {
        // Locals: no special attributes needed
        let attrs = IdenAttrs::LocalAttrs;
        self.symbol_table.add(name, Type::Int, attrs, span);

        // Type-check the initializer expression if it exists
        let checked_init = if let Some(expr) = init {
            Some(self.typecheck_expression(expr)?)
        } else {
            None
        };

        Ok(VariableDecl::new(name, checked_init, storage_class, span))
    }
}
