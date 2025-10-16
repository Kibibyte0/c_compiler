mod expressions;

use std::ops::Range;

pub use expressions::{BinaryOP, Expression, UnaryOP};

#[derive(Debug, Clone)]
pub struct Spanned<T> {
    node: T,
    span: Range<usize>,
}

impl<T> Spanned<T> {
    pub fn new(node: T, span: Range<usize>) -> Self {
        Self { node, span }
    }

    pub fn get_span_copy(&self) -> Range<usize> {
        self.span.clone()
    }

    pub fn get_span_ref(&self) -> &Range<usize> {
        &self.span
    }

    pub fn get_node_ref(&self) -> &T {
        &self.node
    }

    /// discard the span wrapper, returning wrapped data
    pub fn discard_sp(self) -> T {
        self.node
    }

    pub fn into_parts(self) -> (T, Range<usize>) {
        (self.node, self.span)
    }
}

pub struct Program {
    function: Spanned<FunctionDef>,
}

pub struct FunctionDef {
    name: Spanned<Identifier>,
    body: Spanned<Block>,
}

pub struct Block {
    items: Vec<Spanned<BlockItem>>,
}

impl Block {
    pub fn new(items: Vec<Spanned<BlockItem>>) -> Self {
        Self { items }
    }
    pub fn into_parts(self) -> Vec<Spanned<BlockItem>> {
        self.items
    }
}

#[derive(Hash, Eq, PartialEq, Debug, Clone)]
pub struct Identifier(String);

impl Identifier {
    pub fn new(name: String) -> Self {
        Self(name)
    }

    pub fn into_parts(self) -> String {
        self.0
    }

    /// returns a copy of the identifier name
    pub fn get_name_copy(&self) -> String {
        self.0.clone()
    }

    /// get a reference to the identifier name
    pub fn get_name_ref(&self) -> &str {
        &self.0
    }
}

pub struct Declaration {
    name: Spanned<Identifier>,
    init: Option<Spanned<Expression>>,
}

pub enum Statement {
    Return(Spanned<Expression>),
    ExprStatement(Spanned<Expression>),
    IfStatement {
        condition: Spanned<Expression>,
        if_clause: Box<Spanned<Statement>>,
        else_clause: Option<Box<Spanned<Statement>>>,
    },
    Compound(Spanned<Block>),
    Null,
}

pub enum BlockItem {
    D(Spanned<Declaration>),
    S(Spanned<Statement>),
}

//
// Program impl
//

impl Program {
    pub fn new(function: Spanned<FunctionDef>) -> Self {
        Self { function }
    }

    /// get a shared ref to the function
    pub fn get_function_ref(&self) -> &Spanned<FunctionDef> {
        &self.function
    }

    pub fn into_parts(self) -> Spanned<FunctionDef> {
        self.function
    }
}

//
// Function impl
//

impl FunctionDef {
    pub fn new(name: Spanned<Identifier>, body: Spanned<Block>) -> Self {
        Self { name, body }
    }

    /// get a shared reference of the function identifier
    pub fn get_name_ref(&self) -> &Spanned<Identifier> {
        &self.name
    }

    /// get a shared ref to the function body
    pub fn get_body_ref(&self) -> &Spanned<Block> {
        &self.body
    }

    pub fn into_parts(self) -> (Spanned<Identifier>, Spanned<Block>) {
        (self.name, self.body)
    }
}

//
// Declaration impl
//

impl Declaration {
    pub fn new(name: Spanned<Identifier>, init: Option<Spanned<Expression>>) -> Self {
        Self { name, init }
    }

    /// get a shared reference of the declaration identifier
    pub fn get_name_ref(&self) -> &Spanned<Identifier> {
        &self.name
    }

    /// get a shared reference of the expression if it exists
    pub fn get_init(&self) -> &Option<Spanned<Expression>> {
        &self.init
    }

    pub fn into_parts(self) -> (Spanned<Identifier>, Option<Spanned<Expression>>) {
        (self.name, self.init)
    }
}
