use std::collections::HashMap;

use crate::{
    Identifier, OperandSize, convert_type_to_operand_size,
    symbol_table::{EntryType, IdenAttrs, SymbolTable},
};

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum AsmSymbolEntry {
    Obj { size: OperandSize, is_static: bool },
    Fun { is_defined: bool },
}

pub struct AsmSymbolTable {
    table: HashMap<Identifier, AsmSymbolEntry>,
}

impl AsmSymbolTable {
    pub fn new(symbol_table: &SymbolTable) -> Self {
        let mut table = HashMap::new();

        for (iden, entry) in symbol_table.get_table_ref().iter() {
            if let EntryType::Scalar(var_type) = entry.entry_type {
                table.insert(
                    *iden,
                    AsmSymbolEntry::Obj {
                        size: convert_type_to_operand_size(var_type),
                        is_static: entry.is_static(),
                    },
                );
            } else if let IdenAttrs::FunAttrs {
                defined,
                external: _,
            } = entry.attributes
            {
                table.insert(
                    *iden,
                    AsmSymbolEntry::Fun {
                        is_defined: defined,
                    },
                );
            }
        }

        Self { table }
    }

    /// this function paincs if there is no entry for a given identifier
    pub fn get(&self, key: Identifier) -> &AsmSymbolEntry {
        self.table.get(&key).unwrap()
    }
}
