use crate::{
    LoopLabeling,
    semantic_error::{ErrorType, SemanticErr},
};
use parser::ast::*;
use shared_context::{Identifier, Span, source_map::SourceMap, symbol_interner::SymbolInterner};

impl<'src, 'ctx> LoopLabeling<'src, 'ctx> {
    /// Creates a new loop labeling pass.
    ///
    /// # Parameters
    /// - `compiler_ctx`: Shared compiler context containing the source map, interner and symbol table.
    /// - `label_counter`: The initial counter for generating unique loop labels.
    ///
    /// # Purpose
    /// This pass traverses the AST and:
    /// 1. Assigns unique labels to each loop construct (`while`, `do-while`, `for`).
    /// 2. Ensures that `break` and `continue` statements appear only inside loops.
    /// 3. Associates each `break`/`continue` with the label of its nearest enclosing loop.
    pub fn new(
        sy_interner: &'ctx mut SymbolInterner<'src>,
        source_map: &'ctx SourceMap<'src>,
        label_counter: usize,
    ) -> Self {
        Self {
            sy_interner,
            source_map,
            label_counter,
        }
    }

    /// Generates a new unique label identifier for a loop.
    ///
    /// This label will later serve as the target for `break` and `continue`
    /// jumps during code generation.
    fn make_label(&mut self) -> Identifier {
        let s = format!("label_{}", self.label_counter);
        self.label_counter += 1;
        let symbol = self.sy_interner.intern(&s);
        Identifier::new(symbol, 0)
    }

    /// Returns the total number of labels generated so far.
    /// This is used by later compilation stages to avoid variable name conflicts.
    pub fn get_label_count(&self) -> usize {
        self.label_counter
    }

    /// Entry point for the loop labeling pass.
    ///
    /// Traverses all functions in the program, labeling loops and validating control flow.
    /// Returns a new, semantically valid AST with labels inserted.
    pub fn label_program(&mut self, program: Program) -> Result<Program, SemanticErr> {
        let declarations = program.into_parts();
        let mut labeled_declarations = Vec::new();
        for decl in declarations {
            // if it is avariable delcaration skip it
            // else label function declarations
            match decl {
                Declaration::VarDecl(_) => labeled_declarations.push(decl),
                Declaration::FunDecl(fun_decl) => labeled_declarations
                    .push(Declaration::FunDecl(self.label_function_decl(fun_decl)?)),
            }
        }
        Ok(Program::new(labeled_declarations))
    }

    /// Labels loops and control-flow statements within a single function.
    ///
    /// - Recursively labels all loops inside the function body.
    /// - Propagates `SemanticErr` if an invalid `break` or `continue` is found.
    fn label_function_decl(&mut self, function: FunctionDecl) -> Result<FunctionDecl, SemanticErr> {
        let (name, type_id, params, body, storage_class, span) = function.into_parts();
        let labeled_body = if let Some(block) = body {
            match self.label_block(block, None) {
                Ok(block) => Some(block),
                Err(err) => return Err(SemanticErr::new(err, &self.source_map)),
            }
        } else {
            None
        };
        Ok(FunctionDecl::new(
            name,
            type_id,
            params,
            labeled_body,
            storage_class,
            span,
        ))
    }

    /// Labels all loops and statements inside a block.
    ///
    /// # Parameters
    /// - `curr_loop`: The label of the nearest enclosing loop, if any.
    ///   Used to link `break` and `continue` to their correct targets.
    fn label_block(
        &mut self,
        block: Block,
        curr_loop: Option<Identifier>,
    ) -> Result<Block, ErrorType> {
        let (block_items, span) = block.into_parts();
        let mut labeled_block = Vec::new();

        for item in block_items {
            let labeled_item = self.label_block_item(item, curr_loop)?;
            labeled_block.push(labeled_item);
        }

        Ok(Block::new(labeled_block, span))
    }

    /// Labels a single block item (either a statement or a declaration).
    /// Declarations are left unchanged; statements are recursively labeled.
    fn label_block_item(
        &mut self,
        item: BlockItem,
        curr_loop: Option<Identifier>,
    ) -> Result<BlockItem, ErrorType> {
        if let BlockItem::S(stmt) = item {
            Ok(BlockItem::S(self.label_statement(stmt, curr_loop)?))
        } else {
            Ok(item)
        }
    }

    /// Recursively labels a statement and validates loop control usage.
    fn label_statement(
        &mut self,
        stmt: Statement,
        curr_loop: Option<Identifier>,
    ) -> Result<Statement, ErrorType> {
        let (stmt_type, span) = stmt.into_parts();
        let labeled_stmt_type = match stmt_type {
            // Validate that `break` appears inside a loop
            StatementType::Break(_) => self.label_break_statement(curr_loop, span)?,

            // Validate that `continue` appears inside a loop
            StatementType::Continue(_) => self.label_continue_statement(curr_loop, span)?,

            // Recurse into nested blocks
            StatementType::Compound(block) => {
                StatementType::Compound(self.label_block(block, curr_loop)?)
            }

            // Recurse into if/else structures
            StatementType::IfStatement {
                condition,
                if_clause,
                else_clause,
            } => self.label_if_statement(condition, *if_clause, else_clause, curr_loop)?,

            // Label loops and create a new `curr_loop` label for each one
            StatementType::While {
                condition,
                body,
                label: _,
            } => self.label_while_statement(condition, *body)?,
            StatementType::DoWhile {
                condition,
                body,
                label: _,
            } => self.label_do_while_statement(condition, *body)?,
            StatementType::For {
                init,
                condition,
                post,
                body,
                label: _,
            } => self.label_for_statement(init, condition, post, *body)?,

            // Non-control-flow statements are left unchanged
            _ => stmt_type,
        };

        Ok(Statement::new(labeled_stmt_type, span))
    }

    /// Recursively labels `if` statements and their branches.
    /// Carries `curr_loop` forward so inner breaks/continues remain valid.
    fn label_if_statement(
        &mut self,
        condition: Expression,
        if_clause: Statement,
        else_clause: Option<Box<Statement>>,
        curr_loop: Option<Identifier>,
    ) -> Result<StatementType, ErrorType> {
        let if_clause = Box::new(self.label_statement(if_clause, curr_loop)?);

        let else_clause = if let Some(stmt) = else_clause {
            Some(Box::new(self.label_statement(*stmt, curr_loop)?))
        } else {
            None
        };

        Ok(StatementType::IfStatement {
            condition,
            if_clause,
            else_clause,
        })
    }

    /// Labels a `while` loop with a new loop label.
    fn label_while_statement(
        &mut self,
        condition: Expression,
        body: Statement,
    ) -> Result<StatementType, ErrorType> {
        let label = self.make_label();
        let curr_loop = Some(label);
        let body = Box::new(self.label_statement(body, curr_loop)?);
        Ok(StatementType::While {
            condition,
            body,
            label,
        })
    }

    /// Labels a `do-while` loop with a new loop label.
    fn label_do_while_statement(
        &mut self,
        condition: Expression,
        body: Statement,
    ) -> Result<StatementType, ErrorType> {
        let label = self.make_label();
        let curr_loop = Some(label);
        let body = Box::new(self.label_statement(body, curr_loop)?);
        Ok(StatementType::DoWhile {
            condition,
            body,
            label,
        })
    }

    /// Labels a `for` loop, creating a new scope and label for it.
    fn label_for_statement(
        &mut self,
        init: ForInit,
        condition: Option<Expression>,
        post: Option<Expression>,
        body: Statement,
    ) -> Result<StatementType, ErrorType> {
        let label = self.make_label();
        let curr_loop = Some(label);
        let body = Box::new(self.label_statement(body, curr_loop)?);
        Ok(StatementType::For {
            init,
            condition,
            post,
            body,
            label,
        })
    }

    /// Handles and validates `break` statements.
    ///
    /// - If inside a loop, attaches the enclosing loop's label.
    /// - Otherwise, emits a `BreakErr`.
    fn label_break_statement(
        &mut self,
        curr_loop: Option<Identifier>,
        span: Span,
    ) -> Result<StatementType, ErrorType> {
        if let Some(label) = curr_loop {
            Ok(StatementType::Break(label))
        } else {
            Err(ErrorType::BreakErr(span))
        }
    }

    /// Handles and validates `continue` statements.
    ///
    /// - If inside a loop, attaches the enclosing loop's label.
    /// - Otherwise, emits a `ContinueErr`.
    fn label_continue_statement(
        &mut self,
        curr_loop: Option<Identifier>,
        span: Span,
    ) -> Result<StatementType, ErrorType> {
        if let Some(label) = curr_loop {
            Ok(StatementType::Continue(label))
        } else {
            Err(ErrorType::ContinueErr(span))
        }
    }
}
