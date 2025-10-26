use crate::{Identifier, Span, SpannedIdentifier};
use std::{collections::HashMap, usize};

/// The `Type` enum represents the type of an identifier in the symbol table.
/// - `Int` represents a simple integer type.
/// - `FunType(usize)` represents a function type, where `usize` is the number of parameters.
#[derive(Debug, Clone, PartialEq, Copy, Eq)]
pub enum Type {
    Int,
    FunType(usize),
}

/// A `SymbolEntry` represents an entry in the symbol table.
/// It stores information about an identifier.
#[derive(Debug, Clone, PartialEq, Copy)]
pub struct SymbolEntry {
    pub sp_iden: SpannedIdentifier, // The identifier along with its source span
    pub entry_type: Type,           // Type of the identifier (variable or function)
    pub span: Span,                 // Source code span where the identifier was declared
    pub defined: bool,              // Whether the function is defined (only relevant for functions)
}

pub struct SymbolTable {
    table: HashMap<Identifier, SymbolEntry>, // Internal hashmap for fast lookups
}

impl SymbolTable {
    /// Creates a new, empty symbol table.
    pub fn new() -> Self {
        Self {
            table: HashMap::new(),
        }
    }

    /// Retrieves a `SymbolEntry` for a given identifier if it exists.
    /// Returns `None` if the identifier is not found.
    pub fn get(&self, key: Identifier) -> Option<SymbolEntry> {
        self.table.get(&key).cloned() // Clone the entry to avoid ownership issues
    }

    /// Adds a new identifier to the symbol table.
    pub fn add(&mut self, sp_iden: SpannedIdentifier, entry_type: Type, span: Span, defined: bool) {
        self.table.insert(
            sp_iden.get_identifier(), // Use the identifier as the key
            SymbolEntry {
                sp_iden,
                entry_type,
                span,
                defined,
            },
        );
    }
}
