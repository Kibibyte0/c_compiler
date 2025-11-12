// Crate-level imports and re-exports
use crate::symbol_interner::SymbolInterner;
use crate::type_interner::TypeInterner;
pub use bumpalo::Bump; // Memory arena used for efficient allocation
use symbol_interner::Symbol;

pub use symbol_registry::SymbolRegistery;
pub use symbol_table::SymbolTable;

// Submodules
pub mod source_map; // Maps AST positions to source code positions
pub mod symbol_interner; // Deduplicates strings and creates Symbols
pub mod symbol_registry; // Stores symbols types, and their metadata after type checking for infallible access
pub mod symbol_table; // Stores symbols, types, and their metadata while typechecking
pub mod type_interner; // Deduplicate function types

/// Used to deduplicate identifiers and complex types, stores them in an arena
pub struct Interner<'arena> {
    pub sy: SymbolInterner<'arena>,
    pub ty: TypeInterner<'arena>,
}

impl<'arena> Interner<'arena> {
    pub fn new(arena: &'arena Bump) -> Self {
        Self {
            sy: SymbolInterner::new(arena),
            ty: TypeInterner::new(arena),
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
    var_type: Type,
    init: StaticInit,
}

impl StaticVariable {
    pub fn new(name: Identifier, external: bool, var_type: Type, init: StaticInit) -> Self {
        Self {
            name,
            var_type,
            external,
            init,
        }
    }

    /// consume the static variable node and retuen it's components
    pub fn into_parts(self) -> (Identifier, bool, Type, StaticInit) {
        (self.name, self.external, self.var_type, self.init)
    }
}

/// The `Type` enum represents the type of an identifier in the symbol table.
/// - `Int` represents a simple integer type.
/// - `FunType(usize)` represents a function type, where `usize` is the number of parameters.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Type {
    Int,
    Long,
    Uint,
    Ulong,
}

impl Type {
    pub fn size(&self) -> usize {
        match self {
            Type::Long | Type::Ulong => 8,
            Type::Int | Type::Uint => 4,
        }
    }

    pub fn is_signed(&self) -> bool {
        matches!(self, Type::Int | Type::Long)
    }
}

impl Default for Type {
    fn default() -> Self {
        Self::Int
    }
}

// Repressent an constant literal type,
#[derive(Debug, Clone, Copy)]
pub enum Const {
    ConstInt(i32),
    ConstLong(i64),
    ConstUint(u32),
    ConstUlong(u64),
}

// holds the type of initlizer a static variable can have.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum StaticInit {
    IntInit(i32),
    LongInit(i64),
    UintInit(u32),
    UlongInit(u64),
}

/// Represents an operand size in assembly
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum OperandSize {
    LongWord, // long word for 4 byte operands
    QuadWord, // quad word for 8 byte operands
}

pub fn convert_type_to_operand_size(t: Type) -> OperandSize {
    match t {
        Type::Int | Type::Uint => OperandSize::LongWord,
        Type::Long | Type::Ulong => OperandSize::QuadWord,
    }
}

/// convert a const into StaticInit according to the variable type
pub fn convert_constant_value_to_static_init(cons_val: Const, var_type: Type) -> StaticInit {
    // Normalize constant value to a u64 or i64 depending on signedness
    let (unsigned_value, signed_value) = match cons_val {
        Const::ConstInt(v) => (v as u64, v as i64),
        Const::ConstLong(v) => (v as u64, v),
        Const::ConstUint(v) => (v as u64, v as i64),
        Const::ConstUlong(v) => (v, v as i64),
    };

    match var_type {
        Type::Int => StaticInit::IntInit(signed_value as i32),
        Type::Long => StaticInit::LongInit(signed_value as i64),
        Type::Uint => StaticInit::UintInit(unsigned_value as u32),
        Type::Ulong => StaticInit::UlongInit(unsigned_value as u64),
    }
}

/// get the tentative initializer for a static variable
pub fn get_tentative_init(var_type: Type) -> StaticInit {
    match var_type {
        Type::Int => StaticInit::IntInit(0),
        Type::Long => StaticInit::LongInit(0),
        Type::Uint => StaticInit::UintInit(0),
        Type::Ulong => StaticInit::UlongInit(0),
    }
}
