use crate::ast;
use crate::lexer::token::TokenKind;

#[derive(Debug, Clone, PartialEq)]
pub enum TypeInfo {
    Void,
    Bool,
    Char,
    Str,
    Noret,
    AnyType,
    Nil,
    Int(IntWidth, bool),
    Float(FloatWidth),
    Ref(bool, Box<TypeInfo>),
    Pointer(Box<TypeInfo>),
    Optional(Box<TypeInfo>),
    ErrorUnion(Option<Box<TypeInfo>>, Box<TypeInfo>),
    Array(Box<TypeInfo>, Option<u64>),
    Fn(Vec<TypeInfo>, Box<TypeInfo>),
    Struct(String, Vec<(String, TypeInfo, bool)>),
    Enum(String, Vec<(String, Option<TypeInfo>)>),
    Union(String, Vec<(String, TypeInfo)>),
    ErrorSet(String, Vec<String>),
    Builtin(String),
    TypeMeta,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum IntWidth {
    W1,
    W2,
    W4,
    W8,
    W16,
    W32,
    W64,
    W128,
    Arch,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FloatWidth {
    W8,
    W16,
    W32,
    W64,
    W128,
}

impl TypeInfo {
    pub fn display(&self) -> String {
        match self {
            TypeInfo::Void => "void".into(),
            TypeInfo::Bool => "bool".into(),
            TypeInfo::Char => "char".into(),
            TypeInfo::Str => "str".into(),
            TypeInfo::Noret => "noret".into(),
            TypeInfo::AnyType => "anytype".into(),
            TypeInfo::Nil => "nil".into(),
            TypeInfo::Int(w, s) => {
                if *w == IntWidth::Arch {
                    format!("{}size", if *s { "i" } else { "u" })
                } else {
                    let prefix = if *s { "i" } else { "u" };
                    let suffix = match w {
                        IntWidth::W1 => "1",
                        IntWidth::W2 => "2",
                        IntWidth::W4 => "4",
                        IntWidth::W8 => "8",
                        IntWidth::W16 => "16",
                        IntWidth::W32 => "32",
                        IntWidth::W64 => "64",
                        IntWidth::W128 => "128",
                        IntWidth::Arch => unreachable!(),
                    };
                    format!("{}{}", prefix, suffix)
                }
            }
            TypeInfo::Float(w) => {
                let suffix = match w {
                    FloatWidth::W8 => "8",
                    FloatWidth::W16 => "16",
                    FloatWidth::W32 => "32",
                    FloatWidth::W64 => "64",
                    FloatWidth::W128 => "128",
                };
                format!("f{}", suffix)
            }
            TypeInfo::Ref(mut_, inner) => {
                if *mut_ {
                    format!("&mut {}", inner.display())
                } else {
                    format!("&{}", inner.display())
                }
            }
            TypeInfo::Pointer(inner) => format!("*{}", inner.display()),
            TypeInfo::Optional(inner) => format!("?{}", inner.display()),
            TypeInfo::ErrorUnion(err, ok) => {
                if let Some(e) = err {
                    format!("{}!{}", e.display(), ok.display())
                } else {
                    format!("!{}", ok.display())
                }
            }
            TypeInfo::Array(inner, size) => {
                if let Some(s) = size {
                    format!("[{}; {}]", inner.display(), s)
                } else {
                    format!("[{}]", inner.display())
                }
            }
            TypeInfo::Fn(params, ret) => {
                let ps: Vec<String> = params.iter().map(|p| p.display()).collect();
                format!("fn({}) -> {}", ps.join(", "), ret.display())
            }
            TypeInfo::Struct(name, _) => name.clone(),
            TypeInfo::Enum(name, _) => name.clone(),
            TypeInfo::Union(name, _) => name.clone(),
            TypeInfo::ErrorSet(name, _) => name.clone(),
            TypeInfo::Builtin(name) => format!("@{}", name),
            TypeInfo::TypeMeta => "type".into(),
        }
    }

    pub fn is_numeric(&self) -> bool {
        matches!(self, TypeInfo::Int(_, _) | TypeInfo::Float(_))
    }
    pub fn is_integer(&self) -> bool {
        matches!(self, TypeInfo::Int(_, _))
    }
    pub fn is_float(&self) -> bool {
        matches!(self, TypeInfo::Float(_))
    }
    pub fn is_bool(&self) -> bool {
        matches!(self, TypeInfo::Bool)
    }
    pub fn is_noret(&self) -> bool {
        matches!(self, TypeInfo::Noret)
    }
    pub fn is_void(&self) -> bool {
        matches!(self, TypeInfo::Void)
    }
    pub fn is_reference(&self) -> bool {
        matches!(self, TypeInfo::Ref(_, _))
    }
    pub fn is_pointer(&self) -> bool {
        matches!(self, TypeInfo::Pointer(_))
    }
    pub fn is_nil(&self) -> bool {
        matches!(self, TypeInfo::Nil)
    }
    pub fn is_optional(&self) -> bool {
        matches!(self, TypeInfo::Optional(_))
    }
    pub fn is_error_union(&self) -> bool {
        matches!(self, TypeInfo::ErrorUnion(_, _))
    }
    pub fn is_struct(&self) -> bool {
        matches!(self, TypeInfo::Struct(_, _))
    }
    pub fn is_enum(&self) -> bool {
        matches!(self, TypeInfo::Enum(_, _))
    }
    pub fn is_union(&self) -> bool {
        matches!(self, TypeInfo::Union(_, _))
    }
    pub fn is_function(&self) -> bool {
        matches!(self, TypeInfo::Fn(_, _))
    }
    pub fn is_string(&self) -> bool {
        matches!(self, TypeInfo::Str) || matches!(self, TypeInfo::Builtin(n) if n == "str")
    }
    pub fn inner_optional(&self) -> Option<&TypeInfo> {
        if let TypeInfo::Optional(inner) = self {
            Some(inner)
        } else {
            None
        }
    }

    pub fn resolve_builtin(name: &str) -> Option<TypeInfo> {
        match name {
            "void" => Some(TypeInfo::Void),
            "bool" => Some(TypeInfo::Bool),
            "char" => Some(TypeInfo::Char),
            "str" => Some(TypeInfo::Str),
            "noret" => Some(TypeInfo::Noret),
            "anytype" => Some(TypeInfo::AnyType),
            "int" => Some(TypeInfo::Int(IntWidth::W32, true)),
            "uint" => Some(TypeInfo::Int(IntWidth::W32, false)),
            "float" => Some(TypeInfo::Float(FloatWidth::W32)),
            "isize" => Some(TypeInfo::Int(IntWidth::Arch, true)),
            "usize" => Some(TypeInfo::Int(IntWidth::Arch, false)),
            "i1" => Some(TypeInfo::Int(IntWidth::W1, true)),
            "i2" => Some(TypeInfo::Int(IntWidth::W2, true)),
            "i4" => Some(TypeInfo::Int(IntWidth::W4, true)),
            "i8" => Some(TypeInfo::Int(IntWidth::W8, true)),
            "i16" => Some(TypeInfo::Int(IntWidth::W16, true)),
            "i32" => Some(TypeInfo::Int(IntWidth::W32, true)),
            "i64" => Some(TypeInfo::Int(IntWidth::W64, true)),
            "i128" => Some(TypeInfo::Int(IntWidth::W128, true)),
            "u1" => Some(TypeInfo::Int(IntWidth::W1, false)),
            "u2" => Some(TypeInfo::Int(IntWidth::W2, false)),
            "u4" => Some(TypeInfo::Int(IntWidth::W4, false)),
            "u8" => Some(TypeInfo::Int(IntWidth::W8, false)),
            "u16" => Some(TypeInfo::Int(IntWidth::W16, false)),
            "u32" => Some(TypeInfo::Int(IntWidth::W32, false)),
            "u64" => Some(TypeInfo::Int(IntWidth::W64, false)),
            "u128" => Some(TypeInfo::Int(IntWidth::W128, false)),
            "f8" => Some(TypeInfo::Float(FloatWidth::W8)),
            "f16" => Some(TypeInfo::Float(FloatWidth::W16)),
            "f32" => Some(TypeInfo::Float(FloatWidth::W32)),
            "f64" => Some(TypeInfo::Float(FloatWidth::W64)),
            "f128" => Some(TypeInfo::Float(FloatWidth::W128)),
            _ => None,
        }
    }

    pub fn is_assignable_to(&self, target: &TypeInfo) -> bool {
        if self.is_noret() {
            return true;
        }
        if std::mem::discriminant(self) == std::mem::discriminant(target) {
            match (self, target) {
                (TypeInfo::Int(_, _), TypeInfo::Int(_, _)) => true,
                (TypeInfo::Float(_), TypeInfo::Float(_)) => true,
                (TypeInfo::Ref(m1, i1), TypeInfo::Ref(m2, i2)) => {
                    (*m2 || !*m1) && i1.is_assignable_to(i2)
                }
                (TypeInfo::Pointer(i1), TypeInfo::Pointer(i2)) => {
                    i1.is_assignable_to(i2) || i1.is_void() || i2.is_void()
                }
                (TypeInfo::Optional(i1), TypeInfo::Optional(i2)) => i1.is_assignable_to(i2),
                (TypeInfo::ErrorUnion(e1, o1), TypeInfo::ErrorUnion(e2, o2)) => {
                    let err_ok = match (e1, e2) {
                        (Some(ee1), Some(ee2)) => ee1.is_assignable_to(ee2),
                        (None, Some(_)) => true,
                        (Some(_), None) => false,
                        (None, None) => true,
                    };
                    err_ok && o1.is_assignable_to(o2)
                }
                (TypeInfo::Array(i1, s1), TypeInfo::Array(i2, s2)) => {
                    let size_ok = match (s1, s2) {
                        (Some(a), Some(b)) => a == b,
                        (None, None) => true,
                        (Some(_), None) => true,
                        (None, Some(_)) => false,
                    };
                    i1.is_assignable_to(i2) && size_ok
                }
                (TypeInfo::Fn(p1, r1), TypeInfo::Fn(p2, r2)) => {
                    p1.len() == p2.len()
                        && p1.iter().zip(p2.iter()).all(|(a, b)| a.is_assignable_to(b))
                        && r1.is_assignable_to(r2)
                }
                (TypeInfo::Struct(n1, _), TypeInfo::Struct(n2, _)) => n1 == n2,
                (TypeInfo::Enum(n1, _), TypeInfo::Enum(n2, _)) => n1 == n2,
                (TypeInfo::Union(n1, _), TypeInfo::Union(n2, _)) => n1 == n2,
                (TypeInfo::ErrorSet(n1, _), TypeInfo::ErrorSet(n2, _)) => n1 == n2,
                (TypeInfo::Builtin(n1), TypeInfo::Builtin(n2)) => n1 == n2,
                _ => std::mem::discriminant(self) == std::mem::discriminant(target),
            }
        } else {
            if self.is_nil() && (target.is_optional() || target.is_pointer() || target.is_noret()) {
                return true;
            }
            if let TypeInfo::Optional(inner) = target {
                return self.is_assignable_to(inner);
            }
            if let TypeInfo::ErrorUnion(_, ok) = target {
                return self.is_assignable_to(ok);
            }
            false
        }
    }
}

pub fn resolve_ast_type(
    ast_type: &ast::Type,
    resolve_named: &impl Fn(&str) -> Option<TypeInfo>,
) -> Result<TypeInfo, String> {
    match ast_type {
        ast::Type::Primitive(kind) => {
            let name = format!("{}", kind);
            TypeInfo::resolve_builtin(&name)
                .ok_or_else(|| format!("Unknown primitive type '{}'", name))
        }
        ast::Type::Named(name) => {
            resolve_named(name).ok_or_else(|| format!("Unknown type '{}'", name))
        }
        ast::Type::Ref(mutable, inner) => Ok(TypeInfo::Ref(
            *mutable,
            Box::new(resolve_ast_type(inner, resolve_named)?),
        )),
        ast::Type::Pointer(inner) => Ok(TypeInfo::Pointer(Box::new(resolve_ast_type(
            inner,
            resolve_named,
        )?))),
        ast::Type::Optional(inner) => Ok(TypeInfo::Optional(Box::new(resolve_ast_type(
            inner,
            resolve_named,
        )?))),
        ast::Type::ErrorUnion(err, ok) => {
            let err_type = match err {
                Some(e) => Some(Box::new(resolve_ast_type(e, resolve_named)?)),
                None => None,
            };
            Ok(TypeInfo::ErrorUnion(
                err_type,
                Box::new(resolve_ast_type(ok, resolve_named)?),
            ))
        }
        ast::Type::Array(inner, size) => {
            let inner_type = resolve_ast_type(inner, resolve_named)?;
            let size_val = size.as_ref().and_then(|s| {
                if let ast::Expr::Literal(_, val) = s.as_ref() {
                    val.parse::<u64>().ok()
                } else {
                    None
                }
            });
            Ok(TypeInfo::Array(Box::new(inner_type), size_val))
        }
        ast::Type::Slice(inner) => {
            let inner_type = resolve_ast_type(inner, resolve_named)?;
            Ok(TypeInfo::Array(Box::new(inner_type), None))
        }
        ast::Type::Fn(params, ret) => {
            let resolved_params: Vec<TypeInfo> = params
                .iter()
                .map(|p| resolve_ast_type(p, resolve_named))
                .collect::<Result<Vec<_>, _>>()?;
            Ok(TypeInfo::Fn(
                resolved_params,
                Box::new(resolve_ast_type(ret, resolve_named)?),
            ))
        }
        ast::Type::Builtin(name) => {
            Ok(TypeInfo::resolve_builtin(name).unwrap_or(TypeInfo::Builtin(name.clone())))
        }
    }
}

pub fn literal_to_type(kind: &TokenKind, _val: &str) -> TypeInfo {
    match kind {
        TokenKind::IntegerValue => TypeInfo::Int(IntWidth::W32, true),
        TokenKind::FloatValue => TypeInfo::Float(FloatWidth::W32),
        TokenKind::StringValue => TypeInfo::Str,
        TokenKind::CharValue => TypeInfo::Char,
        TokenKind::True | TokenKind::False => TypeInfo::Bool,
        TokenKind::Nil => TypeInfo::Nil,
        _ => TypeInfo::Void,
    }
}
