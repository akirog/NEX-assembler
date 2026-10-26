use std::collections::HashMap;
use std::fmt::{write, Display, Formatter};
use compiler::ast::types::ResolvedType;
use compiler::semantic_analyzer::scope_builder::ScopeBuilder;

/// The scope of a specific block.
pub struct Scope {
    /// String to symbol idx
    pub symbol_names: HashMap<String, usize>,

    /// All symbols in this scope
    pub symbols: Vec<Symbol>,

    /// Index in the scopes vec with parent scope
    pub parent: Option<usize>,

    /// The base stack offset of this scope
    /// For function frames this is always 0
    pub base_offset: usize,

    /// The size of this scope
    pub size: usize,
}

impl Scope {
    fn default() -> Scope {
        Scope {
            symbol_names: HashMap::new(),
            symbols: Vec::new(),
            parent: None,
            base_offset: 0,
            size: 0,
        }
    }
}

impl Display for Scope {
    fn fmt(&self, f: &mut Formatter) -> Result<(), std::fmt::Error> {
        write!(f, "Parent: ")?;
        if let Some(parent) = self.parent {
            write!(f, "{}\n", parent)?;
        } else {
            write!(f, "Global\n")?;
        }

        for idx in self.symbol_names.values() {
            write!(f, "{}\n", self.symbols.get(*idx).unwrap())?;
        }
        write!(f, "\n")
    }
}

/// The symbol of a variable
pub struct Symbol {
    pub name: String,
    pub var_type: ResolvedType,
    pub address: SymbolAddress,
}

impl Display for Symbol {
    fn fmt(&self, f: &mut Formatter) -> Result<(), std::fmt::Error> {
        write!(f, "{}:\t{}\t@ {}", self.name, self.var_type, self.address)
    }
}


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


pub struct ScopeHandler {
    global_scope: Scope,
    pub scopes: Vec<Scope>,
    curr_scope: Option<usize>,
}


impl ScopeHandler {
    pub fn new() -> ScopeHandler {
        ScopeHandler {
            global_scope: Scope::default(),
            scopes: Vec::new(),
            curr_scope: None,
        }
    }

    pub fn register_global(&mut self, name: String, var_type: ResolvedType) {
        self.global_scope.register_symbol(name, var_type);
    }

    pub fn register_symbol(&mut self, name: String, var_type: ResolvedType) {
        let curr_scope = self.get_curr_scope();

        curr_scope.register_symbol(name, var_type);
    }

    pub fn make_new_scope(&mut self) {
        let mut new_scope = Scope::default();
        new_scope.parent = self.curr_scope;
        self.scopes.push(new_scope);
        self.curr_scope = Some(self.scopes.len() - 1);
    }

    pub fn pop_scope(&mut self) {
        self.curr_scope = self.get_curr_scope().parent
    }

    pub fn get_curr_scope(&mut self) -> &mut Scope {
        if let Some(index) = self.curr_scope {
            self.scopes.get_mut(index).unwrap()
        } else {
            &mut self.global_scope
        }
    }
}


impl Display for ScopeHandler {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ScopeHandler:\n")?;
        write!(f, "GlobalScope:\n")?;
        write!(f, "{}\n", self.global_scope)?;
        write!(f, "Scopes:\n")?;
        for (i, scope) in self.scopes.iter().enumerate() {
            write!(f, "Scope: {i}\n{}", scope)?;
        }
        write!(f, "\n")
    }
}


impl Scope {
    fn register_symbol(&mut self, name: String, var_type: ResolvedType) {
        self.symbol_names.insert(name.clone(), self.symbols.len());
        self.symbols.push(Symbol {
            name,
            var_type,
            address: SymbolAddress::StackOffset(0), // todo! Actually implement address thing
        })
    }
}