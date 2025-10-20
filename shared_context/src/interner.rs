use bumpalo::Bump;
use std::collections::HashMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Symbol(pub(crate) usize);

pub struct Interner<'a> {
    arena: &'a Bump,
    map: HashMap<&'a str, Symbol>,
    vec: Vec<&'a str>,
}

impl<'a> Interner<'a> {
    pub fn new(arena: &'a Bump) -> Self {
        // reserve the first position for a default value
        let mut vec = Vec::new();
        vec.push("default");

        Self {
            arena,
            map: HashMap::new(),
            vec,
        }
    }

    pub fn intern(&mut self, s: &str) -> Symbol {
        if let Some(&sym) = self.map.get(s) {
            return sym;
        }

        let sym = Symbol(self.vec.len());

        // Allocate the string in the bump arena
        let stored: &'a str = self.arena.alloc_str(s);

        self.vec.push(stored);
        self.map.insert(stored, sym);

        sym
    }

    pub fn lookup(&self, sym: Symbol) -> &'a str {
        self.vec[sym.0]
    }
}
