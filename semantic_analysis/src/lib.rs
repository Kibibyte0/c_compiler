use std::collections::{HashMap, VecDeque};

use parser::ast::{Identifier, Program, Spanned};

use crate::semantic_error::SemanticErr;

mod semantic_error;
pub mod variable_resolution;

struct ResolverContext {
    scopes: VecDeque<HashMap<Identifier, Spanned<Identifier>>>,
}

impl ResolverContext {
    fn new() -> Self {
        Self {
            scopes: VecDeque::new(),
        }
    }

    /// create a hash table and insert it at the beginning of the queue
    fn create_scope(&mut self) {
        self.scopes.push_front(HashMap::new());
    }

    /// delete the current scope
    fn delete_scope(&mut self) {
        self.scopes.pop_front();
    }

    /// insert a variable at the current scope
    fn insert_variable(&mut self, key: Identifier, value: Spanned<Identifier>) {
        self.scopes
            .front_mut()
            .expect("resolver context is empty")
            .insert(key, value);
    }

    /// searches the variable from the inner most scope
    /// returns None if it was not found
    fn search_scope(&self, key: &Identifier) -> Option<Spanned<Identifier>> {
        for scope in &self.scopes {
            if let Some(sp_name) = scope.get(&key) {
                return Some(sp_name.clone());
            }
        }

        None
    }

    /// search the only the current scope
    fn search_current_scope(&self, key: &Identifier) -> Option<Spanned<Identifier>> {
        self.scopes
            .front()
            .expect("resolver context is empty")
            .get(key)
            .cloned()
    }
}

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
