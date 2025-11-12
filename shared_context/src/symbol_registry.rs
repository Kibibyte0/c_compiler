use crate::symbol_table::{EntryType, IdenAttrs, SymbolTable};
use crate::{Identifier, Type, type_interner::TypeID};
use std::collections::HashMap;

/// A registry of symbols split into distinct immutable tables by category:
/// - **Variables**
/// - **Functions**
///
/// This structure is built from a unified, mutable SymbolTable and provides
/// fast, infallible access to symbol data without repeated pattern matching.
pub struct SymbolRegistery {
    var_table: HashMap<Identifier, VarSy>,
    fun_table: HashMap<Identifier, FunSy>,
}

impl SymbolRegistery {
    /// Builds a new `SymbolRegistery` from a unified `SymbolTable.
    ///
    /// This consumes the symbol table and partitions it into separate,
    /// immutable maps for statics, locals, and functions.
    pub fn build(sytab: SymbolTable) -> Self {
        let mut var_table = HashMap::new();
        let mut fun_table = HashMap::new();

        // Decompose the symbol table into its distinct categories.
        for (iden, entry) in sytab.get_table().into_iter() {
            match (entry.entry_type, entry.attributes) {
                (EntryType::Scalar(ty), IdenAttrs::LocalAttrs) => {
                    let var_sy = VarSy::new(ty, false);
                    var_table.insert(iden, var_sy);
                }
                (EntryType::Scalar(ty), IdenAttrs::StaticAttrs { .. }) => {
                    let var_sy = VarSy::new(ty, true);
                    var_table.insert(iden, var_sy);
                }
                (
                    EntryType::Func(ty_id),
                    IdenAttrs::FunAttrs {
                        defined,
                        external: _,
                    },
                ) => {
                    let fun_sy = FunSy::new(ty_id, defined);
                    fun_table.insert(iden, fun_sy);
                }

                _ => (),
            }
        }

        Self {
            var_table,
            fun_table,
        }
    }

    /// Retrieves a function symbol entry by its `Identifier`.
    ///
    /// # Panics
    /// Panics if the identifier does not exist in the function table.
    pub fn get_function(&self, iden: &Identifier) -> &FunSy {
        &self.fun_table[iden]
    }

    /// Retrieves a static variable symbol entry by its `Identifier`.
    ///
    /// # Panics
    /// Panics if the identifier does not exist in the static variable table.
    pub fn get_variable(&self, iden: &Identifier) -> &VarSy {
        &self.var_table[iden]
    }
}

/// Symbol data for a **static variable**.
///
/// Contains its declared type, whether it’s external (`extern`),
/// and its initial value.
pub struct VarSy {
    ty: Type,
    is_static: bool,
}

impl VarSy {
    /// Creates a new `VarSy` entry.
    pub fn new(ty: Type, is_static: bool) -> Self {
        Self { ty, is_static }
    }

    /// Returns a reference to the type of this static variable.
    pub fn get_type(&self) -> Type {
        self.ty
    }

    pub fn is_static(&self) -> bool {
        self.is_static
    }
}

/// Symbol data for a **function**.
///
/// Includes its function type ID, and flags indicating whether the
/// function is external and whether it has a definition.
pub struct FunSy {
    ty_id: TypeID,
    def: bool,
}

impl FunSy {
    /// Creates a new `FunSy` entry.
    pub fn new(ty_id: TypeID, def: bool) -> Self {
        Self { ty_id, def }
    }

    /// Returns the function’s `TypeID`.
    pub fn get_type_id(&self) -> TypeID {
        self.ty_id
    }

    /// Returns whether this function has a definition in the current translation unit.
    pub fn is_def(&self) -> bool {
        self.def
    }
}
