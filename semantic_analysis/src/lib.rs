use std::collections::{HashMap, VecDeque};

use parser::ast::Program;
use shared_context::{
    SpannedIdentifier,
    interner::{Interner, Symbol},
    source_map::SourceMap,
    symbol_table::SymbolTable,
};

use crate::semantic_error::SemanticErr;

// Modules for specific semantic passes
mod identifier_resolution;
mod loop_labeling;
mod semantic_error;
mod type_checker;

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

/// First pass: resolves variable and function identifiers
/// Builds the symbol table and detects duplicate declarations
pub(crate) struct IdentifierResolver<'src, 'ctx> {
    source_map: &'ctx SourceMap<'src>,
    variable_counter: usize, // Counter for auto-generated variables
}

/// Second pass: labels each loop to support `break` and `continue`
/// Ensures break/continue are used only inside loops
pub(crate) struct LoopLabeling<'src, 'ctx> {
    interner: &'ctx mut Interner<'src>,
    source_map: &'ctx SourceMap<'src>,
    label_counter: usize, // Counter for unique loop labels
}

/// Third pass: type checking
/// Ensures static typing rules are respected and expressions are correctly typed
pub(crate) struct TypeChecker<'src, 'ctx> {
    symbol_table: &'ctx mut SymbolTable,
    source_map: &'ctx SourceMap<'src>,
}

/// Run all semantic analysis passes on the AST.
/// Returns the transformed AST and the final auto-variable counter.
/// - The counter ensures that auto-generated variables in code generation won't collide.
pub fn analize<'src, 'ctx>(
    interner: &'ctx mut Interner<'src>,
    symbol_table: &'ctx mut SymbolTable,
    source_map: &'ctx SourceMap<'src>,
    program: Program,
) -> Result<(Program, usize), SemanticErr> {
    // Identifier Resolution Pass
    let mut id_resolver = IdentifierResolver::new(source_map);
    let resolved_program = id_resolver.resolve_program(program)?;

    // Loop Labeling Pass
    let mut loop_labeling = LoopLabeling::new(interner, source_map, id_resolver.get_var_count());
    let labeled_program = loop_labeling.label_program(resolved_program)?;
    let counter = loop_labeling.get_label_count();

    // Type Checking Pass
    let mut type_checker = TypeChecker::new(symbol_table, source_map);
    let checked_program = type_checker.typecheck_program(labeled_program)?;

    // Return fully processed AST and auto-variable counter
    Ok((checked_program, counter))
}
