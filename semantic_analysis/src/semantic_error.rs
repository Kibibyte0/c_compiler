use core::fmt;
use shared_context::Span;
use shared_context::source_map::SourceMap;

/// Enum representing different types of semantic errors that can occur during compilation.
/// Each variant stores relevant information (like source spans) to generate precise error messages.
#[derive(Debug)]
pub enum ErrorType {
    /// An identifier is declared more than once in the same scope.
    DuplicateDeclaration {
        first: Span,  // Span of the first declaration
        second: Span, // Span of the conflicting second declaration
    },
    /// A redeclaration with incompatible type or linkage
    IncompatibleDecl {
        first: Span,  // Span of the original declaration
        second: Span, // Span of the conflicting declaration
    },
    /// Function call with wrong number of arguments
    WrongNumberOfArgs {
        span: Span,      // Span of the call expression
        expected: usize, // Number of expected arguments
        got: usize,      // Number of arguments provided
    },
    /// Using an identifier that has not been declared
    UseOfUndeclared(Span),
    /// Left-hand side of an assignment is invalid (e.g., literal, function call)
    InvalidLeftValue(Span),
    /// `break` statement used outside of a loop
    BreakErr(Span),
    /// `continue` statement used outside of a loop
    ContinueErr(Span),
    /// Nested function definitions are not allowed in C
    NestedFunctionDecl(Span),
    /// Attempt to call a variable as a function
    VariableAsFunction(Span),
    /// Attempt to use a function as a variable
    FunctionAsVariable(Span),
}

/// SemanticErr wraps a formatted error message for display purposes
#[derive(Debug)]
pub struct SemanticErr {
    formated_error: String,
}

impl SemanticErr {
    /// Construct a new semantic error from an ErrorType
    /// The `source_map` is used to generate human-readable source code references
    pub fn new(err: ErrorType, source_map: &SourceMap) -> Self {
        let formated_error = match err {
            ErrorType::DuplicateDeclaration { first, second } => {
                Self::format_duplicate_decl_err(source_map, first, second)
            }
            ErrorType::IncompatibleDecl { first, second } => {
                Self::format_incompatible_decl_err(source_map, first, second)
            }
            ErrorType::WrongNumberOfArgs {
                span,
                expected,
                got,
            } => Self::format_wrong_number_of_args_err(source_map, span, expected, got),
            ErrorType::UseOfUndeclared(span) => {
                Self::format_use_of_undeclared_err(source_map, span)
            }
            ErrorType::InvalidLeftValue(span) => {
                Self::format_invalid_left_value_err(source_map, span)
            }
            ErrorType::BreakErr(span) => Self::format_break_error(source_map, span),
            ErrorType::ContinueErr(span) => Self::format_continue_error(source_map, span),
            ErrorType::NestedFunctionDecl(span) => {
                Self::format_nested_function_declaration(source_map, span)
            }
            ErrorType::VariableAsFunction(span) => {
                Self::format_variable_as_function_err(source_map, span)
            }
            ErrorType::FunctionAsVariable(span) => {
                Self::format_function_as_variable_err(source_map, span)
            }
        };

        Self { formated_error }
    }

    // Each of these functions generates a human-readable error message
    // including the relevant source code snippet using `source_map`.

    fn format_duplicate_decl_err(source_map: &SourceMap, first: Span, second: Span) -> String {
        format!(
            "duplicate declartion\n\
             first declaration:\n{}\
             second declaration:\n{}",
            source_map.format_message("".to_string(), first),
            source_map.format_message("".to_string(), second),
        )
    }

    fn format_incompatible_decl_err(source_map: &SourceMap, first: Span, second: Span) -> String {
        format!(
            "incompatible declartions\n\
             first declaration:\n{}\
             second declaration:\n{}",
            source_map.format_message("".to_string(), first),
            source_map.format_message("".to_string(), second),
        )
    }

    fn format_wrong_number_of_args_err(
        source_map: &SourceMap,
        span: Span,
        expected: usize,
        got: usize,
    ) -> String {
        format!(
            "expected {} argumnets, got {}\n{}",
            expected,
            got,
            source_map.format_message("".to_string(), span)
        )
    }

    fn format_use_of_undeclared_err(source_map: &SourceMap, span: Span) -> String {
        format!(
            "use of undeclared identifier\n{}",
            source_map.format_message("".to_string(), span)
        )
    }

    fn format_invalid_left_value_err(source_map: &SourceMap, span: Span) -> String {
        format!(
            "invalid left-hand side of assignment\n{}",
            source_map.format_message("".to_string(), span)
        )
    }

    fn format_break_error(source_map: &SourceMap, span: Span) -> String {
        format!(
            "break statement can't exists outside of loop\n{}",
            source_map.format_message("".to_string(), span)
        )
    }

    fn format_continue_error(source_map: &SourceMap, span: Span) -> String {
        format!(
            "continue statement can't exists outside of loop\n{}",
            source_map.format_message("".to_string(), span)
        )
    }

    fn format_nested_function_declaration(source_map: &SourceMap, span: Span) -> String {
        format!(
            "can't define a new function inside the the body of a function\n{}",
            source_map.format_message("".to_string(), span)
        )
    }

    fn format_variable_as_function_err(source_map: &SourceMap, span: Span) -> String {
        format!(
            "can't use a variable as a function\n{}",
            source_map.format_message("".to_string(), span)
        )
    }

    fn format_function_as_variable_err(source_map: &SourceMap, span: Span) -> String {
        format!(
            "can't use a function as a variable\n{}",
            source_map.format_message("".to_string(), span)
        )
    }
}

// Implement `Display` so semantic errors can be printed nicely
impl fmt::Display for SemanticErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.formated_error)
    }
}

// Implement `std::error::Error` to integrate with Rust error handling
impl std::error::Error for SemanticErr {}
