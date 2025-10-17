use crate::interner::Interner;
use crate::source_map::SourceMap;
pub use bumpalo::Bump;
use interner::Symbol;
use std::ops::Range;

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
pub struct Span {
    start: usize,
    end: usize,
}

impl Span {
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    pub fn get_range(&self) -> Range<usize> {
        self.start..self.end
    }
}

impl Default for Span {
    fn default() -> Self {
        Self { start: 0, end: 0 }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Identifier {
    symbol: Symbol,
    id: usize,
    span: Span,
}

impl Identifier {
    pub fn new(symbol: Symbol, id: usize, span: Span) -> Self {
        Self { symbol, id, span }
    }

    pub fn get_symbol(&self) -> Symbol {
        self.symbol
    }

    pub fn get_span(&self) -> Span {
        self.span.clone()
    }

    pub fn into_parts(self) -> (Symbol, usize, Span) {
        (self.symbol, self.id, self.span)
    }
}
