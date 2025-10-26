mod expressions;
use shared_context::Identifier;
use shared_context::{Span, SpannedIdentifier};

pub use expressions::{BinaryOP, Expression, ExpressionType, UnaryOP};

/// Represents the top-level program node in the AST.
///
/// A Program consists of one or more function declarations.
pub struct Program {
    functions: Vec<FunctionDecl>,
}

/// Represents a block scope enclosed by `{}`.
///
/// A Block contains a list of BlockItems (statements or declarations)
/// and its corresponding source Span.
pub struct Block {
    items: Vec<BlockItem>,
    span: Span,
}

impl Block {
    /// Creates a new Block with the given items and span.
    pub fn new(items: Vec<BlockItem>, span: Span) -> Self {
        Self { items, span }
    }

    /// Deconstructs the block into its items and span.
    pub fn into_parts(self) -> (Vec<BlockItem>, Span) {
        (self.items, self.span)
    }
}

/// Represents a declaration within a block or at the global level.
///
/// Can be either a variable declaration or a function declaration.
pub enum Declaration {
    VarDecl(VariableDecl),
    FunDecl(FunctionDecl),
}

/// Represents a function declaration, including its parameters and body.
///
/// Each function has a name, a list of parameters, an optional body (for forward
/// declarations), and a Span describing its location.
pub struct FunctionDecl {
    name: SpannedIdentifier,
    params: Vec<SpannedIdentifier>,
    body: Option<Block>,
    span: Span,
}

impl FunctionDecl {
    /// Creates a new FunctionDecl.
    pub fn new(
        name: SpannedIdentifier,
        params: Vec<SpannedIdentifier>,
        body: Option<Block>,
        span: Span,
    ) -> Self {
        Self {
            name,
            params,
            body,
            span,
        }
    }

    /// Returns `true` if the function has a body (i.e., not just a prototype).
    pub fn has_body(&self) -> bool {
        self.body.is_some()
    }

    /// Returns the source span of the function declaration.
    pub fn get_span(&self) -> Span {
        self.span
    }

    /// Deconstructs the function declaration into its components.
    pub fn into_parts(
        self,
    ) -> (
        SpannedIdentifier,
        Vec<SpannedIdentifier>,
        Option<Block>,
        Span,
    ) {
        (self.name, self.params, self.body, self.span)
    }
}

/// Represents a variable declaration.
///
/// A variable may include an optional initializer expression.
pub struct VariableDecl {
    name: SpannedIdentifier,
    init: Option<Expression>,
    span: Span,
}

impl VariableDecl {
    /// Creates a new [`VariableDecl`].
    pub fn new(name: SpannedIdentifier, init: Option<Expression>, span: Span) -> Self {
        Self { name, init, span }
    }

    /// Deconstructs the variable declaration into its components.
    pub fn into_parts(self) -> (SpannedIdentifier, Option<Expression>, Span) {
        (self.name, self.init, self.span)
    }
}

/// Represents a single statement node in the AST.
///
/// The statement carries its variant and its associated Span.
pub struct Statement {
    stmt: StatementType,
    span: Span,
}

/// Enumerates the various types of statements supported by the language.
pub enum StatementType {
    /// A `return` statement with an expression.
    Return(Expression),

    /// A standalone expression as a statement (e.g. `x++;`).
    ExprStatement(Expression),

    /// An `if` statement, with an optional `else` clause.
    IfStatement {
        condition: Expression,
        if_clause: Box<Statement>,
        else_clause: Option<Box<Statement>>,
    },

    /// A `break` statement with an optional label.
    Break(Identifier),

    /// A `continue` statement with an optional label.
    Continue(Identifier),

    /// A `while` loop.
    While {
        condition: Expression,
        body: Box<Statement>,
        label: Identifier,
    },

    /// A `do...while` loop.
    DoWhile {
        condition: Expression,
        body: Box<Statement>,
        label: Identifier,
    },

    /// A `for` loop, which includes initialization, condition, post-expression, and body.
    For {
        init: ForInit,
        condition: Option<Expression>,
        post: Option<Expression>,
        body: Box<Statement>,
        label: Identifier,
    },

    /// A compound statement, i.e. a block `{ ... }`.
    Compound(Block),

    /// A null statement (e.g., a lone `;`).
    Null,
}

/// Represents the initialization clause of a `for` loop.
///
/// It can either be a declaration (`int i = 0;`) or an optional expression.
pub enum ForInit {
    D(VariableDecl),
    E(Option<Expression>),
}

impl Statement {
    /// Creates a new [`Statement`] of the given type and span.
    pub fn new(stmt: StatementType, span: Span) -> Self {
        Self { stmt, span }
    }

    /// Deconstructs the statement into its variant and span.
    pub fn into_parts(self) -> (StatementType, Span) {
        (self.stmt, self.span)
    }
}

/// Represents a single item within a block — either a declaration or a statement.
pub enum BlockItem {
    D(Declaration),
    S(Statement),
}

//
// Program implementation
//

impl Program {
    /// Creates a new Program with the given set of functions.
    pub fn new(functions: Vec<FunctionDecl>) -> Self {
        Self { functions }
    }

    /// Deconstructs the program into its function list.
    pub fn into_parts(self) -> Vec<FunctionDecl> {
        self.functions
    }
}
