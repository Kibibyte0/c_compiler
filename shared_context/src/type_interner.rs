use crate::Type;
use bumpalo::Bump;
use std::collections::HashMap;

/// A unique identifier for an interned function type.
///
/// This acts as a stable handle that refers to a specific
/// canonical `FunctionType`. Equality between function types
/// can be done by comparing these IDs directly, without
/// comparing parameter lists or return types.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct FuncTypeId(u32);

/// Represents a canonicalized function type.
///
/// Each `FunctionType` stores:
/// - a return type (`ret`)
/// - a slice of parameter types (`params`)
///
/// The parameter list is a reference to data allocated in a
/// `bumpalo::Bump` arena, ensuring immutability and stable memory.
///
/// Once interned, `FunctionType` values are never modified or dropped
/// individually — they live for the entire lifetime of the arena.
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct FunctionType<'a> {
    /// The return type of the function.
    pub ret: Type,
    /// A slice of parameter types (allocated in a bump arena).
    pub params: &'a [Type],
}

/// Stores all unique (interned) types within a compilation context.
///
/// This interner ensures that structurally identical types share a
/// single canonical representation in memory. For now, it only handles
/// function types, but it can be extended later to intern other type
/// forms such as structs, enums, pointers, and arrays.
///
/// The interner uses:
/// - A `HashMap` for deduplication (mapping from type → ID)
/// - A `Vec` for ID-to-type lookup
/// - A shared bump arena for fast, stable allocation
///
/// # Lifetimes
/// `'a` refers to the lifetime of the bump arena.
/// All interned types must live as long as the arena itself.
pub struct TypeInterner<'a> {
    /// The bump allocator used for storing immutable type data.
    arena: &'a Bump,
    /// Maps canonicalized `FunctionType`s to their assigned IDs.
    map: HashMap<FunctionType<'a>, FuncTypeId>,
    /// Stores all interned types; the index corresponds to the ID.
    types: Vec<&'a FunctionType<'a>>,
}

impl<'a> TypeInterner<'a> {
    /// Creates a new, empty type interner using the given arena.
    pub fn new(arena: &'a Bump) -> Self {
        Self {
            arena,
            map: HashMap::new(),
            types: Vec::new(),
        }
    }

    /// Interns a function type composed of `ret` and `params`.
    ///
    /// If an identical function type already exists, its existing ID
    /// is returned instead of creating a duplicate. Otherwise, the
    /// type and its parameter list are allocated in the bump arena.
    pub fn intern(&mut self, ret: Type, params: &[Type]) -> FuncTypeId {
        // Temporary key for deduplication
        let key = FunctionType { ret, params };

        // If this function type was already interned, return its ID
        if let Some(&id) = self.map.get(&key) {
            return id;
        }

        // Otherwise, copy the parameter list into the arena
        let params_copy = self.arena.alloc_slice_copy(params);

        // Allocate the new canonical FunctionType in the arena
        let fty = self.arena.alloc(FunctionType {
            ret,
            params: params_copy,
        });

        // Assign the next available ID
        let id = FuncTypeId(self.types.len() as u32);

        // Record the new type in the map and vector
        self.map.insert(
            FunctionType {
                ret,
                params: params_copy,
            },
            id,
        );
        self.types.push(fty);

        id
    }

    /// Retrieves the canonical [`FunctionType`] corresponding to a previously returned ID.
    ///
    /// # Panics
    /// Panics if the given ID does not correspond to a valid interned type.
    pub fn get(&self, id: FuncTypeId) -> &'a FunctionType<'a> {
        self.types[id.0 as usize]
    }
}
