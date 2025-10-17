mod expressions;
use shared_context::{Identifier, Span};

pub use expressions::{BinaryOP, Expression, ExpressionType, UnaryOP};

pub struct Program {
    function: FunctionDef,
}

pub struct FunctionDef {
    name: Identifier,
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
    name: Identifier,
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
    Compound(Block),
    Null,
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

    // /// get a shared ref to the function
    // pub fn get_function_ref(&self) -> &FunctionDef {
    //     &self.function
    // }

    pub fn into_parts(self) -> FunctionDef {
        self.function
    }
}

//
// Function impl
//

impl FunctionDef {
    pub fn new(name: Identifier, body: Block, span: Span) -> Self {
        Self { name, body, span }
    }

    /// get a shared reference of the function identifier
    // pub fn get_name_ref(&self) -> &Identifier {
    //     &self.name
    // }

    // /// get a shared ref to the function body
    // pub fn get_body_ref(&self) -> &Block {
    //     &self.body
    // }

    pub fn into_parts(self) -> (Identifier, Block, Span) {
        (self.name, self.body, self.span)
    }
}

//
// Declaration impl
//

impl Declaration {
    pub fn new(name: Identifier, init: Option<Expression>, span: Span) -> Self {
        Self { name, init, span }
    }

    /// get a shared reference of the declaration identifier
    pub fn get_name_ref(&self) -> &Identifier {
        &self.name
    }

    /// get a shared reference of the expression if it exists
    pub fn get_init(&self) -> &Option<Expression> {
        &self.init
    }

    pub fn into_parts(self) -> (Identifier, Option<Expression>, Span) {
        (self.name, self.init, self.span)
    }
}
