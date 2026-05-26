use crate::sema::types::TypeInfo;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum Symbol {
    Variable {
        type_: TypeInfo,
        mutable: bool,
        is_const: bool,
    },
    Function(FnSymbol),
    StructType(StructSymbol),
    EnumType(EnumSymbol),
    UnionType(UnionSymbol),
    ErrorSet(ErrorSetSymbol),
    Behave(BehaveSymbol),
    TypeAlias(TypeInfo),
    Parameter {
        type_: TypeInfo,
        mutable: bool,
    },
    BuiltinFn {
        name: String,
        param_count: (usize, Option<usize>),
    },
}

#[derive(Debug, Clone)]
pub struct FnSymbol {
    pub name: String,
    pub generics: Vec<String>,
    pub params: Vec<(String, bool, TypeInfo)>,
    pub return_: Option<TypeInfo>,
    pub is_comptime: bool,
}

#[derive(Debug, Clone)]
pub struct StructSymbol {
    pub name: String,
    pub generics: Vec<String>,
    pub fields: Vec<(String, TypeInfo, bool)>,
    pub methods: Vec<FnSymbol>,
    pub impl_behave: Option<String>,
}

#[derive(Debug, Clone)]
pub struct EnumSymbol {
    pub name: String,
    pub generics: Vec<String>,
    pub variants: Vec<(String, Option<TypeInfo>)>,
    pub methods: Vec<FnSymbol>,
    pub impl_behave: Option<String>,
}

#[derive(Debug, Clone)]
pub struct UnionSymbol {
    pub name: String,
    pub generics: Vec<String>,
    pub variants: Vec<(String, TypeInfo)>,
}

#[derive(Debug, Clone)]
pub struct ErrorSetSymbol {
    pub name: String,
    pub variants: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct BehaveSymbol {
    pub name: String,
    pub generics: Vec<String>,
    pub methods: Vec<FnSymbol>,
}

impl Symbol {
    pub fn get_type(&self) -> Option<TypeInfo> {
        match self {
            Symbol::Variable { type_, .. } => Some(type_.clone()),
            Symbol::Parameter { type_, .. } => Some(type_.clone()),
            Symbol::Function(f) => {
                let params = f.params.iter().map(|(_, _, t)| t.clone()).collect();
                let ret = f.return_.clone().unwrap_or(TypeInfo::Void);
                Some(TypeInfo::Fn(params, Box::new(ret)))
            }
            Symbol::StructType(s) => Some(TypeInfo::Struct(s.name.clone(), s.fields.clone())),
            Symbol::EnumType(e) => Some(TypeInfo::Enum(e.name.clone(), e.variants.clone())),
            Symbol::UnionType(u) => Some(TypeInfo::Union(u.name.clone(), u.variants.clone())),
            Symbol::ErrorSet(e) => Some(TypeInfo::ErrorSet(e.name.clone(), e.variants.clone())),
            Symbol::TypeAlias(t) => Some(t.clone()),
            Symbol::Behave(_) => None,
            Symbol::BuiltinFn { .. } => None,
        }
    }

    pub fn is_mutable(&self) -> bool {
        matches!(
            self,
            Symbol::Variable { mutable: true, .. } | Symbol::Parameter { mutable: true, .. }
        )
    }

    pub fn is_const(&self) -> bool {
        matches!(self, Symbol::Variable { is_const: true, .. })
    }

    pub fn is_function(&self) -> bool {
        matches!(self, Symbol::Function(_) | Symbol::BuiltinFn { .. })
    }
}

#[derive(Debug, Clone)]
struct Scope {
    symbols: HashMap<String, Symbol>,
    parent: Option<usize>,
}

impl Scope {
    fn new(parent: Option<usize>) -> Self {
        Scope {
            symbols: HashMap::new(),
            parent,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SymbolTable {
    scopes: Vec<Scope>,
    current: usize,
}

impl SymbolTable {
    pub fn new() -> Self {
        SymbolTable {
            scopes: vec![Scope::new(None)],
            current: 0,
        }
    }

    pub fn push_scope(&mut self) {
        let new_idx = self.scopes.len();
        self.scopes.push(Scope::new(Some(self.current)));
        self.current = new_idx;
    }

    pub fn pop_scope(&mut self) {
        if let Some(parent) = self.scopes[self.current].parent {
            self.current = parent;
        }
    }

    pub fn insert(&mut self, name: &str, symbol: Symbol) -> Result<(), String> {
        if self.scopes[self.current].symbols.contains_key(name) {
            return Err(format!(
                "Duplicate declaration: symbol '{}' is already defined in this scope",
                name
            ));
        }
        self.scopes[self.current]
            .symbols
            .insert(name.to_string(), symbol);
        Ok(())
    }

    pub fn insert_overwrite(&mut self, name: &str, symbol: Symbol) {
        self.scopes[self.current]
            .symbols
            .insert(name.to_string(), symbol);
    }

    pub fn lookup(&self, name: &str) -> Option<&Symbol> {
        let mut idx = Some(self.current);
        while let Some(i) = idx {
            if let Some(sym) = self.scopes[i].symbols.get(name) {
                return Some(sym);
            }
            idx = self.scopes[i].parent;
        }
        None
    }

    pub fn lookup_in_current(&self, name: &str) -> Option<&Symbol> {
        self.scopes[self.current].symbols.get(name)
    }

    pub fn lookup_type(&self, name: &str) -> Option<TypeInfo> {
        self.lookup(name).and_then(|s| s.get_type())
    }
}
