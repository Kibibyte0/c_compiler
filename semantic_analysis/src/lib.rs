use std::collections::{HashMap, VecDeque};

use parser::ast::{Program, SpannedIdentifier};
use shared_context::{CompilerContext, interner::Symbol};

use crate::semantic_error::SemanticErr;

mod loop_labeling;
mod semantic_error;
mod variable_resolution;

struct ResolverContext {
    // Map from Symbol (variable name) to resolved Identifier (with binding_id)
    scopes: VecDeque<HashMap<Symbol, SpannedIdentifier>>,
}

impl ResolverContext {
    fn new() -> Self {
        Self {
            scopes: VecDeque::new(),
        }
    }

    fn create_scope(&mut self) {
        self.scopes.push_front(HashMap::new());
    }

    fn delete_scope(&mut self) {
        self.scopes.pop_front();
    }

    fn insert_variable(&mut self, key: Symbol, value: SpannedIdentifier) {
        self.scopes
            .front_mut()
            .expect("resolver context is empty")
            .insert(key, value);
    }

    fn search_scope(&self, key: &Symbol) -> Option<SpannedIdentifier> {
        for scope in &self.scopes {
            if let Some(name) = scope.get(key) {
                return Some(*name);
            }
        }
        None
    }

    fn search_current_scope(&self, key: &Symbol) -> Option<SpannedIdentifier> {
        self.scopes
            .front()
            .expect("resolver context is empty")
            .get(key)
            .cloned()
    }
}

pub(crate) struct VariableResolver<'src, 'c> {
    compiler_ctx: &'c CompilerContext<'src>,
    variable_counter: usize,
}

pub(crate) struct LoopLabeling<'src, 'c> {
    compiler_ctx: &'c mut CompilerContext<'src>,
    label_counter: usize,
}

/// put the AST tree though all stages of semantic analysis
/// return the new AST tree and the auto variable generation counter
/// this counter is used during tacky generation to make sure that auto generated variables by both won't conflict
pub fn analize(
    compiler_ctx: &mut CompilerContext,
    program: Program,
) -> Result<(Program, usize), SemanticErr> {
    let mut var_resolver = VariableResolver::new(compiler_ctx);
    let resolved_program = var_resolver.resolve_program(program)?;
    let mut loop_labeling = LoopLabeling::new(compiler_ctx, var_resolver.get_var_count());
    let labeled_program = loop_labeling.label_program(resolved_program)?;
    Ok((labeled_program, loop_labeling.get_label_count()))
}
