use crate::semantic_error::{ErrorType, SemanticErr};
use parser::ast::*;
use shared_context::{
    source_map::SourceMap,
    symbol_table::SymbolTable,
    type_interner::{TypeID, TypeInterner},
};

mod typecheck_expressions;
mod typecheck_functions;
mod typecheck_statements;
mod typecheck_variables;

/// Third pass: type checking
/// Ensures static typing rules are respected and expressions are correctly typed
pub(crate) struct TypeChecker<'src, 'ctx> {
    ty_interner: &'ctx TypeInterner<'src>,
    symbol_table: &'ctx mut SymbolTable,
    source_map: &'ctx SourceMap<'src>,
}

impl<'src, 'ctx> TypeChecker<'src, 'ctx> {
    /// Constructs a new type checker.
    ///
    /// # Purpose
    /// The `TypeChecker` pass performs static type analysis over the entire
    /// Abstract Syntax Tree (AST). It verifies that:
    ///
    /// 1. Every expression and statement is type-consistent.
    /// 2. All variable and function references adhere to declared types.
    /// 3. No invalid operations occur between incompatible types.
    pub fn new(
        symbol_table: &'ctx mut SymbolTable,
        ty_interner: &'ctx TypeInterner<'src>,
        source_map: &'ctx SourceMap<'src>,
    ) -> Self {
        Self {
            ty_interner,
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
        let declarations = program.into_parts();
        let mut typechecked_declarations = Vec::new();

        for decl in declarations {
            let checked_decl = match self.typecheck_global_declaration(decl) {
                Ok(fun) => fun,
                Err(err) => return Err(SemanticErr::new(err, &self.source_map)),
            };
            typechecked_declarations.push(checked_decl);
        }

        Ok(Program::new(typechecked_declarations))
    }

    /// Recursively type checks all statements and declarations in a block.
    /// curr_fun store the ID the current enclosing function, this is used to typecheck return statements
    fn typecheck_block(&mut self, block: Block, curr_fun: TypeID) -> Result<Block, ErrorType> {
        let (block_items, span) = block.into_parts();
        let mut checked_block_items = Vec::new();

        for item in block_items {
            checked_block_items.push(self.typecheck_block_item(item, curr_fun)?);
        }

        Ok(Block::new(checked_block_items, span))
    }

    /// Type checks a single block item.
    fn typecheck_block_item(
        &mut self,
        item: BlockItem,
        curr_fun: TypeID,
    ) -> Result<BlockItem, ErrorType> {
        match item {
            BlockItem::D(decl) => Ok(BlockItem::D(self.typecheck_local_declaration(decl)?)),
            BlockItem::S(stmt) => Ok(BlockItem::S(self.typecheck_statement(stmt, curr_fun)?)),
        }
    }

    /// Type checks a declaration node (function or variable).
    fn typecheck_local_declaration(&mut self, decl: Declaration) -> Result<Declaration, ErrorType> {
        match decl {
            Declaration::FunDecl(fun_decl) => Ok(Declaration::FunDecl(
                self.typecheck_function_declaration(fun_decl)?,
            )),
            Declaration::VarDecl(var_decl) => Ok(Declaration::VarDecl(
                self.typecheck_local_variable_declaration(var_decl)?,
            )),
        }
    }

    fn typecheck_global_declaration(
        &mut self,
        decl: Declaration,
    ) -> Result<Declaration, ErrorType> {
        match decl {
            Declaration::FunDecl(fun_decl) => Ok(Declaration::FunDecl(
                self.typecheck_function_declaration(fun_decl)?,
            )),
            Declaration::VarDecl(var_decl) => Ok(Declaration::VarDecl(
                self.typecheck_global_variable_declaration(var_decl)?,
            )),
        }
    }
}
