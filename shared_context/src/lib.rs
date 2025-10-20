use crate::interner::Interner;
use crate::source_map::SourceMap;
pub use bumpalo::Bump;
use interner::Symbol;

pub mod interner;
pub mod source_map;

// this will contain the global compiler context
// which is the interner, a table for storing identifiers
// and a source map, maps positions in the AST to the corresponding positions in the source file
pub struct CompilerContext<'a> {
    pub interner: Interner<'a>,
    pub source_map: SourceMap<'a>,
}

impl<'a> CompilerContext<'a> {
    pub fn new(arena: &'a Bump, file_name: &'a str, source_code: &'a str) -> Self {
        let interner = Interner::new(arena);
        let source_map = SourceMap::new(file_name, source_code);

        Self {
            interner,
            source_map,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Identifier {
    symbol: Symbol,
    id: usize,
}

impl Identifier {
    pub fn new(symbol: Symbol, id: usize) -> Self {
        Self { symbol, id }
    }

    pub fn get_symbol(&self) -> Symbol {
        self.symbol
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
