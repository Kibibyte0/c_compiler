use bumpalo::Bump;
use std::collections::HashMap;

/// A `Symbol` represents a unique identifier for a string in the interner.
/// Internally, it's just an index into a vector of strings.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Symbol(pub(crate) usize);

/// `Interner` stores unique strings and assigns each a `Symbol`.
/// It uses a bump allocator (`Bump`) for fast memory allocation of strings.
pub struct Interner<'a> {
    arena: &'a Bump,               // Memory arena for allocating strings
    map: HashMap<&'a str, Symbol>, // Map from string slice to its symbol
    vec: Vec<&'a str>,             // Vector to look up a string by its Symbol
}

impl<'a> Interner<'a> {
    /// Creates a new `Interner` with a reference to a bump arena.
    /// The first position in the vector is reserved for a default string.
    pub fn new(arena: &'a Bump) -> Self {
        let mut vec = Vec::new();
        vec.push("default"); // Reserve index 0 as a default value

        Self {
            arena,
            map: HashMap::new(),
            vec,
        }
    }

    /// Interns a string and returns a `Symbol` representing it.
    /// If the string is already interned, returns the existing symbol.
    pub fn intern(&mut self, s: &str) -> Symbol {
        // Check if the string already exists
        if let Some(&sym) = self.map.get(s) {
            return sym;
        }

        // Assign a new symbol (next available index)
        let sym = Symbol(self.vec.len());

        // Allocate the string in the bump arena for fast memory management
        let stored: &'a str = self.arena.alloc_str(s);

        // Store it in the vector for symbol lookup and in the map for string lookup
        self.vec.push(stored);
        self.map.insert(stored, sym);

        sym
    }

    /// Looks up a string by its `Symbol`.
    pub fn lookup(&self, sym: Symbol) -> &'a str {
        self.vec[sym.0] // Simply index into the vector
    }
}
