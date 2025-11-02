// Crate-level imports and re-exports
use crate::interner::Interner;
use crate::source_map::SourceMap;
use crate::symbol_table::SymbolTable;
pub use bumpalo::Bump; // Memory arena used for efficient allocation
use interner::Symbol;

// Submodules
pub mod interner; // Deduplicates strings and creates Symbols
pub mod source_map; // Maps AST positions to source code positions
pub mod symbol_table; // Stores symbols, types, and their metadata

/// Global compiler context
/// Holds the state shared across all compiler stages
/// This includes the interner, symbol table, and source map
pub struct CompilerContext<'a> {
    pub interner: Interner<'a>,    // For interning strings into Symbols
    pub source_map: SourceMap<'a>, // Maps AST nodes to source positions
    pub symbol_table: SymbolTable, // Tracks variable/function declarations
}

impl<'a> CompilerContext<'a> {
    /// Creates a new compiler context
    ///
    /// # Arguments
    /// - `arena`: Memory arena for allocations
    /// - `file_name`: Name of the source file
    /// - `source_code`: The source code itself
    pub fn new(arena: &'a Bump, file_name: &'a str, source_code: &'a str) -> Self {
        let interner = Interner::new(arena);
        let source_map = SourceMap::new(file_name, source_code);
        let symbol_table = SymbolTable::new();

        Self {
            interner,
            source_map,
            symbol_table,
        }
    }
}

/// Represents a region in the source code
/// Used for error reporting and mapping AST nodes to positions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Span {
    pub start: usize, // Starting byte offset in source
    pub end: usize,   // Ending byte offset in source
    pub line: usize,  // Line number in source
}

impl Default for Span {
    fn default() -> Self {
        Self {
            start: 0,
            end: 0,
            line: 0,
        }
    }
}

impl Span {
    pub fn new(start: usize, end: usize, line: usize) -> Self {
        Self { start, end, line }
    }

    /// returns a tuple (start, line)
    /// used in keeping track of the position during parsing
    pub fn get_start_and_line(&self) -> (usize, usize) {
        (self.start, self.line)
    }
}

/// Represents a unique identifier in the program
/// Interned via `Interner` for fast comparison
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Identifier {
    symbol: Symbol, // Interned string representing the identifier name
    id: usize,      // Unique numeric ID for disambiguation
}

impl Identifier {
    pub fn new(symbol: Symbol, id: usize) -> Self {
        Self { symbol, id }
    }

    pub fn get_symbol(&self) -> Symbol {
        self.symbol
    }

    pub fn get_id(&self) -> usize {
        self.id
    }

    pub fn into_parts(self) -> (Symbol, usize) {
        (self.symbol, self.id)
    }
}

impl Default for Identifier {
    fn default() -> Self {
        Self {
            symbol: Symbol(0),
            id: 0,
        }
    }
}

/// A combination of an identifier and its location in the source
/// Useful for precise error reporting and symbol tracking
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct SpannedIdentifier {
    identifier: Identifier, // The identifier itself
    span: Span,             // Location in source code
}

impl SpannedIdentifier {
    pub fn new(identifier: Identifier, span: Span) -> Self {
        Self { identifier, span }
    }

    pub fn get_identifier(&self) -> Identifier {
        self.identifier
    }

    pub fn get_span(&self) -> Span {
        self.span
    }

    pub fn into_parts(self) -> (Identifier, Span) {
        (self.identifier, self.span)
    }
}

/// a static variable in the IR
///
/// exach static variable contains, its identifier, linkage and initializer.
pub struct StaticVariable {
    name: Identifier,
    external: bool,
    init: i32,
}

impl StaticVariable {
    pub fn new(name: Identifier, external: bool, init: i32) -> Self {
        Self {
            name,
            external,
            init,
        }
    }

    /// consume the static variable node and retuen it's components
    pub fn into_parts(self) -> (Identifier, bool, i32) {
        (self.name, self.external, self.init)
    }
}
