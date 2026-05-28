use crate::ast::*;
use crate::lexer::token::{Token, TokenKind};
use std::fmt;

type PResult<T> = Result<T, ParseError>;

#[derive(Debug, Clone)]
pub struct ParseError {
    pub message: String,
    pub line: usize,
    pub col: usize,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Error at {}:{}: {}", self.line, self.col, self.message)
    }
}

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
    source: String,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser {
            tokens,
            pos: 0,
            source: String::new(),
        }
    }

    pub fn new_with_source(tokens: Vec<Token>, source: &str) -> Self {
        Parser {
            tokens,
            pos: 0,
            source: source.to_string(),
        }
    }

    fn mk_err(&self, msg: String) -> ParseError {
        let (line, col) = self
            .tokens
            .get(self.pos)
            .map(|t| (t.line, t.col))
            .unwrap_or((0, 0));
        ParseError {
            message: msg,
            line,
            col,
        }
    }

    pub fn parse(&mut self) -> Result<Program, Vec<ParseError>> {
        let mut decls = Vec::new();
        let mut errors = Vec::new();

        self.skip_comments();
        while self.pos < self.tokens.len() {
            println!("[DEBUG] parse loop: pos={}, token={:?}", self.pos, self.tokens.get(self.pos));
            match self.parse_decl() {
                Ok(d) => decls.push(d),
                Err(e) => {
                    errors.push(e);
                    self.skip_to_decl_boundary();
                }
            }
            self.skip_comments();
        }

        if errors.is_empty() {
            Ok(Program { decls })
        } else {
            Err(errors)
        }
    }

    fn skip_comments(&mut self) {
        while self.pos < self.tokens.len() {
            let k = &self.tokens[self.pos].kind;
            if matches!(k, TokenKind::LineComment | TokenKind::BlockComment) {
                self.pos += 1;
            } else {
                break;
            }
        }
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    fn peek_kind(&self) -> Option<TokenKind> {
        self.tokens.get(self.pos).map(|t| t.kind.clone())
    }

    fn advance(&mut self) -> Option<&Token> {
        let t = self.tokens.get(self.pos);
        self.pos += 1;
        t
    }

    fn expect(&mut self, kind: TokenKind) -> Result<&Token, ParseError> {
        let tok = self.tokens.get(self.pos);
        match tok {
            Some(t) if t.kind == kind => {
                self.pos += 1;
                Ok(t)
            }
            Some(t) => Err(ParseError {
                message: format!("Expected '{}', found '{}'", kind, t.kind),
                line: t.line,
                col: t.col,
            }),
            None => Err(ParseError {
                message: format!("Expected '{}', reached end of file", kind),
                line: self.tokens.last().map(|t| t.line).unwrap_or(0),
                col: self.tokens.last().map(|t| t.col).unwrap_or(0),
            }),
        }
    }

    fn check(&self, kind: TokenKind) -> bool {
        self.tokens.get(self.pos).map_or(false, |t| t.kind == kind)
    }

    fn check_any(&self, kinds: &[TokenKind]) -> bool {
        self.tokens
            .get(self.pos)
            .map_or(false, |t| kinds.contains(&t.kind))
    }

    fn consume_if(&mut self, kind: TokenKind) -> bool {
        if self.check(kind) {
            self.pos += 1;
            true
        } else {
            false
        }
    }

    fn expect_ident(&mut self) -> Result<String, ParseError> {
        let tok = self.tokens.get(self.pos);
        match tok {
            Some(t) if t.kind == TokenKind::Identifier => {
                let name = t.value.clone();
                self.pos += 1;
                Ok(name)
            }
            Some(t) => Err(ParseError {
                message: format!("Expected identifier, found '{}'", t.kind),
                line: t.line,
                col: t.col,
            }),
            None => Err(ParseError {
                message: "Expected identifier, reached end of file".into(),
                line: self.tokens.last().map(|t| t.line).unwrap_or(0),
                col: self.tokens.last().map(|t| t.col).unwrap_or(0),
            }),
        }
    }

    fn at_newline(&self) -> bool {
        let cur = self.tokens.get(self.pos);
        let prev = self.tokens.get(self.pos.wrapping_sub(1));
        match (cur, prev) {
            (Some(c), Some(p)) => c.line > p.line,
            _ => false,
        }
    }

    fn expect_newline_or_semi(&mut self) {
        self.consume_if(TokenKind::Semicolon);
    }

    fn parse_decl(&mut self) -> Result<Decl, ParseError> {
        let attrs = self.parse_attrs();
        let pub_ = self.consume_if(TokenKind::Pub);
        let ext = self.consume_if(TokenKind::Ext);
        let mut_ = self.consume_if(TokenKind::Mut);

        if self.consume_if(TokenKind::Use) {
            return self.parse_use();
        }
        if self.consume_if(TokenKind::Mod) {
            return self.parse_mod(pub_);
        }

        let name = self.expect_ident()?;
        let generics = if self.consume_if(TokenKind::Less) {
            let g = self.parse_generic_params()?;
            self.expect(TokenKind::Greater)?;
            g
        } else {
            Vec::new()
        };

        if self.consume_if(TokenKind::ColonColon) {
            self.parse_const_like(name, generics, pub_, ext, attrs)
        } else if self.consume_if(TokenKind::ColonEquals) {
            if self.check(TokenKind::Fn) {
                self.parse_var_fn_decl(name, mut_, pub_, attrs)
            } else {
                self.parse_var_like(name, mut_, pub_, attrs)
            }
        } else if self.consume_if(TokenKind::Colon) {
            self.parse_explicit_type_decl(name, mut_, pub_, attrs)
        } else {
            let mut expr = Expr::Ident(name);
            expr = self.parse_postfix(expr)?;
            if let Some(op) = self.try_assign_op() {
                let value = self.parse_expr()?;
                self.expect_newline_or_semi();
                Ok(Decl::Const(ConstDecl {
                    name: String::new(),
                    attrs,
                    type_: None,
                    value: Some(Expr::Binary(
                        assign_op_to_binary_op(op),
                        Box::new(expr),
                        Box::new(value),
                    )),
                }))
            } else {
                self.expect_newline_or_semi();
                Ok(Decl::Const(ConstDecl {
                    name: String::new(),
                    attrs,
                    type_: None,
                    value: Some(expr),
                }))
            }
        }
    }

    fn parse_attrs(&mut self) -> Vec<Annotation> {
        let mut attrs = Vec::new();
        loop {
            if self.peek_kind() != Some(TokenKind::At) {
                break;
            }
            let next = self.tokens.get(self.pos + 1);
            let is_attr = match next {
                Some(t) if t.kind == TokenKind::Identifier => true,
                Some(t) if is_type_keyword(&t.kind) => false,
                _ => false,
            };
            if !is_attr {
                break;
            }
            self.advance();
            let name = self.expect_ident().unwrap_or_default();
            let args = if self.consume_if(TokenKind::LeftParen) {
                let mut a = Vec::new();
                loop {
                    if self.check(TokenKind::RightParen) {
                        break;
                    }
                    match self.parse_expr() {
                        Ok(e) => a.push(e),
                        Err(_) => break,
                    }
                    if !self.consume_if(TokenKind::Comma) {
                        break;
                    }
                }
                let _ = self.expect(TokenKind::RightParen);
                a
            } else {
                Vec::new()
            };
            attrs.push(Annotation { name, args });
        }
        attrs
    }

    fn skip_to_decl_boundary(&mut self) {
        loop {
            match self.peek_kind() {
                None | Some(TokenKind::Eof) => return,
                Some(TokenKind::Semicolon) => {
                    self.advance();
                    return;
                }
                Some(TokenKind::Identifier)
                | Some(TokenKind::Pub)
                | Some(TokenKind::Ext)
                | Some(TokenKind::Use)
                | Some(TokenKind::Mod)
                    if self.pos > 0 =>
                {
                    let prev = &self.tokens[self.pos - 1];
                    if prev.line < self.tokens[self.pos].line {
                        return;
                    }
                    self.advance();
                }
                _ => {
                    self.advance();
                }
            }
        }
    }

    fn parse_use(&mut self) -> PResult<Decl> {
        let mut path = Vec::new();
        loop {
            let name = self.expect_ident()?;
            path.push(name);
            if self.consume_if(TokenKind::Dot) {
                continue;
            }
            break;
        }
        self.expect_newline_or_semi();
        Ok(Decl::Use(path))
    }

    fn parse_mod(&mut self, _pub_: bool) -> PResult<Decl> {
        let name = self.expect_ident()?;
        if self.consume_if(TokenKind::LeftBrace) {
            let mut decls = Vec::new();
            self.skip_comments();
            while !self.check(TokenKind::RightBrace) && self.pos < self.tokens.len() {
                decls.push(self.parse_decl()?);
                self.skip_comments();
            }
            self.expect(TokenKind::RightBrace)?;
            Ok(Decl::Mod(name, Some(decls)))
        } else {
            self.expect_newline_or_semi();
            Ok(Decl::Mod(name, None))
        }
    }

    fn parse_generic_params(&mut self) -> PResult<Vec<String>> {
        let mut params = Vec::new();
        loop {
            if self.check(TokenKind::Greater) {
                break;
            }
            let name = self.expect_ident()?;
            params.push(name);
            if !self.consume_if(TokenKind::Comma) {
                break;
            }
        }
        Ok(params)
    }

    fn parse_const_like(
        &mut self,
        name: String,
        generics: Vec<String>,
        pub_: bool,
        ext: bool,
        attrs: Vec<Annotation>,
    ) -> PResult<Decl> {
        if self.check(TokenKind::Fn) {
            return self.parse_fn_decl(name, generics, pub_, ext, attrs);
        }
        if self.check(TokenKind::Struct) {
            return self.parse_struct_decl(name, generics, pub_, attrs);
        }
        if self.check(TokenKind::Union) {
            return self.parse_union_decl(name, generics, pub_, attrs);
        }
        if self.check(TokenKind::Enum) {
            return self.parse_enum_decl(name, generics, pub_, attrs);
        }
        if self.check(TokenKind::ErrorKw) {
            return self.parse_error_decl(name, generics, attrs);
        }
        if self.check(TokenKind::Behave) {
            return self.parse_behave_decl(name, generics, pub_, attrs);
        }
        if self.check(TokenKind::Type) {
            return self.parse_type_alias(name, generics, pub_, attrs);
        }
        if self.check(TokenKind::Test) {
            return self.parse_test_decl(name, pub_, attrs);
        }

        let value = self.parse_expr()?;
        self.expect_newline_or_semi();
        Ok(Decl::Const(ConstDecl {
            name,
            attrs,
            type_: None,
            value: Some(value),
        }))
    }

    fn parse_var_like(
        &mut self,
        name: String,
        mut_: bool,
        pub_: bool,
        attrs: Vec<Annotation>,
    ) -> PResult<Decl> {
        let value = self.parse_expr()?;
        self.expect_newline_or_semi();
        Ok(Decl::Var(VarDecl {
            name,
            mutable: mut_,
            pub_,
            attrs,
            type_: None,
            value: Some(value),
        }))
    }

    fn parse_var_fn_decl(
        &mut self,
        name: String,
        mut_: bool,
        pub_: bool,
        attrs: Vec<Annotation>,
    ) -> PResult<Decl> {
        self.advance(); // consume 'fn'
        self.expect(TokenKind::LeftParen)?;
        let params = self.parse_fn_params()?;
        self.expect(TokenKind::RightParen)?;

        let return_ = if self.consume_if(TokenKind::Arrow) {
            Some(self.parse_type()?)
        } else {
            None
        };

        let body = if self.consume_if(TokenKind::LeftBrace) {
            Some(self.parse_block_contents()?)
        } else {
            self.expect_newline_or_semi();
            None
        };

        let fn_type = Type::Fn(
            params.iter().map(|p| p.type_.clone()).collect(),
            Box::new(return_.clone().unwrap_or(Type::Primitive(TokenKind::Void))),
        );

        Ok(Decl::Var(VarDecl {
            name: name.clone(),
            mutable: mut_,
            pub_,
            attrs: attrs.clone(),
            type_: Some(fn_type),
            value: Some(Expr::Fn(FnDecl {
                name,
                generics: Vec::new(),
                pub_,
                external: false,
                attrs,
                params,
                return_,
                body,
                is_const: false,
                is_variable_fn: true,
            })),
        }))
    }

    fn parse_explicit_type_decl(
        &mut self,
        name: String,
        mut_: bool,
        pub_: bool,
        attrs: Vec<Annotation>,
    ) -> PResult<Decl> {
        let type_ = self.parse_type()?;
        if self.consume_if(TokenKind::ColonEquals) || self.consume_if(TokenKind::Assign) {
            let value = self.parse_expr()?;
            self.expect_newline_or_semi();
            Ok(Decl::Var(VarDecl {
                name,
                mutable: mut_,
                pub_,
                attrs,
                type_: Some(type_),
                value: Some(value),
            }))
        } else if self.consume_if(TokenKind::Colon) {
            let value = self.parse_expr()?;
            self.expect_newline_or_semi();
            Ok(Decl::Const(ConstDecl {
                name,
                attrs,
                type_: Some(type_),
                value: Some(value),
            }))
        } else {
            Err(self.mk_err("Expected ':=', ':', or '=' after type".into()))
        }
    }

    fn parse_fn_decl(
        &mut self,
        name: String,
        generics: Vec<String>,
        pub_: bool,
        ext: bool,
        attrs: Vec<Annotation>,
    ) -> PResult<Decl> {
        self.advance();
        self.expect(TokenKind::LeftParen)?;
        let params = self.parse_fn_params()?;
        self.expect(TokenKind::RightParen)?;

        let return_ = if self.consume_if(TokenKind::Arrow) {
            Some(self.parse_type()?)
        } else {
            None
        };

        let body = if self.consume_if(TokenKind::LeftBrace) {
            Some(self.parse_block_contents()?)
        } else if !ext {
            self.expect_newline_or_semi();
            None
        } else {
            None
        };

        Ok(Decl::Fn(FnDecl {
            name,
            generics,
            pub_,
            external: ext,
            attrs,
            params,
            return_,
            body,
            is_const: true,
            is_variable_fn: false,
        }))
    }

    fn parse_fn_params(&mut self) -> PResult<Vec<Param>> {
        let mut params = Vec::new();
        loop {
            if self.check(TokenKind::RightParen) {
                break;
            }
            let mutable = self.consume_if(TokenKind::Mut);
            let name = self.expect_ident()?;
            self.expect(TokenKind::Colon)?;
            let type_ = self.parse_type()?;
            params.push(Param {
                name,
                mutable,
                type_,
            });
            if !self.consume_if(TokenKind::Comma) {
                break;
            }
        }
        Ok(params)
    }

    fn parse_struct_decl(
        &mut self,
        name: String,
        generics: Vec<String>,
        pub_: bool,
        attrs: Vec<Annotation>,
    ) -> PResult<Decl> {
        self.advance();
        let impl_behave = if self.consume_if(TokenKind::TildeArrow) {
            Some(self.expect_ident()?)
        } else {
            None
        };

        let mut fields = Vec::new();
        let mut methods = Vec::new();

        self.expect(TokenKind::LeftBrace)?;
        self.skip_comments();
        while !self.check(TokenKind::RightBrace) && self.pos < self.tokens.len() {
            if self.peek_kind() == Some(TokenKind::Pub)
                || self.peek_kind() == Some(TokenKind::Identifier)
            {
                let fpub = self.consume_if(TokenKind::Pub);
                let fname = self.expect_ident()?;

                if self.consume_if(TokenKind::ColonColon) {
                    self.expect(TokenKind::Fn)?;
                    let fn_decl = self.parse_fn_body_only(fname, fpub)?;
                    methods.push(fn_decl);
                } else if self.consume_if(TokenKind::Colon) {
                    let type_ = self.parse_type()?;
                    fields.push(Field {
                        name: fname,
                        pub_: fpub,
                        type_,
                    });
                    self.consume_if(TokenKind::Comma);
                } else {
                    return Err(self.mk_err("Expected ':' or '::' after field name".into()));
                }
            } else {
                break;
            }
            self.skip_comments();
        }
        self.expect(TokenKind::RightBrace)?;

        Ok(Decl::Struct(StructDecl {
            name,
            generics,
            impl_behave,
            pub_,
            attrs,
            fields,
            methods,
        }))
    }

    fn parse_union_decl(
        &mut self,
        name: String,
        generics: Vec<String>,
        pub_: bool,
        attrs: Vec<Annotation>,
    ) -> PResult<Decl> {
        self.advance();
        let mut variants = Vec::new();
        self.expect(TokenKind::LeftBrace)?;
        self.skip_comments();
        while !self.check(TokenKind::RightBrace) && self.pos < self.tokens.len() {
            if self.peek_kind() == Some(TokenKind::Identifier) {
                let vname = self.expect_ident()?;
                if self.consume_if(TokenKind::Colon) {
                    let type_ = self.parse_type()?;
                    variants.push(Field {
                        name: vname,
                        pub_: false,
                        type_,
                    });
                } else {
                    return Err(self.mk_err("Expected ':' after variant name".into()));
                }
                self.consume_if(TokenKind::Comma);
            } else {
                break;
            }
            self.skip_comments();
        }
        self.expect(TokenKind::RightBrace)?;
        Ok(Decl::Union(UnionDecl {
            name,
            generics,
            pub_,
            attrs,
            variants,
        }))
    }

    fn parse_enum_decl(
        &mut self,
        name: String,
        generics: Vec<String>,
        pub_: bool,
        attrs: Vec<Annotation>,
    ) -> PResult<Decl> {
        self.advance();
        let impl_behave = if self.consume_if(TokenKind::TildeArrow) {
            Some(self.expect_ident()?)
        } else {
            None
        };

        let mut variants = Vec::new();
        let mut methods = Vec::new();

        self.expect(TokenKind::LeftBrace)?;
        self.skip_comments();
        while !self.check(TokenKind::RightBrace) && self.pos < self.tokens.len() {
            let vname = self.expect_ident()?;

            if self.consume_if(TokenKind::Assign) {
                self.parse_expr()?; // consume the value
                variants.push(EnumVariant {
                    name: vname,
                    type_: None,
                });
                self.consume_if(TokenKind::Comma);
            } else if self.consume_if(TokenKind::ColonColon) {
                self.expect(TokenKind::Fn)?;
                let fn_decl = self.parse_fn_body_only(vname, false)?;
                methods.push(fn_decl);
            } else if self.consume_if(TokenKind::Colon) {
                let type_ = self.parse_type()?;
                variants.push(EnumVariant {
                    name: vname,
                    type_: Some(type_),
                });
                self.consume_if(TokenKind::Comma);
            } else {
                variants.push(EnumVariant {
                    name: vname,
                    type_: None,
                });
                self.consume_if(TokenKind::Comma);
            }
            self.skip_comments();
        }
        self.expect(TokenKind::RightBrace)?;

        Ok(Decl::Enum(EnumDecl {
            name,
            generics,
            impl_behave,
            pub_,
            attrs,
            variants,
            methods,
        }))
    }

    fn parse_error_decl(
        &mut self,
        name: String,
        _generics: Vec<String>,
        _attrs: Vec<Annotation>,
    ) -> PResult<Decl> {
        self.advance();
        let mut variants = Vec::new();
        self.expect(TokenKind::LeftBrace)?;
        self.skip_comments();
        while !self.check(TokenKind::RightBrace) && self.pos < self.tokens.len() {
            let ename = self.expect_ident()?;
            variants.push(EnumVariant {
                name: ename,
                type_: None,
            });
            self.consume_if(TokenKind::Comma);
            self.skip_comments();
        }
        self.expect(TokenKind::RightBrace)?;
        Ok(Decl::Error_(name, variants))
    }

    fn parse_behave_decl(
        &mut self,
        name: String,
        generics: Vec<String>,
        pub_: bool,
        attrs: Vec<Annotation>,
    ) -> PResult<Decl> {
        self.advance();
        let mut methods = Vec::new();
        self.expect(TokenKind::LeftBrace)?;
        self.skip_comments();
        while !self.check(TokenKind::RightBrace) && self.pos < self.tokens.len() {
            let mutable = self.consume_if(TokenKind::Mut);
            let fname = self.expect_ident()?;
            self.expect(TokenKind::ColonColon)?;
            self.expect(TokenKind::Fn)?;
            let mut fn_decl = self.parse_fn_body_only(fname, false)?;
            fn_decl.params.insert(
                0,
                Param {
                    name: "self".into(),
                    mutable,
                    type_: Type::Ref(false, Box::new(Type::Builtin("Self".into()))),
                },
            );
            methods.push(fn_decl);
            self.skip_comments();
        }
        self.expect(TokenKind::RightBrace)?;
        Ok(Decl::Behave(BehaveDecl {
            name,
            generics,
            pub_,
            attrs,
            methods,
        }))
    }

    fn parse_type_alias(
        &mut self,
        name: String,
        _generics: Vec<String>,
        _pub_: bool,
        _attrs: Vec<Annotation>,
    ) -> PResult<Decl> {
        self.advance();
        self.expect(TokenKind::LeftParen)?;
        let type_ = self.parse_type()?;
        self.expect(TokenKind::RightParen)?;
        self.expect_newline_or_semi();
        Ok(Decl::TypeAlias(name, type_))
    }

    fn parse_test_decl(
        &mut self,
        name: String,
        _pub_: bool,
        _attrs: Vec<Annotation>,
    ) -> PResult<Decl> {
        self.advance();
        self.expect(TokenKind::LeftBrace)?;
        let body = self.parse_block_contents()?;
        Ok(Decl::Test(name, body))
    }

    fn parse_fn_body_only(&mut self, name: String, pub_: bool) -> PResult<FnDecl> {
        self.expect(TokenKind::LeftParen)?;
        let params = self.parse_fn_params()?;
        self.expect(TokenKind::RightParen)?;

        let return_ = if self.consume_if(TokenKind::Arrow) {
            Some(self.parse_type()?)
        } else {
            None
        };

        let body = if self.consume_if(TokenKind::LeftBrace) {
            Some(self.parse_block_contents()?)
        } else {
            None
        };

        Ok(FnDecl {
            name,
            generics: Vec::new(),
            pub_,
            external: false,
            attrs: Vec::new(),
            params,
            return_,
            body,
            is_const: true,
            is_variable_fn: false,
        })
    }

    pub fn parse_type(&mut self) -> PResult<Type> {
        if self.consume_if(TokenKind::And) {
            let mutable = self.consume_if(TokenKind::Mut);
            let inner = self.parse_type()?;
            return Ok(Type::Ref(mutable, Box::new(inner)));
        }
        if self.consume_if(TokenKind::Star) {
            let inner = self.parse_type()?;
            return Ok(Type::Pointer(Box::new(inner)));
        }
        if self.consume_if(TokenKind::QuestionMark) {
            let inner = self.parse_type()?;
            return Ok(Type::Optional(Box::new(inner)));
        }
        if self.consume_if(TokenKind::At) {
            let name = if let Some(tk) = self.peek_kind() {
                if is_type_keyword(&tk) {
                    let s = format!("{}", tk);
                    self.advance();
                    s
                } else {
                    self.expect_ident()?
                }
            } else {
                self.expect_ident()?
            };
            // Handle @vec[T], @set[T], @map{K,V} syntax
            if self.consume_if(TokenKind::LeftBracket) {
                let inner = self.parse_type()?;
                if self.consume_if(TokenKind::Semicolon) {
                    let size = self.parse_expr()?;
                    self.expect(TokenKind::RightBracket)?;
                    return Ok(Type::Builtin(format!(
                        "{}[{}; {}]",
                        name,
                        type_to_label(&inner),
                        expr_label(&size)
                    )));
                }
                self.expect(TokenKind::RightBracket)?;
                return Ok(Type::Builtin(format!(
                    "{}[{}]",
                    name,
                    type_to_label(&inner)
                )));
            }
            if self.consume_if(TokenKind::LeftBrace) {
                let mut types = Vec::new();
                loop {
                    types.push(self.parse_type()?);
                    if !self.consume_if(TokenKind::Comma) {
                        break;
                    }
                }
                self.expect(TokenKind::RightBrace)?;
                let inner_str: Vec<String> = types.iter().map(type_to_label).collect();
                return Ok(Type::Builtin(format!(
                    "{}{{{}}}",
                    name,
                    inner_str.join(", ")
                )));
            }
            return Ok(Type::Builtin(name));
        }
        if self.consume_if(TokenKind::AtVec) {
            if self.consume_if(TokenKind::LeftBracket) {
                let inner = self.parse_type()?;
                if self.consume_if(TokenKind::Semicolon) {
                    let size = self.parse_expr()?;
                    self.expect(TokenKind::RightBracket)?;
                    return Ok(Type::Builtin(format!(
                        "@vec[{}; {}]",
                        type_to_label(&inner),
                        expr_label(&size)
                    )));
                }
                self.expect(TokenKind::RightBracket)?;
                return Ok(Type::Builtin(format!(
                    "@vec[{}]",
                    type_to_label(&inner)
                )));
            }
            return Ok(Type::Builtin("@vec".into()));
        }
        if self.consume_if(TokenKind::AtMap) {
            if self.consume_if(TokenKind::LeftBrace) {
                let mut types = Vec::new();
                loop {
                    types.push(self.parse_type()?);
                    if !self.consume_if(TokenKind::Comma) {
                        break;
                    }
                }
                self.expect(TokenKind::RightBrace)?;
                let inner_str: Vec<String> = types.iter().map(type_to_label).collect();
                return Ok(Type::Builtin(format!(
                    "@map{{{}}}",
                    inner_str.join(", ")
                )));
            }
            return Ok(Type::Builtin("@map".into()));
        }
        if self.consume_if(TokenKind::AtSet) {
            if self.consume_if(TokenKind::LeftBracket) {
                let inner = self.parse_type()?;
                if self.consume_if(TokenKind::Semicolon) {
                    let size = self.parse_expr()?;
                    self.expect(TokenKind::RightBracket)?;
                    return Ok(Type::Builtin(format!(
                        "@set[{}; {}]",
                        type_to_label(&inner),
                        expr_label(&size)
                    )));
                }
                self.expect(TokenKind::RightBracket)?;
                return Ok(Type::Builtin(format!(
                    "@set[{}]",
                    type_to_label(&inner)
                )));
            }
            if self.consume_if(TokenKind::LeftBrace) {
                let mut types = Vec::new();
                loop {
                    types.push(self.parse_type()?);
                    if !self.consume_if(TokenKind::Comma) {
                        break;
                    }
                }
                self.expect(TokenKind::RightBrace)?;
                let inner_str: Vec<String> = types.iter().map(type_to_label).collect();
                return Ok(Type::Builtin(format!(
                    "@set{{{}}}",
                    inner_str.join(", ")
                )));
            }
            return Ok(Type::Builtin("@set".into()));
        }
        if self.consume_if(TokenKind::Fn) {
            return self.parse_fn_type();
        }
        if self.consume_if(TokenKind::LeftBracket) {
            return self.parse_array_type();
        }

        if let Some(tk) = self.peek_kind() {
            if is_type_keyword(&tk) {
                self.advance();
                let raw = Type::Primitive(tk.clone());
                if self.consume_if(TokenKind::Bang) {
                    let ok = self.parse_type()?;
                    return Ok(Type::ErrorUnion(Some(Box::new(raw)), Box::new(ok)));
                }
                return Ok(raw);
            }
        }

        if self.peek_kind() == Some(TokenKind::Identifier) {
            let name = self.expect_ident()?;
            if self.consume_if(TokenKind::Bang) {
                let ok = self.parse_type()?;
                return Ok(Type::ErrorUnion(
                    Some(Box::new(Type::Named(name))),
                    Box::new(ok),
                ));
            }
            return Ok(Type::Named(name));
        }

        if self.consume_if(TokenKind::Bang) {
            let ok = self.parse_type()?;
            return Ok(Type::ErrorUnion(None, Box::new(ok)));
        }

        Err(self.mk_err("Expected type".into()))
    }

    fn parse_fn_type(&mut self) -> PResult<Type> {
        self.expect(TokenKind::LeftParen)?;
        let mut params = Vec::new();
        loop {
            if self.check(TokenKind::RightParen) {
                break;
            }
            let t = self.parse_type()?;
            params.push(t);
            if !self.consume_if(TokenKind::Comma) {
                break;
            }
        }
        self.expect(TokenKind::RightParen)?;
        let return_ = if self.consume_if(TokenKind::Arrow) {
            self.parse_type()?
        } else {
            return Err(self.mk_err("Expected '->' and return type for fn type".into()));
        };
        Ok(Type::Fn(params, Box::new(return_)))
    }

    fn parse_array_type(&mut self) -> PResult<Type> {
        let inner = self.parse_type()?;
        let size = if self.consume_if(TokenKind::Semicolon) {
            let expr = self.parse_expr()?;
            Some(Box::new(expr))
        } else {
            None
        };
        self.expect(TokenKind::RightBracket)?;
        match size {
            Some(s) => Ok(Type::Array(Box::new(inner), Some(s))),
            None => Ok(Type::Slice(Box::new(inner))),
        }
    }

    pub fn parse_expr(&mut self) -> PResult<Expr> {
        let mut lhs = self.parse_expr_bp(0)?;
        if let Expr::Ident(_) = &lhs {
            if self.consume_if(TokenKind::LeftBrace) {
                let fields = self.parse_struct_init_fields()?;
                self.expect(TokenKind::RightBrace)?;
                if let Expr::Ident(n) = lhs {
                    lhs = Expr::StructInit(n, fields);
                }
            }
        }
        Ok(lhs)
    }

    fn parse_match_target(&mut self) -> PResult<Expr> {
        self.parse_expr_bp(0)
    }

    fn parse_expr_bp(&mut self, min_bp: u8) -> PResult<Expr> {
        let mut lhs = self.parse_prefix_expr()?;

        loop {
            // Pipe is handled specially: if it starts a capture pattern, break
            // so capture-parsing can consume it; otherwise parse as binary OR.
            if self.peek_kind() == Some(TokenKind::Pipe) {
                if self.is_capture_start() {
                    break;
                }
                // Binary OR precedence (below Caret ^ at 9)
                const PIPE_LBP: u8 = 7;
                const PIPE_RBP: u8 = 8;
                if PIPE_LBP < min_bp {
                    break;
                }
                self.advance();
                let rhs = self.parse_expr_bp(PIPE_RBP)?;
                lhs = Expr::Binary(BinaryOp::BitOr, Box::new(lhs), Box::new(rhs));
                continue;
            }

            let op = match self.peek_kind() {
                Some(ref k) => infix_bp(k),
                None => break,
            };

            match op {
                Some((bp, _)) if bp < min_bp => break,
                Some((_, left_bp)) => {
                    let kind = self.tokens.get(self.pos).map(|t| t.kind.clone());
                    self.advance();
                    let rhs = self.parse_expr_bp(left_bp)?;
                    lhs = self.build_binary(lhs, rhs, kind)?;
                }
                None => break,
            }
        }

        Ok(lhs)
    }

    fn is_capture_start(&self) -> bool {
        let pos = self.pos;
        if pos >= self.tokens.len() || self.tokens[pos].kind != TokenKind::Pipe {
            return false;
        }
        let len = self.tokens.len();
        let mut i = pos + 1;
        if i >= len {
            return false;
        }

        let mut had_content = false;
        loop {
            if i >= len {
                return false;
            }
            match &self.tokens[i].kind {
                TokenKind::And => {
                    had_content = true;
                    i += 1;
                    if i < len && self.tokens[i].kind == TokenKind::Mut {
                        i += 1;
                    }
                    if i < len && self.tokens[i].kind == TokenKind::Identifier {
                        i += 1;
                    } else {
                        return false;
                    }
                }
                TokenKind::Mut => {
                    had_content = true;
                    i += 1;
                    if i < len && self.tokens[i].kind == TokenKind::Identifier {
                        i += 1;
                    } else {
                        return false;
                    }
                }
                TokenKind::Identifier => {
                    had_content = true;
                    i += 1;
                }
                _ => break,
            }
            if i < len && self.tokens[i].kind == TokenKind::Comma {
                i += 1;
                continue;
            }
            break;
        }

        // If there was no content between pipes (e.g., empty ||), it's not a capture
        if !had_content {
            return false;
        }

        if i >= len || self.tokens[i].kind != TokenKind::Pipe {
            return false;
        }
        i += 1;
        if i >= len {
            return false;
        }

        // After closing |, a capture must be followed by {, =>, or be inside a ,-
        // separated list or matched by another |
        matches!(
            &self.tokens[i].kind,
            TokenKind::LeftBrace
                | TokenKind::FatArrow
                | TokenKind::Comma
                | TokenKind::Pipe
                | TokenKind::RightBrace
        )
    }

    fn parse_prefix_expr(&mut self) -> PResult<Expr> {
        let kind = self.peek_kind();
        match kind {
            Some(TokenKind::If) => {
                self.advance();
                return self.parse_if_expr();
            }
            Some(TokenKind::Match) => {
                self.advance();
                return self.parse_match_expr();
            }
            Some(TokenKind::Loop) => {
                self.advance();
                return self.parse_loop_expr();
            }
            Some(TokenKind::Minus) => {
                self.advance();
                let expr = self.parse_expr_bp(prefix_bp(&TokenKind::Minus))?;
                return Ok(Expr::Unary(UnaryOp::Neg, Box::new(expr)));
            }
            Some(TokenKind::Bang) => {
                self.advance();
                let expr = self.parse_expr_bp(prefix_bp(&TokenKind::Bang))?;
                return Ok(Expr::Unary(UnaryOp::Not, Box::new(expr)));
            }
            Some(TokenKind::Tilde) => {
                self.advance();
                let expr = self.parse_expr_bp(prefix_bp(&TokenKind::Tilde))?;
                return Ok(Expr::Unary(UnaryOp::BitNot, Box::new(expr)));
            }
            Some(TokenKind::And) => {
                self.advance();
                let mutable = self.consume_if(TokenKind::Mut);
                let expr = self.parse_expr_bp(prefix_bp(&TokenKind::And))?;
                return if mutable {
                    Ok(Expr::Unary(UnaryOp::RefMut, Box::new(expr)))
                } else {
                    Ok(Expr::Unary(UnaryOp::Ref, Box::new(expr)))
                };
            }
            Some(TokenKind::Star) => {
                self.advance();
                let expr = self.parse_expr_bp(prefix_bp(&TokenKind::Star))?;
                return Ok(Expr::Unary(UnaryOp::Deref, Box::new(expr)));
            }
            Some(TokenKind::QuestionMark) => {
                self.advance();
                let expr = self.parse_expr_bp(prefix_bp(&TokenKind::QuestionMark))?;
                return Ok(Expr::Unary(UnaryOp::Optional, Box::new(expr)));
            }
            Some(TokenKind::Fn) => {
                self.advance();
                self.expect(TokenKind::LeftParen)?;
                let params = self.parse_fn_params()?;
                self.expect(TokenKind::RightParen)?;
                let return_ = if self.consume_if(TokenKind::Arrow) {
                    Some(self.parse_type()?)
                } else {
                    None
                };
                self.expect(TokenKind::LeftBrace)?;
                let body = self.parse_block_contents()?;
                return Ok(Expr::Fn(FnDecl {
                    name: String::new(),
                    generics: Vec::new(),
                    pub_: false,
                    external: false,
                    attrs: Vec::new(),
                    params,
                    return_,
                    body: Some(body),
                    is_const: true,
                    is_variable_fn: false,
                }));
            }
            Some(TokenKind::Ret) => {
                self.advance();
                let value = if !at_stmt_end(self) {
                    Some(Box::new(self.parse_expr()?))
                } else {
                    None
                };
                return Ok(Expr::Ret(value));
            }
            Some(TokenKind::LeftBrace) => {
                self.advance();
                let block = self.parse_block_contents()?;
                return Ok(Expr::Block(block));
            }
            Some(TokenKind::LeftParen) => {
                self.advance();
                let expr = self.parse_expr()?;
                self.expect(TokenKind::RightParen)?;
                let mut result = Expr::Paren(Box::new(expr));
                result = self.parse_postfix(result)?;
                return Ok(result);
            }
            _ => {}
        }

        self.parse_primary_expr()
    }

    fn parse_primary_expr(&mut self) -> PResult<Expr> {
        let tok = self.tokens.get(self.pos).cloned();
        match tok {
            Some(t) if is_literal(&t.kind) => {
                self.advance();
                let mut expr = match t.kind {
                    TokenKind::True => Expr::Literal(TokenKind::True, "true".into()),
                    TokenKind::False => Expr::Literal(TokenKind::False, "false".into()),
                    TokenKind::Nil => Expr::Literal(TokenKind::Nil, "nil".into()),
                    _ => Expr::Literal(t.kind.clone(), t.value.clone()),
                };
                expr = self.parse_postfix(expr)?;
                Ok(expr)
            }
            Some(t) if t.kind == TokenKind::Identifier => {
                self.advance();
                let name = t.value.clone();
                let mut expr = Expr::Ident(name);
                expr = self.parse_postfix(expr)?;
                Ok(expr)
            }
            Some(t) if t.kind == TokenKind::Underscore => {
                self.advance();
                Ok(Expr::Ident("_".into()))
            }
            Some(t) if t.kind == TokenKind::At => {
                self.advance();
                let name = if let Some(tk) = self.peek_kind() {
                    if is_type_keyword(&tk) {
                        let s = format!("{}", tk);
                        self.advance();
                        s
                    } else {
                        self.expect_ident()?
                    }
                } else {
                    self.expect_ident()?
                };
                let mut expr = Expr::Ident(format!("@{}", name));
                expr = self.parse_postfix(expr)?;
                Ok(expr)
            }
            Some(t) if t.kind == TokenKind::AtVec => {
                self.advance();
                let mut expr = Expr::Ident("@vec".into());
                expr = self.parse_postfix(expr)?;
                Ok(expr)
            }
            Some(t) if t.kind == TokenKind::AtMap => {
                self.advance();
                let mut expr = Expr::Ident("@map".into());
                expr = self.parse_postfix(expr)?;
                Ok(expr)
            }
            Some(t) if t.kind == TokenKind::AtSet => {
                self.advance();
                let mut expr = Expr::Ident("@set".into());
                expr = self.parse_postfix(expr)?;
                Ok(expr)
            }
            Some(t) if is_type_keyword(&t.kind) => {
                self.advance();
                let mut expr = Expr::Ident(format!("{}", t.kind));
                expr = self.parse_postfix(expr)?;
                Ok(expr)
            }
            Some(t) => Err(ParseError {
                message: format!("Unexpected token '{}'", t.kind),
                line: t.line,
                col: t.col,
            }),
            None => Err(ParseError {
                message: "Unexpected end of input".into(),
                line: self.tokens.last().map(|t| t.line).unwrap_or(0),
                col: self.tokens.last().map(|t| t.col).unwrap_or(0),
            }),
        }
    }

    fn parse_postfix(&mut self, mut lhs: Expr) -> PResult<Expr> {
        loop {
            if self.consume_if(TokenKind::LeftParen) {
                let mut args = Vec::new();
                loop {
                    if self.check(TokenKind::RightParen) {
                        break;
                    }
                    args.push(self.parse_expr()?);
                    if !self.consume_if(TokenKind::Comma) {
                        break;
                    }
                }
                self.expect(TokenKind::RightParen)?;
                lhs = Expr::Call(Box::new(lhs), args);
            } else if self.consume_if(TokenKind::DotStar) {
                lhs = Expr::Deref(Box::new(lhs));
            } else if self.consume_if(TokenKind::Dot) {
                let field = self.expect_ident()?;
                lhs = Expr::Field(Box::new(lhs), field);
            } else if self.consume_if(TokenKind::LeftBracket) {
                if self.consume_if(TokenKind::RightBracket) {
                    return Err(self.mk_err("Empty index".into()));
                }
                // Use expr_bp with min_bp=20 to prevent `..` (range, LBP=19)
                // from being consumed as a binary operator inside brackets
                let first = self.parse_expr_bp(20)?;
                if self.consume_if(TokenKind::DotDot) {
                    let inclusive = self.consume_if(TokenKind::Assign);
                    let second = self.parse_expr()?;
                    self.expect(TokenKind::RightBracket)?;
                    lhs = Expr::Slice(Box::new(lhs), Box::new(first), Box::new(second), inclusive);
                } else if self.consume_if(TokenKind::DotDotEquals) {
                    let second = self.parse_expr()?;
                    self.expect(TokenKind::RightBracket)?;
                    lhs = Expr::Slice(Box::new(lhs), Box::new(first), Box::new(second), true);
                } else {
                    self.expect(TokenKind::RightBracket)?;
                    lhs = Expr::Index(Box::new(lhs), Box::new(first));
                }
            } else if self.consume_if(TokenKind::At) {
                let method = self.expect_ident()?;
                if self.consume_if(TokenKind::LeftParen) {
                    self.expect(TokenKind::RightParen)?;
                }
                lhs = Expr::AtMethod(Box::new(lhs), method);
            } else if self.consume_if(TokenKind::Catch) {
                let capture = self.parse_pipe_capture();
                if self.consume_if(TokenKind::LeftBrace) {
                    let body = self.parse_block_contents()?;
                    lhs = Expr::Catch(Box::new(lhs), capture, Box::new(body));
                } else {
                    let fallback = self.parse_expr()?;
                    let body = Block {
                        stmts: vec![Stmt::Expr(fallback)],
                    };
                    lhs = Expr::Catch(Box::new(lhs), capture, Box::new(body));
                }
            } else {
                break;
            }
        }
        Ok(lhs)
    }

    fn parse_struct_init_fields(&mut self) -> PResult<Vec<FieldInit>> {
        let mut fields = Vec::new();
        self.skip_comments();
        loop {
            if self.check(TokenKind::RightBrace) {
                break;
            }
            let name = self.expect_ident()?;
            self.expect(TokenKind::Colon)?;
            let value = self.parse_expr()?;
            fields.push(FieldInit { name, value });
            self.consume_if(TokenKind::Comma);
            self.skip_comments();
        }
        Ok(fields)
    }

    fn parse_block_contents(&mut self) -> PResult<Block> {
        let mut stmts = Vec::new();
        self.skip_comments();
        while !self.check(TokenKind::RightBrace) && self.pos < self.tokens.len() {
            println!("[DEBUG]   block loop: pos={}, token={:?}", self.pos, self.tokens.get(self.pos));
            match self.parse_stmt() {
                Ok(stmt) => stmts.push(stmt),
                Err(_) => {
                    self.recover_stmt();
                }
            }
            self.skip_comments();
        }
        self.expect(TokenKind::RightBrace)?;
        Ok(Block { stmts })
    }

    fn recover_stmt(&mut self) {
        loop {
            match self.peek_kind() {
                None => return,
                Some(TokenKind::Eof) => return,
                Some(TokenKind::Semicolon) => {
                    self.advance();
                    return;
                }
                Some(TokenKind::RightBrace) => return,
                Some(TokenKind::RightParen) => {
                    self.advance();
                    continue;
                }
                _ => {
                    self.advance();
                }
            }
        }
    }

    fn parse_stmt(&mut self) -> PResult<Stmt> {
        if self.consume_if(TokenKind::Ret) {
            let expr = if !at_stmt_end(self) {
                Some(self.parse_expr()?)
            } else {
                None
            };
            self.expect_newline_or_semi();
            return Ok(Stmt::Ret(expr));
        }
        if self.consume_if(TokenKind::Stop) {
            self.expect_newline_or_semi();
            return Ok(Stmt::Stop);
        }
        if self.consume_if(TokenKind::Next) {
            self.expect_newline_or_semi();
            return Ok(Stmt::Next);
        }
        if self.consume_if(TokenKind::Defer) {
            let expr = self.parse_expr()?;
            self.expect_newline_or_semi();
            return Ok(Stmt::Defer(Box::new(expr)));
        }
        if self.consume_if(TokenKind::If) {
            return self.parse_if_stmt();
        }
        if self.consume_if(TokenKind::Match) {
            return self.parse_match_stmt();
        }
        if self.consume_if(TokenKind::Loop) {
            return self.parse_loop_stmt();
        }
        if self.consume_if(TokenKind::Try) {
            return self.parse_try_catch_stmt();
        }
        if self.consume_if(TokenKind::LeftBrace) {
            let block = self.parse_block_contents()?;
            return Ok(Stmt::Block(block));
        }

        let mut_ = self.consume_if(TokenKind::Mut);
        let pub_ = self.consume_if(TokenKind::Pub);
        if let Some(name) = self.try_ident() {
            if self.consume_if(TokenKind::ColonEquals) {
                let value = self.parse_expr()?;
                self.expect_newline_or_semi();
                return Ok(Stmt::Var(VarDecl {
                    name,
                    mutable: mut_,
                    pub_,
                    attrs: Vec::new(),
                    type_: None,
                    value: Some(value),
                }));
            }
            if self.consume_if(TokenKind::ColonColon) {
                let value = self.parse_expr()?;
                self.expect_newline_or_semi();
                return Ok(Stmt::Var(VarDecl {
                    name,
                    mutable: mut_,
                    pub_,
                    attrs: Vec::new(),
                    type_: None,
                    value: Some(value),
                }));
            }
            if self.consume_if(TokenKind::Colon) {
                let type_ = self.parse_type()?;
                if self.consume_if(TokenKind::ColonEquals) || self.consume_if(TokenKind::Assign) {
                    let value = self.parse_expr()?;
                    self.expect_newline_or_semi();
                    return Ok(Stmt::Var(VarDecl {
                        name,
                        mutable: mut_,
                        pub_,
                        attrs: Vec::new(),
                        type_: Some(type_),
                        value: Some(value),
                    }));
                }
                if self.consume_if(TokenKind::ColonColon) {
                    let value = self.parse_expr()?;
                    self.expect_newline_or_semi();
                    return Ok(Stmt::Var(VarDecl {
                        name,
                        mutable: mut_,
                        pub_,
                        attrs: Vec::new(),
                        type_: Some(type_),
                        value: Some(value),
                    }));
                }
                return Err(self.mk_err("Expected assignment after type".into()));
            }

            let expr = Expr::Ident(name.clone());
            let expr = self.parse_postfix(expr)?;
            if let Some(op) = self.try_assign_op() {
                let value = self.parse_expr()?;
                self.expect_newline_or_semi();
                return Ok(Stmt::Assign(expr, op, value));
            }
            self.expect_newline_or_semi();
            return Ok(Stmt::Expr(expr));
        }

        let expr = self.parse_expr()?;
        self.expect_newline_or_semi();
        Ok(Stmt::Expr(expr))
    }

    fn try_ident(&mut self) -> Option<String> {
        if self.peek_kind() == Some(TokenKind::Identifier) {
            let t = self.advance()?;
            Some(t.value.clone())
        } else {
            None
        }
    }

    fn try_assign_op(&mut self) -> Option<AssignOp> {
        let k = self.peek_kind()?;
        let op = match k {
            TokenKind::Assign => AssignOp::Eq,
            TokenKind::PlusEquals => AssignOp::AddEq,
            TokenKind::MinusEquals => AssignOp::SubEq,
            TokenKind::StarEquals => AssignOp::MulEq,
            TokenKind::SlashEquals => AssignOp::DivEq,
            TokenKind::PercentEquals => AssignOp::ModEq,
            _ => return None,
        };
        self.advance();
        Some(op)
    }

    fn parse_if_inner(&mut self) -> PResult<If> {
        let cond = self.parse_expr_bp(0)?;
        let capture = self.parse_pipe_capture();
        self.expect(TokenKind::LeftBrace)?;
        let then_block = self.parse_block_contents()?;
        let else_block = if self.consume_if(TokenKind::Else) {
            if self.consume_if(TokenKind::If) {
                let inner = self.parse_if_inner()?;
                Some(Box::new(Stmt::If(inner)))
            } else {
                self.expect(TokenKind::LeftBrace)?;
                let else_block = self.parse_block_contents()?;
                Some(Box::new(Stmt::Block(else_block)))
            }
        } else {
            None
        };
        Ok(If {
            cond,
            capture,
            then_block,
            else_block,
        })
    }

    fn parse_match_inner(&mut self) -> PResult<Match> {
        let target = self.parse_match_target()?;
        self.expect(TokenKind::LeftBrace)?;
        let mut arms = Vec::new();
        self.skip_comments();
        while !self.check(TokenKind::RightBrace) && self.pos < self.tokens.len() {
            let pattern = self.parse_pattern()?;
            let capture = self.parse_pipe_capture();
            if self.consume_if(TokenKind::FatArrow) {
                let value = self.parse_expr()?;
                arms.push(MatchArm {
                    pattern,
                    capture,
                    value,
                });
                self.consume_if(TokenKind::Comma);
            } else {
                return Err(self.mk_err("Expected '=>' in match arm".into()));
            }
            self.skip_comments();
        }
        self.expect(TokenKind::RightBrace)?;
        Ok(Match { target, arms })
    }

    fn parse_loop_inner(&mut self) -> PResult<Loop> {
        if self.check(TokenKind::LeftBrace) {
            self.advance();
            let body = self.parse_block_contents()?;
            return Ok(Loop {
                conds: Vec::new(),
                captures: Vec::new(),
                body,
            });
        }
        let mut conds = Vec::new();
        conds.push(self.parse_expr_bp(0)?);
        while self.consume_if(TokenKind::Comma) {
            // If the next token is | (capture start), stop collecting conds
            if self.peek_kind() == Some(TokenKind::Pipe) {
                break;
            }
            conds.push(self.parse_expr_bp(0)?);
        }
        let captures = self.parse_captures();
        self.expect(TokenKind::LeftBrace)?;
        let body = self.parse_block_contents()?;
        Ok(Loop {
            conds,
            captures,
            body,
        })
    }

    fn parse_if_expr(&mut self) -> PResult<Expr> {
        let if_ = self.parse_if_inner()?;
        Ok(Expr::Block(Block {
            stmts: vec![Stmt::If(if_)],
        }))
    }

    fn parse_match_expr(&mut self) -> PResult<Expr> {
        let m = self.parse_match_inner()?;
        Ok(Expr::Block(Block {
            stmts: vec![Stmt::Match(m)],
        }))
    }

    fn parse_loop_expr(&mut self) -> PResult<Expr> {
        let l = self.parse_loop_inner()?;
        Ok(Expr::Block(Block {
            stmts: vec![Stmt::Loop(l)],
        }))
    }

    fn parse_if_stmt(&mut self) -> PResult<Stmt> {
        let if_ = self.parse_if_inner()?;
        Ok(Stmt::If(if_))
    }

    fn parse_match_stmt(&mut self) -> PResult<Stmt> {
        let m = self.parse_match_inner()?;
        Ok(Stmt::Match(m))
    }

    fn parse_loop_stmt(&mut self) -> PResult<Stmt> {
        let l = self.parse_loop_inner()?;
        Ok(Stmt::Loop(l))
    }

    fn parse_try_catch_stmt(&mut self) -> PResult<Stmt> {
        if self.check(TokenKind::LeftBrace) {
            self.advance();
            let try_body = self.parse_block_contents()?;
            self.expect(TokenKind::Catch)?;
            let capture = self.parse_pipe_capture();
            self.expect(TokenKind::LeftBrace)?;
            let catch_body = self.parse_block_contents()?;
            Ok(Stmt::TryCatch(TryCatch {
                try_body,
                capture,
                catch_body,
            }))
        } else {
            let try_expr = self.parse_expr()?;
            self.expect(TokenKind::Catch)?;
            let capture = self.parse_pipe_capture();
            let catch_body = Block {
                stmts: vec![Stmt::Expr(self.parse_expr()?)],
            };
            Ok(Stmt::TryCatch(TryCatch {
                try_body: Block {
                    stmts: vec![Stmt::Expr(try_expr)],
                },
                capture,
                catch_body,
            }))
        }
    }

    fn parse_pipe_capture(&mut self) -> Vec<String> {
        let caps = self.parse_captures();
        caps.into_iter().map(|c| c.name).collect()
    }

    fn parse_captures(&mut self) -> Vec<Capture> {
        if self.consume_if(TokenKind::Pipe) {
            let mut caps = Vec::new();
            loop {
                if self.check(TokenKind::Pipe) {
                    break;
                }
                let is_ref = self.consume_if(TokenKind::And);
                let mutable = self.consume_if(TokenKind::Mut);
                if self.peek_kind() == Some(TokenKind::Identifier) {
                    let name = self.advance().unwrap().value.clone();
                    caps.push(Capture {
                        name,
                        mutable,
                        is_ref,
                    });
                }
                if !self.consume_if(TokenKind::Comma) {
                    break;
                }
            }
            self.consume_if(TokenKind::Pipe);
            caps
        } else {
            Vec::new()
        }
    }

    fn parse_pattern(&mut self) -> PResult<Pattern> {
        if self.consume_if(TokenKind::Underscore) {
            return Ok(Pattern::Wildcard);
        }
        if is_literal_kind(&self.peek_kind()) {
            let t = self.advance().unwrap();
            return Ok(Pattern::Literal(t.kind.clone(), t.value.clone()));
        }
        if self.peek_kind() == Some(TokenKind::Identifier) {
            let name = self.advance().unwrap().value.clone();
            if self.consume_if(TokenKind::Dot) {
                let variant = self.expect_ident()?;
                let capture = self.parse_single_capture();
                return Ok(Pattern::EnumVariant(name, variant, capture));
            }
            self.pos -= 1;
            let name2 = self.expect_ident()?;
            return Ok(Pattern::Ident(name2));
        }
        Err(self.mk_err("Expected pattern".into()))
    }

    fn parse_single_capture(&mut self) -> Option<String> {
        if self.consume_if(TokenKind::Pipe) {
            let name = self.expect_ident().ok()?;
            self.consume_if(TokenKind::Pipe);
            Some(name)
        } else {
            None
        }
    }
}

fn at_stmt_end(parser: &Parser) -> bool {
    parser.check(TokenKind::Semicolon)
        || parser.check(TokenKind::RightBrace)
        || parser.check(TokenKind::RightParen)
        || parser.pos >= parser.tokens.len()
}

fn line_of(parser: &Parser) -> usize {
    parser.tokens.get(parser.pos).map(|t| t.line).unwrap_or(0)
}

fn is_literal(kind: &TokenKind) -> bool {
    matches!(
        kind,
        TokenKind::IntegerValue
            | TokenKind::FloatValue
            | TokenKind::StringValue
            | TokenKind::CharValue
            | TokenKind::True
            | TokenKind::False
            | TokenKind::Nil
    )
}

fn is_literal_kind(kind: &Option<TokenKind>) -> bool {
    match kind {
        Some(k) => is_literal(k),
        None => false,
    }
}

fn is_type_keyword(kind: &TokenKind) -> bool {
    matches!(
        kind,
        TokenKind::Void
            | TokenKind::Bool
            | TokenKind::Char
            | TokenKind::Str
            | TokenKind::Noret
            | TokenKind::AnyType
            | TokenKind::Int
            | TokenKind::Uint
            | TokenKind::Float
            | TokenKind::Isize
            | TokenKind::Usize
            | TokenKind::I1
            | TokenKind::I2
            | TokenKind::I4
            | TokenKind::I8
            | TokenKind::I16
            | TokenKind::I32
            | TokenKind::I64
            | TokenKind::I128
            | TokenKind::U1
            | TokenKind::U2
            | TokenKind::U4
            | TokenKind::U8
            | TokenKind::U16
            | TokenKind::U32
            | TokenKind::U64
            | TokenKind::U128
            | TokenKind::F8
            | TokenKind::F16
            | TokenKind::F32
            | TokenKind::F64
            | TokenKind::F128
    )
}

fn prefix_bp(kind: &TokenKind) -> u8 {
    match kind {
        TokenKind::Minus
        | TokenKind::Bang
        | TokenKind::Tilde
        | TokenKind::And
        | TokenKind::Star
        | TokenKind::QuestionMark => 14,
        _ => 0,
    }
}

fn infix_bp(kind: &TokenKind) -> Option<(u8, u8)> {
    match kind {
        TokenKind::Assign
        | TokenKind::ColonEquals
        | TokenKind::PlusEquals
        | TokenKind::MinusEquals
        | TokenKind::StarEquals
        | TokenKind::SlashEquals
        | TokenKind::PercentEquals => Some((1, 2)),

        TokenKind::OrOr => Some((3, 4)),
        TokenKind::AndAnd => Some((5, 6)),

        // Pipe is handled by parse_expr_bp after capture-start detection
        TokenKind::Caret => Some((9, 10)),
        TokenKind::And => Some((11, 12)),

        TokenKind::EqualEqual | TokenKind::NotEqual => Some((13, 14)),

        TokenKind::Less | TokenKind::Greater | TokenKind::LessEqual | TokenKind::GreaterEqual => {
            Some((15, 16))
        }

        TokenKind::ShiftLeft | TokenKind::ShiftRight => Some((17, 18)),

        TokenKind::DotDot => Some((19, 20)),
        TokenKind::DotDotEquals => Some((19, 20)),

        TokenKind::Plus | TokenKind::Minus => Some((21, 22)),
        TokenKind::Star | TokenKind::Slash | TokenKind::Percent => Some((23, 24)),

        _ => None,
    }
}

impl Parser {
    fn build_binary(&mut self, lhs: Expr, rhs: Expr, kind: Option<TokenKind>) -> PResult<Expr> {
        let op = match kind {
            Some(TokenKind::Plus) => BinaryOp::Add,
            Some(TokenKind::Minus) => BinaryOp::Sub,
            Some(TokenKind::Star) => BinaryOp::Mul,
            Some(TokenKind::Slash) => BinaryOp::Div,
            Some(TokenKind::Percent) => BinaryOp::Mod,
            Some(TokenKind::PlusEquals) => BinaryOp::AddAssign,
            Some(TokenKind::MinusEquals) => BinaryOp::SubAssign,
            Some(TokenKind::StarEquals) => BinaryOp::MulAssign,
            Some(TokenKind::SlashEquals) => BinaryOp::DivAssign,
            Some(TokenKind::PercentEquals) => BinaryOp::ModAssign,
            Some(TokenKind::EqualEqual) => BinaryOp::Eq,
            Some(TokenKind::NotEqual) => BinaryOp::Ne,
            Some(TokenKind::Less) => BinaryOp::Lt,
            Some(TokenKind::Greater) => BinaryOp::Gt,
            Some(TokenKind::LessEqual) => BinaryOp::Le,
            Some(TokenKind::GreaterEqual) => BinaryOp::Ge,
            Some(TokenKind::AndAnd) => BinaryOp::And,
            Some(TokenKind::OrOr) => BinaryOp::Or,
            Some(TokenKind::And) => BinaryOp::BitAnd,
            Some(TokenKind::Pipe) => BinaryOp::BitOr,
            Some(TokenKind::Caret) => BinaryOp::BitXor,
            Some(TokenKind::ShiftLeft) => BinaryOp::Shl,
            Some(TokenKind::ShiftRight) => BinaryOp::Shr,
            Some(TokenKind::Assign) => BinaryOp::Assign,
            Some(TokenKind::ColonEquals) => BinaryOp::ColonEq,
            Some(TokenKind::DotDot) => BinaryOp::Range,
            Some(TokenKind::DotDotEquals) => BinaryOp::RangeInclusive,
            _ => {
                return Err(ParseError {
                    message: "Unknown binary operator".into(),
                    line: self.tokens.get(self.pos).map(|t| t.line).unwrap_or(0),
                    col: self.tokens.get(self.pos).map(|t| t.col).unwrap_or(0),
                });
            }
        };
        Ok(Expr::Binary(op, Box::new(lhs), Box::new(rhs)))
    }
}

fn assign_op_to_binary_op(op: AssignOp) -> BinaryOp {
    match op {
        AssignOp::Eq => BinaryOp::Assign,
        AssignOp::AddEq => BinaryOp::AddAssign,
        AssignOp::SubEq => BinaryOp::SubAssign,
        AssignOp::MulEq => BinaryOp::MulAssign,
        AssignOp::DivEq => BinaryOp::DivAssign,
        AssignOp::ModEq => BinaryOp::ModAssign,
        AssignOp::ColonEq => BinaryOp::ColonEq,
    }
}

fn type_to_label(t: &Type) -> String {
    match t {
        Type::Primitive(k) => kind_label(k),
        Type::Named(n) => n.clone(),
        Type::Ref(mut_, inner) => {
            if *mut_ {
                format!("&mut {}", type_to_label(inner))
            } else {
                format!("&{}", type_to_label(inner))
            }
        }
        Type::Pointer(inner) => format!("*{}", type_to_label(inner)),
        Type::Optional(inner) => format!("?{}", type_to_label(inner)),
        Type::ErrorUnion(err, ok) => {
            if let Some(e) = err {
                format!("{}!{}", type_to_label(e), type_to_label(ok))
            } else {
                format!("!{}", type_to_label(ok))
            }
        }
        Type::Slice(inner) => format!("[{}]", type_to_label(inner)),
        Type::Array(inner, size) => {
            if let Some(s) = size {
                format!("[{}; {}]", type_to_label(inner), expr_label(s))
            } else {
                format!("[{}]", type_to_label(inner))
            }
        }
        Type::Fn(params, ret) => {
            let ps: Vec<String> = params.iter().map(type_to_label).collect();
            format!("fn({}) -> {}", ps.join(", "), type_to_label(ret))
        }
        Type::Builtin(name) => format!("@{}", name),
    }
}

#[cfg(test)]
mod tests;

fn kind_label(k: &TokenKind) -> String {
    match k {
        TokenKind::Void => "void",
        TokenKind::Bool => "bool",
        TokenKind::Char => "char",
        TokenKind::Str => "str",
        TokenKind::Noret => "noret",
        TokenKind::AnyType => "anytype",
        TokenKind::Int => "int",
        TokenKind::Uint => "uint",
        TokenKind::Float => "float",
        TokenKind::Isize => "isize",
        TokenKind::Usize => "usize",
        TokenKind::I1 => "i1",
        TokenKind::I2 => "i2",
        TokenKind::I4 => "i4",
        TokenKind::I8 => "i8",
        TokenKind::I16 => "i16",
        TokenKind::I32 => "i32",
        TokenKind::I64 => "i64",
        TokenKind::I128 => "i128",
        TokenKind::U1 => "u1",
        TokenKind::U2 => "u2",
        TokenKind::U4 => "u4",
        TokenKind::U8 => "u8",
        TokenKind::U16 => "u16",
        TokenKind::U32 => "u32",
        TokenKind::U64 => "u64",
        TokenKind::U128 => "u128",
        TokenKind::F8 => "f8",
        TokenKind::F16 => "f16",
        TokenKind::F32 => "f32",
        TokenKind::F64 => "f64",
        TokenKind::F128 => "f128",
        TokenKind::Type => "type",
        TokenKind::StringValue => "str",
        _ => "unknown",
    }
    .to_string()
}

fn expr_label(e: &Expr) -> String {
    match e {
        Expr::Literal(_, v) => v.clone(),
        Expr::Ident(name) => name.clone(),
        _ => "expr".into(),
    }
}
