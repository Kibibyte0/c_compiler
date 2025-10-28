use crate::{
    TypeChecker,
    semantic_error::{ErrorType, SemanticErr},
};
use parser::ast::*;
use shared_context::{source_map::SourceMap, symbol_table::SymbolTable, symbol_table::Type};

mod typecheck_expressions;
mod typecheck_functions;
mod typecheck_statements;

impl<'src, 'ctx> TypeChecker<'src, 'ctx> {
    /// Constructs a new type checker.
    ///
    /// # Parameters
    /// - `compiler_ctx`: Shared compiler context containing the source map, interner and symbol table.
    ///
    /// # Purpose
    /// The `TypeChecker` pass performs static type analysis over the entire
    /// Abstract Syntax Tree (AST). It verifies that:
    ///
    /// 1. Every expression and statement is type-consistent.
    /// 2. All variable and function references adhere to declared types.
    /// 3. No invalid operations occur between incompatible types.
    pub fn new(symbol_table: &'ctx mut SymbolTable, source_map: &'ctx SourceMap<'src>) -> Self {
        Self {
            symbol_table,
            source_map,
        }
    }

    /// Performs full type checking on the input program.
    ///
    /// # Returns
    /// A new, type-annotated AST if successful.  
    /// A `SemanticErr` if a type mismatch or invalid use is detected.
    pub fn typecheck_program(&mut self, program: Program) -> Result<Program, SemanticErr> {
        let functions = program.into_parts();
        let mut typechecked_functions = Vec::new();

        for function in functions {
            let checked_function = match self.typecheck_function_declaration(function) {
                Ok(fun) => fun,
                Err(err) => return Err(SemanticErr::new(err, &self.source_map)),
            };
            typechecked_functions.push(checked_function);
        }

        Ok(Program::new(typechecked_functions))
    }

    /// Recursively type checks all statements and declarations in a block.
    fn typecheck_block(&mut self, block: Block) -> Result<Block, ErrorType> {
        let (block_items, span) = block.into_parts();
        let mut checked_block_items = Vec::new();

        for item in block_items {
            checked_block_items.push(self.typecheck_block_item(item)?);
        }

        Ok(Block::new(checked_block_items, span))
    }

    /// Type checks a single block item.
    fn typecheck_block_item(&mut self, item: BlockItem) -> Result<BlockItem, ErrorType> {
        match item {
            BlockItem::D(decl) => Ok(BlockItem::D(self.typecheck_declaration(decl)?)),
            BlockItem::S(stmt) => Ok(BlockItem::S(self.typecheck_statement(stmt)?)),
        }
    }

    /// Type checks a declaration node (function or variable).
    fn typecheck_declaration(&mut self, decl: Declaration) -> Result<Declaration, ErrorType> {
        match decl {
            Declaration::FunDecl(fun_decl) => Ok(Declaration::FunDecl(
                self.typecheck_function_declaration(fun_decl)?,
            )),
            Declaration::VarDecl(var_decl) => Ok(Declaration::VarDecl(
                self.typecheck_variable_declaration(var_decl)?,
            )),
        }
    }

    /// Type checks a variable declaration.
    ///
    /// # Behavior
    /// - Inserts the declared variable into the symbol table with a type
    /// - Type checks the initializer expression (if present) for compatibility.
    ///
    /// # Future Work
    /// Once type inference and multiple primitive types are supported,
    /// this function will need to:
    /// - Compare declared and inferred types.
    /// - Handle implicit type conversions or report mismatches.
    fn typecheck_variable_declaration(
        &mut self,
        var_decl: VariableDecl,
    ) -> Result<VariableDecl, ErrorType> {
        let (sp_iden, init, span) = var_decl.into_parts();

        // Register variable in the compiler's symbol table.
        // Currently, all variables are assumed to be of type `int`.
        self.symbol_table.add(sp_iden, Type::Int, span, false);

        // Recursively type check the initializer (if present).
        let checked_init = match init {
            Some(expr) => Some(self.typecheck_expression(expr)?),
            None => None,
        };

        Ok(VariableDecl::new(sp_iden, checked_init, span))
    }
}
