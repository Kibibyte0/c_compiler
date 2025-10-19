mod expressions;
use shared_context::Identifier;
use std::ops::Range;

pub use expressions::{BinaryOP, Expression, ExpressionType, UnaryOP};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Span {
    start: usize,
    end: usize,
    line: usize,
}

impl Span {
    pub fn new(start: usize, end: usize, line: usize) -> Self {
        Self { start, end, line }
    }

    pub fn get_range(&self) -> Range<usize> {
        self.start..self.end
    }

    pub fn get_line(&self) -> usize {
        self.line
    }
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

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct SpannedIdentifier {
    identifier: Identifier,
    span: Span,
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

pub struct Program {
    function: FunctionDef,
}

pub struct FunctionDef {
    name: SpannedIdentifier,
    body: Block,
    span: Span,
}

pub struct Block {
    items: Vec<BlockItem>,
    span: Span,
}

impl Block {
    pub fn new(items: Vec<BlockItem>, span: Span) -> Self {
        Self { items, span }
    }
    pub fn into_parts(self) -> (Vec<BlockItem>, Span) {
        (self.items, self.span)
    }
}

pub struct Declaration {
    name: SpannedIdentifier,
    init: Option<Expression>,
    span: Span,
}

pub struct Statement {
    stmt: StatementType,
    span: Span,
}

pub enum StatementType {
    Return(Expression),
    ExprStatement(Expression),
    IfStatement {
        condition: Expression,
        if_clause: Box<Statement>,
        else_clause: Option<Box<Statement>>,
    },
    Break(Identifier),
    Continue(Identifier),
    While {
        condition: Expression,
        body: Box<Statement>,
        label: Identifier,
    },
    DoWhile {
        condition: Expression,
        body: Box<Statement>,
        label: Identifier,
    },
    For {
        init: ForInit,
        condition: Option<Expression>,
        post: Option<Expression>,
        body: Box<Statement>,
        label: Identifier,
    },
    Compound(Block),
    Null,
}

pub enum ForInit {
    D(Declaration),
    E(Option<Expression>),
}

impl Statement {
    pub fn new(stmt: StatementType, span: Span) -> Self {
        Self { stmt, span }
    }

    pub fn into_parts(self) -> (StatementType, Span) {
        (self.stmt, self.span)
    }
}

pub enum BlockItem {
    D(Declaration),
    S(Statement),
}

//
// Program impl
//

impl Program {
    pub fn new(function: FunctionDef) -> Self {
        Self { function }
    }

    pub fn into_parts(self) -> FunctionDef {
        self.function
    }
}

//
// Function impl
//

impl FunctionDef {
    pub fn new(name: SpannedIdentifier, body: Block, span: Span) -> Self {
        Self { name, body, span }
    }

    pub fn into_parts(self) -> (SpannedIdentifier, Block, Span) {
        (self.name, self.body, self.span)
    }
}

//
// Declaration impl
//

impl Declaration {
    pub fn new(name: SpannedIdentifier, init: Option<Expression>, span: Span) -> Self {
        Self { name, init, span }
    }

    /// get a shared reference of the declaration identifier
    pub fn get_name_ref(&self) -> &SpannedIdentifier {
        &self.name
    }

    /// get a shared reference of the expression if it exists
    pub fn get_init(&self) -> &Option<Expression> {
        &self.init
    }

    pub fn into_parts(self) -> (SpannedIdentifier, Option<Expression>, Span) {
        (self.name, self.init, self.span)
    }
}
