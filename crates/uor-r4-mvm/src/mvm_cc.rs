//! In-sandbox C-to-RV64IM ELF64 compiler (`R4-37`).
//!
//! Compiles freestanding/POSIX subset C source files directly inside WebAssembly
//! into genuine 64-bit `EM_RISCV` (`243`) static ELF64 executables that run on the
//! `MVM-Wasm` `RV64IMAC` processor via Linux `riscv64` `ecall` system calls.

use std::collections::{HashMap, HashSet};

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
    Dot,
    Arrow,
    Ellipsis,
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

/// Formal R4.MvmLinuxContainer join backslash-continued physical source lines and strip C comments outside string literals (`R4-63`).
fn join_and_strip_c_comments(src: &str) -> String {
    let mut joined = String::with_capacity(src.len());
    let phys_lines: Vec<&str> = src.lines().collect();
    let mut idx = 0usize;
    while idx < phys_lines.len() {
        let mut cur = phys_lines[idx].trim_end_matches('\r').to_string();
        let mut extra_nls = 0usize;
        while cur.ends_with('\\') && idx + 1 < phys_lines.len() {
            cur.pop();
            idx += 1;
            cur.push_str(phys_lines[idx].trim_end_matches('\r'));
            extra_nls += 1;
        }
        joined.push_str(&cur);
        joined.push('\n');
        for _ in 0..extra_nls {
            joined.push('\n');
        }
        idx += 1;
    }

    let bytes = joined.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0usize;
    let mut in_str = false;
    let mut in_chr = false;
    while i < bytes.len() {
        let b = bytes[i];
        if in_str {
            out.push(b);
            if b == b'\\' && i + 1 < bytes.len() {
                i += 1;
                out.push(bytes[i]);
            } else if b == b'"' {
                in_str = false;
            }
            i += 1;
            continue;
        }
        if in_chr {
            out.push(b);
            if b == b'\\' && i + 1 < bytes.len() {
                i += 1;
                out.push(bytes[i]);
            } else if b == b'\'' {
                in_chr = false;
            }
            i += 1;
            continue;
        }
        if b == b'"' {
            in_str = true;
            out.push(b'"');
            i += 1;
            continue;
        }
        if b == b'\'' {
            in_chr = true;
            out.push(b'\'');
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
                    out.push(b'\n');
                }
                i += 1;
            }
            i = (i + 2).min(bytes.len());
            out.push(b' ');
            continue;
        }
        out.push(b);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

/// Formal R4.MvmLinuxContainer normalize a VFS path cleanly resolving `.` and `..` segments (`R4-63`).
fn normalize_vfs_path(base_dir: &str, rel_or_abs: &str) -> String {
    let joined = if rel_or_abs.starts_with('/') {
        rel_or_abs.to_string()
    } else if base_dir == "/" {
        format!("/{rel_or_abs}")
    } else {
        format!("{base_dir}/{rel_or_abs}")
    };
    let mut stack: Vec<&str> = Vec::new();
    for seg in joined.split('/') {
        if seg.is_empty() || seg == "." {
            continue;
        }
        if seg == ".." {
            stack.pop();
        } else {
            stack.push(seg);
        }
    }
    if stack.is_empty() {
        "/".to_string()
    } else {
        format!("/{}", stack.join("/"))
    }
}

/// Formal R4.MvmLinuxContainer recursively preprocess C `#include`, `#pragma once`, `#define`, and `#ifdef`/`#ifndef` conditionals (`R4-63`).
#[allow(clippy::too_many_arguments)]
fn preprocess_unit_recursive<F>(
    cur_path: &str,
    src: &str,
    include_dirs: &[String],
    defines: &mut HashMap<String, String>,
    once_headers: &mut HashSet<String>,
    read_header: &F,
    depth: usize,
    out_text: &mut String,
) -> Result<(), CCompileError>
where
    F: Fn(&str) -> Option<String>,
{
    if depth > 16 {
        return Err(CCompileError {
            line: 1,
            message: format!("#include nesting depth exceeded 16 in `{cur_path}`"),
        });
    }
    let clean = join_and_strip_c_comments(src);
    let cur_dir = match cur_path.rsplit_once('/') {
        Some((d, _)) if !d.is_empty() => d,
        _ => "/workspace",
    };

    #[derive(Clone, Copy)]
    struct CondFrame {
        parent_active: bool,
        this_active: bool,
        branch_taken: bool,
    }
    let mut cond_stack: Vec<CondFrame> = Vec::new();
    let is_active = |st: &[CondFrame]| st.last().is_none_or(|f| f.this_active);

    for (lidx, raw_line) in clean.lines().enumerate() {
        let line_no = lidx + 1;
        let trimmed = raw_line.trim_start();
        if let Some(after_hash) = trimmed.strip_prefix('#') {
            let dir = after_hash.trim();
            if let Some(sym) = dir.strip_prefix("ifdef ") {
                let name = sym.split_whitespace().next().unwrap_or("");
                let parent = is_active(&cond_stack);
                let taken = defines.contains_key(name);
                cond_stack.push(CondFrame {
                    parent_active: parent,
                    this_active: parent && taken,
                    branch_taken: taken,
                });
                out_text.push('\n');
                continue;
            }
            if let Some(sym) = dir.strip_prefix("ifndef ") {
                let name = sym.split_whitespace().next().unwrap_or("");
                let parent = is_active(&cond_stack);
                let taken = !defines.contains_key(name);
                cond_stack.push(CondFrame {
                    parent_active: parent,
                    this_active: parent && taken,
                    branch_taken: taken,
                });
                out_text.push('\n');
                continue;
            }
            if let Some(expr) = dir.strip_prefix("if ") {
                let e = expr.trim();
                let parent = is_active(&cond_stack);
                let taken = if let Some(sym) = e.strip_prefix("defined") {
                    let s = sym
                        .trim()
                        .trim_start_matches('(')
                        .trim_end_matches(')')
                        .trim();
                    defines.contains_key(s)
                } else if let Some(sym) = e.strip_prefix("!defined") {
                    let s = sym
                        .trim()
                        .trim_start_matches('(')
                        .trim_end_matches(')')
                        .trim();
                    !defines.contains_key(s)
                } else {
                    e != "0"
                };
                cond_stack.push(CondFrame {
                    parent_active: parent,
                    this_active: parent && taken,
                    branch_taken: taken,
                });
                out_text.push('\n');
                continue;
            }
            if dir == "else" || dir.starts_with("else ") {
                if let Some(top) = cond_stack.last_mut() {
                    let next_active = top.parent_active && !top.branch_taken;
                    top.this_active = next_active;
                    top.branch_taken = true;
                }
                out_text.push('\n');
                continue;
            }
            if dir == "endif" || dir.starts_with("endif ") {
                cond_stack.pop();
                out_text.push('\n');
                continue;
            }
            if !is_active(&cond_stack) {
                out_text.push('\n');
                continue;
            }
            if dir == "pragma once" {
                once_headers.insert(cur_path.to_string());
                out_text.push('\n');
                continue;
            }
            if let Some(def_rest) = dir.strip_prefix("define ") {
                let trimmed_def = def_rest.trim();
                let mut split_at = 0usize;
                for (bidx, ch) in trimmed_def.char_indices() {
                    if ch.is_ascii_alphanumeric() || ch == '_' {
                        split_at = bidx + ch.len_utf8();
                    } else {
                        break;
                    }
                }
                let name = &trimmed_def[..split_at];
                let remainder = &trimmed_def[split_at..];
                if !name.is_empty() && !remainder.starts_with('(') {
                    let val = remainder.trim();
                    defines.insert(
                        name.to_string(),
                        if val.is_empty() {
                            "1".to_string()
                        } else {
                            val.to_string()
                        },
                    );
                }
                out_text.push('\n');
                continue;
            }
            if let Some(undef_rest) = dir.strip_prefix("undef ") {
                let name = undef_rest.split_whitespace().next().unwrap_or("");
                defines.remove(name);
                out_text.push('\n');
                continue;
            }
            if let Some(inc_rest) = dir.strip_prefix("include") {
                let arg = inc_rest.trim();
                let (quoted, header_rel) = if let Some(s) = arg.strip_prefix('"') {
                    let Some(end) = s.find('"') else {
                        return Err(CCompileError {
                            line: line_no,
                            message: "unterminated #include \"...\"".to_string(),
                        });
                    };
                    (true, &s[..end])
                } else if let Some(s) = arg.strip_prefix('<') {
                    let Some(end) = s.find('>') else {
                        return Err(CCompileError {
                            line: line_no,
                            message: "unterminated #include <...>".to_string(),
                        });
                    };
                    (false, &s[..end])
                } else {
                    return Err(CCompileError {
                        line: line_no,
                        message: format!("malformed #include `{arg}`"),
                    });
                };

                let mut search_paths: Vec<String> = Vec::new();
                if quoted {
                    search_paths.push(normalize_vfs_path(cur_dir, header_rel));
                }
                for inc_d in include_dirs {
                    let base = normalize_vfs_path("/workspace", inc_d);
                    search_paths.push(normalize_vfs_path(&base, header_rel));
                }
                search_paths.push(normalize_vfs_path("/usr/include", header_rel));

                let mut resolved: Option<(String, String)> = None;
                for cand in search_paths {
                    if let Some(txt) = read_header(&cand) {
                        resolved = Some((cand, txt));
                        break;
                    }
                }
                let Some((hdr_path, hdr_src)) = resolved else {
                    return Err(CCompileError {
                        line: line_no,
                        message: format!("header not found: `{header_rel}`"),
                    });
                };
                if !once_headers.contains(&hdr_path) {
                    preprocess_unit_recursive(
                        &hdr_path,
                        &hdr_src,
                        include_dirs,
                        defines,
                        once_headers,
                        read_header,
                        depth + 1,
                        out_text,
                    )?;
                }
                out_text.push('\n');
                continue;
            }
            out_text.push('\n');
        } else if is_active(&cond_stack) {
            out_text.push_str(raw_line);
            out_text.push('\n');
        } else {
            out_text.push('\n');
        }
    }
    Ok(())
}

/// Formal R4.MvmLinuxContainer lex preprocessed C source text with bounded object-like macro expansion (`R4-63`).
fn lex_c_tokens_with_defines(
    cleaned: &str,
    defines: &HashMap<String, String>,
    base_line: usize,
    expand_depth: usize,
) -> Result<Vec<Tok>, CCompileError> {
    let bytes = cleaned.as_bytes();
    let mut i = 0usize;
    let mut line = base_line;
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
            if expand_depth < 8 {
                if let Some(rep) = defines.get(ident) {
                    let sub_toks = lex_c_tokens_with_defines(rep, defines, line, expand_depth + 1)?;
                    for st in sub_toks {
                        if st.kind != TokKind::Eof {
                            toks.push(st);
                        }
                    }
                    continue;
                }
            }
            toks.push(Tok {
                kind: TokKind::Ident(ident.to_string()),
                line,
            });
            continue;
        }

        if i + 2 < bytes.len() && bytes[i] == b'.' && bytes[i + 1] == b'.' && bytes[i + 2] == b'.' {
            toks.push(Tok {
                kind: TokKind::Ellipsis,
                line,
            });
            i += 3;
            continue;
        }

        let two = if i + 1 < bytes.len() {
            Some((bytes[i], bytes[i + 1]))
        } else {
            None
        };
        if let Some(pair) = two {
            let two_tok = match pair {
                (b'-', b'>') => Some(TokKind::Arrow),
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
            b'.' => TokKind::Dot,
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

#[derive(Debug, Clone, PartialEq, Eq)]
enum CType {
    Byte,
    U16,
    U32,
    Word64,
    Ptr(Box<CType>),
    Struct(String),
}

impl CType {
    /// Formal R4.MvmLinuxContainer construct a `char *` pointer type (`R4-63`).
    fn ptr_byte() -> Self {
        Self::Ptr(Box::new(Self::Byte))
    }

    /// Formal R4.MvmLinuxContainer inspect the pointee type when `self` is a pointer (`R4-63`).
    fn deref_inner(&self) -> Option<&CType> {
        match self {
            Self::Ptr(inner) => Some(inner.as_ref()),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
struct StructField {
    name: String,
    offset: usize,
    size: usize,
    ty: CType,
    is_array: bool,
}

#[derive(Debug, Clone)]
struct StructDef {
    size: usize,
    align: usize,
    fields: Vec<StructField>,
}

/// Formal R4.MvmLinuxContainer compute `(size_bytes, align_bytes)` for a C type including structs (`R4-63`).
fn ctype_size_align(ty: &CType, structs: &HashMap<String, StructDef>) -> (usize, usize) {
    match ty {
        CType::Byte => (1, 1),
        CType::U16 => (2, 2),
        CType::U32 => (4, 4),
        CType::Word64 | CType::Ptr(_) => (8, 8),
        CType::Struct(name) => structs
            .get(name)
            .map_or((8, 8), |s| (s.size.max(1), s.align.clamp(1, 8))),
    }
}

#[derive(Debug, Clone)]
enum Expr {
    Int(i64),
    Str(Vec<u8>),
    Var(String),
    Index(Box<Expr>, Box<Expr>),
    Member(Box<Expr>, String),
    PtrMember(Box<Expr>, String),
    Deref(Box<Expr>),
    AddrOf(Box<Expr>),
    SizeOfValue(Box<Expr>),
    Unary(UnaryOp, Box<Expr>),
    Binary(BinOp, Box<Expr>, Box<Expr>),
    Ternary(Box<Expr>, Box<Expr>, Box<Expr>),
    AssignVar(String, Box<Expr>),
    AssignIndex(Box<Expr>, Box<Expr>, Box<Expr>),
    AssignMember(Box<Expr>, String, Box<Expr>),
    AssignPtrMember(Box<Expr>, String, Box<Expr>),
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
    ty: CType,
    is_array: bool,
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
    structs: HashMap<String, StructDef>,
    typedefs: HashMap<String, CType>,
}

impl Parser {
    /// Formal R4.MvmLinuxContainer initialize a multi-TU C parser carrying known struct & typedef tables (`R4-63`).
    fn new(
        toks: Vec<Tok>,
        structs: HashMap<String, StructDef>,
        typedefs: HashMap<String, CType>,
    ) -> Self {
        Self {
            toks,
            pos: 0,
            structs,
            typedefs,
        }
    }

    /// Formal R4.MvmPolyglotToolchain.
    fn peek(&self) -> &Tok {
        &self.toks[self.pos.min(self.toks.len() - 1)]
    }

    /// Formal R4.MvmPolyglotToolchain.
    fn line(&self) -> usize {
        self.peek().line
    }

    /// Formal R4.MvmPolyglotToolchain.
    fn next(&mut self) -> Tok {
        let t = self.peek().clone();
        if self.pos + 1 < self.toks.len() {
            self.pos += 1;
        }
        t
    }

    /// Formal R4.MvmPolyglotToolchain.
    fn eat(&mut self, expected: &TokKind) -> bool {
        if &self.peek().kind == expected {
            self.next();
            true
        } else {
            false
        }
    }

    /// Formal R4.MvmPolyglotToolchain.
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

    /// Formal R4.MvmPolyglotToolchain.
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

    /// Formal R4.MvmLinuxContainer classify whether the next token begins a C type specifier (`R4-63`).
    fn is_type_start(&self) -> bool {
        match &self.peek().kind {
            TokKind::Ident(s) => {
                matches!(
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
                        | "extern"
                        | "inline"
                        | "struct"
                        | "bool"
                        | "_Bool"
                        | "size_t"
                        | "ssize_t"
                        | "socklen_t"
                        | "uint8_t"
                        | "int8_t"
                        | "uint16_t"
                        | "int16_t"
                        | "uint32_t"
                        | "int32_t"
                        | "uint64_t"
                        | "int64_t"
                ) || self.typedefs.contains_key(s.as_str())
            }
            _ => false,
        }
    }

    /// Formal R4.MvmLinuxContainer parse a C struct body with natural field alignment (`R4-63`).
    fn parse_struct_body(&mut self, struct_name: String) -> Result<StructDef, CCompileError> {
        self.expect(&TokKind::LBrace)?;
        let mut fields = Vec::new();
        let mut offset = 0usize;
        let mut max_align = 1usize;
        while !self.eat(&TokKind::RBrace) {
            if self.peek().kind == TokKind::Eof {
                return Err(CCompileError {
                    line: self.line(),
                    message: "unclosed struct definition".to_string(),
                });
            }
            let base_ty = self.parse_type()?;
            loop {
                let mut fty = base_ty.clone();
                while self.eat(&TokKind::Star) {
                    fty = CType::Ptr(Box::new(fty));
                }
                let fname = self.expect_ident()?;
                let mut is_array = false;
                let (elem_sz, align) = ctype_size_align(&fty, &self.structs);
                let mut field_sz = elem_sz;
                if self.eat(&TokKind::LBracket) {
                    if let TokKind::IntLit(n) = self.peek().kind {
                        self.next();
                        field_sz = (n.max(1) as usize) * elem_sz;
                        is_array = true;
                    } else {
                        return Err(CCompileError {
                            line: self.line(),
                            message: "expected constant struct array size".to_string(),
                        });
                    }
                    self.expect(&TokKind::RBracket)?;
                }
                let a = align.clamp(1, 8);
                max_align = max_align.max(a);
                offset = (offset + a - 1) & !(a - 1);
                fields.push(StructField {
                    name: fname,
                    offset,
                    size: field_sz,
                    ty: fty,
                    is_array,
                });
                offset += field_sz;
                if !self.eat(&TokKind::Comma) {
                    break;
                }
            }
            self.expect(&TokKind::Semi)?;
        }
        let total = (offset.max(1) + max_align - 1) & !(max_align - 1);
        let def = StructDef {
            size: total,
            align: max_align,
            fields,
        };
        self.structs.insert(struct_name, def.clone());
        Ok(def)
    }

    /// Formal R4.MvmLinuxContainer parse a C type specifier including structs and typedefs (`R4-63`).
    fn parse_type(&mut self) -> Result<CType, CCompileError> {
        let mut base: Option<CType> = None;
        let mut is_char = false;
        let mut is_short = false;
        let mut is_u32 = false;
        let mut consumed = false;
        while self.is_type_start() {
            if consumed && self.toks.get(self.pos + 1).map(|t| &t.kind) == Some(&TokKind::Semi) {
                break;
            }
            let ident = match &self.peek().kind {
                TokKind::Ident(s) => s.clone(),
                _ => break,
            };
            if let Some(aliased) = self.typedefs.get(&ident).cloned() {
                if consumed {
                    break;
                }
                self.next();
                base = Some(aliased);
                consumed = true;
                break;
            }
            self.next();
            consumed = true;
            if ident == "struct" {
                let sname = if let TokKind::Ident(n) = &self.peek().kind {
                    let s = n.clone();
                    self.next();
                    s
                } else {
                    format!("__anon_struct_{}", self.line())
                };
                if self.peek().kind == TokKind::LBrace {
                    self.parse_struct_body(sname.clone())?;
                }
                base = Some(CType::Struct(sname));
                break;
            }
            match ident.as_str() {
                "char" | "uint8_t" | "int8_t" | "bool" | "_Bool" => is_char = true,
                "short" | "uint16_t" | "int16_t" => is_short = true,
                "uint32_t" => is_u32 = true,
                _ => {}
            }
        }
        if !consumed {
            return Err(CCompileError {
                line: self.line(),
                message: "expected C type specifier".to_string(),
            });
        }
        let mut ty = base.unwrap_or(if is_char {
            CType::Byte
        } else if is_short {
            CType::U16
        } else if is_u32 {
            CType::U32
        } else {
            CType::Word64
        });
        while self.eat(&TokKind::Star) {
            ty = CType::Ptr(Box::new(ty));
        }
        Ok(ty)
    }

    /// Formal R4.MvmPolyglotToolchain.
    fn parse_program(&mut self) -> Result<Program, CCompileError> {
        let mut globals = Vec::new();
        let mut funcs = Vec::new();
        while self.peek().kind != TokKind::Eof {
            if matches!(&self.peek().kind, TokKind::Ident(s) if s == "typedef") {
                self.next();
                let mut base_ty = self.parse_type()?;
                while self.eat(&TokKind::Star) {
                    base_ty = CType::Ptr(Box::new(base_ty));
                }
                let alias = self.expect_ident()?;
                self.expect(&TokKind::Semi)?;
                self.typedefs.insert(alias, base_ty);
                continue;
            }
            let base_ty = self.parse_type()?;
            if self.eat(&TokKind::Semi) {
                // Standalone `struct Tag { ... };` declaration
                continue;
            }
            let mut decl_ty = base_ty.clone();
            while self.eat(&TokKind::Star) {
                decl_ty = CType::Ptr(Box::new(decl_ty));
            }
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
                        let mut arg_idx = 0usize;
                        loop {
                            if self.eat(&TokKind::Ellipsis) {
                                if !self.eat(&TokKind::Comma) {
                                    break;
                                }
                                continue;
                            }
                            let mut pty = self.parse_type()?;
                            while self.eat(&TokKind::Star) {
                                pty = CType::Ptr(Box::new(pty));
                            }
                            let pname = if matches!(
                                &self.peek().kind,
                                TokKind::Comma | TokKind::RParen | TokKind::LBracket
                            ) {
                                format!("__arg_{arg_idx}")
                            } else {
                                self.expect_ident()?
                            };
                            arg_idx += 1;
                            if self.eat(&TokKind::LBracket) {
                                if matches!(self.peek().kind, TokKind::IntLit(_)) {
                                    self.next();
                                }
                                let _ = self.eat(&TokKind::RBracket);
                                pty = CType::Ptr(Box::new(pty));
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
                let (mut size_bytes, _) = ctype_size_align(&decl_ty, &self.structs);
                size_bytes = size_bytes.max(8);
                let mut is_array = false;
                if self.eat(&TokKind::LBracket) {
                    is_array = true;
                    if let TokKind::IntLit(n) = self.peek().kind {
                        self.next();
                        let (elem, _) = ctype_size_align(&decl_ty, &self.structs);
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
                    ty: decl_ty,
                    is_array,
                    init_i64,
                });
            }
        }
        Ok(Program { globals, funcs })
    }

    /// Formal R4.MvmPolyglotToolchain.
    fn parse_decl_stmt(&mut self, eat_semi: bool) -> Result<Stmt, CCompileError> {
        let base_ty = self.parse_type()?;
        if eat_semi && matches!(base_ty, CType::Struct(_)) && self.eat(&TokKind::Semi) {
            return Ok(Stmt::Empty);
        }
        let mut first_ty = base_ty.clone();
        let raw_base = match &base_ty {
            CType::Ptr(inner) => inner.as_ref().clone(),
            other => other.clone(),
        };
        let mut decls = Vec::new();
        loop {
            let mut cur_ty = first_ty.clone();
            while self.eat(&TokKind::Star) {
                cur_ty = CType::Ptr(Box::new(cur_ty));
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
                ty: cur_ty,
                array_elems,
                init,
            });
            first_ty = raw_base.clone();
            if !self.eat(&TokKind::Comma) {
                break;
            }
        }
        if eat_semi {
            self.expect(&TokKind::Semi)?;
        }
        Ok(Stmt::Decl(decls))
    }

    /// Formal R4.MvmPolyglotToolchain.
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

    /// Formal R4.MvmPolyglotToolchain.
    fn parse_expr(&mut self) -> Result<Expr, CCompileError> {
        self.parse_assign()
    }

    /// Formal R4.MvmPolyglotToolchain.
    fn parse_assign(&mut self) -> Result<Expr, CCompileError> {
        let lhs = self.parse_ternary()?;
        if self.eat(&TokKind::Assign) {
            let rhs = self.parse_assign()?;
            return match lhs {
                Expr::Var(name) => Ok(Expr::AssignVar(name, Box::new(rhs))),
                Expr::Index(base, idx) => Ok(Expr::AssignIndex(base, idx, Box::new(rhs))),
                Expr::Member(base, field) => Ok(Expr::AssignMember(base, field, Box::new(rhs))),
                Expr::PtrMember(base, field) => {
                    Ok(Expr::AssignPtrMember(base, field, Box::new(rhs)))
                }
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
                Expr::Member(base, field) => Ok(Expr::AssignMember(
                    base.clone(),
                    field.clone(),
                    Box::new(Expr::Binary(
                        BinOp::Add,
                        Box::new(Expr::Member(base, field)),
                        Box::new(rhs),
                    )),
                )),
                Expr::PtrMember(base, field) => Ok(Expr::AssignPtrMember(
                    base.clone(),
                    field.clone(),
                    Box::new(Expr::Binary(
                        BinOp::Add,
                        Box::new(Expr::PtrMember(base, field)),
                        Box::new(rhs),
                    )),
                )),
                _ => Err(CCompileError {
                    line: self.line(),
                    message: "+= supported on variables and struct fields".to_string(),
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
                Expr::Member(base, field) => Ok(Expr::AssignMember(
                    base.clone(),
                    field.clone(),
                    Box::new(Expr::Binary(
                        BinOp::Sub,
                        Box::new(Expr::Member(base, field)),
                        Box::new(rhs),
                    )),
                )),
                Expr::PtrMember(base, field) => Ok(Expr::AssignPtrMember(
                    base.clone(),
                    field.clone(),
                    Box::new(Expr::Binary(
                        BinOp::Sub,
                        Box::new(Expr::PtrMember(base, field)),
                        Box::new(rhs),
                    )),
                )),
                _ => Err(CCompileError {
                    line: self.line(),
                    message: "-= supported on variables and struct fields".to_string(),
                }),
            };
        }
        Ok(lhs)
    }

    /// Formal R4.MvmPolyglotToolchain.
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

    /// Formal R4.MvmPolyglotToolchain.
    fn parse_log_or(&mut self) -> Result<Expr, CCompileError> {
        let mut lhs = self.parse_log_and()?;
        while self.eat(&TokKind::PipePipe) {
            let rhs = self.parse_log_and()?;
            lhs = Expr::Binary(BinOp::LogOr, Box::new(lhs), Box::new(rhs));
        }
        Ok(lhs)
    }

    /// Formal R4.MvmPolyglotToolchain.
    fn parse_log_and(&mut self) -> Result<Expr, CCompileError> {
        let mut lhs = self.parse_bit_or()?;
        while self.eat(&TokKind::AmpAmp) {
            let rhs = self.parse_bit_or()?;
            lhs = Expr::Binary(BinOp::LogAnd, Box::new(lhs), Box::new(rhs));
        }
        Ok(lhs)
    }

    /// Formal R4.MvmPolyglotToolchain.
    fn parse_bit_or(&mut self) -> Result<Expr, CCompileError> {
        let mut lhs = self.parse_bit_xor()?;
        while self.eat(&TokKind::Pipe) {
            let rhs = self.parse_bit_xor()?;
            lhs = Expr::Binary(BinOp::BitOr, Box::new(lhs), Box::new(rhs));
        }
        Ok(lhs)
    }

    /// Formal R4.MvmPolyglotToolchain.
    fn parse_bit_xor(&mut self) -> Result<Expr, CCompileError> {
        let mut lhs = self.parse_bit_and()?;
        while self.eat(&TokKind::Caret) {
            let rhs = self.parse_bit_and()?;
            lhs = Expr::Binary(BinOp::BitXor, Box::new(lhs), Box::new(rhs));
        }
        Ok(lhs)
    }

    /// Formal R4.MvmPolyglotToolchain.
    fn parse_bit_and(&mut self) -> Result<Expr, CCompileError> {
        let mut lhs = self.parse_eq()?;
        while self.eat(&TokKind::Amp) {
            let rhs = self.parse_eq()?;
            lhs = Expr::Binary(BinOp::BitAnd, Box::new(lhs), Box::new(rhs));
        }
        Ok(lhs)
    }

    /// Formal R4.MvmPolyglotToolchain.
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

    /// Formal R4.MvmPolyglotToolchain.
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

    /// Formal R4.MvmPolyglotToolchain.
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

    /// Formal R4.MvmPolyglotToolchain.
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

    /// Formal R4.MvmPolyglotToolchain.
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

    /// Formal R4.MvmPolyglotToolchain.
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
            let inner = self.parse_unary()?;
            return Ok(Expr::AddrOf(Box::new(inner)));
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

    /// Formal R4.MvmPolyglotToolchain.
    fn parse_postfix(&mut self) -> Result<Expr, CCompileError> {
        let mut expr = self.parse_primary()?;
        loop {
            if self.eat(&TokKind::LBracket) {
                let idx = self.parse_expr()?;
                self.expect(&TokKind::RBracket)?;
                expr = Expr::Index(Box::new(expr), Box::new(idx));
            } else if self.eat(&TokKind::Dot) {
                let field = self.expect_ident()?;
                expr = Expr::Member(Box::new(expr), field);
            } else if self.eat(&TokKind::Arrow) {
                let field = self.expect_ident()?;
                expr = Expr::PtrMember(Box::new(expr), field);
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

    /// Formal R4.MvmPolyglotToolchain.
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
                    if self.is_type_start() {
                        let t = self.parse_type()?;
                        let sz = ctype_size_align(&t, &self.structs).0 as i64;
                        self.expect(&TokKind::RParen)?;
                        return Ok(Expr::Int(sz));
                    }
                    let e = self.parse_expr()?;
                    self.expect(&TokKind::RParen)?;
                    return Ok(Expr::SizeOfValue(Box::new(e)));
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
const A3: u32 = 13;
const A4: u32 = 14;
const A5: u32 = 15;
const A7: u32 = 17;

#[derive(Clone)]
struct LocalSlot {
    offset_from_fp: i32,
    ty: CType,
    is_array: bool,
    byte_size: usize,
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
    structs: HashMap<String, StructDef>,
    global_offsets: HashMap<String, (usize, CType, bool, usize)>,
    func_inst_idx: HashMap<String, usize>,
    fixups: Vec<Fixup>,
    locals: HashMap<String, LocalSlot>,
    frame_size: i32,
    epilogue_jumps: Vec<usize>,
    break_jumps: Vec<Vec<usize>>,
}

impl Codegen {
    /// Formal R4.MvmPolyglotToolchain.
    fn emit(&mut self, inst: u32) -> usize {
        let idx = self.insts.len();
        self.insts.push(inst);
        idx
    }

    /// Formal R4.MvmPolyglotToolchain.
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

    /// Formal R4.MvmPolyglotToolchain.
    fn emit_fp_addr(&mut self, rd: u32, offset: i32) {
        if (-2048..=2047).contains(&offset) {
            self.emit(enc_i(offset, S0, 0, rd, 0x13));
        } else {
            self.emit_li(rd, offset as i64);
            self.emit(enc_r(0, S0, rd, 0, rd, 0x33));
        }
    }

    /// Formal R4.MvmPolyglotToolchain.
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

    /// Formal R4.MvmPolyglotToolchain.
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

    /// Formal R4.MvmPolyglotToolchain.
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

    /// Formal R4.MvmPolyglotToolchain.
    fn emit_syscall_call(
        &mut self,
        args: &[Expr],
        reg_targets: &[u32],
        defaults: &[(u32, i32)],
        syscall_nr: i32,
    ) -> Result<(), CCompileError> {
        for a in args {
            self.compile_expr(a)?;
            self.push_a0();
        }
        for &reg in reg_targets.iter().rev() {
            self.pop_to(reg);
        }
        for &(reg, imm) in defaults {
            self.emit(enc_i(imm, X0, 0, reg, 0x13));
        }
        self.emit(enc_i(syscall_nr, X0, 0, A7, 0x13));
        self.emit(0x0000_0073);
        Ok(())
    }

    /// Formal R4.MvmPolyglotToolchain.
    fn push_a0(&mut self) {
        self.emit(enc_i(-8, SP, 0, SP, 0x13)); // addi sp, sp, -8
        self.emit(enc_s(0, A0, SP, 3, 0x23)); // sd a0, 0(sp)
    }

    /// Formal R4.MvmPolyglotToolchain.
    fn pop_to(&mut self, reg: u32) {
        self.emit(enc_i(0, SP, 3, reg, 0x03)); // ld reg, 0(sp)
        self.emit(enc_i(8, SP, 0, SP, 0x13)); // addi sp, sp, 8
    }

    /// Formal R4.MvmPolyglotToolchain.
    fn patch_b(&mut self, inst_idx: usize, target_idx: usize, rs2: u32, rs1: u32, funct3: u32) {
        let off = ((target_idx as isize) - (inst_idx as isize)) * 4;
        self.insts[inst_idx] = enc_b(off as i32, rs2, rs1, funct3, 0x63);
    }

    /// Formal R4.MvmPolyglotToolchain.
    fn patch_j(&mut self, inst_idx: usize, target_idx: usize, rd: u32) {
        let off = ((target_idx as isize) - (inst_idx as isize)) * 4;
        self.insts[inst_idx] = enc_j(off as i32, rd, 0x6f);
    }

    /// Formal R4.MvmPolyglotToolchain.
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
        self.emit(enc_s(0, A0, SP, 3, 0x23)); // save cstr byte len in 0(sp)
        self.emit(enc_i(8, SP, 3, T0, 0x03)); // append_newline?
        let skip_nl = self.emit(0);
        self.emit(enc_i(10, X0, 0, A0, 0x13)); // '\n'
        self.emit_call("__rt_putchar");
        self.emit(enc_i(0, SP, 3, T0, 0x03));
        self.emit(enc_i(1, T0, 0, A0, 0x13));
        let done_all = self.emit(0);
        let after_nl = self.insts.len();
        self.patch_b(skip_nl, after_nl, X0, T0, 0);
        self.emit(enc_i(0, SP, 3, A0, 0x03));
        let puts_exit = self.insts.len();
        self.patch_j(done_all, puts_exit, X0);
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

        // __rt_print_hex64(a0 = u64): lowercase hex with no prefix
        self.func_inst_idx
            .insert("__rt_print_hex64".to_string(), self.insts.len());
        self.emit(enc_i(-48, SP, 0, SP, 0x13));
        self.emit(enc_s(40, RA, SP, 3, 0x23));
        self.emit(enc_i(0, A0, 0, T0, 0x13)); // t0 = val
        self.emit(enc_i(32, SP, 0, T2, 0x13)); // cursor at sp+32
        self.emit(enc_i(15, X0, 0, T1, 0x13)); // mask 0xf
        let hex_loop = self.insts.len();
        self.emit(enc_r(0, T1, T0, 7, A1, 0x33)); // and a1, t0, 0xf
        self.emit(enc_i(10, X0, 0, A2, 0x13));
        let br_digit = self.emit(0); // blt a1, 10
        self.emit(enc_i((b'a' - 10) as i32, A1, 0, A1, 0x13));
        let j_store = self.emit(0);
        let is_dec = self.insts.len();
        self.patch_b(br_digit, is_dec, A2, A1, 4); // blt a1, a2
        self.emit(enc_i(b'0' as i32, A1, 0, A1, 0x13));
        let do_store = self.insts.len();
        self.patch_j(j_store, do_store, X0);
        self.emit(enc_i(-1, T2, 0, T2, 0x13));
        self.emit(enc_s(0, A1, T2, 0, 0x23));
        self.emit(enc_i(4, T0, 5, T0, 0x13)); // srli t0, t0, 4
        let br_hex_more = self.emit(0);
        self.patch_b(br_hex_more, hex_loop, X0, T0, 1); // bne t0, x0
        self.emit(enc_i(32, SP, 0, A2, 0x13));
        self.emit(enc_r(0x20, T2, A2, 0, A2, 0x33));
        self.emit(enc_i(0, T2, 0, A1, 0x13));
        self.emit(enc_i(1, X0, 0, A0, 0x13));
        self.emit(enc_i(64, X0, 0, A7, 0x13));
        self.emit(0x0000_0073);
        self.emit(enc_i(40, SP, 3, RA, 0x03));
        self.emit(enc_i(48, SP, 0, SP, 0x13));
        self.emit(enc_i(0, RA, 0, X0, 0x67));

        // __rt_strlen(a0 = s) -> a0
        self.func_inst_idx
            .insert("__rt_strlen".to_string(), self.insts.len());
        self.emit(enc_i(0, X0, 0, T0, 0x13));
        let sl_loop = self.insts.len();
        self.emit(enc_r(0, T0, A0, 0, T1, 0x33));
        self.emit(enc_i(0, T1, 4, T2, 0x03));
        let sl_done = self.emit(0);
        self.emit(enc_i(1, T0, 0, T0, 0x13));
        let sl_back = self.emit(0);
        self.patch_j(sl_back, sl_loop, X0);
        let sl_exit = self.insts.len();
        self.patch_b(sl_done, sl_exit, X0, T2, 0);
        self.emit(enc_i(0, T0, 0, A0, 0x13));
        self.emit(enc_i(0, RA, 0, X0, 0x67));

        // __rt_strcmp(a0 = s1, a1 = s2) -> a0
        self.func_inst_idx
            .insert("__rt_strcmp".to_string(), self.insts.len());
        let sc_loop = self.insts.len();
        self.emit(enc_i(0, A0, 4, T0, 0x03));
        self.emit(enc_i(0, A1, 4, T1, 0x03));
        let sc_diff = self.emit(0); // bne t0, t1
        let sc_end = self.emit(0); // beq t0, x0
        self.emit(enc_i(1, A0, 0, A0, 0x13));
        self.emit(enc_i(1, A1, 0, A1, 0x13));
        let sc_back = self.emit(0);
        self.patch_j(sc_back, sc_loop, X0);
        let sc_done = self.insts.len();
        self.patch_b(sc_diff, sc_done, T1, T0, 1);
        self.patch_b(sc_end, sc_done, X0, T0, 0);
        self.emit(enc_r(0x20, T1, T0, 0, A0, 0x33));
        self.emit(enc_i(0, RA, 0, X0, 0x67));

        // __rt_strncmp(a0 = s1, a1 = s2, a2 = n) -> a0
        self.func_inst_idx
            .insert("__rt_strncmp".to_string(), self.insts.len());
        let sn_loop = self.insts.len();
        let sn_zero = self.emit(0); // beq a2, x0
        self.emit(enc_i(0, A0, 4, T0, 0x03));
        self.emit(enc_i(0, A1, 4, T1, 0x03));
        let sn_diff = self.emit(0); // bne t0, t1
        let sn_nul = self.emit(0); // beq t0, x0
        self.emit(enc_i(1, A0, 0, A0, 0x13));
        self.emit(enc_i(1, A1, 0, A1, 0x13));
        self.emit(enc_i(-1, A2, 0, A2, 0x13));
        let sn_back = self.emit(0);
        self.patch_j(sn_back, sn_loop, X0);
        let sn_z = self.insts.len();
        self.patch_b(sn_zero, sn_z, X0, A2, 0);
        self.emit(enc_i(0, X0, 0, A0, 0x13));
        self.emit(enc_i(0, RA, 0, X0, 0x67));
        let sn_ret = self.insts.len();
        self.patch_b(sn_diff, sn_ret, T1, T0, 1);
        self.patch_b(sn_nul, sn_ret, X0, T0, 0);
        self.emit(enc_r(0x20, T1, T0, 0, A0, 0x33));
        self.emit(enc_i(0, RA, 0, X0, 0x67));

        // __rt_strcpy(a0 = dst, a1 = src) -> a0 = orig dst
        self.func_inst_idx
            .insert("__rt_strcpy".to_string(), self.insts.len());
        self.emit(enc_i(0, A0, 0, T0, 0x13));
        let cy_loop = self.insts.len();
        self.emit(enc_i(0, A1, 4, T1, 0x03));
        self.emit(enc_s(0, T1, T0, 0, 0x23));
        self.emit(enc_i(1, T0, 0, T0, 0x13));
        self.emit(enc_i(1, A1, 0, A1, 0x13));
        let cy_more = self.emit(0);
        self.patch_b(cy_more, cy_loop, X0, T1, 1);
        self.emit(enc_i(0, RA, 0, X0, 0x67));

        // __rt_memcpy(a0 = dst, a1 = src, a2 = n) -> a0 = orig dst
        self.func_inst_idx
            .insert("__rt_memcpy".to_string(), self.insts.len());
        self.emit(enc_i(0, A0, 0, T0, 0x13));
        let mc_loop = self.insts.len();
        let mc_done = self.emit(0);
        self.emit(enc_i(0, A1, 4, T1, 0x03));
        self.emit(enc_s(0, T1, T0, 0, 0x23));
        self.emit(enc_i(1, T0, 0, T0, 0x13));
        self.emit(enc_i(1, A1, 0, A1, 0x13));
        self.emit(enc_i(-1, A2, 0, A2, 0x13));
        let mc_back = self.emit(0);
        self.patch_j(mc_back, mc_loop, X0);
        let mc_exit = self.insts.len();
        self.patch_b(mc_done, mc_exit, X0, A2, 0);
        self.emit(enc_i(0, RA, 0, X0, 0x67));

        // __rt_memset(a0 = dst, a1 = byte, a2 = n) -> a0 = orig dst
        self.func_inst_idx
            .insert("__rt_memset".to_string(), self.insts.len());
        self.emit(enc_i(0, A0, 0, T0, 0x13));
        let ms_loop = self.insts.len();
        let ms_done = self.emit(0);
        self.emit(enc_s(0, A1, T0, 0, 0x23));
        self.emit(enc_i(1, T0, 0, T0, 0x13));
        self.emit(enc_i(-1, A2, 0, A2, 0x13));
        let ms_back = self.emit(0);
        self.patch_j(ms_back, ms_loop, X0);
        let ms_exit = self.insts.len();
        self.patch_b(ms_done, ms_exit, X0, A2, 0);
        self.emit(enc_i(0, RA, 0, X0, 0x67));

        // __rt_malloc(a0 = size) -> a0 (8-byte aligned SYS_brk = 214, R4-67)
        self.func_inst_idx
            .insert("__rt_malloc".to_string(), self.insts.len());
        self.emit(enc_i(7, A0, 0, T2, 0x13)); // addi t2, a0, 7
        self.emit(enc_i(-8, T2, 7, T2, 0x13)); // andi t2, t2, -8
        let ml_nz = self.emit(0); // bne t2, x0, ml_do_brk
        self.emit(enc_i(8, X0, 0, T2, 0x13)); // min 8 bytes
        let ml_do_brk = self.insts.len();
        self.patch_b(ml_nz, ml_do_brk, X0, T2, 1);
        self.emit(enc_i(0, X0, 0, A0, 0x13)); // a0 = 0 (query current brk)
        self.emit(enc_i(214, X0, 0, A7, 0x13)); // SYS_brk
        self.emit(0x0000_0073);
        self.emit(enc_i(0, A0, 0, T0, 0x13)); // t0 = old_brk
        self.emit(enc_r(0, T2, T0, 0, A0, 0x33)); // a0 = old_brk + aligned_size
        self.emit(enc_i(214, X0, 0, A7, 0x13)); // SYS_brk
        self.emit(0x0000_0073);
        let ml_ok = self.emit(0); // bne a0, t0, ml_ret_ptr
        self.emit(enc_i(0, X0, 0, A0, 0x13)); // NULL on failure
        self.emit(enc_i(0, RA, 0, X0, 0x67));
        let ml_ret_ptr = self.insts.len();
        self.patch_b(ml_ok, ml_ret_ptr, T0, A0, 1);
        self.emit(enc_i(0, T0, 0, A0, 0x13)); // return old_brk
        self.emit(enc_i(0, RA, 0, X0, 0x67));

        // __rt_calloc(a0 = nmemb, a1 = size) -> a0 (zero-initialized SYS_brk allocation, R4-67)
        self.func_inst_idx
            .insert("__rt_calloc".to_string(), self.insts.len());
        self.emit(enc_i(-32, SP, 0, SP, 0x13));
        self.emit(enc_s(24, RA, SP, 3, 0x23));
        self.emit(enc_s(16, S0, SP, 3, 0x23));
        self.emit(enc_r(0x01, A1, A0, 0, S0, 0x33)); // mul s0, a0, a1
        self.emit(enc_i(0, S0, 0, A0, 0x13));
        self.emit_call("__rt_malloc");
        let cl_null = self.emit(0); // beq a0, x0, cl_done
        self.emit(enc_i(0, X0, 0, A1, 0x13)); // byte = 0
        self.emit(enc_i(0, S0, 0, A2, 0x13)); // n = s0
        self.emit_call("__rt_memset");
        let cl_done = self.insts.len();
        self.patch_b(cl_null, cl_done, X0, A0, 0);
        self.emit(enc_i(24, SP, 3, RA, 0x03));
        self.emit(enc_i(16, SP, 3, S0, 0x03));
        self.emit(enc_i(32, SP, 0, SP, 0x13));
        self.emit(enc_i(0, RA, 0, X0, 0x67));

        // __rt_free(a0 = ptr) -> a0 = 0 (no-op arena free, R4-67)
        self.func_inst_idx
            .insert("__rt_free".to_string(), self.insts.len());
        self.emit(enc_i(0, X0, 0, A0, 0x13));
        self.emit(enc_i(0, RA, 0, X0, 0x67));

        // __rt_abs(a0 = x) -> a0 (R4-67)
        self.func_inst_idx
            .insert("__rt_abs".to_string(), self.insts.len());
        let abs_pos = self.emit(0); // bge a0, x0, abs_ret
        self.emit(enc_r(0x20, A0, X0, 0, A0, 0x33)); // sub a0, x0, a0
        let abs_ret = self.insts.len();
        self.patch_b(abs_pos, abs_ret, X0, A0, 5);
        self.emit(enc_i(0, RA, 0, X0, 0x67));

        // __rt_atoi(a0 = s) -> a0 (signed base-10 ASCII integer parser, R4-67)
        self.func_inst_idx
            .insert("__rt_atoi".to_string(), self.insts.len());
        self.emit(enc_i(0, X0, 0, T0, 0x13)); // t0 = acc = 0
        self.emit(enc_i(1, X0, 0, T1, 0x13)); // t1 = sign = 1
        self.emit(enc_i(32, X0, 0, A2, 0x13)); // a2 = ' ' (32)
        let atoi_ws = self.insts.len();
        self.emit(enc_i(0, A0, 4, T2, 0x03)); // lbu t2, 0(a0)
        let atoi_ws_nul = self.emit(0); // beq t2, x0, atoi_done
        let atoi_ws_end = self.emit(0); // blt a2, t2, atoi_sign (32 < t2)
        self.emit(enc_i(1, A0, 0, A0, 0x13));
        let atoi_ws_back = self.emit(0);
        self.patch_j(atoi_ws_back, atoi_ws, X0);
        let atoi_sign = self.insts.len();
        self.patch_b(atoi_ws_end, atoi_sign, T2, A2, 4);
        self.emit(enc_i(b'-' as i32, X0, 0, A1, 0x13));
        let atoi_not_minus = self.emit(0); // bne t2, '-', atoi_chk_plus
        self.emit(enc_i(-1, X0, 0, T1, 0x13));
        self.emit(enc_i(1, A0, 0, A0, 0x13));
        let atoi_to_dig = self.emit(0);
        let atoi_chk_plus = self.insts.len();
        self.patch_b(atoi_not_minus, atoi_chk_plus, A1, T2, 1);
        self.emit(enc_i(b'+' as i32, X0, 0, A1, 0x13));
        let atoi_not_plus = self.emit(0); // bne t2, '+', atoi_dig
        self.emit(enc_i(1, A0, 0, A0, 0x13));
        let atoi_dig = self.insts.len();
        self.patch_j(atoi_to_dig, atoi_dig, X0);
        self.patch_b(atoi_not_plus, atoi_dig, A1, T2, 1);
        self.emit(enc_i(0, A0, 4, T2, 0x03)); // lbu t2, 0(a0)
        self.emit(enc_i(b'0' as i32, X0, 0, A1, 0x13));
        let atoi_lt0 = self.emit(0); // blt t2, '0', atoi_done
        self.emit(enc_i((b'9' + 1) as i32, X0, 0, A1, 0x13));
        let atoi_gt9 = self.emit(0); // bge t2, '9'+1, atoi_done
        self.emit(enc_i(-(b'0' as i32), T2, 0, T2, 0x13));
        self.emit(enc_i(10, X0, 0, A1, 0x13));
        self.emit(enc_r(0x01, A1, T0, 0, T0, 0x33)); // t0 *= 10
        self.emit(enc_r(0, T2, T0, 0, T0, 0x33)); // t0 += digit
        self.emit(enc_i(1, A0, 0, A0, 0x13));
        let atoi_dig_back = self.emit(0);
        self.patch_j(atoi_dig_back, atoi_dig, X0);
        let atoi_done = self.insts.len();
        self.patch_b(atoi_ws_nul, atoi_done, X0, T2, 0);
        self.patch_b(atoi_lt0, atoi_done, A1, T2, 4);
        self.patch_b(atoi_gt9, atoi_done, A1, T2, 5);
        self.emit(enc_r(0x01, T1, T0, 0, A0, 0x33)); // a0 = acc * sign
        self.emit(enc_i(0, RA, 0, X0, 0x67));
    }

    fn count_stmt_slots(&self, stmt: &Stmt) -> usize {
        match stmt {
            Stmt::Block(list) => list.iter().map(|s| self.count_stmt_slots(s)).sum(),
            Stmt::Decl(decls) => decls
                .iter()
                .map(|d| {
                    let (elem_sz, _) = ctype_size_align(&d.ty, &self.structs);
                    let total_bytes = d.array_elems.unwrap_or(1).max(1) * elem_sz;
                    total_bytes.div_ceil(8).max(1) + 1
                })
                .sum(),
            Stmt::If(_, t, e) => {
                self.count_stmt_slots(t) + e.as_deref().map_or(0, |es| self.count_stmt_slots(es))
            }
            Stmt::While(_, b) => self.count_stmt_slots(b),
            Stmt::For(i, _, _, b) => {
                i.as_deref().map_or(0, |is| self.count_stmt_slots(is)) + self.count_stmt_slots(b)
            }
            _ => 0,
        }
    }

    /// Formal R4.MvmPolyglotToolchain.
    fn compile_func(&mut self, func: &FuncDef) -> Result<(), CCompileError> {
        self.func_inst_idx
            .insert(func.name.clone(), self.insts.len());
        self.locals.clear();
        self.epilogue_jumps.clear();
        self.break_jumps.clear();

        let body_slots: usize = func.body.iter().map(|s| self.count_stmt_slots(s)).sum();
        let local_slots: usize = func.params.len() + body_slots + 6;
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
            let byte_size = ctype_size_align(pty, &self.structs).0;
            let slot = LocalSlot {
                offset_from_fp: next_off,
                ty: pty.clone(),
                is_array: false,
                byte_size,
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

    /// Formal R4.MvmPolyglotToolchain.
    fn compile_stmt(&mut self, stmt: &Stmt, next_off: &mut i32) -> Result<(), CCompileError> {
        match stmt {
            Stmt::Empty => Ok(()),
            Stmt::Block(list) => {
                let saved_locals = self.locals.clone();
                let res = (|| {
                    for s in list {
                        self.compile_stmt(s, next_off)?;
                    }
                    Ok(())
                })();
                self.locals = saved_locals;
                res
            }
            Stmt::Decl(decls) => {
                for d in decls {
                    let (elem_sz, _) = ctype_size_align(&d.ty, &self.structs);
                    if let Some(elems) = d.array_elems {
                        let total = elems.max(1) * elem_sz;
                        let aligned_bytes = (total as i32 + 7) & !7;
                        *next_off -= aligned_bytes;
                        let base_off = *next_off + 8;
                        self.locals.insert(
                            d.name.clone(),
                            LocalSlot {
                                offset_from_fp: base_off,
                                ty: d.ty.clone(),
                                is_array: true,
                                byte_size: total,
                            },
                        );
                        *next_off -= 8;
                    } else if matches!(d.ty, CType::Struct(_)) {
                        let aligned_bytes = (elem_sz as i32 + 7) & !7;
                        *next_off -= aligned_bytes;
                        let base_off = *next_off + 8;
                        self.locals.insert(
                            d.name.clone(),
                            LocalSlot {
                                offset_from_fp: base_off,
                                ty: d.ty.clone(),
                                is_array: false,
                                byte_size: elem_sz,
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
                                ty: d.ty.clone(),
                                is_array: false,
                                byte_size: elem_sz,
                            },
                        );
                        if let Some(init) = &d.init {
                            self.compile_expr(init)?;
                            self.emit_fp_addr(T0, off);
                            self.emit_store_a0_to_t0_by_type(&d.ty);
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
                let saved_locals = self.locals.clone();
                let res = (|| -> Result<(), CCompileError> {
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
                })();
                self.locals = saved_locals;
                res
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

    /// Formal R4.MvmLinuxContainer infer the static `CType` (and optional aggregate byte size) of an expression (`R4-63`).
    fn infer_expr_type_and_size(&self, expr: &Expr) -> (CType, usize) {
        match expr {
            Expr::Var(name) => {
                if let Some(loc) = self.locals.get(name) {
                    if loc.is_array {
                        (CType::Ptr(Box::new(loc.ty.clone())), loc.byte_size)
                    } else {
                        (loc.ty.clone(), loc.byte_size)
                    }
                } else if let Some((_, gty, is_array, size_bytes)) = self.global_offsets.get(name) {
                    if *is_array {
                        (CType::Ptr(Box::new(gty.clone())), *size_bytes)
                    } else {
                        let sz = ctype_size_align(gty, &self.structs).0;
                        (gty.clone(), sz)
                    }
                } else {
                    (CType::Word64, 8)
                }
            }
            Expr::Str(b) => (CType::ptr_byte(), b.len() + 1),
            Expr::AddrOf(inner) => {
                let (it, _) = self.infer_expr_type_and_size(inner);
                (CType::Ptr(Box::new(it)), 8)
            }
            Expr::Deref(inner) => {
                let (it, _) = self.infer_expr_type_and_size(inner);
                let dt = it.deref_inner().cloned().unwrap_or(CType::Word64);
                let sz = ctype_size_align(&dt, &self.structs).0;
                (dt, sz)
            }
            Expr::Index(base, _) => {
                let (bt, _) = self.infer_expr_type_and_size(base);
                let et = bt.deref_inner().cloned().unwrap_or(CType::Word64);
                let sz = ctype_size_align(&et, &self.structs).0;
                (et, sz)
            }
            Expr::Member(base, fname) => {
                let (bt, _) = self.infer_expr_type_and_size(base);
                if let CType::Struct(sname) = bt {
                    if let Some(f) = self
                        .structs
                        .get(&sname)
                        .and_then(|s| s.fields.iter().find(|sf| sf.name == *fname))
                    {
                        let rty = if f.is_array {
                            CType::Ptr(Box::new(f.ty.clone()))
                        } else {
                            f.ty.clone()
                        };
                        return (rty, f.size);
                    }
                }
                (CType::Word64, 8)
            }
            Expr::PtrMember(base, fname) => {
                let (bt, _) = self.infer_expr_type_and_size(base);
                if let Some(CType::Struct(sname)) = bt.deref_inner() {
                    if let Some(f) = self
                        .structs
                        .get(sname)
                        .and_then(|s| s.fields.iter().find(|sf| sf.name == *fname))
                    {
                        let rty = if f.is_array {
                            CType::Ptr(Box::new(f.ty.clone()))
                        } else {
                            f.ty.clone()
                        };
                        return (rty, f.size);
                    }
                }
                (CType::Word64, 8)
            }
            _ => (CType::Word64, 8),
        }
    }

    /// Formal R4.MvmLinuxContainer emit `A0 <- &lvalue` and return `(field_type, is_array)` (`R4-63`).
    fn compile_lvalue_addr(&mut self, expr: &Expr) -> Result<(CType, bool), CCompileError> {
        match expr {
            Expr::Var(name) => {
                if let Some(loc) = self.locals.get(name).cloned() {
                    self.emit_fp_addr(A0, loc.offset_from_fp);
                    Ok((loc.ty, loc.is_array))
                } else if let Some((goff, gty, is_array, _)) = self.global_offsets.get(name).cloned() {
                    self.emit_load_data_addr(A0, goff);
                    Ok((gty, is_array))
                } else {
                    Err(CCompileError {
                        line: 1,
                        message: format!("undefined variable `{name}`"),
                    })
                }
            }
            Expr::Deref(inner) => {
                let (it, _) = self.infer_expr_type_and_size(inner);
                let dt = it.deref_inner().cloned().unwrap_or(CType::Word64);
                self.compile_expr(inner)?;
                Ok((dt, false))
            }
            Expr::Index(base, idx) => {
                let (bt, _) = self.infer_expr_type_and_size(base);
                let et = bt.deref_inner().cloned().unwrap_or(CType::Word64);
                let (stride, _) = ctype_size_align(&et, &self.structs);
                self.compile_expr(base)?;
                self.push_a0();
                self.compile_expr(idx)?;
                if stride > 1 {
                    self.emit_li(T0, stride as i64);
                    self.emit(enc_r(0x01, T0, A0, 0, A0, 0x33)); // mul a0, a0, t0
                }
                self.pop_to(T0);
                self.emit(enc_r(0, A0, T0, 0, A0, 0x33)); // a0 = base + idx*stride
                Ok((et, false))
            }
            Expr::Member(base, fname) => {
                let (base_ty, _) = self.compile_lvalue_addr(base)?;
                let CType::Struct(sname) = base_ty else {
                    return Err(CCompileError {
                        line: 1,
                        message: format!("member `.{fname}` requires a struct value"),
                    });
                };
                let field = self
                    .structs
                    .get(&sname)
                    .and_then(|s| s.fields.iter().find(|f| f.name == *fname))
                    .cloned()
                    .ok_or_else(|| CCompileError {
                        line: 1,
                        message: format!("struct `{sname}` has no field `{fname}`"),
                    })?;
                if field.offset > 0 {
                    self.emit_li(T0, field.offset as i64);
                    self.emit(enc_r(0, T0, A0, 0, A0, 0x33));
                }
                Ok((field.ty, field.is_array))
            }
            Expr::PtrMember(base, fname) => {
                let (bt, _) = self.infer_expr_type_and_size(base);
                let Some(CType::Struct(sname)) = bt.deref_inner().cloned() else {
                    return Err(CCompileError {
                        line: 1,
                        message: format!("member `->{fname}` requires a pointer to struct"),
                    });
                };
                let field = self
                    .structs
                    .get(&sname)
                    .and_then(|s| s.fields.iter().find(|f| f.name == *fname))
                    .cloned()
                    .ok_or_else(|| CCompileError {
                        line: 1,
                        message: format!("struct `{sname}` has no field `{fname}`"),
                    })?;
                self.compile_expr(base)?;
                if field.offset > 0 {
                    self.emit_li(T0, field.offset as i64);
                    self.emit(enc_r(0, T0, A0, 0, A0, 0x33));
                }
                Ok((field.ty, field.is_array))
            }
            _ => Err(CCompileError {
                line: 1,
                message: "expression has no addressable lvalue location".to_string(),
            }),
        }
    }

    /// Formal R4.MvmLinuxContainer load a scalar from address in `A0` according to `CType` width (`R4-63`).
    fn emit_load_from_a0_by_type(&mut self, ty: &CType, is_array: bool) {
        if is_array || matches!(ty, CType::Struct(_)) {
            return;
        }
        match ty {
            CType::Byte => {
                self.emit(enc_i(0, A0, 4, A0, 0x03)); // lbu a0, 0(a0)
            }
            CType::U16 => {
                self.emit(enc_i(0, A0, 5, A0, 0x03)); // lhu a0, 0(a0)
            }
            CType::U32 => {
                self.emit(enc_i(0, A0, 6, A0, 0x03)); // lwu a0, 0(a0)
            }
            CType::Word64 | CType::Ptr(_) => {
                self.emit(enc_i(0, A0, 3, A0, 0x03)); // ld a0, 0(a0)
            }
            CType::Struct(_) => {}
        }
    }

    /// Formal R4.MvmLinuxContainer store `A0` into address in `T0` according to `CType` width (`R4-63`).
    fn emit_store_a0_to_t0_by_type(&mut self, ty: &CType) {
        match ty {
            CType::Byte => {
                self.emit(enc_s(0, A0, T0, 0, 0x23)); // sb a0, 0(t0)
            }
            CType::U16 => {
                self.emit(enc_s(0, A0, T0, 1, 0x23)); // sh a0, 0(t0)
            }
            CType::U32 => {
                self.emit(enc_s(0, A0, T0, 2, 0x23)); // sw a0, 0(t0)
            }
            CType::Word64 | CType::Ptr(_) | CType::Struct(_) => {
                self.emit(enc_s(0, A0, T0, 3, 0x23)); // sd a0, 0(t0)
            }
        }
    }

    /// Formal R4.MvmPolyglotToolchain.
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
        self.emit(enc_i(-16, SP, 0, SP, 0x13)); // 0(sp) holds printf total written bytes
        self.emit(enc_s(0, X0, SP, 3, 0x23));
        let add_acc = |cg: &mut Self| {
            cg.emit(enc_i(0, SP, 3, T0, 0x03));
            cg.emit(enc_r(0, T0, A0, 0, T0, 0x33));
            cg.emit(enc_s(0, T0, SP, 3, 0x23));
        };
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
                    add_acc(self);
                }
                i += 1;
                if fmt_bytes[i] == b'%' {
                    self.emit_li(A0, b'%' as i64);
                    self.emit_call("__rt_putchar");
                    add_acc(self);
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
                    let stack_off = ((n_vals - 1 - cur_val) * 8 + 16) as i32;
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
                        b'x' => {
                            self.emit_call("__rt_print_hex64");
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
                    add_acc(self);
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
            add_acc(self);
        }
        self.emit(enc_i(0, SP, 3, A0, 0x03));
        self.emit_li(T0, ((n_vals * 8) + 16) as i64);
        self.emit(enc_r(0, T0, SP, 0, SP, 0x33));
        Ok(())
    }

    /// Formal R4.MvmPolyglotToolchain.
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
            Expr::Var(_) | Expr::Member(_, _) | Expr::PtrMember(_, _) | Expr::Index(_, _) => {
                let (ty, is_array) = self.compile_lvalue_addr(expr)?;
                self.emit_load_from_a0_by_type(&ty, is_array);
                Ok(())
            }
            Expr::AddrOf(inner) => {
                let _ = self.compile_lvalue_addr(inner)?;
                Ok(())
            }
            Expr::SizeOfValue(inner) => {
                let (_, sz) = self.infer_expr_type_and_size(inner);
                self.emit_li(A0, sz as i64);
                Ok(())
            }
            Expr::Deref(inner) => {
                let (it, _) = self.infer_expr_type_and_size(inner);
                let dt = it.deref_inner().cloned().unwrap_or(CType::Word64);
                self.compile_expr(inner)?;
                self.emit_load_from_a0_by_type(&dt, false);
                Ok(())
            }
            Expr::AssignVar(name, rhs) => {
                self.compile_expr(rhs)?;
                if let Some(loc) = self.locals.get(name).cloned() {
                    self.emit_fp_addr(T0, loc.offset_from_fp);
                    self.emit_store_a0_to_t0_by_type(&loc.ty);
                    Ok(())
                } else if let Some((goff, gty, _, _)) = self.global_offsets.get(name).cloned() {
                    self.emit_load_data_addr(T0, goff);
                    self.emit_store_a0_to_t0_by_type(&gty);
                    Ok(())
                } else {
                    Err(CCompileError {
                        line: 1,
                        message: format!("assignment to undefined variable `{name}`"),
                    })
                }
            }
            Expr::AssignMember(base, field, rhs) => {
                let target = Expr::Member(base.clone(), field.clone());
                let (fty, _) = self.compile_lvalue_addr(&target)?;
                self.push_a0();
                self.compile_expr(rhs)?;
                self.pop_to(T0);
                self.emit_store_a0_to_t0_by_type(&fty);
                Ok(())
            }
            Expr::AssignPtrMember(base, field, rhs) => {
                let target = Expr::PtrMember(base.clone(), field.clone());
                let (fty, _) = self.compile_lvalue_addr(&target)?;
                self.push_a0();
                self.compile_expr(rhs)?;
                self.pop_to(T0);
                self.emit_store_a0_to_t0_by_type(&fty);
                Ok(())
            }
            Expr::AssignIndex(base, idx, rhs) => {
                let target = Expr::Index(base.clone(), idx.clone());
                let (ety, _) = self.compile_lvalue_addr(&target)?;
                self.push_a0();
                self.compile_expr(rhs)?;
                self.pop_to(T0);
                self.emit_store_a0_to_t0_by_type(&ety);
                Ok(())
            }
            Expr::AssignDeref(ptr, rhs) => {
                let (pt, _) = self.infer_expr_type_and_size(ptr);
                let dt = pt.deref_inner().cloned().unwrap_or(CType::Word64);
                self.compile_expr(ptr)?;
                self.push_a0();
                self.compile_expr(rhs)?;
                self.pop_to(T0);
                self.emit_store_a0_to_t0_by_type(&dt);
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
                if (name == "htons" || name == "ntohs") && args.len() == 1 {
                    self.compile_expr(&args[0])?;
                    self.emit(enc_i(0xff, A0, 7, T0, 0x13));
                    self.emit(enc_i(8, T0, 1, T0, 0x13));
                    self.emit(enc_i(8, A0, 5, T1, 0x13));
                    self.emit(enc_i(0xff, T1, 7, T1, 0x13));
                    self.emit(enc_r(0, T1, T0, 6, A0, 0x33));
                    return Ok(());
                }
                if (name == "htonl" || name == "ntohl") && args.len() == 1 {
                    self.compile_expr(&args[0])?;
                    self.emit(enc_i(0xff, A0, 7, T0, 0x13));
                    self.emit(enc_i(24, T0, 1, T0, 0x13));
                    self.emit(enc_i(8, A0, 5, T1, 0x13));
                    self.emit(enc_i(0xff, T1, 7, T1, 0x13));
                    self.emit(enc_i(16, T1, 1, T1, 0x13));
                    self.emit(enc_r(0, T1, T0, 6, T0, 0x33));
                    self.emit(enc_i(16, A0, 5, T1, 0x13));
                    self.emit(enc_i(0xff, T1, 7, T1, 0x13));
                    self.emit(enc_i(8, T1, 1, T1, 0x13));
                    self.emit(enc_r(0, T1, T0, 6, T0, 0x33));
                    self.emit(enc_i(24, A0, 5, T1, 0x13));
                    self.emit(enc_i(0xff, T1, 7, T1, 0x13));
                    self.emit(enc_r(0, T1, T0, 6, A0, 0x33));
                    return Ok(());
                }
                match (name.as_str(), args.len()) {
                    ("exit", 1) => return self.emit_syscall_call(args, &[A0], &[], 94),
                    ("close", 1) => return self.emit_syscall_call(args, &[A0], &[], 57),
                    ("unlink", 1) => {
                        return self.emit_syscall_call(args, &[A1], &[(A0, -100), (A2, 0)], 35)
                    }
                    ("unlinkat", 3) => return self.emit_syscall_call(args, &[A0, A1, A2], &[], 35),
                    ("open", 2) => {
                        return self.emit_syscall_call(
                            args,
                            &[A1, A2],
                            &[(A0, -100), (A3, 0o644)],
                            56,
                        )
                    }
                    ("open", 3) => {
                        return self.emit_syscall_call(args, &[A1, A2, A3], &[(A0, -100)], 56)
                    }
                    ("openat", 3) => {
                        return self.emit_syscall_call(args, &[A0, A1, A2], &[(A3, 0)], 56)
                    }
                    ("openat", 4) => {
                        return self.emit_syscall_call(args, &[A0, A1, A2, A3], &[], 56)
                    }
                    ("read", 3) => return self.emit_syscall_call(args, &[A0, A1, A2], &[], 63),
                    ("write", 3) => return self.emit_syscall_call(args, &[A0, A1, A2], &[], 64),
                    ("socket", 3) => return self.emit_syscall_call(args, &[A0, A1, A2], &[], 198),
                    ("connect", 3) => return self.emit_syscall_call(args, &[A0, A1, A2], &[], 203),
                    ("send", 4) => {
                        return self.emit_syscall_call(
                            args,
                            &[A0, A1, A2, A3],
                            &[(A4, 0), (A5, 0)],
                            206,
                        )
                    }
                    ("sendto", 6) => {
                        return self.emit_syscall_call(args, &[A0, A1, A2, A3, A4, A5], &[], 206)
                    }
                    ("recv", 4) => {
                        return self.emit_syscall_call(
                            args,
                            &[A0, A1, A2, A3],
                            &[(A4, 0), (A5, 0)],
                            207,
                        )
                    }
                    ("recvfrom", 6) => {
                        return self.emit_syscall_call(args, &[A0, A1, A2, A3, A4, A5], &[], 207)
                    }
                    _ => {}
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
                let callee = match (name.as_str(), args.len()) {
                    ("strlen", 1) => "__rt_strlen",
                    ("strcmp", 2) => "__rt_strcmp",
                    ("strncmp", 3) => "__rt_strncmp",
                    ("strcpy", 2) => "__rt_strcpy",
                    ("memcpy", 3) => "__rt_memcpy",
                    ("memset", 3) => "__rt_memset",
                    ("malloc", 1) => "__rt_malloc",
                    ("calloc", 2) => "__rt_calloc",
                    ("free", 1) => "__rt_free",
                    ("abs", 1) => "__rt_abs",
                    ("atoi", 1) => "__rt_atoi",
                    _ => name.as_str(),
                };
                self.emit_call(callee);
                Ok(())
            }
        }
    }
}

/// Formal R4.MvmLinuxContainer compile multiple C translation units with recursive `#include`, `-I`, `-D`, and `struct` support into a static RV64 ELF64 executable (`R4-63`).
pub fn compile_c_units_to_rv64_elf<F>(
    units: &[(&str, &str)],
    include_dirs: &[String],
    defines: &[(String, String)],
    read_header: F,
) -> Result<Vec<u8>, CCompileError>
where
    F: Fn(&str) -> Option<String>,
{
    if units.is_empty() {
        return Err(CCompileError {
            line: 1,
            message: "no C translation units supplied".to_string(),
        });
    }

    let mut base_defines = HashMap::new();
    base_defines.insert("__riscv".to_string(), "1".to_string());
    base_defines.insert("__riscv_xlen".to_string(), "64".to_string());
    base_defines.insert("__UOR_MVM__".to_string(), "1".to_string());
    for (k, v) in defines {
        base_defines.insert(k.clone(), v.clone());
    }

    let mut all_globals = Vec::new();
    let mut all_funcs = Vec::new();
    let mut all_structs: HashMap<String, StructDef> = HashMap::new();
    let mut seen_funcs: HashSet<String> = HashSet::new();
    let mut seen_globals: HashSet<String> = HashSet::new();

    for &(unit_path, unit_src) in units {
        let mut tu_defines = base_defines.clone();
        let mut once_headers = HashSet::new();
        let mut preprocessed = String::new();
        preprocess_unit_recursive(
            unit_path,
            unit_src,
            include_dirs,
            &mut tu_defines,
            &mut once_headers,
            &read_header,
            0,
            &mut preprocessed,
        )?;
        let toks = lex_c_tokens_with_defines(&preprocessed, &tu_defines, 1, 0)?;
        let mut parser = Parser::new(toks, HashMap::new(), HashMap::new());
        let unit_prog = parser.parse_program()?;
        all_structs.extend(parser.structs);
        for g in unit_prog.globals {
            if seen_globals.insert(g.name.clone()) {
                all_globals.push(g);
            }
        }
        for f in unit_prog.funcs {
            if !seen_funcs.insert(f.name.clone()) {
                return Err(CCompileError {
                    line: 1,
                    message: format!("duplicate definition of `{}` in `{unit_path}`", f.name),
                });
            }
            all_funcs.push(f);
        }
    }

    if !all_funcs.iter().any(|f| f.name == "main") {
        return Err(CCompileError {
            line: 1,
            message: "missing `main` entry function".to_string(),
        });
    }

    let mut cg = Codegen {
        insts: Vec::new(),
        data: Vec::new(),
        structs: all_structs,
        global_offsets: HashMap::new(),
        func_inst_idx: HashMap::new(),
        fixups: Vec::new(),
        locals: HashMap::new(),
        frame_size: 64,
        epilogue_jumps: Vec::new(),
        break_jumps: Vec::new(),
    };

    for g in &all_globals {
        while !cg.data.len().is_multiple_of(8) {
            cg.data.push(0);
        }
        let off = cg.data.len();
        cg.data.extend_from_slice(&g.init_i64.to_le_bytes());
        while cg.data.len() < off + g.size_bytes {
            cg.data.push(0);
        }
        cg.global_offsets
            .insert(g.name.clone(), (off, g.ty.clone(), g.is_array, g.size_bytes));
    }

    // _start at text[0]: Load argc/argv from initial sp, call main, SYS_exit_group(a0)
    cg.emit(enc_i(0, SP, 3, A0, 0x03)); // ld a0, 0(sp)  (argc)
    cg.emit(enc_i(8, SP, 0, A1, 0x13)); // addi a1, sp, 8 (argv)
    cg.emit_call("main");
    cg.emit(enc_i(94, X0, 0, A7, 0x13)); // SYS_exit_group
    cg.emit(0x0000_0073); // ecall

    cg.emit_runtime_helpers();
    for f in &all_funcs {
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

/// Compile a C translation unit into a static 64-bit `EM_RISCV` (`243`) ELF64 executable.
pub fn compile_c_to_rv64_elf(source: &str) -> Result<Vec<u8>, CCompileError> {
    compile_c_units_to_rv64_elf(&[("main.c", source)], &[], &[], |_| None)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compile_multi_tu_struct_and_include_r4_63() {
        let hdr = r#"
#pragma once
typedef unsigned int u32_t;
struct Pair {
    u32_t a;
    u32_t b;
};
int pair_sum(struct Pair *p);
"#;
        let util_c = r#"
#include "pair.h"
int pair_sum(struct Pair *p) {
    return p->a + p->b;
}
"#;
        let main_c = r#"
#include "pair.h"
#include <stdint.h>
int main() {
    struct Pair p;
    p.a = 19;
    p.b = 23;
    return pair_sum(&p) + sizeof(struct Pair) - 8;
}
"#;
        let inc = vec!["/usr/include".to_string(), "/workspace".to_string()];
        let elf = compile_c_units_to_rv64_elf(
            &[("/workspace/util.c", util_c), ("/workspace/main.c", main_c)],
            &inc,
            &[("MVM_TEST".to_string(), "1".to_string())],
            |path| match path {
                "/workspace/pair.h" => Some(hdr.to_string()),
                "/usr/include/stdint.h" => Some("#pragma once\n".to_string()),
                _ => None,
            },
        )
        .expect("multi-TU struct compile must succeed");
        assert_eq!(&elf[0..4], b"\x7fELF");
        assert_eq!(u16::from_le_bytes([elf[18], elf[19]]), 243);
    }
}
