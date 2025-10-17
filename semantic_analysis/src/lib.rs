use std::collections::{HashMap, VecDeque};

use parser::ast::Program;
use shared_context::{CompilerContext, Identifier, interner::Symbol};

use crate::semantic_error::SemanticErr;

mod semantic_error;
pub mod variable_resolution;

struct ResolverContext {
    // Map from Symbol (variable name) to resolved Identifier (with binding_id)
    scopes: VecDeque<HashMap<Symbol, Identifier>>,
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

    fn insert_variable(&mut self, key: Symbol, value: Identifier) {
        self.scopes
            .front_mut()
            .expect("resolver context is empty")
            .insert(key, value);
    }

    fn search_scope(&self, key: &Symbol) -> Option<Identifier> {
        for scope in &self.scopes {
            if let Some(name) = scope.get(key) {
                return Some(*name);
            }
        }
        None
    }

    fn search_current_scope(&self, key: &Symbol) -> Option<Identifier> {
        self.scopes
            .front()
            .expect("resolver context is empty")
            .get(key)
            .cloned()
    }
}

pub(crate) struct VariableResolver<'a, 'c> {
    compiler_ctx: &'c CompilerContext<'a>,
    variable_counter: usize,
}

/// put the AST tree though all stages of semantic analysis
/// return the new AST tree and the auto variable generation counter
/// this counter is used during tacky generation to make sure that auto generated variables by both won't conflict
pub fn analize<'a, 'c>(
    compiler_ctx: &'c CompilerContext<'a>,
    program: Program,
) -> Result<(Program, usize), SemanticErr> {
    let mut var_resolver = VariableResolver::new(compiler_ctx);
    let resolved_program = var_resolver.resolve_program(program)?;
    Ok((resolved_program, var_resolver.get_var_count()))
}
