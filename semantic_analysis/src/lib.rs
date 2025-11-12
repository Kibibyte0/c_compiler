use identifier_resolution::IdentifierResolver;
use loop_labeling::LoopLabeling;
use parser::ast::Program;
use shared_context::{
    source_map::SourceMap, symbol_interner::SymbolInterner, symbol_table::SymbolTable,
    type_interner::TypeInterner,
};
use type_checker::TypeChecker;

use crate::semantic_error::SemanticErr;

// Modules for specific semantic passes
mod identifier_resolution;
mod loop_labeling;
mod semantic_error;
mod type_checker;

/// Run all semantic analysis passes on the AST.
/// Returns the transformed AST and the final auto-variable counter.
/// - The counter ensures that auto-generated variables in code generation won't collide.
pub fn analize<'src, 'ctx>(
    ty_interner: &'ctx TypeInterner<'src>,
    sy_interner: &'ctx mut SymbolInterner<'src>,
    symbol_table: &'ctx mut SymbolTable,
    source_map: &'ctx SourceMap<'src>,
    program: Program,
) -> Result<(Program, usize), SemanticErr> {
    // Identifier Resolution Pass
    let mut id_resolver = IdentifierResolver::new(source_map);
    let resolved_program = id_resolver.resolve_program(program)?;

    // Loop Labeling Pass
    let mut loop_labeling = LoopLabeling::new(sy_interner, source_map, id_resolver.get_var_count());
    let labeled_program = loop_labeling.label_program(resolved_program)?;
    let counter = loop_labeling.get_label_count();

    // Type Checking Pass
    let mut type_checker = TypeChecker::new(symbol_table, ty_interner, source_map);
    let checked_program = type_checker.typecheck_program(labeled_program)?;

    // Return fully processed AST and auto-variable counter
    Ok((checked_program, counter))
}
