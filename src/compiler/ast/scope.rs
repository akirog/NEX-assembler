use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use crate::compiler::*;

/// The scope of a specific block.
pub struct Scope {
    /// String to symbol idx
    pub symbol_names: HashMap<String, SymbolId>,

    /// Index in the scopes vec with parent scope
    pub parent: Option<ScopeId>,

    /// The base stack offset of this scope
    /// For function frames this is always 0
    pub base_offset: usize,

    /// The size of this scope
    pub size: usize,

    /// Name used for debug purposes
    pub debug_name: String,
}

impl Scope {
    fn default() -> Scope {
        Scope {
            symbol_names: HashMap::new(),
            parent: None,
            base_offset: 0,
            size: 0,
            debug_name: String::new(),
        }
    }
}

impl Display for Scope {
    fn fmt(&self, f: &mut Formatter) -> Result<(), std::fmt::Error> {
        write!(f, "scope of |{}|\n", self.debug_name)?;
        write!(f, "Parent: ")?;
        if let Some(parent) = &self.parent {
            write!(f, "{:?}\n", parent)?;
        } else {
            write!(f, "Global\n")?;
        };

        write!(f, "base offset: {}\n", self.base_offset)?;
        write!(f, "size: {}\n", self.size)?;

        for id in self.symbol_names.values() {
            write!(f, "{:?}\n", id)?;
        }
        write!(f, "\n")
    }
}

/// The symbol of a variable
#[derive(Hash, Eq, PartialEq, Copy, Clone, Debug)]
pub struct SymbolId(uid::Id<u16>);


#[derive(Debug)]
pub struct Symbol {
    pub name: String,
    pub var_type: ResolvedType,
    pub address: SymbolAddress,
}

impl Display for Symbol {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let ty = self.var_type.to_string();
        write!(f, "{}:\n\ttype: {}\n\t@ {:>20}", self.name, ty, self.address)
    }
}


#[derive(Debug)]
pub enum SymbolAddress {
    StackOffset(usize),
    GlobalVar(String),
    FunctionLabel(String),
}

impl Display for SymbolAddress {
    fn fmt(&self, f: &mut Formatter) -> Result<(), std::fmt::Error> {
        match self {
            SymbolAddress::StackOffset(offset) => write!(f, "[bp - {}]", offset),
            SymbolAddress::GlobalVar(name) => write!(f, "global: {}", name),
            SymbolAddress::FunctionLabel(name) => write!(f, "function: {}", name),
        }
    }
}


#[derive(Debug, Copy, Clone, Eq, Hash, PartialEq)]
pub struct ScopeId(uid::Id<u16>);

pub struct ScopeHandler {
    global_scope: ScopeId,
    pub scopes: HashMap<ScopeId, Scope>,
    curr_scope: ScopeId,
    block_scopes: HashMap<NodeId, ScopeId>,
    symbols: HashMap<SymbolId, Symbol>,
}


impl ScopeHandler {
    pub fn new() -> ScopeHandler {
        let mut scopes = HashMap::new();
        let global_id = ScopeId(uid::Id::new());
        scopes.insert(global_id,
            Scope {
                parent: None,
                debug_name: "Global scope".to_string(),
                ..Scope::default()
            }
        );

        ScopeHandler {
            global_scope: global_id,
            scopes,
            curr_scope: global_id,
            block_scopes: HashMap::new(),
            symbols: HashMap::new(),
        }
    }

    pub fn register_global(&mut self, name: String, var_type: ResolvedType, address: SymbolAddress) {
        assert!(matches!(address, SymbolAddress::GlobalVar(_)) || matches!(address, SymbolAddress::FunctionLabel(_)), "Compiler error, global var address is not a global var label or function label");

        let id = SymbolId(uid::Id::new());
        self.scopes.get_mut(&self.global_scope).unwrap().register_symbol(name.clone(), id);

        self.symbols.insert(
            id,
            Symbol {
                name,
                var_type,
                address,
            }
        );
    }

    pub fn register_symbol(&mut self, name: String, var_type: ResolvedType, address: SymbolAddress) {
        let curr_scope = self.get_curr_scope_mut();

        let id = SymbolId(uid::Id::new());
        curr_scope.register_symbol(name.clone(), id);

        self.symbols.insert(
            id,
            Symbol {
                name,
                var_type,
                address,
            }
        );
    }

    pub fn make_new_scope(&mut self, block_id: NodeId) -> ScopeId {
        let mut new_scope = Scope::default();
        new_scope.parent = Some(self.curr_scope);

        let id = ScopeId(uid::Id::new());
        self.scopes.insert(id, new_scope);
        
        self.curr_scope = id;

        self.block_scopes.insert(block_id, self.curr_scope);

        id
    }

    pub fn enter_scope(&mut self, block_id: NodeId) {
        if let Some(scope) = self.block_scopes.get(&block_id) {
            self.curr_scope = *scope;
            return;
        }
        panic!("Scope not found.");
    }
    
    pub fn pop_scope(&mut self) {
        self.curr_scope = self.get_curr_scope().parent.unwrap();
    }

    fn get_scope(&self, id: ScopeId) -> &Scope {
        self.scopes.get(&id).unwrap()
    }

    pub fn get_curr_scope(&self) -> &Scope {
        self.get_scope(self.curr_scope)
    }

    pub fn get_curr_scope_mut(&mut self) -> &mut Scope {
        self.scopes.get_mut(&self.curr_scope).unwrap()
    }

    pub fn lookup_symbol(&mut self, name: &str) -> Option<SymbolId> {
        let mut scope = self.get_curr_scope();
        loop {
            let id = scope.symbol_names.get(name).cloned();
            if let Some(id) = id {
                return Some(id);
            }
            scope = self.get_scope(scope.parent?);
        }
    }

    pub fn get_symbol(&self, id: SymbolId) -> Option<&Symbol> {
        self.symbols.get(&id)
    }
}


impl Display for ScopeHandler {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ScopeHandler:\n")?;
        write!(f, "GlobalScope:\n")?;
        write!(f, "{:?}\n", self.global_scope)?;
        write!(f, "Scopes:\n")?;
        for (i, scope) in &self.scopes {
            write!(f, "Scope: {:?}\n{}", i, scope)?;
        }
        write!(f, "\n")
    }
}


impl Scope {
    fn register_symbol(&mut self, name: String, id: SymbolId) {
        self.symbol_names.insert(name.clone(), id);
    }
}