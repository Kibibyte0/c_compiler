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

// the identifier attributes type hold metadata about the identifier
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum IdenAttrs {
    // holds whether the function is external or not, and whether it's defined or not
    FunAttrs {
        defined: bool,
        external: bool,
    },
    // holds the initial value, and whether it's external or not
    StaticAttrs {
        init_value: InitValue,
        external: bool,
    },
    // repressent variables with automatic storage duration
    LocalAttrs,
}

impl IdenAttrs {
    pub fn is_external(&self) -> bool {
        match self {
            IdenAttrs::FunAttrs {
                defined: _,
                external,
            } => *external,
            IdenAttrs::StaticAttrs {
                init_value: _,
                external,
            } => *external,
            IdenAttrs::LocalAttrs => false,
        }
    }

    pub fn is_defined(&self) -> bool {
        match self {
            IdenAttrs::FunAttrs {
                defined,
                external: _,
            } => *defined,
            IdenAttrs::StaticAttrs {
                init_value: _,
                external: _,
            } => true,
            IdenAttrs::LocalAttrs => true,
        }
    }

    pub fn get_init_value(&self) -> Option<InitValue> {
        match self {
            IdenAttrs::StaticAttrs {
                init_value,
                external: _,
            } => Some(*init_value),
            _ => None,
        }
    }
}

// holds metadata about the variable initializer (eg. tentative or not initialized)
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum InitValue {
    Tentative,
    Initial(i32),
    NoInitializer,
}

impl InitValue {
    pub fn is_constant(&self) -> bool {
        match self {
            InitValue::Initial(_) => true,
            _ => false,
        }
    }
}

/// A `SymbolEntry` represents an entry in the symbol table.
/// It stores information about an identifier.
#[derive(Debug, Clone, PartialEq, Copy)]
pub struct SymbolEntry {
    pub sp_iden: SpannedIdentifier, // The identifier along with its source span
    pub entry_type: Type,           // Type of the identifier (variable or function)
    pub attributes: IdenAttrs,      // hold metadata about the identifier
    pub span: Span,                 // Source code span where the identifier was declared
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
    pub fn add(
        &mut self,
        sp_iden: SpannedIdentifier,
        entry_type: Type,
        attributes: IdenAttrs,
        span: Span,
    ) {
        self.table.insert(
            sp_iden.get_identifier(), // Use the identifier as the key
            SymbolEntry {
                sp_iden,
                attributes,
                entry_type,
                span,
            },
        );
    }

    /// get a reference to the hash map in symbol table
    pub fn get_table_ref(&self) -> &HashMap<Identifier, SymbolEntry> {
        &self.table
    }
}
