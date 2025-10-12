use parser::ast::{Program, Spanned};

use crate::semantic_error::SemanticErr;

mod semantic_error;
pub mod variable_resolution;

pub(crate) struct VariableResolver<'a> {
    variable_counter: usize,
    file_name: &'a str,
    source_code: &'a str,
}

/// put the AST tree though all stages of semantic analysis
/// return the new AST tree and the auto variable generation counter
/// this counter is used during tacky generation to make sure that auto generated variables by both won't conflict
pub fn analize<'a>(
    file_name: &'a str,
    source_code: &'a str,
    sp_program: Spanned<Program>,
) -> Result<(Spanned<Program>, usize), SemanticErr> {
    let mut var_resolver = VariableResolver::new(file_name, source_code);
    let resolved_program = var_resolver.resolve_program(sp_program)?;
    Ok((resolved_program, var_resolver.get_var_count()))
}
