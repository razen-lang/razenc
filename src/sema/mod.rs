pub mod checker;
pub mod scope;
pub mod types;

use crate::ast::*;
use crate::sema::checker::*;
use crate::sema::scope::*;
use crate::sema::types::*;

pub struct SemanticAnalyzer {
    pub table: SymbolTable,
    pub checker: TypeChecker,
}

impl SemanticAnalyzer {
    pub fn new() -> Self {
        let mut table = SymbolTable::new();
        register_builtins(&mut table);
        SemanticAnalyzer {
            table,
            checker: TypeChecker::new(),
        }
    }

    pub fn analyze(&mut self, program: &Program) -> Result<(), Vec<SemanticError>> {
        for decl in &program.decls {
            if let Err(e) = self.register_decl(decl, false) {
                self.checker.errors.push(SemanticError::new("SEMA-0003", e));
            }
        }

        for decl in &program.decls {
            self.analyze_decl_bodies(decl);
        }

        if self.checker.has_errors() {
            Err(self.checker.errors.clone())
        } else {
            Ok(())
        }
    }

    fn register_decl(&mut self, decl: &Decl, _nested: bool) -> Result<(), String> {
        match decl {
            Decl::Use(path) => self.table.insert(
                &path.join("."),
                Symbol::Variable {
                    type_: TypeInfo::Void,
                    mutable: false,
                    is_const: true,
                },
            ),
            Decl::Mod(name, body) => {
                self.table.push_scope();
                if let Some(decls) = body {
                    for d in decls {
                        self.register_decl(d, true)?;
                    }
                }
                self.table.pop_scope();
                self.table.insert(
                    name,
                    Symbol::Variable {
                        type_: TypeInfo::Void,
                        mutable: false,
                        is_const: true,
                    },
                )
            }
            Decl::Fn(f) => {
                let params: Vec<(String, bool, TypeInfo)> = f
                    .params
                    .iter()
                    .map(|p| {
                        let resolve = |n: &str| self.table.lookup_type(n);
                        (
                            p.name.clone(),
                            p.mutable,
                            resolve_ast_type(&p.type_, &resolve).unwrap_or(TypeInfo::Void),
                        )
                    })
                    .collect();
                let return_ = f.return_.as_ref().and_then(|rt| {
                    let resolve = |n: &str| self.table.lookup_type(n);
                    resolve_ast_type(rt, &resolve).ok()
                });
                let fs = FnSymbol {
                    name: f.name.clone(),
                    generics: f.generics.clone(),
                    params,
                    return_,
                    is_comptime: true,
                };
                if self.table.lookup_in_current(&f.name).is_some() {
                    return Err(format!(
                        "Duplicate declaration: symbol '{}' is already defined in this scope",
                        f.name
                    ));
                }
                self.table.insert(&f.name, Symbol::Function(fs))
            }
            Decl::Struct(s) => {
                let fields: Vec<(String, TypeInfo, bool)> = s
                    .fields
                    .iter()
                    .map(|f| {
                        let resolve = |n: &str| self.table.lookup_type(n);
                        (
                            f.name.clone(),
                            resolve_ast_type(&f.type_, &resolve).unwrap_or(TypeInfo::Void),
                            f.pub_,
                        )
                    })
                    .collect();
                let methods: Vec<FnSymbol> = s
                    .methods
                    .iter()
                    .map(|m| {
                        let params: Vec<(String, bool, TypeInfo)> = m
                            .params
                            .iter()
                            .map(|p| {
                                let resolve = |n: &str| self.table.lookup_type(n);
                                (
                                    p.name.clone(),
                                    p.mutable,
                                    resolve_ast_type(&p.type_, &resolve).unwrap_or(TypeInfo::Void),
                                )
                            })
                            .collect();
                        let ret = m.return_.as_ref().and_then(|rt| {
                            let resolve = |n: &str| self.table.lookup_type(n);
                            resolve_ast_type(rt, &resolve).ok()
                        });
                        FnSymbol {
                            name: m.name.clone(),
                            generics: m.generics.clone(),
                            params,
                            return_: ret,
                            is_comptime: true,
                        }
                    })
                    .collect();
                let impl_behave = s.impl_behave.clone();
                let ss = StructSymbol {
                    name: s.name.clone(),
                    generics: s.generics.clone(),
                    fields,
                    methods,
                    impl_behave: impl_behave.clone(),
                };
                if self.table.lookup_in_current(&s.name).is_some() {
                    return Err(format!(
                        "Duplicate declaration: symbol '{}' is already defined in this scope",
                        s.name
                    ));
                }
                self.table.insert(&s.name, Symbol::StructType(ss))?;
                if let Some(ref bhv) = impl_behave {
                    let ms: Vec<FnSymbol> = s
                        .methods
                        .iter()
                        .map(|m| {
                            let params: Vec<(String, bool, TypeInfo)> = m
                                .params
                                .iter()
                                .map(|p| {
                                    let resolve = |n: &str| self.table.lookup_type(n);
                                    (
                                        p.name.clone(),
                                        p.mutable,
                                        resolve_ast_type(&p.type_, &resolve)
                                            .unwrap_or(TypeInfo::Void),
                                    )
                                })
                                .collect();
                            let ret = m.return_.as_ref().and_then(|rt| {
                                let resolve = |n: &str| self.table.lookup_type(n);
                                resolve_ast_type(rt, &resolve).ok()
                            });
                            FnSymbol {
                                name: m.name.clone(),
                                generics: m.generics.clone(),
                                params,
                                return_: ret,
                                is_comptime: true,
                            }
                        })
                        .collect();
                    self.checker
                        .check_behave_impl(&s.name, bhv, &ms, &self.table);
                }
                Ok(())
            }
            Decl::Enum(e) => {
                let variants: Vec<(String, Option<TypeInfo>)> = e
                    .variants
                    .iter()
                    .map(|v| {
                        let vt = v.type_.as_ref().and_then(|t| {
                            let resolve = |n: &str| self.table.lookup_type(n);
                            resolve_ast_type(t, &resolve).ok()
                        });
                        (v.name.clone(), vt)
                    })
                    .collect();
                let methods: Vec<FnSymbol> = e
                    .methods
                    .iter()
                    .map(|m| {
                        let params: Vec<(String, bool, TypeInfo)> = m
                            .params
                            .iter()
                            .map(|p| {
                                let resolve = |n: &str| self.table.lookup_type(n);
                                (
                                    p.name.clone(),
                                    p.mutable,
                                    resolve_ast_type(&p.type_, &resolve).unwrap_or(TypeInfo::Void),
                                )
                            })
                            .collect();
                        let ret = m.return_.as_ref().and_then(|rt| {
                            let resolve = |n: &str| self.table.lookup_type(n);
                            resolve_ast_type(rt, &resolve).ok()
                        });
                        FnSymbol {
                            name: m.name.clone(),
                            generics: m.generics.clone(),
                            params,
                            return_: ret,
                            is_comptime: true,
                        }
                    })
                    .collect();
                let es = EnumSymbol {
                    name: e.name.clone(),
                    generics: e.generics.clone(),
                    variants,
                    methods,
                    impl_behave: e.impl_behave.clone(),
                };
                if self.table.lookup_in_current(&e.name).is_some() {
                    return Err(format!(
                        "Duplicate declaration: symbol '{}' is already defined in this scope",
                        e.name
                    ));
                }
                self.table.insert(&e.name, Symbol::EnumType(es))
            }
            Decl::Union(u) => {
                let variants: Vec<(String, TypeInfo)> = u
                    .variants
                    .iter()
                    .map(|v| {
                        let resolve = |n: &str| self.table.lookup_type(n);
                        (
                            v.name.clone(),
                            resolve_ast_type(&v.type_, &resolve).unwrap_or(TypeInfo::Void),
                        )
                    })
                    .collect();
                let us = UnionSymbol {
                    name: u.name.clone(),
                    generics: u.generics.clone(),
                    variants,
                };
                if self.table.lookup_in_current(&u.name).is_some() {
                    return Err(format!(
                        "Duplicate declaration: symbol '{}' is already defined in this scope",
                        u.name
                    ));
                }
                self.table.insert(&u.name, Symbol::UnionType(us))
            }
            Decl::Error_(name, variants) => {
                let vnames: Vec<String> = variants.iter().map(|v| v.name.clone()).collect();
                if self.table.lookup_in_current(name).is_some() {
                    return Err(format!(
                        "Duplicate declaration: symbol '{}' is already defined in this scope",
                        name
                    ));
                }
                self.table.insert(
                    name,
                    Symbol::ErrorSet(ErrorSetSymbol {
                        name: name.clone(),
                        variants: vnames,
                    }),
                )
            }
            Decl::Behave(b) => {
                let methods: Vec<FnSymbol> = b
                    .methods
                    .iter()
                    .map(|m| {
                        let params: Vec<(String, bool, TypeInfo)> = m
                            .params
                            .iter()
                            .map(|p| {
                                let resolve = |n: &str| self.table.lookup_type(n);
                                (
                                    p.name.clone(),
                                    p.mutable,
                                    resolve_ast_type(&p.type_, &resolve).unwrap_or(TypeInfo::Void),
                                )
                            })
                            .collect();
                        let ret = m.return_.as_ref().and_then(|rt| {
                            let resolve = |n: &str| self.table.lookup_type(n);
                            resolve_ast_type(rt, &resolve).ok()
                        });
                        FnSymbol {
                            name: m.name.clone(),
                            generics: m.generics.clone(),
                            params,
                            return_: ret,
                            is_comptime: true,
                        }
                    })
                    .collect();
                let bs = BehaveSymbol {
                    name: b.name.clone(),
                    generics: b.generics.clone(),
                    methods,
                };
                if self.table.lookup_in_current(&b.name).is_some() {
                    return Err(format!(
                        "Duplicate declaration: symbol '{}' is already defined in this scope",
                        b.name
                    ));
                }
                self.table.insert(&b.name, Symbol::Behave(bs))
            }
            Decl::Var(v) => {
                let var_type = v
                    .type_
                    .as_ref()
                    .and_then(|t| {
                        let resolve = |n: &str| self.table.lookup_type(n);
                        resolve_ast_type(t, &resolve).ok()
                    })
                    .unwrap_or(TypeInfo::Void);
                let sym = Symbol::Variable {
                    type_: var_type,
                    mutable: v.mutable,
                    is_const: false,
                };
                if self.table.lookup_in_current(&v.name).is_some() {
                    return Err(format!(
                        "Duplicate declaration: symbol '{}' is already defined in this scope",
                        v.name
                    ));
                }
                self.table.insert(&v.name, sym)
            }
            Decl::Const(c) => {
                let const_type = c
                    .type_
                    .as_ref()
                    .and_then(|t| {
                        let resolve = |n: &str| self.table.lookup_type(n);
                        resolve_ast_type(t, &resolve).ok()
                    })
                    .unwrap_or(TypeInfo::Void);
                let sym = Symbol::Variable {
                    type_: const_type,
                    mutable: false,
                    is_const: true,
                };
                if self.table.lookup_in_current(&c.name).is_some() {
                    return Err(format!(
                        "Duplicate declaration: symbol '{}' is already defined in this scope",
                        c.name
                    ));
                }
                self.table.insert(&c.name, sym)
            }
            Decl::TypeAlias(name, type_) => {
                let resolve = |n: &str| self.table.lookup_type(n);
                let t = resolve_ast_type(type_, &resolve).unwrap_or(TypeInfo::Void);
                if self.table.lookup_in_current(name).is_some() {
                    return Err(format!(
                        "Duplicate declaration: symbol '{}' is already defined in this scope",
                        name
                    ));
                }
                self.table.insert(name, Symbol::TypeAlias(t))
            }
            Decl::Test(name, _block) => {
                if self.table.lookup_in_current(name).is_some() {
                    return Err(format!(
                        "Duplicate declaration: symbol '{}' is already defined in this scope",
                        name
                    ));
                }
                self.table.insert(
                    name,
                    Symbol::Variable {
                        type_: TypeInfo::Void,
                        mutable: false,
                        is_const: true,
                    },
                )
            }
        }
    }

    fn analyze_decl_bodies(&mut self, decl: &Decl) {
        match decl {
            Decl::Fn(f) => {
                if let Some(ref body) = f.body {
                    let resolve = |n: &str| self.table.lookup_type(n);
                    let ret_type = f
                        .return_
                        .as_ref()
                        .and_then(|rt| resolve_ast_type(rt, &resolve).ok());
                    let params: Vec<(String, bool, TypeInfo)> = f
                        .params
                        .iter()
                        .map(|p| {
                            let resolve = |n: &str| self.table.lookup_type(n);
                            (
                                p.name.clone(),
                                p.mutable,
                                resolve_ast_type(&p.type_, &resolve).unwrap_or(TypeInfo::Void),
                            )
                        })
                        .collect();
                    let fs = FnSymbol {
                        name: f.name.clone(),
                        generics: f.generics.clone(),
                        params,
                        return_: ret_type,
                        is_comptime: true,
                    };
                    self.check_fn_body(body, &fs);
                }
            }
            Decl::Struct(s) => {
                for m in &s.methods {
                    if let Some(ref body) = m.body {
                        let params: Vec<(String, bool, TypeInfo)> = m
                            .params
                            .iter()
                            .map(|p| {
                                let resolve = |n: &str| self.table.lookup_type(n);
                                (
                                    p.name.clone(),
                                    p.mutable,
                                    resolve_ast_type(&p.type_, &resolve).unwrap_or(TypeInfo::Void),
                                )
                            })
                            .collect();
                        let ret = m.return_.as_ref().and_then(|rt| {
                            let resolve = |n: &str| self.table.lookup_type(n);
                            resolve_ast_type(rt, &resolve).ok()
                        });
                        let fs = FnSymbol {
                            name: m.name.clone(),
                            generics: m.generics.clone(),
                            params,
                            return_: ret,
                            is_comptime: true,
                        };
                        self.check_fn_body(body, &fs);
                    }
                }
            }
            Decl::Enum(e) => {
                for m in &e.methods {
                    if let Some(ref body) = m.body {
                        let params: Vec<(String, bool, TypeInfo)> = m
                            .params
                            .iter()
                            .map(|p| {
                                let resolve = |n: &str| self.table.lookup_type(n);
                                (
                                    p.name.clone(),
                                    p.mutable,
                                    resolve_ast_type(&p.type_, &resolve).unwrap_or(TypeInfo::Void),
                                )
                            })
                            .collect();
                        let ret = m.return_.as_ref().and_then(|rt| {
                            let resolve = |n: &str| self.table.lookup_type(n);
                            resolve_ast_type(rt, &resolve).ok()
                        });
                        let fs = FnSymbol {
                            name: m.name.clone(),
                            generics: m.generics.clone(),
                            params,
                            return_: ret,
                            is_comptime: true,
                        };
                        self.check_fn_body(body, &fs);
                    }
                }
            }
            Decl::Var(v) => {
                if let Some(ref val) = v.value {
                    let declared = v.type_.as_ref().and_then(|t| {
                        let resolve = |n: &str| self.table.lookup_type(n);
                        resolve_ast_type(t, &resolve).ok()
                    });
                    let vt = self.checker.check_expr(val, &mut self.table);
                    if let (Some(vt), Some(dt)) = (&vt, &declared) {
                        if !vt.is_assignable_to(dt) && !vt.is_noret() {
                            self.checker.errors.push(SemanticError::new(
                                "SEMA-0001",
                                format!(
                                    "Type mismatch: expected '{}', found type '{}'",
                                    dt.display(),
                                    vt.display()
                                ),
                            ));
                        }
                    }
                    if let Some(sym) = self.table.lookup_in_current(&v.name) {
                        if let Some(vi) = vt {
                            if let Symbol::Variable { type_, .. } = sym {
                                if type_.is_void() {
                                    let _ = self.table.insert_overwrite(
                                        &v.name,
                                        Symbol::Variable {
                                            type_: vi.clone(),
                                            mutable: v.mutable,
                                            is_const: false,
                                        },
                                    );
                                }
                            }
                        }
                    }
                }
            }
            Decl::Const(c) => {
                if let Some(ref val) = c.value {
                    let declared = c.type_.as_ref().and_then(|t| {
                        let resolve = |n: &str| self.table.lookup_type(n);
                        resolve_ast_type(t, &resolve).ok()
                    });
                    let vt = self.checker.check_expr(val, &mut self.table);
                    if let (Some(vt), Some(dt)) = (&vt, &declared) {
                        if !vt.is_assignable_to(dt) && !vt.is_noret() {
                            self.checker.errors.push(SemanticError::new(
                                "SEMA-0001",
                                format!(
                                    "Type mismatch: expected '{}', found type '{}'",
                                    dt.display(),
                                    vt.display()
                                ),
                            ));
                        }
                    }
                    if let Some(sym) = self.table.lookup_in_current(&c.name) {
                        if let Some(vi) = vt {
                            if let Symbol::Variable { type_, .. } = sym {
                                if type_.is_void() {
                                    let _ = self.table.insert_overwrite(
                                        &c.name,
                                        Symbol::Variable {
                                            type_: vi.clone(),
                                            mutable: false,
                                            is_const: true,
                                        },
                                    );
                                }
                            }
                        }
                    }
                }
            }
            Decl::Test(_name, block) => {
                self.checker.check_block(block, &mut self.table);
            }
            Decl::Mod(_, body) => {
                if let Some(decls) = body {
                    for d in decls {
                        self.analyze_decl_bodies(d);
                    }
                }
            }
            _ => {}
        }
    }

    fn check_fn_body(&mut self, block: &Block, fn_sym: &FnSymbol) {
        self.table.push_scope();
        for (pname, pmut, ptype) in &fn_sym.params {
            self.table.insert_overwrite(
                pname,
                Symbol::Parameter {
                    type_: ptype.clone(),
                    mutable: *pmut,
                },
            );
        }

        // Set up return type checking
        let declared_ret = fn_sym.return_.clone();
        self.checker.current_return_type = declared_ret.clone();
        self.checker.reached_end = false;
        self.checker.inferred_return_type = None;

        self.checker.check_block(block, &mut self.table);

        // S-SEMA-02: Control flow - verify all paths return for non-void/non-noret
        if let Some(ref ret_type) = declared_ret {
            if !ret_type.is_void() && !ret_type.is_noret() {
                if !self.checker.reached_end {
                    self.checker.error(
                        "SEMA-0009",
                        format!(
                            "Function '{}' has non-void return type '{}' but does not return a value on all paths",
                            fn_sym.name, ret_type.display()
                        ),
                    );
                }
            }
        }

        // S-SEMA-03: Type inference - if no declared return type, use inferred
        if declared_ret.is_none() {
            if self.checker.inferred_return_type.is_some() {
                // Inferred return type is available for future use
                // (e.g., callers that reference this function)
            }
        }

        self.checker.current_return_type = None;
        self.checker.reached_end = false;
        self.checker.inferred_return_type = None;
        self.table.pop_scope();
    }
}

#[cfg(test)]
mod tests;

fn register_builtins(table: &mut SymbolTable) {
    let builtins: Vec<(&str, usize, Option<usize>)> = vec![
        ("TypeOf", 1, Some(1)),
        ("SizeOf", 1, Some(1)),
        ("AlignOf", 1, Some(1)),
        ("TypeName", 1, Some(1)),
        ("EnumCount", 1, Some(1)),
        ("Fields", 1, Some(1)),
        ("as", 2, Some(2)),
        ("bitCast", 2, Some(2)),
        ("ptrCast", 2, Some(2)),
        ("intToPtr", 2, Some(2)),
        ("ptrToInt", 1, Some(1)),
        ("memcpy", 3, Some(3)),
        ("memset", 3, Some(3)),
        ("memmove", 3, Some(3)),
        ("pageAlloc", 1, Some(1)),
        ("pageFree", 2, Some(2)),
        ("comptimeDefaultAllocator", 0, Some(0)),
        ("comptime", 0, None),
        ("compileLog", 0, None),
        ("compileError", 1, Some(1)),
        ("embedFile", 1, Some(1)),
        ("panic", 1, Some(1)),
        ("breakpoint", 0, Some(0)),
        ("trap", 0, Some(0)),
        ("sysCall", 1, None),
        ("str.from_raw", 2, Some(2)),
        ("str.from", 1, Some(1)),
        ("vec", 0, None),
        ("set", 0, None),
        ("map", 0, None),
        ("addWithOverflow", 2, Some(2)),
        ("subWithOverflow", 2, Some(2)),
        ("mulWithOverflow", 2, Some(2)),
        ("ctz", 1, Some(1)),
        ("clz", 1, Some(1)),
        ("popCount", 1, Some(1)),
        ("bswap", 1, Some(1)),
        ("atomicLoad", 2, Some(2)),
        ("atomicStore", 3, Some(3)),
        ("cmpxchg", 5, Some(5)),
    ];
    for (name, _, _) in builtins {
        table.insert_overwrite(
            name,
            Symbol::BuiltinFn {
                name: name.to_string(),
                param_count: (0, None),
            },
        );
    }
}
