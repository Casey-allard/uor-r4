//! In-sandbox C-to-RV64IM ELF64 compiler (`R4-37`).
//!
//! Compiles freestanding/POSIX subset C source files directly inside WebAssembly
//! into genuine 64-bit `EM_RISCV` (`243`) static ELF64 executables that run on the
//! `MVM-Wasm` `RV64IMAC` processor via Linux `riscv64` `ecall` system calls.

use std::collections::HashMap;

/// Base virtual address for `PT_LOAD` in compiled RV64 ELF64 executables.
pub const RV64_BASE_VADDR: u64 = 0x0001_0000;
/// Offset of `.text` (`_start`) after the 64-byte `Ehdr` and 56-byte `Phdr` (padded to 128B).
pub const RV64_TEXT_OFFSET: usize = 0x80;

/// Typed compilation diagnostic emitted by `compile_c_to_rv64_elf`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CCompileError {
    /// Line number (1-based) where compilation failed.
    pub line: usize,
    /// Human-readable diagnostic message.
    pub message: String,
}

impl std::fmt::Display for CCompileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "mvm-cc:{}: {}", self.line, self.message)
    }
}

impl std::error::Error for CCompileError {}

#[derive(Debug, Clone, PartialEq, Eq)]
enum TokKind {
    Ident(String),
    IntLit(i64),
    StrLit(Vec<u8>),
    CharLit(u8),
    LParen,
    RParen,
    LBrace,
    RBrace,
    LBracket,
    RBracket,
    Semi,
    Comma,
    Assign,
    PlusAssign,
    MinusAssign,
    PlusPlus,
    MinusMinus,
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Amp,
    Pipe,
    Caret,
    Tilde,
    Bang,
    AmpAmp,
    PipePipe,
    EqEq,
    BangEq,
    Lt,
    Le,
    Gt,
    Ge,
    Shl,
    Shr,
    Question,
    Colon,
    Eof,
}

#[derive(Debug, Clone)]
struct Tok {
    kind: TokKind,
    line: usize,
}

fn preprocess_and_lex(src: &str) -> Result<Vec<Tok>, CCompileError> {
    let mut defines: HashMap<String, String> = HashMap::new();
    let mut cleaned = String::with_capacity(src.len());
    for raw_line in src.lines() {
        let trimmed = raw_line.trim_start();
        if let Some(rest) = trimmed.strip_prefix('#') {
            let directive = rest.trim();
            if let Some(def_rest) = directive.strip_prefix("define ") {
                let mut parts = def_rest.trim().splitn(2, char::is_whitespace);
                if let Some(name) = parts.next() {
                    let val = parts.next().unwrap_or("1").trim();
                    if !name.contains('(') {
                        defines.insert(name.to_string(), val.to_string());
                    }
                }
            }
            cleaned.push('\n');
        } else {
            cleaned.push_str(raw_line);
            cleaned.push('\n');
        }
    }

    let bytes = cleaned.as_bytes();
    let mut i = 0usize;
    let mut line = 1usize;
    let mut toks = Vec::new();

    while i < bytes.len() {
        let b = bytes[i];
        if b == b'\n' {
            line += 1;
            i += 1;
            continue;
        }
        if b.is_ascii_whitespace() {
            i += 1;
            continue;
        }
        if b == b'/' && i + 1 < bytes.len() && bytes[i + 1] == b'/' {
            while i < bytes.len() && bytes[i] != b'\n' {
                i += 1;
            }
            continue;
        }
        if b == b'/' && i + 1 < bytes.len() && bytes[i + 1] == b'*' {
            i += 2;
            while i + 1 < bytes.len() && !(bytes[i] == b'*' && bytes[i + 1] == b'/') {
                if bytes[i] == b'\n' {
                    line += 1;
                }
                i += 1;
            }
            i = (i + 2).min(bytes.len());
            continue;
        }
        if b == b'"' {
            i += 1;
            let mut s = Vec::new();
            while i < bytes.len() && bytes[i] != b'"' {
                if bytes[i] == b'\\' && i + 1 < bytes.len() {
                    i += 1;
                    let esc = match bytes[i] {
                        b'n' => b'\n',
                        b't' => b'\t',
                        b'r' => b'\r',
                        b'0' => 0,
                        b'\\' => b'\\',
                        b'"' => b'"',
                        other => other,
                    };
                    s.push(esc);
                } else {
                    if bytes[i] == b'\n' {
                        line += 1;
                    }
                    s.push(bytes[i]);
                }
                i += 1;
            }
            if i >= bytes.len() {
                return Err(CCompileError {
                    line,
                    message: "unterminated string literal".to_string(),
                });
            }
            i += 1;
            toks.push(Tok {
                kind: TokKind::StrLit(s),
                line,
            });
            continue;
        }
        if b == b'\'' {
            i += 1;
            if i >= bytes.len() {
                return Err(CCompileError {
                    line,
                    message: "unterminated char literal".to_string(),
                });
            }
            let ch = if bytes[i] == b'\\' && i + 1 < bytes.len() {
                i += 1;
                match bytes[i] {
                    b'n' => b'\n',
                    b't' => b'\t',
                    b'r' => b'\r',
                    b'0' => 0,
                    other => other,
                }
            } else {
                bytes[i]
            };
            i += 1;
            if i >= bytes.len() || bytes[i] != b'\'' {
                return Err(CCompileError {
                    line,
                    message: "expected closing quote for char literal".to_string(),
                });
            }
            i += 1;
            toks.push(Tok {
                kind: TokKind::CharLit(ch),
                line,
            });
            continue;
        }
        if b.is_ascii_digit() {
            let start = i;
            if b == b'0' && i + 1 < bytes.len() && (bytes[i + 1] == b'x' || bytes[i + 1] == b'X') {
                i += 2;
                let hex_start = i;
                while i < bytes.len() && bytes[i].is_ascii_hexdigit() {
                    i += 1;
                }
                let val =
                    u64::from_str_radix(&cleaned[hex_start..i], 16).map_err(|_| CCompileError {
                        line,
                        message: "invalid hex literal".to_string(),
                    })? as i64;
                while i < bytes.len() && matches!(bytes[i], b'u' | b'U' | b'l' | b'L') {
                    i += 1;
                }
                toks.push(Tok {
                    kind: TokKind::IntLit(val),
                    line,
                });
                continue;
            }
            while i < bytes.len() && bytes[i].is_ascii_digit() {
                i += 1;
            }
            let val = cleaned[start..i]
                .parse::<i64>()
                .map_err(|_| CCompileError {
                    line,
                    message: "invalid integer literal".to_string(),
                })?;
            while i < bytes.len() && matches!(bytes[i], b'u' | b'U' | b'l' | b'L') {
                i += 1;
            }
            toks.push(Tok {
                kind: TokKind::IntLit(val),
                line,
            });
            continue;
        }
        if b.is_ascii_alphabetic() || b == b'_' {
            let start = i;
            while i < bytes.len() && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_') {
                i += 1;
            }
            let ident = &cleaned[start..i];
            if let Some(rep) = defines.get(ident) {
                if let Ok(v) = rep.parse::<i64>() {
                    toks.push(Tok {
                        kind: TokKind::IntLit(v),
                        line,
                    });
                    continue;
                }
            }
            toks.push(Tok {
                kind: TokKind::Ident(ident.to_string()),
                line,
            });
            continue;
        }

        let two = if i + 1 < bytes.len() {
            Some((bytes[i], bytes[i + 1]))
        } else {
            None
        };
        if let Some(pair) = two {
            let two_tok = match pair {
                (b'+', b'=') => Some(TokKind::PlusAssign),
                (b'-', b'=') => Some(TokKind::MinusAssign),
                (b'+', b'+') => Some(TokKind::PlusPlus),
                (b'-', b'-') => Some(TokKind::MinusMinus),
                (b'&', b'&') => Some(TokKind::AmpAmp),
                (b'|', b'|') => Some(TokKind::PipePipe),
                (b'=', b'=') => Some(TokKind::EqEq),
                (b'!', b'=') => Some(TokKind::BangEq),
                (b'<', b'=') => Some(TokKind::Le),
                (b'>', b'=') => Some(TokKind::Ge),
                (b'<', b'<') => Some(TokKind::Shl),
                (b'>', b'>') => Some(TokKind::Shr),
                _ => None,
            };
            if let Some(kind) = two_tok {
                toks.push(Tok { kind, line });
                i += 2;
                continue;
            }
        }

        let single = match b {
            b'(' => TokKind::LParen,
            b')' => TokKind::RParen,
            b'{' => TokKind::LBrace,
            b'}' => TokKind::RBrace,
            b'[' => TokKind::LBracket,
            b']' => TokKind::RBracket,
            b';' => TokKind::Semi,
            b',' => TokKind::Comma,
            b'=' => TokKind::Assign,
            b'+' => TokKind::Plus,
            b'-' => TokKind::Minus,
            b'*' => TokKind::Star,
            b'/' => TokKind::Slash,
            b'%' => TokKind::Percent,
            b'&' => TokKind::Amp,
            b'|' => TokKind::Pipe,
            b'^' => TokKind::Caret,
            b'~' => TokKind::Tilde,
            b'!' => TokKind::Bang,
            b'<' => TokKind::Lt,
            b'>' => TokKind::Gt,
            b'?' => TokKind::Question,
            b':' => TokKind::Colon,
            _ => {
                return Err(CCompileError {
                    line,
                    message: format!("unexpected byte '{}'", b as char),
                })
            }
        };
        toks.push(Tok { kind: single, line });
        i += 1;
    }

    toks.push(Tok {
        kind: TokKind::Eof,
        line,
    });
    Ok(toks)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CType {
    Byte,
    Word64,
    PtrByte,
    PtrWord64,
}

#[derive(Debug, Clone)]
enum Expr {
    Int(i64),
    Str(Vec<u8>),
    Var(String),
    Index(Box<Expr>, Box<Expr>),
    Deref(Box<Expr>),
    AddrOf(String),
    Unary(UnaryOp, Box<Expr>),
    Binary(BinOp, Box<Expr>, Box<Expr>),
    Ternary(Box<Expr>, Box<Expr>, Box<Expr>),
    AssignVar(String, Box<Expr>),
    AssignIndex(Box<Expr>, Box<Expr>, Box<Expr>),
    AssignDeref(Box<Expr>, Box<Expr>),
    Call(String, Vec<Expr>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum UnaryOp {
    Neg,
    BitNot,
    LogNot,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    Shl,
    Shr,
    BitAnd,
    BitOr,
    BitXor,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    LogAnd,
    LogOr,
}

#[derive(Debug, Clone)]
struct LocalDecl {
    name: String,
    ty: CType,
    array_elems: Option<usize>,
    init: Option<Expr>,
}

#[derive(Debug, Clone)]
#[allow(clippy::enum_variant_names)]
enum Stmt {
    Block(Vec<Stmt>),
    Decl(Vec<LocalDecl>),
    If(Expr, Box<Stmt>, Option<Box<Stmt>>),
    While(Expr, Box<Stmt>),
    For(Option<Box<Stmt>>, Option<Expr>, Option<Expr>, Box<Stmt>),
    Return(Option<Expr>),
    Break,
    ExprStmt(Expr),
    Empty,
}

#[derive(Debug, Clone)]
struct FuncDef {
    name: String,
    params: Vec<(String, CType)>,
    body: Vec<Stmt>,
}

#[derive(Debug, Clone)]
struct GlobalVar {
    name: String,
    size_bytes: usize,
    init_i64: i64,
}

#[derive(Debug, Clone)]
struct Program {
    globals: Vec<GlobalVar>,
    funcs: Vec<FuncDef>,
}

struct Parser {
    toks: Vec<Tok>,
    pos: usize,
}

impl Parser {
    fn peek(&self) -> &Tok {
        &self.toks[self.pos.min(self.toks.len() - 1)]
    }

    fn line(&self) -> usize {
        self.peek().line
    }

    fn next(&mut self) -> Tok {
        let t = self.peek().clone();
        if self.pos + 1 < self.toks.len() {
            self.pos += 1;
        }
        t
    }

    fn eat(&mut self, expected: &TokKind) -> bool {
        if &self.peek().kind == expected {
            self.next();
            true
        } else {
            false
        }
    }

    fn expect(&mut self, expected: &TokKind) -> Result<(), CCompileError> {
        if self.eat(expected) {
            Ok(())
        } else {
            Err(CCompileError {
                line: self.line(),
                message: format!("expected {:?}, got {:?}", expected, self.peek().kind),
            })
        }
    }

    fn expect_ident(&mut self) -> Result<String, CCompileError> {
        match &self.peek().kind {
            TokKind::Ident(s) => {
                let out = s.clone();
                self.next();
                Ok(out)
            }
            other => Err(CCompileError {
                line: self.line(),
                message: format!("expected identifier, got {:?}", other),
            }),
        }
    }

    fn is_type_start(&self) -> bool {
        matches!(
            &self.peek().kind,
            TokKind::Ident(s)
                if matches!(
                    s.as_str(),
                    "void"
                        | "int"
                        | "long"
                        | "char"
                        | "short"
                        | "unsigned"
                        | "signed"
                        | "const"
                        | "static"
                        | "size_t"
                        | "ssize_t"
                        | "uint8_t"
                        | "uint32_t"
                        | "uint64_t"
                        | "int64_t"
                )
        )
    }

    fn parse_type(&mut self) -> Result<CType, CCompileError> {
        let mut is_char = false;
        let mut stars = 0usize;
        let mut consumed = false;
        while self.is_type_start() {
            if let TokKind::Ident(s) = &self.peek().kind {
                if s == "char" || s == "uint8_t" {
                    is_char = true;
                }
            }
            self.next();
            consumed = true;
        }
        if !consumed {
            return Err(CCompileError {
                line: self.line(),
                message: "expected C type specifier".to_string(),
            });
        }
        while self.eat(&TokKind::Star) {
            stars += 1;
        }
        Ok(if stars > 0 {
            if is_char {
                CType::PtrByte
            } else {
                CType::PtrWord64
            }
        } else if is_char {
            CType::Byte
        } else {
            CType::Word64
        })
    }

    fn parse_program(&mut self) -> Result<Program, CCompileError> {
        let mut globals = Vec::new();
        let mut funcs = Vec::new();
        while self.peek().kind != TokKind::Eof {
            let base_ty = self.parse_type()?;
            let name = self.expect_ident()?;
            if self.eat(&TokKind::LParen) {
                let mut params = Vec::new();
                if !self.eat(&TokKind::RParen) {
                    if matches!(&self.peek().kind, TokKind::Ident(s) if s == "void")
                        && self.toks.get(self.pos + 1).map(|t| &t.kind) == Some(&TokKind::RParen)
                    {
                        self.next();
                        self.expect(&TokKind::RParen)?;
                    } else {
                        loop {
                            let mut pty = self.parse_type()?;
                            let pname = self.expect_ident()?;
                            if self.eat(&TokKind::LBracket) {
                                let _ = self.eat(&TokKind::RBracket);
                                pty = if pty == CType::Byte {
                                    CType::PtrByte
                                } else {
                                    CType::PtrWord64
                                };
                            }
                            params.push((pname, pty));
                            if !self.eat(&TokKind::Comma) {
                                break;
                            }
                        }
                        self.expect(&TokKind::RParen)?;
                    }
                }
                if self.eat(&TokKind::Semi) {
                    // Forward prototype declaration
                    continue;
                }
                self.expect(&TokKind::LBrace)?;
                let mut body = Vec::new();
                while !self.eat(&TokKind::RBrace) {
                    if self.peek().kind == TokKind::Eof {
                        return Err(CCompileError {
                            line: self.line(),
                            message: "unclosed function body".to_string(),
                        });
                    }
                    body.push(self.parse_stmt()?);
                }
                funcs.push(FuncDef { name, params, body });
            } else {
                let mut size_bytes = 8usize;
                if self.eat(&TokKind::LBracket) {
                    if let TokKind::IntLit(n) = self.peek().kind {
                        self.next();
                        let elem = if base_ty == CType::Byte { 1 } else { 8 };
                        size_bytes = ((n.max(1) as usize) * elem + 7) & !7;
                    }
                    self.expect(&TokKind::RBracket)?;
                }
                let mut init_i64 = 0i64;
                if self.eat(&TokKind::Assign) {
                    if let TokKind::IntLit(v) = self.peek().kind {
                        init_i64 = v;
                        self.next();
                    } else if self.eat(&TokKind::Minus) {
                        if let TokKind::IntLit(v) = self.peek().kind {
                            init_i64 = -v;
                            self.next();
                        }
                    }
                }
                self.expect(&TokKind::Semi)?;
                globals.push(GlobalVar {
                    name,
                    size_bytes,
                    init_i64,
                });
            }
        }
        Ok(Program { globals, funcs })
    }

    fn parse_decl_stmt(&mut self, eat_semi: bool) -> Result<Stmt, CCompileError> {
        let base_ty = self.parse_type()?;
        let mut first_stars_applied = base_ty;
        let mut decls = Vec::new();
        loop {
            while self.eat(&TokKind::Star) {
                first_stars_applied = CType::PtrWord64;
            }
            let name = self.expect_ident()?;
            let mut array_elems = None;
            if self.eat(&TokKind::LBracket) {
                if let TokKind::IntLit(n) = self.peek().kind {
                    self.next();
                    array_elems = Some(n.max(1) as usize);
                } else {
                    return Err(CCompileError {
                        line: self.line(),
                        message: "expected constant array length".to_string(),
                    });
                }
                self.expect(&TokKind::RBracket)?;
            }
            let init = if self.eat(&TokKind::Assign) {
                Some(self.parse_expr()?)
            } else {
                None
            };
            decls.push(LocalDecl {
                name,
                ty: first_stars_applied,
                array_elems,
                init,
            });
            first_stars_applied = if base_ty == CType::PtrByte || base_ty == CType::Byte {
                CType::Byte
            } else {
                CType::Word64
            };
            if !self.eat(&TokKind::Comma) {
                break;
            }
        }
        if eat_semi {
            self.expect(&TokKind::Semi)?;
        }
        Ok(Stmt::Decl(decls))
    }

    fn parse_stmt(&mut self) -> Result<Stmt, CCompileError> {
        if self.eat(&TokKind::Semi) {
            return Ok(Stmt::Empty);
        }
        if self.eat(&TokKind::LBrace) {
            let mut stmts = Vec::new();
            while !self.eat(&TokKind::RBrace) {
                if self.peek().kind == TokKind::Eof {
                    return Err(CCompileError {
                        line: self.line(),
                        message: "unclosed block".to_string(),
                    });
                }
                stmts.push(self.parse_stmt()?);
            }
            return Ok(Stmt::Block(stmts));
        }
        if self.is_type_start() {
            return self.parse_decl_stmt(true);
        }
        if let TokKind::Ident(kw) = &self.peek().kind {
            match kw.as_str() {
                "if" => {
                    self.next();
                    self.expect(&TokKind::LParen)?;
                    let cond = self.parse_expr()?;
                    self.expect(&TokKind::RParen)?;
                    let then_s = Box::new(self.parse_stmt()?);
                    let else_s = if matches!(&self.peek().kind, TokKind::Ident(s) if s == "else") {
                        self.next();
                        Some(Box::new(self.parse_stmt()?))
                    } else {
                        None
                    };
                    return Ok(Stmt::If(cond, then_s, else_s));
                }
                "while" => {
                    self.next();
                    self.expect(&TokKind::LParen)?;
                    let cond = self.parse_expr()?;
                    self.expect(&TokKind::RParen)?;
                    let body = Box::new(self.parse_stmt()?);
                    return Ok(Stmt::While(cond, body));
                }
                "for" => {
                    self.next();
                    self.expect(&TokKind::LParen)?;
                    let init = if self.eat(&TokKind::Semi) {
                        None
                    } else if self.is_type_start() {
                        let d = self.parse_decl_stmt(true)?;
                        Some(Box::new(d))
                    } else {
                        let e = self.parse_expr()?;
                        self.expect(&TokKind::Semi)?;
                        Some(Box::new(Stmt::ExprStmt(e)))
                    };
                    let cond = if self.eat(&TokKind::Semi) {
                        None
                    } else {
                        let c = self.parse_expr()?;
                        self.expect(&TokKind::Semi)?;
                        Some(c)
                    };
                    let step = if self.eat(&TokKind::RParen) {
                        None
                    } else {
                        let s = self.parse_expr()?;
                        self.expect(&TokKind::RParen)?;
                        Some(s)
                    };
                    let body = Box::new(self.parse_stmt()?);
                    return Ok(Stmt::For(init, cond, step, body));
                }
                "return" => {
                    self.next();
                    if self.eat(&TokKind::Semi) {
                        return Ok(Stmt::Return(None));
                    }
                    let e = self.parse_expr()?;
                    self.expect(&TokKind::Semi)?;
                    return Ok(Stmt::Return(Some(e)));
                }
                "break" => {
                    self.next();
                    self.expect(&TokKind::Semi)?;
                    return Ok(Stmt::Break);
                }
                _ => {}
            }
        }
        let expr = self.parse_expr()?;
        self.expect(&TokKind::Semi)?;
        Ok(Stmt::ExprStmt(expr))
    }

    fn parse_expr(&mut self) -> Result<Expr, CCompileError> {
        self.parse_assign()
    }

    fn parse_assign(&mut self) -> Result<Expr, CCompileError> {
        let lhs = self.parse_ternary()?;
        if self.eat(&TokKind::Assign) {
            let rhs = self.parse_assign()?;
            return match lhs {
                Expr::Var(name) => Ok(Expr::AssignVar(name, Box::new(rhs))),
                Expr::Index(base, idx) => Ok(Expr::AssignIndex(base, idx, Box::new(rhs))),
                Expr::Deref(ptr) => Ok(Expr::AssignDeref(ptr, Box::new(rhs))),
                _ => Err(CCompileError {
                    line: self.line(),
                    message: "invalid assignment target".to_string(),
                }),
            };
        }
        if self.eat(&TokKind::PlusAssign) {
            let rhs = self.parse_assign()?;
            return match lhs {
                Expr::Var(name) => Ok(Expr::AssignVar(
                    name.clone(),
                    Box::new(Expr::Binary(
                        BinOp::Add,
                        Box::new(Expr::Var(name)),
                        Box::new(rhs),
                    )),
                )),
                _ => Err(CCompileError {
                    line: self.line(),
                    message: "+= supported on variables".to_string(),
                }),
            };
        }
        if self.eat(&TokKind::MinusAssign) {
            let rhs = self.parse_assign()?;
            return match lhs {
                Expr::Var(name) => Ok(Expr::AssignVar(
                    name.clone(),
                    Box::new(Expr::Binary(
                        BinOp::Sub,
                        Box::new(Expr::Var(name)),
                        Box::new(rhs),
                    )),
                )),
                _ => Err(CCompileError {
                    line: self.line(),
                    message: "-= supported on variables".to_string(),
                }),
            };
        }
        Ok(lhs)
    }

    fn parse_ternary(&mut self) -> Result<Expr, CCompileError> {
        let cond = self.parse_log_or()?;
        if self.eat(&TokKind::Question) {
            let t = self.parse_expr()?;
            self.expect(&TokKind::Colon)?;
            let f = self.parse_ternary()?;
            Ok(Expr::Ternary(Box::new(cond), Box::new(t), Box::new(f)))
        } else {
            Ok(cond)
        }
    }

    fn parse_log_or(&mut self) -> Result<Expr, CCompileError> {
        let mut lhs = self.parse_log_and()?;
        while self.eat(&TokKind::PipePipe) {
            let rhs = self.parse_log_and()?;
            lhs = Expr::Binary(BinOp::LogOr, Box::new(lhs), Box::new(rhs));
        }
        Ok(lhs)
    }

    fn parse_log_and(&mut self) -> Result<Expr, CCompileError> {
        let mut lhs = self.parse_bit_or()?;
        while self.eat(&TokKind::AmpAmp) {
            let rhs = self.parse_bit_or()?;
            lhs = Expr::Binary(BinOp::LogAnd, Box::new(lhs), Box::new(rhs));
        }
        Ok(lhs)
    }

    fn parse_bit_or(&mut self) -> Result<Expr, CCompileError> {
        let mut lhs = self.parse_bit_xor()?;
        while self.eat(&TokKind::Pipe) {
            let rhs = self.parse_bit_xor()?;
            lhs = Expr::Binary(BinOp::BitOr, Box::new(lhs), Box::new(rhs));
        }
        Ok(lhs)
    }

    fn parse_bit_xor(&mut self) -> Result<Expr, CCompileError> {
        let mut lhs = self.parse_bit_and()?;
        while self.eat(&TokKind::Caret) {
            let rhs = self.parse_bit_and()?;
            lhs = Expr::Binary(BinOp::BitXor, Box::new(lhs), Box::new(rhs));
        }
        Ok(lhs)
    }

    fn parse_bit_and(&mut self) -> Result<Expr, CCompileError> {
        let mut lhs = self.parse_eq()?;
        while self.eat(&TokKind::Amp) {
            let rhs = self.parse_eq()?;
            lhs = Expr::Binary(BinOp::BitAnd, Box::new(lhs), Box::new(rhs));
        }
        Ok(lhs)
    }

    fn parse_eq(&mut self) -> Result<Expr, CCompileError> {
        let mut lhs = self.parse_rel()?;
        loop {
            if self.eat(&TokKind::EqEq) {
                let rhs = self.parse_rel()?;
                lhs = Expr::Binary(BinOp::Eq, Box::new(lhs), Box::new(rhs));
            } else if self.eat(&TokKind::BangEq) {
                let rhs = self.parse_rel()?;
                lhs = Expr::Binary(BinOp::Ne, Box::new(lhs), Box::new(rhs));
            } else {
                break;
            }
        }
        Ok(lhs)
    }

    fn parse_rel(&mut self) -> Result<Expr, CCompileError> {
        let mut lhs = self.parse_shift()?;
        loop {
            if self.eat(&TokKind::Lt) {
                let rhs = self.parse_shift()?;
                lhs = Expr::Binary(BinOp::Lt, Box::new(lhs), Box::new(rhs));
            } else if self.eat(&TokKind::Le) {
                let rhs = self.parse_shift()?;
                lhs = Expr::Binary(BinOp::Le, Box::new(lhs), Box::new(rhs));
            } else if self.eat(&TokKind::Gt) {
                let rhs = self.parse_shift()?;
                lhs = Expr::Binary(BinOp::Gt, Box::new(lhs), Box::new(rhs));
            } else if self.eat(&TokKind::Ge) {
                let rhs = self.parse_shift()?;
                lhs = Expr::Binary(BinOp::Ge, Box::new(lhs), Box::new(rhs));
            } else {
                break;
            }
        }
        Ok(lhs)
    }

    fn parse_shift(&mut self) -> Result<Expr, CCompileError> {
        let mut lhs = self.parse_add()?;
        loop {
            if self.eat(&TokKind::Shl) {
                let rhs = self.parse_add()?;
                lhs = Expr::Binary(BinOp::Shl, Box::new(lhs), Box::new(rhs));
            } else if self.eat(&TokKind::Shr) {
                let rhs = self.parse_add()?;
                lhs = Expr::Binary(BinOp::Shr, Box::new(lhs), Box::new(rhs));
            } else {
                break;
            }
        }
        Ok(lhs)
    }

    fn parse_add(&mut self) -> Result<Expr, CCompileError> {
        let mut lhs = self.parse_mul()?;
        loop {
            if self.eat(&TokKind::Plus) {
                let rhs = self.parse_mul()?;
                lhs = Expr::Binary(BinOp::Add, Box::new(lhs), Box::new(rhs));
            } else if self.eat(&TokKind::Minus) {
                let rhs = self.parse_mul()?;
                lhs = Expr::Binary(BinOp::Sub, Box::new(lhs), Box::new(rhs));
            } else {
                break;
            }
        }
        Ok(lhs)
    }

    fn parse_mul(&mut self) -> Result<Expr, CCompileError> {
        let mut lhs = self.parse_unary()?;
        loop {
            if self.eat(&TokKind::Star) {
                let rhs = self.parse_unary()?;
                lhs = Expr::Binary(BinOp::Mul, Box::new(lhs), Box::new(rhs));
            } else if self.eat(&TokKind::Slash) {
                let rhs = self.parse_unary()?;
                lhs = Expr::Binary(BinOp::Div, Box::new(lhs), Box::new(rhs));
            } else if self.eat(&TokKind::Percent) {
                let rhs = self.parse_unary()?;
                lhs = Expr::Binary(BinOp::Rem, Box::new(lhs), Box::new(rhs));
            } else {
                break;
            }
        }
        Ok(lhs)
    }

    fn parse_unary(&mut self) -> Result<Expr, CCompileError> {
        if self.eat(&TokKind::Minus) {
            return Ok(Expr::Unary(UnaryOp::Neg, Box::new(self.parse_unary()?)));
        }
        if self.eat(&TokKind::Plus) {
            return self.parse_unary();
        }
        if self.eat(&TokKind::Tilde) {
            return Ok(Expr::Unary(UnaryOp::BitNot, Box::new(self.parse_unary()?)));
        }
        if self.eat(&TokKind::Bang) {
            return Ok(Expr::Unary(UnaryOp::LogNot, Box::new(self.parse_unary()?)));
        }
        if self.eat(&TokKind::Star) {
            return Ok(Expr::Deref(Box::new(self.parse_unary()?)));
        }
        if self.eat(&TokKind::Amp) {
            let id = self.expect_ident()?;
            return Ok(Expr::AddrOf(id));
        }
        if self.eat(&TokKind::PlusPlus) {
            let id = self.expect_ident()?;
            return Ok(Expr::AssignVar(
                id.clone(),
                Box::new(Expr::Binary(
                    BinOp::Add,
                    Box::new(Expr::Var(id)),
                    Box::new(Expr::Int(1)),
                )),
            ));
        }
        if self.eat(&TokKind::MinusMinus) {
            let id = self.expect_ident()?;
            return Ok(Expr::AssignVar(
                id.clone(),
                Box::new(Expr::Binary(
                    BinOp::Sub,
                    Box::new(Expr::Var(id)),
                    Box::new(Expr::Int(1)),
                )),
            ));
        }
        self.parse_postfix()
    }

    fn parse_postfix(&mut self) -> Result<Expr, CCompileError> {
        let mut expr = self.parse_primary()?;
        loop {
            if self.eat(&TokKind::LBracket) {
                let idx = self.parse_expr()?;
                self.expect(&TokKind::RBracket)?;
                expr = Expr::Index(Box::new(expr), Box::new(idx));
            } else if self.eat(&TokKind::PlusPlus) {
                if let Expr::Var(name) = expr {
                    expr = Expr::AssignVar(
                        name.clone(),
                        Box::new(Expr::Binary(
                            BinOp::Add,
                            Box::new(Expr::Var(name)),
                            Box::new(Expr::Int(1)),
                        )),
                    );
                }
            } else if self.eat(&TokKind::MinusMinus) {
                if let Expr::Var(name) = expr {
                    expr = Expr::AssignVar(
                        name.clone(),
                        Box::new(Expr::Binary(
                            BinOp::Sub,
                            Box::new(Expr::Var(name)),
                            Box::new(Expr::Int(1)),
                        )),
                    );
                }
            } else {
                break;
            }
        }
        Ok(expr)
    }

    fn parse_primary(&mut self) -> Result<Expr, CCompileError> {
        match &self.peek().kind {
            TokKind::IntLit(v) => {
                let val = *v;
                self.next();
                Ok(Expr::Int(val))
            }
            TokKind::CharLit(c) => {
                let val = *c as i64;
                self.next();
                Ok(Expr::Int(val))
            }
            TokKind::StrLit(s) => {
                let mut bytes = s.clone();
                self.next();
                while let TokKind::StrLit(more) = &self.peek().kind {
                    bytes.extend_from_slice(more);
                    self.next();
                }
                Ok(Expr::Str(bytes))
            }
            TokKind::Ident(id) => {
                let name = id.clone();
                self.next();
                if name == "sizeof" {
                    self.expect(&TokKind::LParen)?;
                    let sz = if self.is_type_start() {
                        let t = self.parse_type()?;
                        if t == CType::Byte {
                            1
                        } else {
                            8
                        }
                    } else {
                        let _ = self.parse_expr()?;
                        8
                    };
                    self.expect(&TokKind::RParen)?;
                    return Ok(Expr::Int(sz));
                }
                if self.eat(&TokKind::LParen) {
                    let mut args = Vec::new();
                    if !self.eat(&TokKind::RParen) {
                        loop {
                            args.push(self.parse_expr()?);
                            if !self.eat(&TokKind::Comma) {
                                break;
                            }
                        }
                        self.expect(&TokKind::RParen)?;
                    }
                    Ok(Expr::Call(name, args))
                } else {
                    Ok(Expr::Var(name))
                }
            }
            TokKind::LParen => {
                self.next();
                if self.is_type_start() {
                    let _ = self.parse_type()?;
                    self.expect(&TokKind::RParen)?;
                    return self.parse_unary();
                }
                let e = self.parse_expr()?;
                self.expect(&TokKind::RParen)?;
                Ok(e)
            }
            other => Err(CCompileError {
                line: self.line(),
                message: format!("unexpected token in expression: {:?}", other),
            }),
        }
    }
}

// RV64IM instruction encoders (little-endian 32-bit words)
fn enc_r(funct7: u32, rs2: u32, rs1: u32, funct3: u32, rd: u32, opcode: u32) -> u32 {
    (funct7 << 25) | (rs2 << 20) | (rs1 << 15) | (funct3 << 12) | (rd << 7) | opcode
}
fn enc_i(imm12: i32, rs1: u32, funct3: u32, rd: u32, opcode: u32) -> u32 {
    (((imm12 as u32) & 0xfff) << 20) | (rs1 << 15) | (funct3 << 12) | (rd << 7) | opcode
}
fn enc_s(imm12: i32, rs2: u32, rs1: u32, funct3: u32, opcode: u32) -> u32 {
    let u = (imm12 as u32) & 0xfff;
    ((u >> 5) << 25) | (rs2 << 20) | (rs1 << 15) | (funct3 << 12) | ((u & 0x1f) << 7) | opcode
}
fn enc_b(off: i32, rs2: u32, rs1: u32, funct3: u32, opcode: u32) -> u32 {
    let u = off as u32;
    let b12 = (u >> 12) & 1;
    let b11 = (u >> 11) & 1;
    let b10_5 = (u >> 5) & 0x3f;
    let b4_1 = (u >> 1) & 0xf;
    (b12 << 31)
        | (b10_5 << 25)
        | (rs2 << 20)
        | (rs1 << 15)
        | (funct3 << 12)
        | (b4_1 << 8)
        | (b11 << 7)
        | opcode
}
fn enc_u(imm20: u32, rd: u32, opcode: u32) -> u32 {
    ((imm20 & 0xfffff) << 12) | (rd << 7) | opcode
}
fn enc_j(off: i32, rd: u32, opcode: u32) -> u32 {
    let u = off as u32;
    let b20 = (u >> 20) & 1;
    let b10_1 = (u >> 1) & 0x3ff;
    let b11 = (u >> 11) & 1;
    let b19_12 = (u >> 12) & 0xff;
    (b20 << 31) | (b10_1 << 21) | (b11 << 20) | (b19_12 << 12) | (rd << 7) | opcode
}

const X0: u32 = 0;
const RA: u32 = 1;
const SP: u32 = 2;
const T0: u32 = 5;
const T1: u32 = 6;
const T2: u32 = 7;
const S0: u32 = 8;
const A0: u32 = 10;
const A1: u32 = 11;
const A2: u32 = 12;
const A7: u32 = 17;

#[derive(Clone, Copy)]
struct LocalSlot {
    offset_from_fp: i32,
    ty: CType,
    is_array: bool,
}

#[derive(Clone, Copy)]
enum FixupKind {
    CallFunc,
    LoadDataAddr,
}

struct Fixup {
    inst_idx: usize,
    kind: FixupKind,
    target: String,
    data_offset: usize,
    rd: u32,
}

struct Codegen {
    insts: Vec<u32>,
    data: Vec<u8>,
    global_offsets: HashMap<String, (usize, CType)>,
    func_inst_idx: HashMap<String, usize>,
    fixups: Vec<Fixup>,
    locals: HashMap<String, LocalSlot>,
    frame_size: i32,
    epilogue_jumps: Vec<usize>,
    break_jumps: Vec<Vec<usize>>,
}

impl Codegen {
    fn emit(&mut self, inst: u32) -> usize {
        let idx = self.insts.len();
        self.insts.push(inst);
        idx
    }

    fn emit_li(&mut self, rd: u32, val: i64) {
        if (-2048..=2047).contains(&val) {
            self.emit(enc_i(val as i32, X0, 0, rd, 0x13)); // addi rd, x0, imm
            return;
        }
        if (-0x8000_0000..=0x7fff_ffff).contains(&val) {
            let v32 = val as i32;
            let lo = (v32 << 20) >> 20;
            let hi = ((v32 - lo) >> 12) as u32;
            self.emit(enc_u(hi, rd, 0x37)); // lui rd, hi
            if lo != 0 {
                self.emit(enc_i(lo, rd, 0, rd, 0x13)); // addiw/addi
            }
            return;
        }
        // Full 64-bit constant construction via 12-bit chunks
        let hi32 = val >> 32;
        self.emit_li(rd, hi32);
        self.emit(enc_i(32, rd, 1, rd, 0x13)); // slli rd, rd, 32
        let lo32 = (val as u64) & 0xffff_ffff;
        let c2 = ((lo32 >> 21) & 0x7ff) as i32;
        let c1 = ((lo32 >> 10) & 0x7ff) as i32;
        let c0 = (lo32 & 0x3ff) as i32;
        self.emit_li(T2, c2 as i64);
        self.emit(enc_i(21, T2, 1, T2, 0x13));
        self.emit(enc_r(0, T2, rd, 6, rd, 0x33)); // or rd, rd, t2
        self.emit_li(T2, c1 as i64);
        self.emit(enc_i(10, T2, 1, T2, 0x13));
        self.emit(enc_r(0, T2, rd, 6, rd, 0x33));
        self.emit(enc_i(c0, rd, 6, rd, 0x13)); // ori rd, rd, c0
    }

    fn emit_fp_addr(&mut self, rd: u32, offset: i32) {
        if (-2048..=2047).contains(&offset) {
            self.emit(enc_i(offset, S0, 0, rd, 0x13));
        } else {
            self.emit_li(rd, offset as i64);
            self.emit(enc_r(0, S0, rd, 0, rd, 0x33));
        }
    }

    fn add_data_bytes(&mut self, bytes: &[u8], nul_terminate: bool) -> usize {
        while !self.data.len().is_multiple_of(8) {
            self.data.push(0);
        }
        let off = self.data.len();
        self.data.extend_from_slice(bytes);
        if nul_terminate {
            self.data.push(0);
        }
        off
    }

    fn emit_load_data_addr(&mut self, rd: u32, data_offset: usize) {
        let idx = self.emit(0);
        self.emit(0); // 2-instruction lui + addi slot
        self.fixups.push(Fixup {
            inst_idx: idx,
            kind: FixupKind::LoadDataAddr,
            target: String::new(),
            data_offset,
            rd,
        });
    }

    fn emit_call(&mut self, target: &str) {
        let idx = self.emit(0);
        self.fixups.push(Fixup {
            inst_idx: idx,
            kind: FixupKind::CallFunc,
            target: target.to_string(),
            data_offset: 0,
            rd: RA,
        });
    }

    fn push_a0(&mut self) {
        self.emit(enc_i(-8, SP, 0, SP, 0x13)); // addi sp, sp, -8
        self.emit(enc_s(0, A0, SP, 3, 0x23)); // sd a0, 0(sp)
    }

    fn pop_to(&mut self, reg: u32) {
        self.emit(enc_i(0, SP, 3, reg, 0x03)); // ld reg, 0(sp)
        self.emit(enc_i(8, SP, 0, SP, 0x13)); // addi sp, sp, 8
    }

    fn patch_b(&mut self, inst_idx: usize, target_idx: usize, rs2: u32, rs1: u32, funct3: u32) {
        let off = ((target_idx as isize) - (inst_idx as isize)) * 4;
        self.insts[inst_idx] = enc_b(off as i32, rs2, rs1, funct3, 0x63);
    }

    fn patch_j(&mut self, inst_idx: usize, target_idx: usize, rd: u32) {
        let off = ((target_idx as isize) - (inst_idx as isize)) * 4;
        self.insts[inst_idx] = enc_j(off as i32, rd, 0x6f);
    }

    fn emit_runtime_helpers(&mut self) {
        // __rt_putchar(a0 = byte): write 1 byte to stdout (fd=1)
        self.func_inst_idx
            .insert("__rt_putchar".to_string(), self.insts.len());
        self.emit(enc_i(-16, SP, 0, SP, 0x13));
        self.emit(enc_s(0, A0, SP, 0, 0x23)); // sb a0, 0(sp)
        self.emit(enc_i(1, X0, 0, A0, 0x13)); // a0 = 1
        self.emit(enc_i(0, SP, 0, A1, 0x13)); // a1 = sp
        self.emit(enc_i(1, X0, 0, A2, 0x13)); // a2 = 1
        self.emit(enc_i(64, X0, 0, A7, 0x13)); // a7 = SYS_write (64)
        self.emit(0x0000_0073); // ecall
        self.emit(enc_i(16, SP, 0, SP, 0x13));
        self.emit(enc_i(0, RA, 0, X0, 0x67)); // ret

        // __rt_write_slice(a0 = ptr, a1 = len)
        self.func_inst_idx
            .insert("__rt_write_slice".to_string(), self.insts.len());
        self.emit(enc_i(0, A1, 0, A2, 0x13)); // a2 = len
        self.emit(enc_i(0, A0, 0, A1, 0x13)); // a1 = ptr
        self.emit(enc_i(1, X0, 0, A0, 0x13)); // a0 = 1
        self.emit(enc_i(64, X0, 0, A7, 0x13)); // SYS_write
        self.emit(0x0000_0073);
        self.emit(enc_i(0, RA, 0, X0, 0x67));

        // __rt_puts_cstr(a0 = cstr, a1 = append_newline)
        self.func_inst_idx
            .insert("__rt_puts_cstr".to_string(), self.insts.len());
        self.emit(enc_i(-32, SP, 0, SP, 0x13));
        self.emit(enc_s(24, RA, SP, 3, 0x23));
        self.emit(enc_s(16, A0, SP, 3, 0x23));
        self.emit(enc_s(8, A1, SP, 3, 0x23));
        self.emit(enc_i(0, X0, 0, T1, 0x13)); // len = 0
        let loop_idx = self.insts.len();
        self.emit(enc_r(0, T1, A0, 0, T2, 0x33)); // t2 = a0 + len
        self.emit(enc_i(0, T2, 4, T0, 0x03)); // lbu t0, 0(t2)
        let br_done = self.emit(0);
        self.emit(enc_i(1, T1, 0, T1, 0x13)); // len++
        let j_back = self.emit(0);
        self.patch_j(j_back, loop_idx, X0);
        let done_idx = self.insts.len();
        self.patch_b(br_done, done_idx, X0, T0, 0); // beq t0, x0, done
        self.emit(enc_i(16, SP, 3, A1, 0x03)); // a1 = cstr
        self.emit(enc_i(0, T1, 0, A2, 0x13)); // a2 = len
        self.emit(enc_i(1, X0, 0, A0, 0x13)); // a0 = 1
        self.emit(enc_i(64, X0, 0, A7, 0x13));
        self.emit(0x0000_0073);
        self.emit(enc_i(8, SP, 3, T0, 0x03)); // append_newline?
        let skip_nl = self.emit(0);
        self.emit(enc_i(10, X0, 0, A0, 0x13)); // '\n'
        self.emit_call("__rt_putchar");
        let after_nl = self.insts.len();
        self.patch_b(skip_nl, after_nl, X0, T0, 0);
        self.emit(enc_i(24, SP, 3, RA, 0x03));
        self.emit(enc_i(32, SP, 0, SP, 0x13));
        self.emit(enc_i(0, RA, 0, X0, 0x67));

        // __rt_print_i64(a0 = signed 64-bit integer, a1 = is_signed (1 or 0))
        self.func_inst_idx
            .insert("__rt_print_i64".to_string(), self.insts.len());
        self.emit(enc_i(-48, SP, 0, SP, 0x13));
        self.emit(enc_s(40, RA, SP, 3, 0x23));
        // t0 = unsigned magnitude, t1 = is_negative
        self.emit(enc_i(0, A0, 0, T0, 0x13));
        self.emit(enc_i(0, X0, 0, T1, 0x13));
        let skip_sign = self.emit(0); // beq a1, x0, skip_sign
        let pos_branch = self.emit(0); // bge a0, x0, skip_sign
        self.emit(enc_r(0x20, A0, X0, 0, T0, 0x33)); // sub t0, x0, a0
        self.emit(enc_i(1, X0, 0, T1, 0x13)); // t1 = 1
        let mag_ready = self.insts.len();
        self.patch_b(skip_sign, mag_ready, X0, A1, 0);
        self.patch_b(pos_branch, mag_ready, X0, A0, 5); // bge a0, x0

        // t2 = cursor at sp + 32
        self.emit(enc_i(32, SP, 0, T2, 0x13));
        self.emit(enc_i(10, X0, 0, A2, 0x13)); // divisor = 10
        let zero_chk = self.emit(0); // bne t0, x0, dig_loop
        self.emit(enc_i(-1, T2, 0, T2, 0x13));
        self.emit(enc_i(b'0' as i32, X0, 0, A1, 0x13));
        self.emit(enc_s(0, A1, T2, 0, 0x23));
        let j_sign = self.emit(0);

        let dig_loop = self.insts.len();
        self.patch_b(zero_chk, dig_loop, X0, T0, 1); // bne t0, x0
        self.emit(enc_r(0x01, A2, T0, 7, A1, 0x33)); // remu a1, t0, a2
        self.emit(enc_r(0x01, A2, T0, 5, T0, 0x33)); // divu t0, t0, a2
        self.emit(enc_i(b'0' as i32, A1, 0, A1, 0x13));
        self.emit(enc_i(-1, T2, 0, T2, 0x13));
        self.emit(enc_s(0, A1, T2, 0, 0x23));
        let b_more = self.emit(0);
        self.patch_b(b_more, dig_loop, X0, T0, 1); // bne t0, x0, dig_loop

        let sign_chk = self.insts.len();
        self.patch_j(j_sign, sign_chk, X0);
        let b_no_minus = self.emit(0);
        self.emit(enc_i(-1, T2, 0, T2, 0x13));
        self.emit(enc_i(b'-' as i32, X0, 0, A1, 0x13));
        self.emit(enc_s(0, A1, T2, 0, 0x23));
        let emit_sys = self.insts.len();
        self.patch_b(b_no_minus, emit_sys, X0, T1, 0);
        // len = (sp + 32) - t2
        self.emit(enc_i(32, SP, 0, A2, 0x13));
        self.emit(enc_r(0x20, T2, A2, 0, A2, 0x33)); // sub a2, a2, t2
        self.emit(enc_i(0, T2, 0, A1, 0x13)); // a1 = buf
        self.emit(enc_i(1, X0, 0, A0, 0x13)); // fd = 1
        self.emit(enc_i(64, X0, 0, A7, 0x13)); // SYS_write
        self.emit(0x0000_0073);
        self.emit(enc_i(40, SP, 3, RA, 0x03));
        self.emit(enc_i(48, SP, 0, SP, 0x13));
        self.emit(enc_i(0, RA, 0, X0, 0x67));
    }

    fn count_stmt_slots(stmt: &Stmt) -> usize {
        match stmt {
            Stmt::Block(list) => list.iter().map(Self::count_stmt_slots).sum(),
            Stmt::Decl(decls) => decls
                .iter()
                .map(|d| d.array_elems.unwrap_or(1).max(1))
                .sum(),
            Stmt::If(_, t, e) => {
                Self::count_stmt_slots(t) + e.as_deref().map_or(0, Self::count_stmt_slots)
            }
            Stmt::While(_, b) => Self::count_stmt_slots(b),
            Stmt::For(i, _, _, b) => {
                i.as_deref().map_or(0, Self::count_stmt_slots) + Self::count_stmt_slots(b)
            }
            _ => 0,
        }
    }

    fn compile_func(&mut self, func: &FuncDef) -> Result<(), CCompileError> {
        self.func_inst_idx
            .insert(func.name.clone(), self.insts.len());
        self.locals.clear();
        self.epilogue_jumps.clear();
        self.break_jumps.clear();

        let local_slots: usize =
            func.params.len() + func.body.iter().map(Self::count_stmt_slots).sum::<usize>() + 4;
        let raw_bytes = ((local_slots + 2) * 8) as i32;
        self.frame_size = (raw_bytes + 15) & !15;

        // Prologue
        self.emit_li(T0, -self.frame_size as i64);
        self.emit(enc_r(0, T0, SP, 0, SP, 0x33)); // sp -= frame_size
        self.emit_li(T0, (self.frame_size - 8) as i64);
        self.emit(enc_r(0, T0, SP, 0, T1, 0x33));
        self.emit(enc_s(0, RA, T1, 3, 0x23)); // save ra
        self.emit(enc_s(-8, S0, T1, 3, 0x23)); // save s0
        self.emit_li(T0, self.frame_size as i64);
        self.emit(enc_r(0, T0, SP, 0, S0, 0x33)); // s0 = old sp

        let mut next_off = -24i32;
        for (idx, (pname, pty)) in func.params.iter().enumerate() {
            let slot = LocalSlot {
                offset_from_fp: next_off,
                ty: *pty,
                is_array: false,
            };
            self.locals.insert(pname.clone(), slot);
            if idx < 8 {
                let arg_reg = 10 + (idx as u32);
                self.emit_fp_addr(T0, next_off);
                self.emit(enc_s(0, arg_reg, T0, 3, 0x23));
            }
            next_off -= 8;
        }

        for s in &func.body {
            self.compile_stmt(s, &mut next_off)?;
        }

        // Default return 0
        self.emit(enc_i(0, X0, 0, A0, 0x13));
        let epilogue_idx = self.insts.len();
        let jumps = std::mem::take(&mut self.epilogue_jumps);
        for j in jumps {
            self.patch_j(j, epilogue_idx, X0);
        }

        self.emit_fp_addr(T0, -8);
        self.emit(enc_i(0, T0, 3, RA, 0x03));
        self.emit_fp_addr(T0, -16);
        self.emit(enc_i(0, T0, 3, T1, 0x03)); // saved s0 in t1
        self.emit(enc_i(0, S0, 0, SP, 0x13)); // sp = s0
        self.emit(enc_i(0, T1, 0, S0, 0x13)); // s0 = saved s0
        self.emit(enc_i(0, RA, 0, X0, 0x67)); // ret
        Ok(())
    }

    fn compile_stmt(&mut self, stmt: &Stmt, next_off: &mut i32) -> Result<(), CCompileError> {
        match stmt {
            Stmt::Empty => Ok(()),
            Stmt::Block(list) => {
                for s in list {
                    self.compile_stmt(s, next_off)?;
                }
                Ok(())
            }
            Stmt::Decl(decls) => {
                for d in decls {
                    if let Some(elems) = d.array_elems {
                        let bytes = ((elems * 8) as i32 + 7) & !7;
                        *next_off -= bytes;
                        let base_off = *next_off + 8;
                        self.locals.insert(
                            d.name.clone(),
                            LocalSlot {
                                offset_from_fp: base_off,
                                ty: d.ty,
                                is_array: true,
                            },
                        );
                        *next_off -= 8;
                    } else {
                        let off = *next_off;
                        *next_off -= 8;
                        self.locals.insert(
                            d.name.clone(),
                            LocalSlot {
                                offset_from_fp: off,
                                ty: d.ty,
                                is_array: false,
                            },
                        );
                        if let Some(init) = &d.init {
                            self.compile_expr(init)?;
                            self.emit_fp_addr(T0, off);
                            self.emit(enc_s(0, A0, T0, 3, 0x23));
                        }
                    }
                }
                Ok(())
            }
            Stmt::If(cond, then_s, else_s) => {
                self.compile_expr(cond)?;
                let br_false = self.emit(0);
                self.compile_stmt(then_s, next_off)?;
                if let Some(es) = else_s {
                    let j_end = self.emit(0);
                    let else_idx = self.insts.len();
                    self.patch_b(br_false, else_idx, X0, A0, 0);
                    self.compile_stmt(es, next_off)?;
                    let end_idx = self.insts.len();
                    self.patch_j(j_end, end_idx, X0);
                } else {
                    let end_idx = self.insts.len();
                    self.patch_b(br_false, end_idx, X0, A0, 0);
                }
                Ok(())
            }
            Stmt::While(cond, body) => {
                self.break_jumps.push(Vec::new());
                let top = self.insts.len();
                self.compile_expr(cond)?;
                let br_exit = self.emit(0);
                self.compile_stmt(body, next_off)?;
                let j_top = self.emit(0);
                self.patch_j(j_top, top, X0);
                let exit_idx = self.insts.len();
                self.patch_b(br_exit, exit_idx, X0, A0, 0);
                if let Some(bjs) = self.break_jumps.pop() {
                    for bj in bjs {
                        self.patch_j(bj, exit_idx, X0);
                    }
                }
                Ok(())
            }
            Stmt::For(init, cond, step, body) => {
                if let Some(init_s) = init {
                    self.compile_stmt(init_s, next_off)?;
                }
                self.break_jumps.push(Vec::new());
                let top = self.insts.len();
                let br_exit = if let Some(c) = cond {
                    self.compile_expr(c)?;
                    Some(self.emit(0))
                } else {
                    None
                };
                self.compile_stmt(body, next_off)?;
                if let Some(st) = step {
                    self.compile_expr(st)?;
                }
                let j_top = self.emit(0);
                self.patch_j(j_top, top, X0);
                let exit_idx = self.insts.len();
                if let Some(be) = br_exit {
                    self.patch_b(be, exit_idx, X0, A0, 0);
                }
                if let Some(bjs) = self.break_jumps.pop() {
                    for bj in bjs {
                        self.patch_j(bj, exit_idx, X0);
                    }
                }
                Ok(())
            }
            Stmt::Break => {
                let j = self.emit(0);
                if let Some(last) = self.break_jumps.last_mut() {
                    last.push(j);
                }
                Ok(())
            }
            Stmt::Return(opt_e) => {
                if let Some(e) = opt_e {
                    self.compile_expr(e)?;
                } else {
                    self.emit(enc_i(0, X0, 0, A0, 0x13));
                }
                let j = self.emit(0);
                self.epilogue_jumps.push(j);
                Ok(())
            }
            Stmt::ExprStmt(e) => self.compile_expr(e),
        }
    }

    fn expr_elem_stride(&self, base: &Expr) -> i64 {
        if let Expr::Var(name) = base {
            if let Some(loc) = self.locals.get(name) {
                if loc.ty == CType::PtrByte || (loc.is_array && loc.ty == CType::Byte) {
                    return 1;
                }
            }
        }
        8
    }

    fn compile_printf(&mut self, args: &[Expr]) -> Result<(), CCompileError> {
        if args.is_empty() {
            self.emit(enc_i(0, X0, 0, A0, 0x13));
            return Ok(());
        }
        let fmt_bytes = match &args[0] {
            Expr::Str(b) => b.clone(),
            other => {
                self.compile_expr(other)?;
                self.emit(enc_i(0, X0, 0, A1, 0x13));
                self.emit_call("__rt_puts_cstr");
                return Ok(());
            }
        };
        let val_args = &args[1..];
        for arg in val_args {
            self.compile_expr(arg)?;
            self.push_a0();
        }
        let n_vals = val_args.len();
        let mut cur_val = 0usize;
        let mut lit_start = 0usize;
        let mut i = 0usize;
        while i < fmt_bytes.len() {
            if fmt_bytes[i] == b'%' && i + 1 < fmt_bytes.len() {
                if lit_start < i {
                    let off = self.add_data_bytes(&fmt_bytes[lit_start..i], false);
                    let slice_len = (i - lit_start) as i64;
                    self.emit_load_data_addr(A0, off);
                    self.emit_li(A1, slice_len);
                    self.emit_call("__rt_write_slice");
                }
                i += 1;
                if fmt_bytes[i] == b'%' {
                    self.emit_li(A0, b'%' as i64);
                    self.emit_call("__rt_putchar");
                    i += 1;
                    lit_start = i;
                    continue;
                }
                if fmt_bytes[i] == b'l' && i + 1 < fmt_bytes.len() {
                    i += 1;
                }
                let spec = fmt_bytes[i];
                i += 1;
                lit_start = i;
                if cur_val < n_vals {
                    let stack_off = ((n_vals - 1 - cur_val) * 8) as i32;
                    self.emit(enc_i(stack_off, SP, 3, A0, 0x03));
                    cur_val += 1;
                    match spec {
                        b'd' | b'i' => {
                            self.emit(enc_i(1, X0, 0, A1, 0x13));
                            self.emit_call("__rt_print_i64");
                        }
                        b'u' => {
                            self.emit(enc_i(0, X0, 0, A1, 0x13));
                            self.emit_call("__rt_print_i64");
                        }
                        b'c' => {
                            self.emit_call("__rt_putchar");
                        }
                        b's' => {
                            self.emit(enc_i(0, X0, 0, A1, 0x13));
                            self.emit_call("__rt_puts_cstr");
                        }
                        _ => {
                            self.emit(enc_i(1, X0, 0, A1, 0x13));
                            self.emit_call("__rt_print_i64");
                        }
                    }
                }
            } else {
                i += 1;
            }
        }
        if lit_start < fmt_bytes.len() {
            let off = self.add_data_bytes(&fmt_bytes[lit_start..], false);
            let slice_len = (fmt_bytes.len() - lit_start) as i64;
            self.emit_load_data_addr(A0, off);
            self.emit_li(A1, slice_len);
            self.emit_call("__rt_write_slice");
        }
        if n_vals > 0 {
            self.emit_li(T0, (n_vals * 8) as i64);
            self.emit(enc_r(0, T0, SP, 0, SP, 0x33));
        }
        self.emit(enc_i(0, X0, 0, A0, 0x13));
        Ok(())
    }

    fn compile_expr(&mut self, expr: &Expr) -> Result<(), CCompileError> {
        match expr {
            Expr::Int(v) => {
                self.emit_li(A0, *v);
                Ok(())
            }
            Expr::Str(bytes) => {
                let off = self.add_data_bytes(bytes, true);
                self.emit_load_data_addr(A0, off);
                Ok(())
            }
            Expr::Var(name) => {
                if let Some(loc) = self.locals.get(name).copied() {
                    self.emit_fp_addr(A0, loc.offset_from_fp);
                    if !loc.is_array {
                        self.emit(enc_i(0, A0, 3, A0, 0x03)); // ld a0, 0(a0)
                    }
                    Ok(())
                } else if let Some((goff, _)) = self.global_offsets.get(name).copied() {
                    self.emit_load_data_addr(A0, goff);
                    self.emit(enc_i(0, A0, 3, A0, 0x03));
                    Ok(())
                } else {
                    Err(CCompileError {
                        line: 1,
                        message: format!("undefined variable `{name}`"),
                    })
                }
            }
            Expr::AddrOf(name) => {
                if let Some(loc) = self.locals.get(name).copied() {
                    self.emit_fp_addr(A0, loc.offset_from_fp);
                    Ok(())
                } else if let Some((goff, _)) = self.global_offsets.get(name).copied() {
                    self.emit_load_data_addr(A0, goff);
                    Ok(())
                } else {
                    Err(CCompileError {
                        line: 1,
                        message: format!("undefined variable `&{name}`"),
                    })
                }
            }
            Expr::Deref(inner) => {
                self.compile_expr(inner)?;
                self.emit(enc_i(0, A0, 3, A0, 0x03));
                Ok(())
            }
            Expr::Index(base, idx) => {
                let stride = self.expr_elem_stride(base);
                self.compile_expr(base)?;
                self.push_a0();
                self.compile_expr(idx)?;
                if stride == 8 {
                    self.emit(enc_i(3, A0, 1, A0, 0x13)); // slli a0, a0, 3
                }
                self.pop_to(T0);
                self.emit(enc_r(0, A0, T0, 0, T0, 0x33)); // t0 = base + scaled_idx
                if stride == 1 {
                    self.emit(enc_i(0, T0, 4, A0, 0x03)); // lbu a0, 0(t0)
                } else {
                    self.emit(enc_i(0, T0, 3, A0, 0x03)); // ld a0, 0(t0)
                }
                Ok(())
            }
            Expr::AssignVar(name, rhs) => {
                self.compile_expr(rhs)?;
                if let Some(loc) = self.locals.get(name).copied() {
                    self.emit_fp_addr(T0, loc.offset_from_fp);
                    self.emit(enc_s(0, A0, T0, 3, 0x23));
                    Ok(())
                } else if let Some((goff, _)) = self.global_offsets.get(name).copied() {
                    self.emit_load_data_addr(T0, goff);
                    self.emit(enc_s(0, A0, T0, 3, 0x23));
                    Ok(())
                } else {
                    Err(CCompileError {
                        line: 1,
                        message: format!("assignment to undefined variable `{name}`"),
                    })
                }
            }
            Expr::AssignIndex(base, idx, rhs) => {
                let stride = self.expr_elem_stride(base);
                self.compile_expr(base)?;
                self.push_a0();
                self.compile_expr(idx)?;
                if stride == 8 {
                    self.emit(enc_i(3, A0, 1, A0, 0x13));
                }
                self.pop_to(T0);
                self.emit(enc_r(0, A0, T0, 0, A0, 0x33));
                self.push_a0(); // save element address
                self.compile_expr(rhs)?;
                self.pop_to(T0);
                if stride == 1 {
                    self.emit(enc_s(0, A0, T0, 0, 0x23)); // sb
                } else {
                    self.emit(enc_s(0, A0, T0, 3, 0x23)); // sd
                }
                Ok(())
            }
            Expr::AssignDeref(ptr, rhs) => {
                self.compile_expr(ptr)?;
                self.push_a0();
                self.compile_expr(rhs)?;
                self.pop_to(T0);
                self.emit(enc_s(0, A0, T0, 3, 0x23));
                Ok(())
            }
            Expr::Unary(op, inner) => {
                self.compile_expr(inner)?;
                match op {
                    UnaryOp::Neg => {
                        self.emit(enc_r(0x20, A0, X0, 0, A0, 0x33)); // sub a0, x0, a0
                    }
                    UnaryOp::BitNot => {
                        self.emit(enc_i(-1, A0, 4, A0, 0x13)); // xori a0, a0, -1
                    }
                    UnaryOp::LogNot => {
                        self.emit(enc_i(1, A0, 3, A0, 0x13)); // sltiu a0, a0, 1 (seqz)
                    }
                }
                Ok(())
            }
            Expr::Ternary(c, t, f) => {
                self.compile_expr(c)?;
                let br_f = self.emit(0);
                self.compile_expr(t)?;
                let j_end = self.emit(0);
                let f_idx = self.insts.len();
                self.patch_b(br_f, f_idx, X0, A0, 0);
                self.compile_expr(f)?;
                let end_idx = self.insts.len();
                self.patch_j(j_end, end_idx, X0);
                Ok(())
            }
            Expr::Binary(BinOp::LogAnd, l, r) => {
                self.compile_expr(l)?;
                let br_false1 = self.emit(0);
                self.compile_expr(r)?;
                let br_false2 = self.emit(0);
                self.emit(enc_i(1, X0, 0, A0, 0x13));
                let j_end = self.emit(0);
                let false_idx = self.insts.len();
                self.patch_b(br_false1, false_idx, X0, A0, 0);
                self.patch_b(br_false2, false_idx, X0, A0, 0);
                self.emit(enc_i(0, X0, 0, A0, 0x13));
                let end_idx = self.insts.len();
                self.patch_j(j_end, end_idx, X0);
                Ok(())
            }
            Expr::Binary(BinOp::LogOr, l, r) => {
                self.compile_expr(l)?;
                let br_true1 = self.emit(0);
                self.compile_expr(r)?;
                let br_true2 = self.emit(0);
                self.emit(enc_i(0, X0, 0, A0, 0x13));
                let j_end = self.emit(0);
                let true_idx = self.insts.len();
                self.patch_b(br_true1, true_idx, X0, A0, 1); // bne a0, x0
                self.patch_b(br_true2, true_idx, X0, A0, 1);
                self.emit(enc_i(1, X0, 0, A0, 0x13));
                let end_idx = self.insts.len();
                self.patch_j(j_end, end_idx, X0);
                Ok(())
            }
            Expr::Binary(op, l, r) => {
                self.compile_expr(l)?;
                self.push_a0();
                self.compile_expr(r)?;
                self.emit(enc_i(0, A0, 0, A1, 0x13)); // a1 = right
                self.pop_to(A0); // a0 = left
                match op {
                    BinOp::Add => {
                        self.emit(enc_r(0, A1, A0, 0, A0, 0x33));
                    }
                    BinOp::Sub => {
                        self.emit(enc_r(0x20, A1, A0, 0, A0, 0x33));
                    }
                    BinOp::Mul => {
                        self.emit(enc_r(0x01, A1, A0, 0, A0, 0x33));
                    }
                    BinOp::Div => {
                        self.emit(enc_r(0x01, A1, A0, 4, A0, 0x33));
                    }
                    BinOp::Rem => {
                        self.emit(enc_r(0x01, A1, A0, 6, A0, 0x33));
                    }
                    BinOp::Shl => {
                        self.emit(enc_r(0, A1, A0, 1, A0, 0x33));
                    }
                    BinOp::Shr => {
                        self.emit(enc_r(0x20, A1, A0, 5, A0, 0x33));
                    }
                    BinOp::BitAnd => {
                        self.emit(enc_r(0, A1, A0, 7, A0, 0x33));
                    }
                    BinOp::BitOr => {
                        self.emit(enc_r(0, A1, A0, 6, A0, 0x33));
                    }
                    BinOp::BitXor => {
                        self.emit(enc_r(0, A1, A0, 4, A0, 0x33));
                    }
                    BinOp::Eq => {
                        self.emit(enc_r(0x20, A1, A0, 0, A0, 0x33)); // sub
                        self.emit(enc_i(1, A0, 3, A0, 0x13)); // seqz
                    }
                    BinOp::Ne => {
                        self.emit(enc_r(0x20, A1, A0, 0, A0, 0x33));
                        self.emit(enc_r(0, A0, X0, 3, A0, 0x33)); // sltu a0, x0, a0 (snez)
                    }
                    BinOp::Lt => {
                        self.emit(enc_r(0, A1, A0, 2, A0, 0x33)); // slt
                    }
                    BinOp::Gt => {
                        self.emit(enc_r(0, A0, A1, 2, A0, 0x33)); // slt a0, a1, a0
                    }
                    BinOp::Le => {
                        self.emit(enc_r(0, A0, A1, 2, A0, 0x33)); // a1 < a0
                        self.emit(enc_i(1, A0, 4, A0, 0x13)); // xori a0, a0, 1
                    }
                    BinOp::Ge => {
                        self.emit(enc_r(0, A1, A0, 2, A0, 0x33)); // a0 < a1
                        self.emit(enc_i(1, A0, 4, A0, 0x13)); // xori a0, a0, 1
                    }
                    BinOp::LogAnd | BinOp::LogOr => {}
                }
                Ok(())
            }
            Expr::Call(name, args) => {
                if name == "printf" {
                    return self.compile_printf(args);
                }
                if name == "puts" && args.len() == 1 {
                    self.compile_expr(&args[0])?;
                    self.emit(enc_i(1, X0, 0, A1, 0x13));
                    self.emit_call("__rt_puts_cstr");
                    return Ok(());
                }
                if name == "putchar" && args.len() == 1 {
                    self.compile_expr(&args[0])?;
                    self.emit_call("__rt_putchar");
                    return Ok(());
                }
                if name == "exit" && args.len() == 1 {
                    self.compile_expr(&args[0])?;
                    self.emit(enc_i(94, X0, 0, A7, 0x13));
                    self.emit(0x0000_0073);
                    return Ok(());
                }
                if name == "write" && args.len() == 3 {
                    for a in args {
                        self.compile_expr(a)?;
                        self.push_a0();
                    }
                    self.pop_to(A2);
                    self.pop_to(A1);
                    self.pop_to(A0);
                    self.emit(enc_i(64, X0, 0, A7, 0x13));
                    self.emit(0x0000_0073);
                    return Ok(());
                }
                if args.len() > 8 {
                    return Err(CCompileError {
                        line: 1,
                        message: format!("call to `{name}` exceeds 8 register arguments"),
                    });
                }
                for a in args {
                    self.compile_expr(a)?;
                    self.push_a0();
                }
                for idx in (0..args.len()).rev() {
                    self.pop_to(10 + (idx as u32));
                }
                self.emit_call(name);
                Ok(())
            }
        }
    }
}

/// Compile a C translation unit into a static 64-bit `EM_RISCV` (`243`) ELF64 executable.
pub fn compile_c_to_rv64_elf(source: &str) -> Result<Vec<u8>, CCompileError> {
    let toks = preprocess_and_lex(source)?;
    let mut parser = Parser { toks, pos: 0 };
    let prog = parser.parse_program()?;

    if !prog.funcs.iter().any(|f| f.name == "main") {
        return Err(CCompileError {
            line: 1,
            message: "missing `main` entry function".to_string(),
        });
    }

    let mut cg = Codegen {
        insts: Vec::new(),
        data: Vec::new(),
        global_offsets: HashMap::new(),
        func_inst_idx: HashMap::new(),
        fixups: Vec::new(),
        locals: HashMap::new(),
        frame_size: 64,
        epilogue_jumps: Vec::new(),
        break_jumps: Vec::new(),
    };

    for g in &prog.globals {
        while !cg.data.len().is_multiple_of(8) {
            cg.data.push(0);
        }
        let off = cg.data.len();
        cg.data.extend_from_slice(&g.init_i64.to_le_bytes());
        while cg.data.len() < off + g.size_bytes {
            cg.data.push(0);
        }
        cg.global_offsets
            .insert(g.name.clone(), (off, CType::Word64));
    }

    // _start at text[0]: Load argc/argv from initial sp, call main, SYS_exit_group(a0)
    cg.emit(enc_i(0, SP, 3, A0, 0x03)); // ld a0, 0(sp)  (argc)
    cg.emit(enc_i(8, SP, 0, A1, 0x13)); // addi a1, sp, 8 (argv)
    cg.emit_call("main");
    cg.emit(enc_i(94, X0, 0, A7, 0x13)); // SYS_exit_group
    cg.emit(0x0000_0073); // ecall

    cg.emit_runtime_helpers();
    for f in &prog.funcs {
        cg.compile_func(f)?;
    }

    let text_bytes_len = cg.insts.len() * 4;
    let data_file_offset = (RV64_TEXT_OFFSET + text_bytes_len + 7) & !7;
    let data_vaddr = RV64_BASE_VADDR + (data_file_offset as u64);

    let fixups = std::mem::take(&mut cg.fixups);
    for fx in fixups {
        match fx.kind {
            FixupKind::CallFunc => {
                let Some(&target_idx) = cg.func_inst_idx.get(&fx.target) else {
                    return Err(CCompileError {
                        line: 1,
                        message: format!("undefined function `{}`", fx.target),
                    });
                };
                let off = ((target_idx as isize) - (fx.inst_idx as isize)) * 4;
                cg.insts[fx.inst_idx] = enc_j(off as i32, fx.rd, 0x6f);
            }
            FixupKind::LoadDataAddr => {
                let addr = (data_vaddr + (fx.data_offset as u64)) as i32;
                let lo = (addr << 20) >> 20;
                let hi = ((addr - lo) >> 12) as u32;
                cg.insts[fx.inst_idx] = enc_u(hi, fx.rd, 0x37);
                cg.insts[fx.inst_idx + 1] = enc_i(lo, fx.rd, 0, fx.rd, 0x13);
            }
        }
    }

    let total_file_size = data_file_offset + cg.data.len();
    let mut elf = vec![0u8; total_file_size];

    // ELF64 Header (64 bytes)
    elf[0..4].copy_from_slice(b"\x7fELF");
    elf[4] = 2; // ELFCLASS64
    elf[5] = 1; // ELFDATA2LSB
    elf[6] = 1; // EV_CURRENT
    elf[16..18].copy_from_slice(&2u16.to_le_bytes()); // ET_EXEC
    elf[18..20].copy_from_slice(&243u16.to_le_bytes()); // EM_RISCV (243)
    elf[20..24].copy_from_slice(&1u32.to_le_bytes()); // e_version = 1
    let entry_vaddr = RV64_BASE_VADDR + (RV64_TEXT_OFFSET as u64);
    elf[24..32].copy_from_slice(&entry_vaddr.to_le_bytes()); // e_entry
    elf[32..40].copy_from_slice(&64u64.to_le_bytes()); // e_phoff = 64
    elf[52..54].copy_from_slice(&64u16.to_le_bytes()); // e_ehsize
    elf[54..56].copy_from_slice(&56u16.to_le_bytes()); // e_phentsize
    elf[56..58].copy_from_slice(&1u16.to_le_bytes()); // e_phnum = 1

    // ELF64 Program Header (56 bytes at offset 64)
    elf[64..68].copy_from_slice(&1u32.to_le_bytes()); // PT_LOAD
    elf[68..72].copy_from_slice(&7u32.to_le_bytes()); // PF_R | PF_W | PF_X
    elf[72..80].copy_from_slice(&0u64.to_le_bytes()); // p_offset = 0
    elf[80..88].copy_from_slice(&RV64_BASE_VADDR.to_le_bytes()); // p_vaddr
    elf[88..96].copy_from_slice(&RV64_BASE_VADDR.to_le_bytes()); // p_paddr
    let filesz = total_file_size as u64;
    let memsz = filesz + 4096;
    elf[96..104].copy_from_slice(&filesz.to_le_bytes()); // p_filesz
    elf[104..112].copy_from_slice(&memsz.to_le_bytes()); // p_memsz
    elf[112..120].copy_from_slice(&4096u64.to_le_bytes()); // p_align

    for (idx, &inst) in cg.insts.iter().enumerate() {
        let off = RV64_TEXT_OFFSET + idx * 4;
        elf[off..off + 4].copy_from_slice(&inst.to_le_bytes());
    }
    elf[data_file_offset..data_file_offset + cg.data.len()].copy_from_slice(&cg.data);

    Ok(elf)
}
