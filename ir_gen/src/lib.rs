// IR (Intermediate Representation) generator.
//
// This module transforms the parsed AST into a lower-level
// intermediate representation. The IR, often called “Tacky”,
// is a flattened, instruction-based form that simplifies later optimization
// and code generation phases.

use parser::ast::{self, StorageClass};
use shared_context::{
    Identifier, StaticVariable,
    interner::Interner,
    symbol_table::{IdenAttrs, InitValue, SymbolTable},
};

use crate::tacky::TopLevel;

mod gen_expressions;
mod gen_statements;
pub mod print_ir;
pub mod tacky;

/// the enrty point for IR generation
/// Consumes an AST and produce a tacky program
pub fn lower_to_tacky(
    program: ast::Program,
    interner: &mut Interner,
    symbol_table: &SymbolTable,
    var_counter: usize,
) -> tacky::Program {
    let mut ir_gen = IRgen::new(var_counter, interner, symbol_table);
    let program_tacky = ir_gen.gen_tacky(program);
    program_tacky
}

/// Generates the compiler’s intermediate representation (IR)
/// from a high-level abstract syntax tree (AST).
///
/// The IRgen struct manages temporary variable allocation,
/// label generation, and identifier interning during translation.
/// It converts parsed program structures into tacky::Programs.
pub struct IRgen<'src, 'ctx> {
    /// Counter used to generate unique temporary variables and labels.
    var_counter: usize,

    /// Reference to the string interner used to manage symbol deduplication.
    interner: &'ctx mut Interner<'src>,
    symbol_table: &'ctx SymbolTable,
}

impl<'src, 'ctx> IRgen<'src, 'ctx> {
    /// Creates a new IR generator instance.
    pub fn new(
        var_counter: usize,
        interner: &'ctx mut Interner<'src>,
        symbol_table: &'ctx SymbolTable,
    ) -> Self {
        Self {
            var_counter,
            interner,
            symbol_table,
        }
    }

    /// Creates a new temporary variable (e.g., `tmp.0`, `tmp.1`, …)
    /// and returns it as a tacky::Value::Var.
    ///
    /// Used to hold intermediate computation results during expression lowering.
    fn make_temp_var(&mut self) -> tacky::Value {
        let s = format!("tmp.{}", self.var_counter);
        self.var_counter += 1;
        let symbol = self.interner.intern(&s);
        let temp_id = Identifier::new(symbol, 0);
        tacky::Value::Var(temp_id)
    }

    /// Generates a unique label identifier (e.g., `label_0`, `label_1`, …).
    ///
    /// Used for control flow constructs such as loops and conditionals.
    fn make_label(&mut self) -> Identifier {
        let s = format!("label_{}", self.var_counter);
        self.var_counter += 1;
        let symbol = self.interner.intern(&s);
        Identifier::new(symbol, 0)
    }

    /// Converts a label into a corresponding “break” label.
    ///
    /// For example, `label_3` becomes `label_3_break`.
    fn convert_to_break_label(&mut self, label: Identifier) -> Identifier {
        let symbol = label.get_symbol();
        let s = format!("{}_break", self.interner.lookup(symbol));
        Identifier::new(self.interner.intern(&s), 0)
    }

    /// Converts a label into a corresponding “continue” label.
    ///
    /// For example, `label_3` becomes `label_3_continue`.
    fn convert_to_continue_label(&mut self, label: Identifier) -> Identifier {
        let symbol = label.get_symbol();
        let s = format!("{}_continue", self.interner.lookup(symbol));
        Identifier::new(self.interner.intern(&s), 0)
    }

    /// Translates a parsed program ast::Program into its IR form tacky::Program.
    ///
    /// Each function definition in the AST is lowered into a corresponding
    /// tacky::FunctionDef. Function declarations (without bodies) are ignored.
    pub fn gen_tacky(&mut self, program: ast::Program) -> tacky::Program {
        let declarations = program.into_parts();
        let mut tacky_items = Vec::new();

        // generate function defintions
        for decl in declarations {
            match decl {
                ast::Declaration::FunDecl(fun_decl) => {
                    if let Some(tacky_function) = self.gen_function_def(fun_decl) {
                        tacky_items.push(tacky::TopLevel::F(tacky_function));
                    }
                }
                // skip file scope variable declarations
                ast::Declaration::VarDecl(_) => continue,
            }
        }

        // generate static variables defintions
        self.gen_static_variable_defintions(&mut tacky_items);
        tacky::Program::new(tacky_items)
    }

    /// generate static variables defintions using the symbol table
    ///
    /// each static variable in the symbol table is lowerd to its corresponding tacky::StaticVariable
    /// other entries in the table are ignored (local variabels and function declarations)
    fn gen_static_variable_defintions(&self, tacky_items: &mut Vec<TopLevel>) {
        for (iden, entry) in self.symbol_table.get_table_ref().iter() {
            match entry.attributes {
                IdenAttrs::StaticAttrs {
                    init_value,
                    external,
                } => match init_value {
                    InitValue::Initial(int) => {
                        tacky_items.push(TopLevel::S(StaticVariable::new(*iden, external, int)))
                    }
                    InitValue::Tentative => {
                        tacky_items.push(TopLevel::S(StaticVariable::new(*iden, external, 0)))
                    }
                    // skip unintialized variables
                    InitValue::NoInitializer => continue,
                },
                // skip local variables and function declarations
                _ => continue,
            }
        }
    }

    /// Generates IR for a single function definition.
    ///
    /// Returns `None` if the given AST node represents only a function declaration.
    fn gen_function_def(&mut self, function: ast::FunctionDecl) -> Option<tacky::FunctionDef> {
        let (name, params, body, _, _) = function.into_parts();

        match body {
            Some(block) => {
                // Convert parameter identifiers
                let tacky_params = params
                    .iter()
                    .map(|sp_iden| sp_iden.get_identifier())
                    .collect();

                let mut instructions = Vec::new();
                self.gen_function_block(block, &mut instructions);
                let identifier = name.get_identifier();

                Some(tacky::FunctionDef::new(
                    identifier,
                    self.get_function_linkage(identifier),
                    tacky_params,
                    instructions,
                ))
            }
            None => None, // Skip pure declarations
        }
    }

    /// get the linkage of the function definition
    fn get_function_linkage(&self, iden: Identifier) -> bool {
        // every defined function is gaurnteed to be in the symbol table at this point.
        self.symbol_table
            .get(iden)
            .unwrap()
            .attributes
            .is_external()
    }

    /// Generates a full function body block.
    ///
    /// Automatically appends a `return 0` instruction at the end if
    /// no explicit return statement is encountered.
    fn gen_function_block(
        &mut self,
        block: ast::Block,
        instructions: &mut Vec<tacky::Instruction>,
    ) {
        let (block_items, _) = block.into_parts();
        for item in block_items {
            self.gen_block_item(item, instructions);
        }

        // Default return for functions without explicit `return` statements
        instructions.push(tacky::Instruction::Ret(tacky::Value::Constant(0)));
    }

    /// Generates IR for a standard block (without adding implicit returns).
    fn gen_block(&mut self, block: ast::Block, instructions: &mut Vec<tacky::Instruction>) {
        let (block_items, _) = block.into_parts();
        for item in block_items {
            self.gen_block_item(item, instructions);
        }
    }

    /// Dispatches the generation of a block item (declaration or statement).
    fn gen_block_item(&mut self, item: ast::BlockItem, instructions: &mut Vec<tacky::Instruction>) {
        match item {
            ast::BlockItem::D(sp_decl) => self.gen_declaration(sp_decl, instructions),
            ast::BlockItem::S(sp_stmt) => self.gen_statements(sp_stmt, instructions),
        }
    }

    /// Handles variable or function declarations inside a block.
    ///
    /// Function declarations are ignored at this stage; variable declarations
    /// are lowered to IR `Copy` instructions when initialized.
    fn gen_declaration(
        &mut self,
        decl: ast::Declaration,
        instructions: &mut Vec<tacky::Instruction>,
    ) {
        match decl {
            ast::Declaration::FunDecl(_) => return, // Skip pure declarations
            ast::Declaration::VarDecl(var_decl) => {
                self.gen_variable_declaration(var_decl, instructions)
            }
        }
    }

    /// Lowers a variable declaration into IR form.
    ///
    /// If the variable includes an initializer, a `Copy` instruction is generated:
    /// Uninitialized variables are ignored (discarded).
    fn gen_variable_declaration(
        &mut self,
        var_decl: ast::VariableDecl,
        instructions: &mut Vec<tacky::Instruction>,
    ) {
        let (name, init, storage_class, _) = var_decl.into_parts();

        // if a variable is declared with static keyword, skip it
        // static variable definitions will be handled separatly
        if storage_class == StorageClass::Static {
            return;
        }

        match init {
            Some(init) => {
                // Lower the initializer expression
                let value = self.gen_expression(init, instructions);

                // Emit IR assignment
                let instr = tacky::Instruction::Copy {
                    src: value,
                    dst: tacky::Value::Var(name.get_identifier()),
                };
                instructions.push(instr);
            }
            None => return, // No initializer: skip emission
        }
    }
}
