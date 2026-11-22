//! RV64 MicroVM shell parser, initramfs gzip codec, and POSIX/jq userland (`R4-78`).

use super::*;

pub(super) enum StepEvent {
    CloneChild,
    Wait4 { target_pid: i64, status_ptr: u64 },
    VsockPacketEmitted(VsockPacket),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ChainOp {
    First,
    Semi,
    And,
    Or,
}

#[derive(Debug, Clone)]
pub(super) struct HeredocBody {
    pub(super) body: String,
    pub(super) expand_vars: bool,
}

#[derive(Debug, Clone)]
pub(super) struct ShellCommandSegment {
    pub(super) op: ChainOp,
    pub(super) raw: String,
    pub(super) stages: Vec<String>,
    pub(super) heredocs: Vec<HeredocBody>,
}

#[derive(Debug, Clone, Default)]
pub(super) struct StageRedirects {
    pub(super) stdin_file: Option<String>,
    pub(super) stdin_heredoc: Option<usize>,
    pub(super) stdout_file: Option<(String, bool)>,
    pub(super) stderr_file: Option<(String, bool)>,
    pub(super) stderr_to_stdout: bool,
    pub(super) stdout_to_stderr: bool,
}

#[derive(Debug, Clone)]
pub(super) struct ParsedStage {
    pub(super) assignments: Vec<(String, String)>,
    pub(super) words: Vec<String>,
    pub(super) quoted_flags: Vec<bool>,
    pub(super) redirects: StageRedirects,
}

pub(super) struct StageExecBytesOutcome {
    pub(super) exit_code: i32,
    pub(super) stdout: Vec<u8>,
    pub(super) stderr: String,
    pub(super) instructions_retired: u64,
    pub(super) vsock_frames: Vec<VsockPacket>,
    pub(super) compiled_elf_machine: Option<u16>,
}

impl StageExecBytesOutcome {
    pub(super) fn ok_empty() -> Self {
        Self {
            exit_code: 0,
            stdout: Vec::new(),
            stderr: String::new(),
            instructions_retired: 0,
            vsock_frames: Vec::new(),
            compiled_elf_machine: None,
        }
    }

    pub(super) fn ok_bytes(stdout: Vec<u8>) -> Self {
        Self {
            exit_code: 0,
            stdout,
            stderr: String::new(),
            instructions_retired: 0,
            vsock_frames: Vec::new(),
            compiled_elf_machine: None,
        }
    }

    pub(super) fn ok_text(stdout: String) -> Self {
        Self::ok_bytes(stdout.into_bytes())
    }

    pub(super) fn err(exit_code: i32, stderr: String) -> Self {
        Self {
            exit_code,
            stdout: Vec::new(),
            stderr,
            instructions_retired: 0,
            vsock_frames: Vec::new(),
            compiled_elf_machine: None,
        }
    }

    pub(super) fn from_exec_outcome(out: MvmExecOutcome) -> Self {
        Self {
            exit_code: out.exit_code,
            stdout: out.stdout.into_bytes(),
            stderr: out.stderr,
            instructions_retired: out.instructions_retired,
            vsock_frames: out.vsock_frames,
            compiled_elf_machine: out.compiled_elf_machine,
        }
    }
}

/// Parse a shell script into chained segments with attached heredoc bodies (`R4-57`).
pub(super) fn parse_shell_segments(script: &str) -> Vec<ShellCommandSegment> {
    struct PendingHeredoc {
        delim: String,
        strip_tabs: bool,
        expand_vars: bool,
    }

    let mut out = Vec::new();
    let bytes = script.as_bytes();
    let len = bytes.len();
    let mut i = 0usize;
    let mut cur: Vec<u8> = Vec::new();
    let mut next_op = ChainOp::First;
    let mut quote = 0u8;
    let mut pending_heredocs: Vec<PendingHeredoc> = Vec::new();
    let mut line_segments_start = 0usize;

    let attach_heredocs_to_line = |out_slice: &mut [ShellCommandSegment],
                                   pending: &mut Vec<PendingHeredoc>,
                                   bytes_all: &[u8],
                                   cursor: &mut usize| {
        if pending.is_empty() {
            return;
        }
        let mut bodies: Vec<HeredocBody> = Vec::with_capacity(pending.len());
        for ph in pending.drain(..) {
            let mut body = String::new();
            while *cursor < bytes_all.len() {
                let line_start = *cursor;
                while *cursor < bytes_all.len() && bytes_all[*cursor] != b'\n' {
                    *cursor += 1;
                }
                let raw_line = String::from_utf8_lossy(&bytes_all[line_start..*cursor]);
                let raw_line = raw_line.strip_suffix('\r').unwrap_or(&raw_line);
                if *cursor < bytes_all.len() && bytes_all[*cursor] == b'\n' {
                    *cursor += 1;
                }
                let candidate = if ph.strip_tabs {
                    raw_line.trim_start_matches('\t')
                } else {
                    raw_line
                };
                if candidate == ph.delim {
                    break;
                }
                body.push_str(candidate);
                body.push('\n');
            }
            bodies.push(HeredocBody {
                body,
                expand_vars: ph.expand_vars,
            });
        }
        for seg in out_slice.iter_mut() {
            seg.heredocs = bodies.clone();
        }
    };

    let split_pipes = |seg: &str| -> Vec<String> {
        let mut stages_out = Vec::new();
        let mut stage_cur: Vec<u8> = Vec::new();
        let sb = seg.as_bytes();
        let mut sq = 0u8;
        let mut si = 0usize;
        while si < sb.len() {
            if sq == 0 && sb[si] == b'\\' && si + 1 < sb.len() {
                stage_cur.push(b'\\');
                stage_cur.push(sb[si + 1]);
                si += 2;
                continue;
            }
            if sq == b'"' && sb[si] == b'\\' && si + 1 < sb.len() {
                stage_cur.push(b'\\');
                stage_cur.push(sb[si + 1]);
                si += 2;
                continue;
            }
            if sb[si] == b'\'' || sb[si] == b'"' {
                if sq == sb[si] {
                    sq = 0;
                } else if sq == 0 {
                    sq = sb[si];
                }
                stage_cur.push(sb[si]);
                si += 1;
                continue;
            }
            if sq == 0
                && sb[si] == b'|'
                && (si == 0 || sb[si - 1] != b'|')
                && (si + 1 >= sb.len() || sb[si + 1] != b'|')
            {
                let trimmed = String::from_utf8_lossy(&stage_cur).trim().to_string();
                stages_out.push(trimmed);
                stage_cur.clear();
                si += 1;
                continue;
            }
            stage_cur.push(sb[si]);
            si += 1;
        }
        let tail = String::from_utf8_lossy(&stage_cur).trim().to_string();
        if !tail.is_empty() {
            stages_out.push(tail);
        }
        stages_out
    };

    while i < len {
        let c = bytes[i];
        if quote != 0 {
            if quote == b'"' && c == b'\\' && i + 1 < len {
                cur.push(b'\\');
                cur.push(bytes[i + 1]);
                i += 2;
                continue;
            }
            if c == quote {
                quote = 0;
            }
            cur.push(c);
            i += 1;
            continue;
        }

        if c == b'\\' && i + 1 < len {
            if bytes[i + 1] == b'\n' {
                i += 2;
                continue;
            }
            cur.push(b'\\');
            cur.push(bytes[i + 1]);
            i += 2;
            continue;
        }

        if c == b'\'' || c == b'"' {
            quote = c;
            cur.push(c);
            i += 1;
            continue;
        }

        if c == b'#'
            && (cur.iter().all(|b| b.is_ascii_whitespace())
                || cur.ends_with(b" ")
                || cur.ends_with(b"\t"))
        {
            while i < len && bytes[i] != b'\n' {
                i += 1;
            }
            continue;
        }

        if c == b'<'
            && i + 1 < len
            && bytes[i + 1] == b'<'
            && (i + 2 >= len || bytes[i + 2] != b'<')
        {
            i += 2;
            let strip_tabs = if i < len && bytes[i] == b'-' {
                i += 1;
                true
            } else {
                false
            };
            while i < len && (bytes[i] == b' ' || bytes[i] == b'\t') {
                i += 1;
            }
            let mut delim: Vec<u8> = Vec::new();
            let mut expand_vars = true;
            if i < len && (bytes[i] == b'\'' || bytes[i] == b'"') {
                let q = bytes[i];
                expand_vars = false;
                i += 1;
                while i < len && bytes[i] != q && bytes[i] != b'\n' {
                    delim.push(bytes[i]);
                    i += 1;
                }
                if i < len && bytes[i] == q {
                    i += 1;
                }
            } else {
                while i < len
                    && !bytes[i].is_ascii_whitespace()
                    && !matches!(bytes[i], b'|' | b';' | b'&' | b'<' | b'>')
                {
                    if bytes[i] == b'\\' && i + 1 < len {
                        expand_vars = false;
                        delim.push(bytes[i + 1]);
                        i += 2;
                    } else {
                        delim.push(bytes[i]);
                        i += 1;
                    }
                }
            }
            let id = pending_heredocs.len();
            pending_heredocs.push(PendingHeredoc {
                delim: String::from_utf8_lossy(&delim).into_owned(),
                strip_tabs,
                expand_vars,
            });
            cur.extend_from_slice(format!(" <<__MVM_HEREDOC_{id}__ ").as_bytes());
            continue;
        }

        if c == b'&' && i + 1 < len && bytes[i + 1] == b'&' {
            let seg_raw = String::from_utf8_lossy(&cur).trim().to_string();
            if !seg_raw.is_empty() {
                let stages = split_pipes(&seg_raw);
                out.push(ShellCommandSegment {
                    op: next_op,
                    raw: seg_raw,
                    stages,
                    heredocs: Vec::new(),
                });
            }
            cur.clear();
            next_op = ChainOp::And;
            i += 2;
            continue;
        }
        if c == b'|' && i + 1 < len && bytes[i + 1] == b'|' {
            let seg_raw = String::from_utf8_lossy(&cur).trim().to_string();
            if !seg_raw.is_empty() {
                let stages = split_pipes(&seg_raw);
                out.push(ShellCommandSegment {
                    op: next_op,
                    raw: seg_raw,
                    stages,
                    heredocs: Vec::new(),
                });
            }
            cur.clear();
            next_op = ChainOp::Or;
            i += 2;
            continue;
        }
        if c == b';' {
            let seg_raw = String::from_utf8_lossy(&cur).trim().to_string();
            if !seg_raw.is_empty() {
                let stages = split_pipes(&seg_raw);
                out.push(ShellCommandSegment {
                    op: next_op,
                    raw: seg_raw,
                    stages,
                    heredocs: Vec::new(),
                });
            }
            cur.clear();
            next_op = ChainOp::Semi;
            i += 1;
            continue;
        }
        if c == b'\n' {
            let seg_raw = String::from_utf8_lossy(&cur).trim().to_string();
            if !seg_raw.is_empty() {
                let stages = split_pipes(&seg_raw);
                out.push(ShellCommandSegment {
                    op: next_op,
                    raw: seg_raw,
                    stages,
                    heredocs: Vec::new(),
                });
                next_op = ChainOp::Semi;
            }
            cur.clear();
            i += 1;
            attach_heredocs_to_line(
                &mut out[line_segments_start..],
                &mut pending_heredocs,
                bytes,
                &mut i,
            );
            line_segments_start = out.len();
            continue;
        }
        cur.push(c);
        i += 1;
    }

    let seg_raw = String::from_utf8_lossy(&cur).trim().to_string();
    if !seg_raw.is_empty() {
        let stages = split_pipes(&seg_raw);
        out.push(ShellCommandSegment {
            op: next_op,
            raw: seg_raw,
            stages,
            heredocs: Vec::new(),
        });
    }
    if !pending_heredocs.is_empty() && line_segments_start < out.len() {
        let _ = pending_heredocs;
    }
    out
}

pub(super) fn expand_var_at(
    bytes: &[u8],
    pos: usize,
    env: &BTreeMap<String, String>,
    last_exit: i32,
    pid: i64,
) -> (String, usize) {
    let len = bytes.len();
    if pos + 1 >= len {
        return ("$".to_string(), pos + 1);
    }
    let next = bytes[pos + 1];
    if next == b'?' {
        return (last_exit.to_string(), pos + 2);
    }
    if next == b'$' {
        return (pid.to_string(), pos + 2);
    }
    if next == b'{' {
        let mut end = pos + 2;
        while end < len && bytes[end] != b'}' {
            end += 1;
        }
        if end < len && bytes[end] == b'}' {
            let inner = String::from_utf8_lossy(&bytes[pos + 2..end]);
            let val = if let Some((name, default)) = inner.split_once(":-") {
                match env.get(name) {
                    Some(v) if !v.is_empty() => v.clone(),
                    _ => expand_shell_vars(default, env, last_exit, pid),
                }
            } else if let Some((name, default)) = inner.split_once('-') {
                env.get(name)
                    .cloned()
                    .unwrap_or_else(|| expand_shell_vars(default, env, last_exit, pid))
            } else {
                env.get(inner.as_ref()).cloned().unwrap_or_default()
            };
            return (val, end + 1);
        }
        return ("${".to_string(), pos + 2);
    }
    if next.is_ascii_alphabetic() || next == b'_' {
        let mut end = pos + 2;
        while end < len && (bytes[end].is_ascii_alphanumeric() || bytes[end] == b'_') {
            end += 1;
        }
        let name = String::from_utf8_lossy(&bytes[pos + 1..end]);
        return (env.get(name.as_ref()).cloned().unwrap_or_default(), end);
    }
    if next.is_ascii_digit() {
        return (String::new(), pos + 2);
    }
    ("$".to_string(), pos + 1)
}

pub(super) fn expand_shell_vars(
    raw: &str,
    env: &BTreeMap<String, String>,
    last_exit: i32,
    pid: i64,
) -> String {
    let bytes = raw.as_bytes();
    let mut out = String::with_capacity(raw.len());
    let mut i = 0usize;
    while i < bytes.len() {
        if bytes[i] == b'\\' && i + 1 < bytes.len() && bytes[i + 1] == b'$' {
            out.push('$');
            i += 2;
            continue;
        }
        if bytes[i] == b'$' {
            let (val, next_i) = expand_var_at(bytes, i, env, last_exit, pid);
            out.push_str(&val);
            i = next_i;
            continue;
        }
        let ch = raw[i..].chars().next().unwrap_or('\0');
        out.push(ch);
        i += ch.len_utf8();
    }
    out
}

pub(super) fn is_valid_env_name(name: &str) -> bool {
    let mut chars = name.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    (first.is_ascii_alphabetic() || first == '_')
        && chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
}

pub(super) fn parse_stage_words_and_redirects(
    stage: &str,
    env: &BTreeMap<String, String>,
    last_exit: i32,
    pid: i64,
) -> ParsedStage {
    #[derive(Clone, Copy, PartialEq, Eq)]
    enum TokKind {
        Word,
        Op,
    }
    let mut raw_tokens: Vec<(String, bool, bool, TokKind)> = Vec::new();
    let bytes = stage.as_bytes();
    let len = bytes.len();
    let mut cur = String::new();
    let mut cur_quoted = false;
    let mut cur_lhs_clean = true;
    let mut cur_can_assign = false;
    let mut has_token = false;
    let mut i = 0usize;
    let mut q = 0u8;

    while i < len {
        let b = bytes[i];
        if q != 0 {
            if q == b'"' && b == b'\\' && i + 1 < len {
                let ch = stage[i + 1..].chars().next().unwrap_or('\0');
                if matches!(ch, '"' | '\\' | '$' | '`') {
                    cur.push(ch);
                } else {
                    cur.push('\\');
                    cur.push(ch);
                }
                i += 1 + ch.len_utf8();
                continue;
            }
            if q == b'"' && b == b'$' {
                let (val, next_i) = expand_var_at(bytes, i, env, last_exit, pid);
                cur.push_str(&val);
                i = next_i;
                continue;
            }
            if b == q {
                q = 0;
                i += 1;
                continue;
            }
            let ch = stage[i..].chars().next().unwrap_or('\0');
            cur.push(ch);
            i += ch.len_utf8();
            continue;
        }

        if b == b'\\' && i + 1 < len {
            let ch = stage[i + 1..].chars().next().unwrap_or('\0');
            cur.push(ch);
            cur_lhs_clean = false;
            has_token = true;
            i += 1 + ch.len_utf8();
            continue;
        }
        if b == b'\'' || b == b'"' {
            q = b;
            cur_quoted = true;
            cur_lhs_clean = false;
            has_token = true;
            i += 1;
            continue;
        }
        if b.is_ascii_whitespace() {
            if has_token {
                raw_tokens.push((
                    std::mem::take(&mut cur),
                    cur_quoted,
                    cur_can_assign,
                    TokKind::Word,
                ));
                cur_quoted = false;
                cur_lhs_clean = true;
                cur_can_assign = false;
                has_token = false;
            }
            i += 1;
            continue;
        }
        if b == b'$' {
            let (val, next_i) = expand_var_at(bytes, i, env, last_exit, pid);
            cur.push_str(&val);
            cur_lhs_clean = false;
            has_token = true;
            i = next_i;
            continue;
        }

        // Redirection operators outside quotes
        let rest = &stage[i..];
        let mut matched_op: Option<&str> = None;
        for cand in [
            "2>&1", "1>&2", "&>>", "2>>", "&>", "2>", ">>", "<<", ">", "<",
        ] {
            if rest.starts_with(cand) {
                if (cand == "2>" || cand == "2>>") && !cur.is_empty() {
                    break;
                }
                if (cand == "2>" || cand == "2>>") && has_token {
                    break;
                }
                matched_op = Some(cand);
                break;
            }
        }
        if let Some(op) = matched_op {
            if has_token {
                raw_tokens.push((
                    std::mem::take(&mut cur),
                    cur_quoted,
                    cur_can_assign,
                    TokKind::Word,
                ));
                cur_quoted = false;
                cur_lhs_clean = true;
                cur_can_assign = false;
                has_token = false;
            }
            raw_tokens.push((op.to_string(), false, false, TokKind::Op));
            i += op.len();
            continue;
        }

        if b == b'=' && !cur_can_assign {
            if cur_lhs_clean && is_valid_env_name(&cur) {
                cur_can_assign = true;
            } else {
                cur_lhs_clean = false;
            }
        }

        let ch = stage[i..].chars().next().unwrap_or('\0');
        cur.push(ch);
        has_token = true;
        i += ch.len_utf8();
    }
    if has_token {
        raw_tokens.push((cur, cur_quoted, cur_can_assign, TokKind::Word));
    }

    let mut redirects = StageRedirects::default();
    let mut words = Vec::new();
    let mut quoted_flags = Vec::new();
    let mut assignments = Vec::new();
    let mut idx = 0usize;
    while idx < raw_tokens.len() {
        let (ref tok, was_quoted, can_assign, kind) = raw_tokens[idx];
        if kind == TokKind::Op && !was_quoted {
            match tok.as_str() {
                "2>&1" => {
                    redirects.stderr_to_stdout = true;
                    idx += 1;
                    continue;
                }
                "1>&2" => {
                    redirects.stdout_to_stderr = true;
                    idx += 1;
                    continue;
                }
                ">" | ">>" => {
                    if idx + 1 < raw_tokens.len() {
                        redirects.stdout_file = Some((raw_tokens[idx + 1].0.clone(), tok == ">>"));
                        idx += 2;
                        continue;
                    }
                }
                "2>" | "2>>" => {
                    if idx + 1 < raw_tokens.len() {
                        redirects.stderr_file = Some((raw_tokens[idx + 1].0.clone(), tok == "2>>"));
                        idx += 2;
                        continue;
                    }
                }
                "&>" | "&>>" => {
                    if idx + 1 < raw_tokens.len() {
                        let target = raw_tokens[idx + 1].0.clone();
                        redirects.stdout_file = Some((target, tok == "&>>"));
                        redirects.stderr_to_stdout = true;
                        idx += 2;
                        continue;
                    }
                }
                "<" => {
                    if idx + 1 < raw_tokens.len() {
                        redirects.stdin_file = Some(raw_tokens[idx + 1].0.clone());
                        idx += 2;
                        continue;
                    }
                }
                "<<" if idx + 1 < raw_tokens.len() => {
                    let marker = &raw_tokens[idx + 1].0;
                    if let Some(num_s) = marker
                        .strip_prefix("__MVM_HEREDOC_")
                        .and_then(|s| s.strip_suffix("__"))
                    {
                        if let Ok(hid) = num_s.parse::<usize>() {
                            redirects.stdin_heredoc = Some(hid);
                        }
                    }
                    idx += 2;
                    continue;
                }
                _ => {}
            }
        }
        if words.is_empty() && can_assign {
            if let Some((k, v)) = tok.split_once('=') {
                if is_valid_env_name(k) {
                    assignments.push((k.to_string(), v.to_string()));
                    idx += 1;
                    continue;
                }
            }
        }
        words.push(tok.clone());
        quoted_flags.push(was_quoted);
        idx += 1;
    }

    ParsedStage {
        assignments,
        words,
        quoted_flags,
        redirects,
    }
}

/// Formal R4.MvmLinuxContainer.
pub(super) fn glob_match(pattern: &str, text: &str) -> bool {
    let p: Vec<char> = pattern.chars().collect();
    let t: Vec<char> = text.chars().collect();
    let (mut pi, mut ti) = (0usize, 0usize);
    let mut star_pi: Option<usize> = None;
    let mut star_ti = 0usize;
    while ti < t.len() {
        if pi < p.len() && p[pi] == '*' {
            star_pi = Some(pi);
            star_ti = ti;
            pi += 1;
        } else if pi < p.len() && (p[pi] == '?' || p[pi] == t[ti]) {
            pi += 1;
            ti += 1;
        } else if pi < p.len() && p[pi] == '[' {
            if let Some(close_rel) = p[pi + 1..].iter().position(|&c| c == ']') {
                let inner = &p[pi + 1..pi + 1 + close_rel];
                let (neg, body) = if inner.first() == Some(&'!') || inner.first() == Some(&'^') {
                    (true, &inner[1..])
                } else {
                    (false, inner)
                };
                let mut hit = false;
                let mut j = 0usize;
                while j < body.len() {
                    if j + 2 < body.len() && body[j + 1] == '-' {
                        if t[ti] >= body[j] && t[ti] <= body[j + 2] {
                            hit = true;
                        }
                        j += 3;
                    } else {
                        if t[ti] == body[j] {
                            hit = true;
                        }
                        j += 1;
                    }
                }
                if hit != neg {
                    pi = pi + 1 + close_rel + 1;
                    ti += 1;
                    continue;
                }
            }
            if let Some(sp) = star_pi {
                pi = sp + 1;
                star_ti += 1;
                ti = star_ti;
            } else {
                return false;
            }
        } else if let Some(sp) = star_pi {
            pi = sp + 1;
            star_ti += 1;
            ti = star_ti;
        } else {
            return false;
        }
    }
    while pi < p.len() && p[pi] == '*' {
        pi += 1;
    }
    pi == p.len()
}

// ============================================================================
// Pure Integer CRC-32 + RFC 1952 Gzip + RFC 1951 Deflate/Inflate + USTAR Tar
// ============================================================================

pub(super) fn crc32_ieee(data: &[u8]) -> u32 {
    let mut crc = 0xFFFF_FFFFu32;
    for &b in data {
        crc ^= b as u32;
        let mut k = 0u8;
        while k < 8 {
            let mask = (crc & 1).wrapping_neg();
            crc = (crc >> 1) ^ (0xEDB8_8320u32 & mask);
            k += 1;
        }
    }
    !crc
}

pub(super) fn gzip_decompress_bytes(bytes: &[u8]) -> Result<Vec<u8>, &'static str> {
    struct DeflateBitReader<'a> {
        data: &'a [u8],
        byte_pos: usize,
        bit_buf: u32,
        bit_cnt: u8,
    }

    impl<'a> DeflateBitReader<'a> {
        fn new(data: &'a [u8]) -> Self {
            Self {
                data,
                byte_pos: 0,
                bit_buf: 0,
                bit_cnt: 0,
            }
        }

        fn ensure(&mut self, n: u8) -> Result<(), &'static str> {
            while self.bit_cnt < n {
                if self.byte_pos >= self.data.len() {
                    return Err("unexpected end of deflate stream");
                }
                self.bit_buf |= (self.data[self.byte_pos] as u32) << self.bit_cnt;
                self.byte_pos += 1;
                self.bit_cnt += 8;
            }
            Ok(())
        }

        fn read_bits(&mut self, n: u8) -> Result<u32, &'static str> {
            if n == 0 {
                return Ok(0);
            }
            self.ensure(n)?;
            let mask = if n == 32 { u32::MAX } else { (1u32 << n) - 1 };
            let val = self.bit_buf & mask;
            self.bit_buf >>= n;
            self.bit_cnt -= n;
            Ok(val)
        }

        fn align_byte(&mut self) {
            let drop_bits = self.bit_cnt & 7;
            self.bit_buf >>= drop_bits;
            self.bit_cnt -= drop_bits;
        }

        fn read_aligned_u16(&mut self) -> Result<u16, &'static str> {
            self.align_byte();
            let lo = self.read_bits(8)? as u16;
            let hi = self.read_bits(8)? as u16;
            Ok(lo | (hi << 8))
        }

        fn read_aligned_bytes(&mut self, n: usize, out: &mut Vec<u8>) -> Result<(), &'static str> {
            self.align_byte();
            let mut rem = n;
            while rem > 0 && self.bit_cnt >= 8 {
                out.push((self.bit_buf & 0xff) as u8);
                self.bit_buf >>= 8;
                self.bit_cnt -= 8;
                rem -= 1;
            }
            if self.byte_pos.saturating_add(rem) > self.data.len() {
                return Err("truncated stored block");
            }
            out.extend_from_slice(&self.data[self.byte_pos..self.byte_pos + rem]);
            self.byte_pos += rem;
            Ok(())
        }
    }

    struct HuffmanTree {
        counts: [u16; 16],
        symbols: Vec<u16>,
    }

    impl HuffmanTree {
        fn from_lengths(lengths: &[u8]) -> Result<Self, &'static str> {
            let mut counts = [0u16; 16];
            for &l in lengths {
                if (l as usize) >= 16 {
                    return Err("huffman bit length out of range");
                }
                if l > 0 {
                    counts[l as usize] += 1;
                }
            }
            let mut left = 1i32;
            for &c in counts.iter().skip(1) {
                left <<= 1;
                left -= c as i32;
                if left < 0 {
                    return Err("oversubscribed huffman tree");
                }
            }
            let mut offsets = [0u16; 16];
            let mut total = 0u16;
            for len in 1..16 {
                offsets[len] = total;
                total = total.saturating_add(counts[len]);
            }
            let mut symbols = vec![0u16; lengths.len()];
            for (sym, &len) in lengths.iter().enumerate() {
                if len > 0 {
                    let off = offsets[len as usize] as usize;
                    offsets[len as usize] += 1;
                    if off < symbols.len() {
                        symbols[off] = sym as u16;
                    }
                }
            }
            Ok(Self { counts, symbols })
        }

        fn decode(&self, br: &mut DeflateBitReader<'_>) -> Result<u16, &'static str> {
            let mut code = 0i32;
            let mut first = 0i32;
            let mut index = 0i32;
            for len in 1..16 {
                code |= br.read_bits(1)? as i32;
                let count = self.counts[len] as i32;
                if code - count < first {
                    let idx = (index + (code - first)) as usize;
                    return self
                        .symbols
                        .get(idx)
                        .copied()
                        .ok_or("invalid huffman symbol index");
                }
                index += count;
                first += count;
                first <<= 1;
                code <<= 1;
            }
            Err("invalid huffman code")
        }
    }

    let inflate_codes = |br: &mut DeflateBitReader<'_>,
                         lit_tree: &HuffmanTree,
                         dist_tree: &HuffmanTree,
                         out: &mut Vec<u8>|
     -> Result<(), &'static str> {
        const LEN_BASE: [u16; 29] = [
            3, 4, 5, 6, 7, 8, 9, 10, 11, 13, 15, 17, 19, 23, 27, 31, 35, 43, 51, 59, 67, 83, 99,
            115, 131, 163, 195, 227, 258,
        ];
        const LEN_EXTRA: [u8; 29] = [
            0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 4, 4, 4, 4, 5, 5, 5, 5, 0,
        ];
        const DIST_BASE: [u16; 30] = [
            1, 2, 3, 4, 5, 7, 9, 13, 17, 25, 33, 49, 65, 97, 129, 193, 257, 385, 513, 769, 1025,
            1537, 2049, 3073, 4097, 6145, 8193, 12289, 16385, 24577,
        ];
        const DIST_EXTRA: [u8; 30] = [
            0, 0, 0, 0, 1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 6, 6, 7, 7, 8, 8, 9, 9, 10, 10, 11, 11, 12,
            12, 13, 13,
        ];
        let mut steps = 0usize;
        while steps < 32 * 1024 * 1024 {
            steps += 1;
            let sym = lit_tree.decode(br)?;
            if sym < 256 {
                out.push(sym as u8);
                continue;
            }
            if sym == 256 {
                return Ok(());
            }
            let lidx = (sym as usize).saturating_sub(257);
            if lidx >= LEN_BASE.len() {
                return Err("invalid length symbol");
            }
            let length = (LEN_BASE[lidx] as usize) + (br.read_bits(LEN_EXTRA[lidx])? as usize);
            let dsym = dist_tree.decode(br)? as usize;
            if dsym >= DIST_BASE.len() {
                return Err("invalid distance symbol");
            }
            let dist = (DIST_BASE[dsym] as usize) + (br.read_bits(DIST_EXTRA[dsym])? as usize);
            if dist == 0 || dist > out.len() {
                return Err("invalid lz77 backreference");
            }
            if out.len().saturating_add(length) > 32 * 1024 * 1024 {
                return Err("inflated payload exceeds 32 MiB cap");
            }
            for _ in 0..length {
                let b = out[out.len() - dist];
                out.push(b);
            }
        }
        Err("deflate stream exceeded step limit")
    };

    if bytes.len() < 18 || bytes[0] != 0x1f || bytes[1] != 0x8b || bytes[2] != 0x08 {
        return Err("invalid gzip header");
    }
    let flg = bytes[3];
    let mut pos = 10usize;
    if (flg & 0x04) != 0 {
        if pos + 2 > bytes.len() {
            return Err("truncated gzip FEXTRA");
        }
        let xlen = u16::from_le_bytes([bytes[pos], bytes[pos + 1]]) as usize;
        pos = pos.saturating_add(2 + xlen);
    }
    if (flg & 0x08) != 0 {
        while pos < bytes.len() && bytes[pos] != 0 {
            pos += 1;
        }
        pos += 1;
    }
    if (flg & 0x10) != 0 {
        while pos < bytes.len() && bytes[pos] != 0 {
            pos += 1;
        }
        pos += 1;
    }
    if (flg & 0x02) != 0 {
        pos += 2;
    }
    if pos + 8 > bytes.len() {
        return Err("truncated gzip body");
    }

    let deflate_slice = &bytes[pos..bytes.len() - 8];
    let mut br = DeflateBitReader::new(deflate_slice);
    let mut out = Vec::new();
    let mut is_final = false;
    while !is_final {
        is_final = br.read_bits(1)? == 1;
        let btype = br.read_bits(2)?;
        match btype {
            0 => {
                let len = br.read_aligned_u16()?;
                let nlen = br.read_aligned_u16()?;
                if len != !nlen {
                    return Err("corrupt stored deflate block length");
                }
                br.read_aligned_bytes(len as usize, &mut out)?;
            }
            1 => {
                let mut lit_lens = [0u8; 288];
                lit_lens[0..144].fill(8);
                lit_lens[144..256].fill(9);
                lit_lens[256..280].fill(7);
                lit_lens[280..288].fill(8);
                let dist_lens = [5u8; 32];
                let lt = HuffmanTree::from_lengths(&lit_lens)?;
                let dt = HuffmanTree::from_lengths(&dist_lens)?;
                inflate_codes(&mut br, &lt, &dt, &mut out)?;
            }
            2 => {
                const CL_ORDER: [usize; 19] = [
                    16, 17, 18, 0, 8, 7, 9, 6, 10, 5, 11, 4, 12, 3, 13, 2, 14, 1, 15,
                ];
                let hlit = (br.read_bits(5)? as usize) + 257;
                let hdist = (br.read_bits(5)? as usize) + 1;
                let hclen = (br.read_bits(4)? as usize) + 4;
                if hlit > 286 || hdist > 30 {
                    return Err("invalid dynamic huffman counts");
                }
                let mut cl_lens = [0u8; 19];
                for i in 0..hclen {
                    cl_lens[CL_ORDER[i]] = br.read_bits(3)? as u8;
                }
                let cl_tree = HuffmanTree::from_lengths(&cl_lens)?;
                let total = hlit + hdist;
                let mut lens = Vec::with_capacity(total);
                while lens.len() < total {
                    let sym = cl_tree.decode(&mut br)?;
                    match sym {
                        0..=15 => lens.push(sym as u8),
                        16 => {
                            let prev = *lens.last().ok_or("invalid RLE 16")?;
                            let rep = (br.read_bits(2)? as usize) + 3;
                            for _ in 0..rep {
                                if lens.len() < total {
                                    lens.push(prev);
                                }
                            }
                        }
                        17 => {
                            let rep = (br.read_bits(3)? as usize) + 3;
                            for _ in 0..rep {
                                if lens.len() < total {
                                    lens.push(0);
                                }
                            }
                        }
                        18 => {
                            let rep = (br.read_bits(7)? as usize) + 11;
                            for _ in 0..rep {
                                if lens.len() < total {
                                    lens.push(0);
                                }
                            }
                        }
                        _ => return Err("invalid code length symbol"),
                    }
                }
                let lt = HuffmanTree::from_lengths(&lens[..hlit])?;
                let dt = HuffmanTree::from_lengths(&lens[hlit..])?;
                inflate_codes(&mut br, &lt, &dt, &mut out)?;
            }
            _ => return Err("invalid deflate block type"),
        }
    }

    let trailer = &bytes[bytes.len() - 8..];
    let expected_crc = u32::from_le_bytes([trailer[0], trailer[1], trailer[2], trailer[3]]);
    let expected_isize = u32::from_le_bytes([trailer[4], trailer[5], trailer[6], trailer[7]]);
    if crc32_ieee(&out) != expected_crc || (out.len() as u32) != expected_isize {
        return Err("gzip crc32 or size mismatch");
    }
    Ok(out)
}

/// Formal R4.MvmLinuxContainer compress bytes into an RFC 1952 gzip container using RFC 1951 Fixed-Huffman LZ77 Deflate (`R4-63`).
pub(super) fn gzip_compress_bytes(input: &[u8]) -> Vec<u8> {
    struct DeflateBitWriter {
        out: Vec<u8>,
        bit_buf: u32,
        bit_cnt: u8,
    }

    impl DeflateBitWriter {
        fn new(cap: usize) -> Self {
            Self {
                out: Vec::with_capacity(cap),
                bit_buf: 0,
                bit_cnt: 0,
            }
        }

        fn write_bit(&mut self, bit: u32) {
            self.bit_buf |= (bit & 1) << self.bit_cnt;
            self.bit_cnt += 1;
            if self.bit_cnt == 8 {
                self.out.push(self.bit_buf as u8);
                self.bit_buf = 0;
                self.bit_cnt = 0;
            }
        }

        fn write_bits_lsb(&mut self, val: u32, nbits: u8) {
            for i in 0..nbits {
                self.write_bit(val >> i);
            }
        }

        fn write_huff_msb(&mut self, code: u32, nbits: u8) {
            for i in (0..nbits).rev() {
                self.write_bit(code >> i);
            }
        }

        fn write_fixed_lit(&mut self, sym: u32) {
            if sym < 144 {
                self.write_huff_msb(0x30 + sym, 8);
            } else if sym < 256 {
                self.write_huff_msb(0x190 + (sym - 144), 9);
            } else if sym < 280 {
                self.write_huff_msb(sym - 256, 7);
            } else {
                self.write_huff_msb(0xC0 + (sym - 280), 8);
            }
        }

        fn finish(mut self) -> Vec<u8> {
            if self.bit_cnt > 0 {
                self.out.push(self.bit_buf as u8);
            }
            self.out
        }
    }

    const LEN_BASE: [usize; 29] = [
        3, 4, 5, 6, 7, 8, 9, 10, 11, 13, 15, 17, 19, 23, 27, 31, 35, 43, 51, 59, 67, 83, 99, 115,
        131, 163, 195, 227, 258,
    ];
    const LEN_EXTRA: [u8; 29] = [
        0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 4, 4, 4, 4, 5, 5, 5, 5, 0,
    ];
    const DIST_BASE: [usize; 30] = [
        1, 2, 3, 4, 5, 7, 9, 13, 17, 25, 33, 49, 65, 97, 129, 193, 257, 385, 513, 769, 1025, 1537,
        2049, 3073, 4097, 6145, 8193, 12289, 16385, 24577,
    ];
    const DIST_EXTRA: [u8; 30] = [
        0, 0, 0, 0, 1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 6, 6, 7, 7, 8, 8, 9, 9, 10, 10, 11, 11, 12, 12,
        13, 13,
    ];

    let build_stored = || -> Vec<u8> {
        let mut out = Vec::with_capacity(input.len() + 24);
        out.extend_from_slice(&[0x1f, 0x8b, 0x08, 0x00, 0, 0, 0, 0, 0, 0x03]);
        if input.is_empty() {
            out.push(0x01);
            out.extend_from_slice(&0u16.to_le_bytes());
            out.extend_from_slice(&(!0u16).to_le_bytes());
        } else {
            let chunks = input.chunks(65_535);
            let total_chunks = chunks.len();
            for (idx, chunk) in chunks.enumerate() {
                let bfinal = if idx + 1 == total_chunks {
                    0x01u8
                } else {
                    0x00u8
                };
                out.push(bfinal);
                let len16 = chunk.len() as u16;
                out.extend_from_slice(&len16.to_le_bytes());
                out.extend_from_slice(&(!len16).to_le_bytes());
                out.extend_from_slice(chunk);
            }
        }
        out.extend_from_slice(&crc32_ieee(input).to_le_bytes());
        out.extend_from_slice(&(input.len() as u32).to_le_bytes());
        out
    };

    if input.len() < 16 {
        return build_stored();
    }

    let mut bw = DeflateBitWriter::new(input.len() / 2 + 32);
    bw.out
        .extend_from_slice(&[0x1f, 0x8b, 0x08, 0x00, 0, 0, 0, 0, 0, 0x03]);
    bw.write_bits_lsb(1, 1); // BFINAL = 1
    bw.write_bits_lsb(1, 2); // BTYPE = 01 (Fixed Huffman)

    let hash3 = |idx: usize| -> usize {
        let a = input[idx] as usize;
        let b = input[idx + 1] as usize;
        let c = input[idx + 2] as usize;
        ((a << 10) ^ (b << 5) ^ c) & 4095
    };
    let mut head = vec![usize::MAX; 4096];
    let mut prev = vec![usize::MAX; 32768];
    let mut pos = 0usize;

    while pos < input.len() {
        let mut best_len = 0usize;
        let mut best_dist = 0usize;
        if pos + 3 <= input.len() {
            let h = hash3(pos);
            let mut cand = head[h];
            let mut hops = 0usize;
            while cand != usize::MAX && hops < 16 && pos - cand <= 32768 {
                let max_cmp = (input.len() - pos).min(258);
                let mut l = 0usize;
                while l < max_cmp && input[cand + l] == input[pos + l] {
                    l += 1;
                }
                if l >= 3 && l > best_len {
                    best_len = l;
                    best_dist = pos - cand;
                    if best_len == 258 {
                        break;
                    }
                }
                cand = prev[cand & 32767];
                hops += 1;
            }
        }

        if best_len >= 3 {
            let mut lidx = 0usize;
            while lidx + 1 < LEN_BASE.len() && best_len >= LEN_BASE[lidx + 1] {
                lidx += 1;
            }
            bw.write_fixed_lit((257 + lidx) as u32);
            let lextra = (best_len - LEN_BASE[lidx]) as u32;
            if LEN_EXTRA[lidx] > 0 {
                bw.write_bits_lsb(lextra, LEN_EXTRA[lidx]);
            }

            let mut didx = 0usize;
            while didx + 1 < DIST_BASE.len() && best_dist >= DIST_BASE[didx + 1] {
                didx += 1;
            }
            bw.write_huff_msb(didx as u32, 5);
            let dextra = (best_dist - DIST_BASE[didx]) as u32;
            if DIST_EXTRA[didx] > 0 {
                bw.write_bits_lsb(dextra, DIST_EXTRA[didx]);
            }

            let end_pos = (pos + best_len).min(input.len().saturating_sub(2));
            for k in pos..end_pos {
                let h = hash3(k);
                prev[k & 32767] = head[h];
                head[h] = k;
            }
            pos += best_len;
        } else {
            if pos + 3 <= input.len() {
                let h = hash3(pos);
                prev[pos & 32767] = head[h];
                head[h] = pos;
            }
            bw.write_fixed_lit(input[pos] as u32);
            pos += 1;
        }
    }

    bw.write_fixed_lit(256); // End of block
    let mut deflated = bw.finish();
    deflated.extend_from_slice(&crc32_ieee(input).to_le_bytes());
    deflated.extend_from_slice(&(input.len() as u32).to_le_bytes());
    if deflated.len() < input.len() + 24 {
        deflated
    } else {
        build_stored()
    }
}

/// Formal R4.MvmLinuxContainer pack a raw `UORMVM01` initramfs image into a compressed `UORMVMZ1` container (`R4-63`).
pub fn pack_compressed_initramfs_uormvmz1(uormvm01_bytes: &[u8]) -> Result<Vec<u8>, MvmError> {
    if uormvm01_bytes.len() < 12 || &uormvm01_bytes[0..8] != b"UORMVM01" {
        return Err(MvmError::CorruptInitramfs(
            "expected UORMVM01 payload for compression".to_string(),
        ));
    }
    let gz = gzip_compress_bytes(uormvm01_bytes);
    let mut packed = Vec::with_capacity(8 + gz.len());
    packed.extend_from_slice(b"UORMVMZ1");
    packed.extend_from_slice(&gz);
    Ok(packed)
}

// ============================================================================
// Pure-Rust Native Coreutils Applets (`R4-57`)
// ============================================================================

impl MvmSandbox {
    pub(super) fn try_exec_native_coreutil(
        &mut self,
        prog: &str,
        args: &[String],
        cwd: &mut String,
        stdin: &[u8],
        stage_env: &BTreeMap<String, String>,
    ) -> Option<StageExecBytesOutcome> {
        if let Some(mut out) = self
            .exec_coreutil_archive(prog, args, cwd, stdin)
            .or_else(|| self.exec_coreutil_text_lang(prog, args, cwd, stdin))
        {
            let _ = self.cow_state_sha256();
            if out.exit_code != 0
                && out.stderr.is_empty()
                && prog != "diff"
                && prog != "grep"
                && prog != "rg"
            {
                out.stderr = format!("{prog}: operation failed\n");
            }
            return Some(out);
        }

        fn unescape_c_escapes(s: &str) -> (Vec<u8>, bool) {
            let bytes = s.as_bytes();
            let mut out = Vec::with_capacity(bytes.len());
            let mut i = 0usize;
            while i < bytes.len() {
                if bytes[i] == b'\\' {
                    if i + 1 < bytes.len() && bytes[i + 1] == b'c' {
                        return (out, true);
                    }
                    let (seq, adv) = parse_c_escape_at(bytes, i);
                    out.extend_from_slice(&seq);
                    i += adv;
                } else {
                    out.push(bytes[i]);
                    i += 1;
                }
            }
            (out, false)
        }

        fn utc_from_epoch(epoch_secs: i64) -> (i64, u32, u32, u32, u32, u32, i64) {
            let days = epoch_secs.div_euclid(86_400);
            let rem = epoch_secs.rem_euclid(86_400) as u32;
            let hr = rem / 3_600;
            let mn = (rem % 3_600) / 60;
            let sc = rem % 60;
            let wday = (days + 4).rem_euclid(7);
            // Howard Hinnant's civil_from_days
            let z = days + 719_468;
            let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
            let doe = (z - era * 146_097) as u64;
            let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
            let y = (yoe as i64) + era * 400;
            let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
            let mp = (5 * doy + 2) / 153;
            let d = doy - (153 * mp + 2) / 5 + 1;
            let m = if mp < 10 { mp + 3 } else { mp - 9 };
            let year = y + if m <= 2 { 1 } else { 0 };
            (year, m as u32, d as u32, hr, mn, sc, wday)
        }

        fn apply_chmod_spec(cur_perm: u32, spec: &str) -> Result<u32, ()> {
            if spec.chars().all(|c| ('0'..='7').contains(&c)) && !spec.is_empty() {
                return u32::from_str_radix(spec, 8).map_err(|_| ());
            }
            let mut mode = cur_perm;
            for clause in spec.split(',') {
                let chars: Vec<char> = clause.chars().collect();
                let mut idx = 0usize;
                let mut who = 0u32;
                while idx < chars.len() && matches!(chars[idx], 'u' | 'g' | 'o' | 'a') {
                    match chars[idx] {
                        'u' => who |= 0o700,
                        'g' => who |= 0o070,
                        'o' => who |= 0o007,
                        'a' => who |= 0o777,
                        _ => {}
                    }
                    idx += 1;
                }
                if who == 0 {
                    who = 0o777;
                }
                if idx >= chars.len() || !matches!(chars[idx], '+' | '-' | '=') {
                    return Err(());
                }
                let op = chars[idx];
                idx += 1;
                let mut bits = 0u32;
                while idx < chars.len() {
                    match chars[idx] {
                        'r' => bits |= 0o444,
                        'w' => bits |= 0o222,
                        'x' | 'X' => bits |= 0o111,
                        _ => return Err(()),
                    }
                    idx += 1;
                }
                let masked = bits & who;
                match op {
                    '+' => mode |= masked,
                    '-' => mode &= !masked,
                    '=' => mode = (mode & !who) | masked,
                    _ => return Err(()),
                }
            }
            Ok(mode)
        }

        fn ls_collect_dir(
            sandbox: &MvmSandbox,
            dir_norm: &str,
            show_all: bool,
            long_fmt: bool,
            recursive: bool,
            show_header: bool,
            out: &mut String,
        ) {
            if show_header || recursive {
                out.push_str(&format!("{dir_norm}:\n"));
            }
            let entries = sandbox.list_dir_entries(dir_norm);
            let mut subdirs = Vec::new();
            for (dtype, name) in entries {
                if !show_all && name.starts_with('.') {
                    continue;
                }
                if name == "." || name == ".." {
                    if show_all {
                        out.push_str(&format!("{name}\n"));
                    }
                    continue;
                }
                let full = if dir_norm == "/" {
                    format!("/{name}")
                } else {
                    format!("{dir_norm}/{name}")
                };
                if dtype == 4 && recursive {
                    subdirs.push(full.clone());
                }
                if long_fmt {
                    let (perm, size) = if dtype == 4 {
                        ("drwxr-xr-x", 0usize)
                    } else if dtype == 10 {
                        ("lrwxrwxrwx", 0usize)
                    } else {
                        let mode = sandbox
                            .inodes
                            .get(&full)
                            .map(|i| i.mode)
                            .unwrap_or(0o100644);
                        let perm_s = if (mode & 0o111) != 0 {
                            "-rwxr-xr-x"
                        } else {
                            "-rw-r--r--"
                        };
                        (
                            perm_s,
                            sandbox.read_guest_file(&full).map(|b| b.len()).unwrap_or(0),
                        )
                    };
                    out.push_str(&format!("{perm} 1 lexlean lexlean {size:>8} {name}\n"));
                } else {
                    out.push_str(&format!("{name}\n"));
                }
            }
            for sd in subdirs {
                out.push('\n');
                ls_collect_dir(sandbox, &sd, show_all, long_fmt, recursive, true, out);
            }
        }

        let mut out = match prog {
            "true" => StageExecBytesOutcome::ok_empty(),
            "false" => StageExecBytesOutcome::err(1, String::new()),
            "pwd" => StageExecBytesOutcome::ok_text(format!("{cwd}\n")),
            "echo" => {
                let mut newline = true;
                let mut escapes = false;
                let mut idx = 0usize;
                while idx < args.len() {
                    let a = &args[idx];
                    if a.starts_with('-')
                        && a.len() > 1
                        && a[1..].chars().all(|c| matches!(c, 'n' | 'e' | 'E'))
                    {
                        for ch in a[1..].chars() {
                            match ch {
                                'n' => newline = false,
                                'e' => escapes = true,
                                'E' => escapes = false,
                                _ => {}
                            }
                        }
                        idx += 1;
                    } else {
                        break;
                    }
                }
                let joined = args[idx..].join(" ");
                let mut out = if escapes {
                    unescape_c_escapes(&joined).0
                } else {
                    joined.into_bytes()
                };
                if newline {
                    out.push(b'\n');
                }
                StageExecBytesOutcome::ok_bytes(out)
            }
            "printf" => (|| -> StageExecBytesOutcome {
                if args.is_empty() {
                    return StageExecBytesOutcome::err(
                        1,
                        "printf: usage: printf format [arguments]\n".to_string(),
                    );
                }
                let fmt = &args[0];
                let mut vals = &args[1..];
                let mut out = Vec::new();
                loop {
                    let mut arg_idx = 0usize;
                    let bytes = fmt.as_bytes();
                    let mut i = 0usize;
                    while i < bytes.len() {
                        if bytes[i] == b'\\' {
                            let (seq, adv) = parse_c_escape_at(bytes, i);
                            out.extend_from_slice(&seq);
                            i += adv;
                            continue;
                        }
                        if bytes[i] == b'%' {
                            if i + 1 < bytes.len() && bytes[i + 1] == b'%' {
                                out.push(b'%');
                                i += 2;
                                continue;
                            }
                            let mut j = i + 1;
                            let mut left_align = false;
                            let mut zero_pad = false;
                            while j < bytes.len() && matches!(bytes[j], b'-' | b'0' | b'+' | b' ') {
                                if bytes[j] == b'-' {
                                    left_align = true;
                                }
                                if bytes[j] == b'0' {
                                    zero_pad = true;
                                }
                                j += 1;
                            }
                            let w_start = j;
                            while j < bytes.len() && bytes[j].is_ascii_digit() {
                                j += 1;
                            }
                            let width: usize = std::str::from_utf8(&bytes[w_start..j])
                                .ok()
                                .and_then(|s| s.parse().ok())
                                .unwrap_or(0);
                            let mut prec: Option<usize> = None;
                            if j < bytes.len() && bytes[j] == b'.' {
                                j += 1;
                                let p_start = j;
                                while j < bytes.len() && bytes[j].is_ascii_digit() {
                                    j += 1;
                                }
                                prec = Some(
                                    std::str::from_utf8(&bytes[p_start..j])
                                        .ok()
                                        .and_then(|s| s.parse().ok())
                                        .unwrap_or(0),
                                );
                            }
                            if let Some(spec) = fmt.get(j..).and_then(|s| s.chars().next()) {
                                let arg_val = vals.get(arg_idx).map(String::as_str).unwrap_or("");
                                arg_idx += 1;
                                let rendered = match spec {
                                    's' => {
                                        let mut s = arg_val.to_string();
                                        if let Some(p) = prec {
                                            s = s.chars().take(p).collect();
                                        }
                                        pad_formatted_field(&s, width, left_align, ' ')
                                    }
                                    'b' => {
                                        let (b, _) = unescape_c_escapes(arg_val);
                                        let s = String::from_utf8_lossy(&b).into_owned();
                                        pad_formatted_field(&s, width, left_align, ' ')
                                    }
                                    'c' => {
                                        let ch = arg_val.chars().next().unwrap_or('\0');
                                        pad_formatted_field(&ch.to_string(), width, left_align, ' ')
                                    }
                                    'd' | 'i' => {
                                        let n = parse_printf_i64(arg_val);
                                        let neg = n < 0;
                                        let abs_s = n.unsigned_abs().to_string();
                                        let pad_c = if zero_pad && !left_align { '0' } else { ' ' };
                                        let raw_s = if neg && pad_c == '0' {
                                            let padded = pad_formatted_field(
                                                &abs_s,
                                                width.saturating_sub(1),
                                                false,
                                                '0',
                                            );
                                            format!("-{padded}")
                                        } else {
                                            let signed =
                                                if neg { format!("-{abs_s}") } else { abs_s };
                                            pad_formatted_field(&signed, width, left_align, pad_c)
                                        };
                                        raw_s
                                    }
                                    'u' => {
                                        let n = parse_printf_i64(arg_val) as u64;
                                        pad_formatted_field(
                                            &n.to_string(),
                                            width,
                                            left_align,
                                            if zero_pad && !left_align { '0' } else { ' ' },
                                        )
                                    }
                                    'x' | 'X' => {
                                        let n = parse_printf_i64(arg_val) as u64;
                                        let s = if spec == 'x' {
                                            format!("{n:x}")
                                        } else {
                                            format!("{n:X}")
                                        };
                                        pad_formatted_field(
                                            &s,
                                            width,
                                            left_align,
                                            if zero_pad && !left_align { '0' } else { ' ' },
                                        )
                                    }
                                    'o' => {
                                        let n = parse_printf_i64(arg_val) as u64;
                                        pad_formatted_field(
                                            &format!("{n:o}"),
                                            width,
                                            left_align,
                                            if zero_pad && !left_align { '0' } else { ' ' },
                                        )
                                    }
                                    other => format!("%{other}"),
                                };
                                out.extend_from_slice(rendered.as_bytes());
                                i = j + spec.len_utf8();
                                continue;
                            }
                        }
                        let ch = fmt.get(i..).and_then(|s| s.chars().next()).unwrap_or('\0');
                        let mut tmp = [0u8; 4];
                        out.extend_from_slice(ch.encode_utf8(&mut tmp).as_bytes());
                        i += ch.len_utf8();
                    }
                    if arg_idx == 0 || arg_idx >= vals.len() {
                        break;
                    }
                    vals = &vals[arg_idx..];
                }
                StageExecBytesOutcome::ok_bytes(out)
            })(),
            "env" => (|| -> StageExecBytesOutcome {
                let mut cur_env = stage_env.clone();
                let mut idx = 0usize;
                while idx < args.len() {
                    let a = &args[idx];
                    if a == "-i" || a == "--ignore-environment" {
                        cur_env.clear();
                        idx += 1;
                    } else if a == "-u" && idx + 1 < args.len() {
                        cur_env.remove(&args[idx + 1]);
                        idx += 2;
                    } else if let Some((k, v)) = a.split_once('=') {
                        cur_env.insert(k.to_string(), v.to_string());
                        idx += 1;
                    } else {
                        break;
                    }
                }
                if idx >= args.len() {
                    let mut out = String::new();
                    for (k, v) in &cur_env {
                        out.push_str(&format!("{k}={v}\n"));
                    }
                    return StageExecBytesOutcome::ok_text(out);
                }
                self.exec_single_command_words(&args[idx..], cwd, stdin, &cur_env)
                    .unwrap_or_else(|e| StageExecBytesOutcome::err(1, format!("{e}\n")))
            })(),
            "which" => (|| -> StageExecBytesOutcome {
                let mut show_all = false;
                let mut targets = Vec::new();
                for a in args {
                    if a == "-a" {
                        show_all = true;
                    } else if !a.starts_with('-') {
                        targets.push(a.as_str());
                    }
                }
                if targets.is_empty() {
                    return StageExecBytesOutcome::err(
                        1,
                        "which: missing command argument\n".to_string(),
                    );
                }
                let path_val = stage_env
                    .get("PATH")
                    .map(String::as_str)
                    .unwrap_or("/usr/local/bin:/bin:/sbin:/usr/bin:/workspace");
                let mut stdout = String::new();
                let mut stderr = String::new();
                let mut all_found = true;
                for name in targets {
                    let mut found_list = Vec::new();
                    if name.contains('/') {
                        let p = Self::normalize_guest_path(&self.current_cwd, name);
                        if self.inodes.contains_key(&p) {
                            found_list.push(p);
                        }
                    } else {
                        for dir in path_val.split(':') {
                            let cand = if dir == "/" {
                                format!("/{name}")
                            } else {
                                format!("{dir}/{name}")
                            };
                            if self.inodes.contains_key(&cand) && !found_list.contains(&cand) {
                                found_list.push(cand);
                            }
                        }
                        if is_native_coreutil_or_builtin(name) {
                            let builtin_path = format!("/usr/bin/{name}");
                            if !found_list.contains(&builtin_path) {
                                found_list.push(builtin_path);
                            }
                        }
                    }
                    if found_list.is_empty() {
                        all_found = false;
                        stderr.push_str(&format!("which: no {name} in ({path_val})\n"));
                    } else if show_all {
                        for f in found_list {
                            stdout.push_str(&f);
                            stdout.push('\n');
                        }
                    } else {
                        stdout.push_str(&found_list[0]);
                        stdout.push('\n');
                    }
                }
                StageExecBytesOutcome {
                    exit_code: if all_found { 0 } else { 1 },
                    stdout: stdout.into_bytes(),
                    stderr,
                    instructions_retired: 0,
                    vsock_frames: Vec::new(),
                    compiled_elf_machine: None,
                }
            })(),
            "ps" => {
                let aux = args.iter().any(|a| a == "aux" || a == "-ef" || a == "-e");
                let out = if aux && args.iter().any(|a| a == "aux") {
                    format!(
                "USER       PID %CPU %MEM    VSZ   RSS TTY      STAT START   TIME COMMAND\n\
                 lexlean      1  0.0  0.1   8192  1024 ?        Ss   00:00   0:00 /sbin/mvm-agent --vsock\n\
                 lexlean     10  0.0  0.1   8192  1024 pts/0    S    00:00   0:00 /bin/mvmbox sh\n\
                 lexlean   {:>4}  0.0  0.1   4096   512 pts/0    R+   00:00   0:00 ps\n",
                self.next_pid
            )
                } else {
                    format!(
                        "  PID TTY          TIME CMD\n\
                     1 ?        00:00:00 /sbin/mvm-agent\n\
                    10 pts/0    00:00:00 /bin/mvmbox sh\n\
                 {:>5} pts/0    00:00:00 ps\n",
                        self.next_pid
                    )
                };
                StageExecBytesOutcome::ok_text(out)
            }
            "uname" => (|| -> StageExecBytesOutcome {
                let mut s = false;
                let mut n = false;
                let mut r = false;
                let mut v = false;
                let mut m = false;
                let mut p = false;
                let mut i_flag = false;
                let mut o = false;
                if args.is_empty() {
                    s = true;
                } else {
                    for a in args {
                        if !a.starts_with('-') || a.len() == 1 {
                            return StageExecBytesOutcome::err(
                                1,
                                format!("uname: extra operand '{a}'\n"),
                            );
                        }
                        for ch in a[1..].chars() {
                            match ch {
                                'a' => {
                                    s = true;
                                    n = true;
                                    r = true;
                                    v = true;
                                    m = true;
                                    o = true;
                                }
                                's' => s = true,
                                'n' => n = true,
                                'r' => r = true,
                                'v' => v = true,
                                'm' => m = true,
                                'p' => p = true,
                                'i' => i_flag = true,
                                'o' => o = true,
                                other => {
                                    return StageExecBytesOutcome::err(
                                        1,
                                        format!("uname: invalid option -- '{other}'\n"),
                                    );
                                }
                            }
                        }
                    }
                }
                let mut parts = Vec::new();
                if s {
                    parts.push("Linux");
                }
                if n {
                    parts.push("mvm-wasm");
                }
                if r {
                    parts.push("6.6.0-uor-rv64imac");
                }
                if v {
                    parts.push("#1 SMP PREEMPT_DYNAMIC");
                }
                if m {
                    parts.push("riscv64");
                }
                if p {
                    parts.push("riscv64");
                }
                if i_flag {
                    parts.push("riscv64");
                }
                if o {
                    parts.push("GNU/Linux");
                }
                StageExecBytesOutcome::ok_text(format!("{}\n", parts.join(" ")))
            })(),
            "date" => (|| -> StageExecBytesOutcome {
                let epoch_secs = 1_794_332_000i64;
                let mut fmt: Option<String> = None;
                let mut iso = false;
                let mut rfc = false;
                for a in args {
                    if let Some(f) = a.strip_prefix('+') {
                        fmt = Some(f.to_string());
                    } else if a == "-u" || a == "--utc" {
                        continue;
                    } else if a == "-I" || a == "--iso-8601" {
                        iso = true;
                    } else if a == "-R" || a == "--rfc-email" {
                        rfc = true;
                    } else if a.starts_with('-') {
                        return StageExecBytesOutcome::err(
                            1,
                            format!("date: invalid option '{a}'\n"),
                        );
                    }
                }
                let (yr, mo, dy, hr, mn, sc, wday) = utc_from_epoch(epoch_secs);
                const WDAYS: [&str; 7] = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];
                const MONTHS: [&str; 12] = [
                    "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov",
                    "Dec",
                ];
                let wname = WDAYS[(wday as usize) % 7];
                let mname = MONTHS[((mo - 1) as usize) % 12];
                let out = if iso {
                    format!("{yr:04}-{mo:02}-{dy:02}\n")
                } else if rfc {
                    format!("{wname}, {dy:02} {mname} {yr:04} {hr:02}:{mn:02}:{sc:02} +0000\n")
                } else if let Some(f) = fmt {
                    let mut rendered = String::new();
                    let fc: Vec<char> = f.chars().collect();
                    let mut idx = 0usize;
                    while idx < fc.len() {
                        if fc[idx] == '%' && idx + 1 < fc.len() {
                            match fc[idx + 1] {
                                'Y' => rendered.push_str(&format!("{yr:04}")),
                                'm' => rendered.push_str(&format!("{mo:02}")),
                                'd' => rendered.push_str(&format!("{dy:02}")),
                                'H' => rendered.push_str(&format!("{hr:02}")),
                                'M' => rendered.push_str(&format!("{mn:02}")),
                                'S' => rendered.push_str(&format!("{sc:02}")),
                                's' => rendered.push_str(&format!("{epoch_secs}")),
                                'F' => rendered.push_str(&format!("{yr:04}-{mo:02}-{dy:02}")),
                                'T' => rendered.push_str(&format!("{hr:02}:{mn:02}:{sc:02}")),
                                'a' => rendered.push_str(wname),
                                'b' | 'h' => rendered.push_str(mname),
                                'Z' => rendered.push_str("UTC"),
                                'z' => rendered.push_str("+0000"),
                                'n' => rendered.push('\n'),
                                't' => rendered.push('\t'),
                                '%' => rendered.push('%'),
                                other => {
                                    rendered.push('%');
                                    rendered.push(other);
                                }
                            }
                            idx += 2;
                        } else {
                            rendered.push(fc[idx]);
                            idx += 1;
                        }
                    }
                    rendered.push('\n');
                    rendered
                } else {
                    format!("{wname} {mname} {dy:02} {hr:02}:{mn:02}:{sc:02} UTC {yr:04}\n")
                };
                StageExecBytesOutcome::ok_text(out)
            })(),
            "cat" => (|| -> StageExecBytesOutcome {
                let mut number_lines = false;
                let mut squeeze = false;
                let mut files = Vec::new();
                for a in args {
                    if a == "-" {
                        files.push(a.clone());
                    } else if a.starts_with('-') && a.len() > 1 {
                        for ch in a[1..].chars() {
                            match ch {
                                'n' => number_lines = true,
                                's' => squeeze = true,
                                'A' | 'v' | 'E' | 'T' => {}
                                _ => {
                                    return StageExecBytesOutcome::err(
                                        1,
                                        format!("cat: invalid option -- '{ch}'\n"),
                                    )
                                }
                            }
                        }
                    } else {
                        files.push(a.clone());
                    }
                }
                let sources = match self.read_input_sources(&files, cwd, stdin, "cat") {
                    Ok(s) => s,
                    Err(e) => return StageExecBytesOutcome::err(1, e),
                };
                if !number_lines && !squeeze {
                    let mut out = Vec::new();
                    for (_, b) in sources {
                        out.extend_from_slice(&b);
                    }
                    return StageExecBytesOutcome::ok_bytes(out);
                }
                let mut out = String::new();
                let mut line_num = 1usize;
                let mut prev_blank = false;
                for (_, b) in sources {
                    let text = String::from_utf8_lossy(&b);
                    for line in text.split_inclusive('\n') {
                        let is_blank = line == "\n" || line.is_empty();
                        if squeeze && is_blank && prev_blank {
                            continue;
                        }
                        prev_blank = is_blank;
                        if number_lines {
                            out.push_str(&format!("{line_num:>6}\t{line}"));
                            line_num += 1;
                        } else {
                            out.push_str(line);
                        }
                    }
                }
                StageExecBytesOutcome::ok_text(out)
            })(),
            "ls" => (|| -> StageExecBytesOutcome {
                let mut long_fmt = false;
                let mut show_all = false;
                let mut recursive = false;
                let mut targets = Vec::new();
                for a in args {
                    if a.starts_with('-') && a.len() > 1 {
                        for ch in a[1..].chars() {
                            match ch {
                                'l' => long_fmt = true,
                                'a' | 'A' => show_all = true,
                                'R' => recursive = true,
                                '1' | 'h' | 'F' => {}
                                _ => {
                                    return StageExecBytesOutcome::err(
                                        1,
                                        format!("ls: invalid option -- '{ch}'\n"),
                                    )
                                }
                            }
                        }
                    } else {
                        targets.push(a.as_str());
                    }
                }
                if targets.is_empty() {
                    targets.push(".");
                }
                let mut out = String::new();
                let mut err = String::new();
                let mut exit_code = 0i32;
                let multi = targets.len() > 1;
                for raw_t in targets {
                    let norm = Self::normalize_guest_path(cwd, raw_t);
                    if self.inodes.contains_key(&norm) && !self.dirs.contains(&norm) {
                        if long_fmt {
                            let size = self.read_guest_file(&norm).map(|b| b.len()).unwrap_or(0);
                            out.push_str(&format!(
                                "-rw-r--r-- 1 lexlean lexlean {size:>8} {raw_t}\n"
                            ));
                        } else {
                            out.push_str(&format!("{raw_t}\n"));
                        }
                        continue;
                    }
                    if !self.dirs.contains(&norm) {
                        exit_code = 1;
                        err.push_str(&format!(
                            "ls: cannot access '{raw_t}': No such file or directory\n"
                        ));
                        continue;
                    }
                    ls_collect_dir(self, &norm, show_all, long_fmt, recursive, multi, &mut out);
                }
                StageExecBytesOutcome {
                    exit_code,
                    stdout: out.into_bytes(),
                    stderr: err,
                    instructions_retired: 0,
                    vsock_frames: Vec::new(),
                    compiled_elf_machine: None,
                }
            })(),
            "mkdir" => (|| -> StageExecBytesOutcome {
                let mut parents = false;
                let mut targets = Vec::new();
                for a in args {
                    if a == "-p" || a == "--parents" {
                        parents = true;
                    } else if !a.starts_with('-') {
                        targets.push(a.as_str());
                    }
                }
                if targets.is_empty() {
                    return StageExecBytesOutcome::err(1, "mkdir: missing operand\n".to_string());
                }
                for raw_t in targets {
                    let norm = Self::normalize_guest_path(cwd, raw_t);
                    if self.inodes.contains_key(&norm) {
                        return StageExecBytesOutcome::err(
                            1,
                            format!("mkdir: cannot create directory '{raw_t}': File exists\n"),
                        );
                    }
                    if self.dirs.contains(&norm) {
                        if parents {
                            continue;
                        }
                        return StageExecBytesOutcome::err(
                            1,
                            format!("mkdir: cannot create directory '{raw_t}': File exists\n"),
                        );
                    }
                    if Self::is_protected_guest_path(&norm) {
                        return StageExecBytesOutcome::err(
                            1,
                            format!("mvm: refused mkdir on protected path '{norm}'\n"),
                        );
                    }
                    if !parents {
                        let parent = norm
                            .rsplit_once('/')
                            .map(|(p, _)| if p.is_empty() { "/" } else { p })
                            .unwrap_or("/");
                        if !self.dirs.contains(parent) {
                            return StageExecBytesOutcome::err(
                        1,
                        format!("mkdir: cannot create directory '{raw_t}': No such file or directory\n"),
                    );
                        }
                    }
                    self.ensure_parent_dirs(&norm);
                    self.dirs.insert(norm);
                }
                StageExecBytesOutcome::ok_empty()
            })(),
            "rmdir" => (|| -> StageExecBytesOutcome {
                let targets: Vec<&str> = args
                    .iter()
                    .filter(|a| !a.starts_with('-'))
                    .map(String::as_str)
                    .collect();
                if targets.is_empty() {
                    return StageExecBytesOutcome::err(1, "rmdir: missing operand\n".to_string());
                }
                for raw_t in targets {
                    let norm = Self::normalize_guest_path(cwd, raw_t);
                    if Self::is_protected_guest_path(&norm) {
                        return StageExecBytesOutcome::err(
                            1,
                            format!("rmdir: refusing to remove protected path '{norm}'\n"),
                        );
                    }
                    if !self.dirs.contains(&norm) {
                        return StageExecBytesOutcome::err(
                            1,
                            format!(
                                "rmdir: failed to remove '{raw_t}': No such file or directory\n"
                            ),
                        );
                    }
                    let prefix = format!("{norm}/");
                    let non_empty = self.inodes.keys().any(|k| k.starts_with(&prefix))
                        || self.dirs.iter().any(|d| d.starts_with(&prefix));
                    if non_empty {
                        return StageExecBytesOutcome::err(
                            1,
                            format!("rmdir: failed to remove '{raw_t}': Directory not empty\n"),
                        );
                    }
                    self.dirs.remove(&norm);
                }
                StageExecBytesOutcome::ok_empty()
            })(),
            "cp" => (|| -> StageExecBytesOutcome {
                let mut recursive = false;
                let mut paths = Vec::new();
                for a in args {
                    if a.starts_with('-') && a.len() > 1 {
                        for ch in a[1..].chars() {
                            match ch {
                                'r' | 'R' | 'a' => recursive = true,
                                'f' | 'p' | 'v' => {}
                                _ => {
                                    return StageExecBytesOutcome::err(
                                        1,
                                        format!("cp: invalid option -- '{ch}'\n"),
                                    )
                                }
                            }
                        }
                    } else {
                        paths.push(a.as_str());
                    }
                }
                if paths.len() < 2 {
                    return StageExecBytesOutcome::err(1, "cp: missing file operand\n".to_string());
                }
                let dst_raw = paths[paths.len() - 1];
                let dst_norm = Self::normalize_guest_path(cwd, dst_raw);
                let dst_is_dir = self.dirs.contains(&dst_norm);
                if paths.len() > 2 && !dst_is_dir {
                    return StageExecBytesOutcome::err(
                        1,
                        format!("cp: target '{dst_raw}' is not a directory\n"),
                    );
                }
                for &src_raw in &paths[..paths.len() - 1] {
                    let src_norm = Self::normalize_guest_path(cwd, src_raw);
                    if self.dirs.contains(&src_norm) && !self.inodes.contains_key(&src_norm) {
                        if !recursive {
                            return StageExecBytesOutcome::err(
                                1,
                                format!("cp: -r not specified; omitting directory '{src_raw}'\n"),
                            );
                        }
                        let src_base = src_norm.rsplit('/').next().unwrap_or("dir");
                        let target_root = if dst_is_dir {
                            Self::normalize_guest_path(&dst_norm, src_base)
                        } else {
                            dst_norm.clone()
                        };
                        if Self::is_protected_guest_path(&target_root) {
                            return StageExecBytesOutcome::err(
                                1,
                                format!("mvm: refused copy to protected path '{target_root}'\n"),
                            );
                        }
                        let prefix = format!("{src_norm}/");
                        let child_dirs: Vec<String> = self
                            .dirs
                            .iter()
                            .filter(|d| d.starts_with(&prefix))
                            .cloned()
                            .collect();
                        let mut dst_dirs = Vec::with_capacity(child_dirs.len());
                        for cd in &child_dirs {
                            if let Some(rel) = cd.strip_prefix(&prefix) {
                                let dst_cd = Self::normalize_guest_path(&target_root, rel);
                                if Self::is_protected_guest_path(&dst_cd) {
                                    return StageExecBytesOutcome::err(
                                        1,
                                        format!("mvm: refused copy to protected path '{dst_cd}'\n"),
                                    );
                                }
                                dst_dirs.push(dst_cd);
                            }
                        }
                        let child_files: Vec<(String, u32, Vec<u8>)> = self
                            .inodes
                            .iter()
                            .filter(|(k, _)| k.starts_with(&prefix))
                            .filter_map(|(k, ino)| {
                                self.read_guest_file(k).map(|b| (k.clone(), ino.mode, b))
                            })
                            .collect();
                        for (cf, _, _) in &child_files {
                            if let Some(rel) = cf.strip_prefix(&prefix) {
                                let out_p = Self::normalize_guest_path(&target_root, rel);
                                if Self::is_protected_guest_path(&out_p) {
                                    return StageExecBytesOutcome::err(
                                        1,
                                        format!("mvm: refused write to protected canonical path '{out_p}'\n"),
                                    );
                                }
                            }
                        }
                        self.ensure_parent_dirs(&target_root);
                        self.dirs.insert(target_root.clone());
                        for dst_cd in dst_dirs {
                            self.dirs.insert(dst_cd);
                        }
                        for (cf, mode, bytes) in child_files {
                            if let Some(rel) = cf.strip_prefix(&prefix) {
                                let out_p = Self::normalize_guest_path(&target_root, rel);
                                if let Err(msg) = self.write_redirected_bytes(
                                    "/",
                                    &out_p,
                                    &bytes,
                                    false,
                                    Some(mode),
                                ) {
                                    return StageExecBytesOutcome::err(1, format!("{msg}\n"));
                                }
                            }
                        }
                    } else if let Some(bytes) = self.read_guest_file(&src_norm) {
                        let mode = self
                            .inodes
                            .get(&src_norm)
                            .map(|i| i.mode)
                            .unwrap_or(0o100644);
                        let out_p = if dst_is_dir {
                            let base = src_norm.rsplit('/').next().unwrap_or("file");
                            if dst_norm == "/" {
                                format!("/{base}")
                            } else {
                                format!("{dst_norm}/{base}")
                            }
                        } else {
                            dst_norm.clone()
                        };
                        if let Err(msg) =
                            self.write_redirected_bytes("/", &out_p, &bytes, false, Some(mode))
                        {
                            return StageExecBytesOutcome::err(1, format!("{msg}\n"));
                        }
                    } else {
                        return StageExecBytesOutcome::err(
                            1,
                            format!("cp: cannot stat '{src_raw}': No such file or directory\n"),
                        );
                    }
                }
                StageExecBytesOutcome::ok_empty()
            })(),
            "mv" => (|| -> StageExecBytesOutcome {
                let paths: Vec<&str> = args
                    .iter()
                    .filter(|a| !a.starts_with('-'))
                    .map(String::as_str)
                    .collect();
                if paths.len() < 2 {
                    return StageExecBytesOutcome::err(1, "mv: missing file operand\n".to_string());
                }
                let dst_raw = paths[paths.len() - 1];
                let dst_norm = Self::normalize_guest_path(cwd, dst_raw);
                let dst_is_dir = self.dirs.contains(&dst_norm);
                if paths.len() > 2 && !dst_is_dir {
                    return StageExecBytesOutcome::err(
                        1,
                        format!("mv: target '{dst_raw}' is not a directory\n"),
                    );
                }
                for &src_raw in &paths[..paths.len() - 1] {
                    let src_norm = Self::normalize_guest_path(cwd, src_raw);
                    if Self::is_protected_guest_path(&src_norm) {
                        return StageExecBytesOutcome::err(
                            1,
                            format!("mvm: refused move of protected canonical path '{src_norm}'\n"),
                        );
                    }
                    if self.dirs.contains(&src_norm) && !self.inodes.contains_key(&src_norm) {
                        let target_root = if dst_is_dir {
                            let base = src_norm.rsplit('/').next().unwrap_or("dir");
                            Self::normalize_guest_path(&dst_norm, base)
                        } else {
                            dst_norm.clone()
                        };
                        if Self::is_protected_guest_path(&target_root) {
                            return StageExecBytesOutcome::err(
                                1,
                                format!("mvm: refused write to protected canonical path '{target_root}'\n"),
                            );
                        }
                        let prefix = format!("{src_norm}/");
                        let child_files: Vec<(String, GuestInode)> = self
                            .inodes
                            .iter()
                            .filter(|(k, _)| k.starts_with(&prefix))
                            .map(|(k, v)| (k.clone(), v.clone()))
                            .collect();
                        for (cf, _) in &child_files {
                            if Self::is_protected_guest_path(cf) {
                                return StageExecBytesOutcome::err(
                                    1,
                                    format!(
                                        "mvm: refused move of protected canonical path '{cf}'\n"
                                    ),
                                );
                            }
                            if let Some(rel) = cf.strip_prefix(&prefix) {
                                let np = Self::normalize_guest_path(&target_root, rel);
                                if Self::is_protected_guest_path(&np) {
                                    return StageExecBytesOutcome::err(
                                        1,
                                        format!("mvm: refused write to protected canonical path '{np}'\n"),
                                    );
                                }
                            }
                        }
                        self.ensure_parent_dirs(&target_root);
                        self.dirs.insert(target_root.clone());
                        let child_dirs: Vec<String> = self
                            .dirs
                            .iter()
                            .filter(|d| d.starts_with(&prefix))
                            .cloned()
                            .collect();
                        for cd in child_dirs {
                            self.dirs.remove(&cd);
                            if let Some(rel) = cd.strip_prefix(&prefix) {
                                self.dirs.insert(Self::normalize_guest_path(&target_root, rel));
                            }
                        }
                        self.dirs.remove(&src_norm);
                        for (cf, ino) in child_files {
                            self.inodes.remove(&cf);
                            if let Some(rel) = cf.strip_prefix(&prefix) {
                                let dst_child = Self::normalize_guest_path(&target_root, rel);
                                if let Some(old) = self.inodes.remove(&dst_child) {
                                    self.free_inode_sectors(&old);
                                }
                                self.inodes.insert(dst_child, ino);
                            }
                        }
                    } else if let Some(ino) = self.inodes.get(&src_norm).cloned() {
                        let out_p = if dst_is_dir {
                            let base = src_norm.rsplit('/').next().unwrap_or("file");
                            if dst_norm == "/" {
                                format!("/{base}")
                            } else {
                                format!("{dst_norm}/{base}")
                            }
                        } else {
                            dst_norm.clone()
                        };
                        if Self::is_protected_guest_path(&out_p) {
                            return StageExecBytesOutcome::err(
                                1,
                                format!(
                                    "mvm: refused write to protected canonical path '{out_p}'\n"
                                ),
                            );
                        }
                        self.ensure_parent_dirs(&out_p);
                        self.inodes.remove(&src_norm);
                        if let Some(old) = self.inodes.remove(&out_p) {
                            self.free_inode_sectors(&old);
                        }
                        self.inodes.insert(out_p, ino);
                    } else {
                        return StageExecBytesOutcome::err(
                            1,
                            format!("mv: cannot stat '{src_raw}': No such file or directory\n"),
                        );
                    }
                }
                StageExecBytesOutcome::ok_empty()
            })(),
            "touch" => (|| -> StageExecBytesOutcome {
                let mut no_create = false;
                let mut files = Vec::new();
                for a in args {
                    if a == "-c" || a == "--no-create" {
                        no_create = true;
                    } else if a == "-a" || a == "-m" {
                        continue;
                    } else if a.starts_with('-') {
                        return StageExecBytesOutcome::err(
                            1,
                            format!("touch: invalid option '{a}'\n"),
                        );
                    } else {
                        files.push(a.as_str());
                    }
                }
                if files.is_empty() {
                    return StageExecBytesOutcome::err(
                        1,
                        "touch: missing file operand\n".to_string(),
                    );
                }
                for raw_f in files {
                    let norm = Self::normalize_guest_path(cwd, raw_f);
                    if Self::is_protected_guest_path(&norm) {
                        return StageExecBytesOutcome::err(
                            1,
                            format!("mvm: refused write to protected canonical path '{norm}'\n"),
                        );
                    }
                    if self.dirs.contains(&norm) || self.inodes.contains_key(&norm) {
                        continue;
                    }
                    if !no_create {
                        if let Err(msg) =
                            self.write_redirected_bytes("/", &norm, &[], false, Some(0o100644))
                        {
                            return StageExecBytesOutcome::err(1, format!("{msg}\n"));
                        }
                    }
                }
                StageExecBytesOutcome::ok_empty()
            })(),
            "chmod" => (|| -> StageExecBytesOutcome {
                let mut recursive = false;
                let mut pos_args = Vec::new();
                for a in args {
                    if a == "-R" {
                        recursive = true;
                    } else {
                        pos_args.push(a.as_str());
                    }
                }
                if pos_args.len() < 2 {
                    return StageExecBytesOutcome::err(1, "chmod: missing operand\n".to_string());
                }
                let mode_spec = pos_args[0];
                let mut updated_paths = Vec::new();
                for &raw_t in &pos_args[1..] {
                    let norm = Self::normalize_guest_path(cwd, raw_t);
                    if Self::is_protected_guest_path(&norm) {
                        return StageExecBytesOutcome::err(
                            1,
                            format!("mvm: refused chmod on protected canonical path '{norm}'\n"),
                        );
                    }
                    if self.inodes.contains_key(&norm) {
                        updated_paths.push(norm);
                    } else if self.dirs.contains(&norm) {
                        if recursive {
                            let prefix = format!("{norm}/");
                            for k in self.inodes.keys() {
                                if k.starts_with(&prefix) {
                                    if Self::is_protected_guest_path(k) {
                                        return StageExecBytesOutcome::err(
                                            1,
                                            format!(
                                                "mvm: refused chmod on protected canonical path '{k}'\n"
                                            ),
                                        );
                                    }
                                    updated_paths.push(k.clone());
                                }
                            }
                        }
                    } else {
                        return StageExecBytesOutcome::err(
                            1,
                            format!("chmod: cannot access '{raw_t}': No such file or directory\n"),
                        );
                    }
                }
                let mut parsed_updates = Vec::with_capacity(updated_paths.len());
                for path in updated_paths {
                    if let Some(ino) = self.inodes.get(&path) {
                        let kind_bits = ino.mode & 0o170000;
                        let perm = ino.mode & 0o7777;
                        let Ok(new_perm) = apply_chmod_spec(perm, mode_spec) else {
                            return StageExecBytesOutcome::err(
                                1,
                                format!("chmod: invalid mode: '{mode_spec}'\n"),
                            );
                        };
                        parsed_updates.push((path, kind_bits | new_perm));
                    }
                }
                for (path, next_mode) in parsed_updates {
                    if let Some(ino) = self.inodes.get_mut(&path) {
                        ino.mode = next_mode;
                    }
                }
                StageExecBytesOutcome::ok_empty()
            })(),
            "ln" => (|| -> StageExecBytesOutcome {
                let mut symlink = false;
                let mut force = false;
                let mut paths = Vec::new();
                for a in args {
                    if a.starts_with('-') && a.len() > 1 {
                        for ch in a[1..].chars() {
                            match ch {
                                's' => symlink = true,
                                'f' => force = true,
                                'n' | 'v' => {}
                                _ => {
                                    return StageExecBytesOutcome::err(
                                        1,
                                        format!("ln: invalid option -- '{ch}'\n"),
                                    )
                                }
                            }
                        }
                    } else {
                        paths.push(a.as_str());
                    }
                }
                if paths.len() < 2 {
                    return StageExecBytesOutcome::err(1, "ln: missing file operand\n".to_string());
                }
                let target_raw = paths[0];
                let link_raw = paths[1];
                let mut link_norm = Self::normalize_guest_path(cwd, link_raw);
                if self.dirs.contains(&link_norm) {
                    let trimmed_target = target_raw.trim_end_matches('/');
                    let base = trimmed_target.rsplit('/').next().unwrap_or("");
                    if base.is_empty() || base == "." || base == ".." {
                        return StageExecBytesOutcome::err(
                            1,
                            format!("ln: failed to create link '{target_raw}': invalid link name\n"),
                        );
                    }
                    link_norm = Self::normalize_guest_path(&link_norm, base);
                }
                if Self::is_protected_guest_path(&link_norm) {
                    return StageExecBytesOutcome::err(
                        1,
                        format!("mvm: refused write to protected canonical path '{link_norm}'\n"),
                    );
                }
                if !force && self.inodes.contains_key(&link_norm) {
                    return StageExecBytesOutcome::err(
                        1,
                        format!("ln: failed to create link '{link_raw}': File exists\n"),
                    );
                }
                if symlink {
                    if let Err(msg) = self.write_symlink_inode(&link_norm, target_raw) {
                        return StageExecBytesOutcome::err(1, format!("{msg}\n"));
                    }
                    StageExecBytesOutcome::ok_empty()
                } else {
                    let src_norm = Self::normalize_guest_path(cwd, target_raw);
                    let Some(bytes) = self.read_guest_file(&src_norm) else {
                        return StageExecBytesOutcome::err(
                            1,
                            format!(
                                "ln: failed to access '{target_raw}': No such file or directory\n"
                            ),
                        );
                    };
                    let mode = self
                        .inodes
                        .get(&src_norm)
                        .map(|i| i.mode)
                        .unwrap_or(0o100644);
                    if let Err(msg) =
                        self.write_redirected_bytes("/", &link_norm, &bytes, false, Some(mode))
                    {
                        return StageExecBytesOutcome::err(1, format!("{msg}\n"));
                    }
                    StageExecBytesOutcome::ok_empty()
                }
            })(),
            "tee" => (|| -> StageExecBytesOutcome {
                let mut append = false;
                let mut files = Vec::new();
                for a in args {
                    if a == "-a" || a == "--append" {
                        append = true;
                    } else if !a.starts_with('-') {
                        files.push(a.as_str());
                    }
                }
                for f in files {
                    if let Err(msg) = self.write_redirected_bytes(cwd, f, stdin, append, None) {
                        return StageExecBytesOutcome::err(1, format!("{msg}\n"));
                    }
                }
                StageExecBytesOutcome::ok_bytes(stdin.to_vec())
            })(),
            "dirname" => (|| -> StageExecBytesOutcome {
                let Some(p) = args.iter().find(|a| !a.starts_with('-')) else {
                    return StageExecBytesOutcome::err(1, "dirname: missing operand\n".to_string());
                };
                let trimmed = p.trim_end_matches('/');
                let dir = if trimmed.is_empty() {
                    "/"
                } else if let Some((left, _)) = trimmed.rsplit_once('/') {
                    if left.is_empty() {
                        "/"
                    } else {
                        left
                    }
                } else {
                    "."
                };
                StageExecBytesOutcome::ok_text(format!("{dir}\n"))
            })(),
            "basename" => (|| -> StageExecBytesOutcome {
                let pos: Vec<&str> = args
                    .iter()
                    .filter(|a| !a.starts_with('-'))
                    .map(String::as_str)
                    .collect();
                let Some(&p) = pos.first() else {
                    return StageExecBytesOutcome::err(
                        1,
                        "basename: missing operand\n".to_string(),
                    );
                };
                let trimmed = p.trim_end_matches('/');
                let mut base = if trimmed.is_empty() {
                    "/"
                } else {
                    trimmed.rsplit('/').next().unwrap_or(trimmed)
                };
                if let Some(&suf) = pos.get(1) {
                    if base != suf {
                        if let Some(stripped) = base.strip_suffix(suf) {
                            base = stripped;
                        }
                    }
                }
                StageExecBytesOutcome::ok_text(format!("{base}\n"))
            })(),
            "readlink" | "realpath" => (|| -> StageExecBytesOutcome {
                let mut canonicalize = prog == "realpath";
                let mut targets = Vec::new();
                for a in args {
                    if a == "-f" || a == "-e" {
                        canonicalize = true;
                    } else if !a.starts_with('-') {
                        targets.push(a.as_str());
                    }
                }
                let Some(&raw) = targets.first() else {
                    return StageExecBytesOutcome::err(1, format!("{prog}: missing operand\n"));
                };
                let mut cur = Self::normalize_guest_path(cwd, raw);
                if !canonicalize {
                    return match self.inodes.get(&cur) {
                        Some(GuestInode {
                            kind: InodeKind::Symlink { target },
                            ..
                        }) => StageExecBytesOutcome::ok_text(format!("{target}\n")),
                        _ => {
                            StageExecBytesOutcome::err(1, format!("{prog}: {raw}: not a symlink\n"))
                        }
                    };
                }
                for _ in 0..8 {
                    if let Some(GuestInode {
                        kind: InodeKind::Symlink { target },
                        ..
                    }) = self.inodes.get(&cur)
                    {
                        cur = Self::resolve_symlink_step(&cur, target);
                    } else {
                        break;
                    }
                }
                if !self.inodes.contains_key(&cur) && !self.dirs.contains(&cur) {
                    return StageExecBytesOutcome::err(
                        1,
                        format!("{prog}: {raw}: No such file or directory\n"),
                    );
                }
                StageExecBytesOutcome::ok_text(format!("{cur}\n"))
            })(),
            "stat" => (|| -> StageExecBytesOutcome {
                let Some(raw) = args.iter().find(|a| !a.starts_with('-')) else {
                    return StageExecBytesOutcome::err(1, "stat: missing operand\n".to_string());
                };
                let norm = Self::normalize_guest_path(cwd, raw);
                if let Some(ino) = self.inodes.get(&norm) {
                    let (kind_s, size) = match &ino.kind {
                        InodeKind::Regular { byte_len, .. } => ("regular file", *byte_len),
                        InodeKind::Symlink { target } => ("symbolic link", target.len()),
                    };
                    StageExecBytesOutcome::ok_text(format!(
                        "  File: {raw}\n  Size: {size}\tKind: {kind_s}\nAccess: ({:04o})\n",
                        ino.mode & 0o7777
                    ))
                } else if self.dirs.contains(&norm) {
                    StageExecBytesOutcome::ok_text(format!(
                        "  File: {raw}\n  Size: 0\tKind: directory\nAccess: (0755)\n"
                    ))
                } else {
                    StageExecBytesOutcome::err(
                        1,
                        format!("stat: cannot stat '{raw}': No such file or directory\n"),
                    )
                }
            })(),
            "test" | "[" => (|| -> StageExecBytesOutcome {
                let slice = if prog == "[" {
                    if args.last().map(String::as_str) != Some("]") {
                        return StageExecBytesOutcome::err(2, "[: missing `]`\n".to_string());
                    }
                    &args[..args.len() - 1]
                } else {
                    args
                };
                let ok = match slice {
                    [] => false,
                    [s] => !s.is_empty(),
                    [flag, arg] if flag == "-n" => !arg.is_empty(),
                    [flag, arg] if flag == "-z" => arg.is_empty(),
                    [flag, arg] if flag == "-f" => {
                        let n = Self::normalize_guest_path(cwd, arg);
                        self.inodes.contains_key(&n)
                    }
                    [flag, arg] if flag == "-d" => {
                        let n = Self::normalize_guest_path(cwd, arg);
                        self.dirs.contains(&n)
                    }
                    [flag, arg] if flag == "-e" => {
                        let n = Self::normalize_guest_path(cwd, arg);
                        self.inodes.contains_key(&n) || self.dirs.contains(&n)
                    }
                    [a, op, b] => match op.as_str() {
                        "=" | "==" => a == b,
                        "!=" => a != b,
                        "-eq" => {
                            a.parse::<i64>().ok() == b.parse::<i64>().ok()
                                && a.parse::<i64>().is_ok()
                        }
                        "-ne" => a.parse::<i64>().ok() != b.parse::<i64>().ok(),
                        "-lt" => a.parse::<i64>().unwrap_or(0) < b.parse::<i64>().unwrap_or(0),
                        "-le" => a.parse::<i64>().unwrap_or(0) <= b.parse::<i64>().unwrap_or(0),
                        "-gt" => a.parse::<i64>().unwrap_or(0) > b.parse::<i64>().unwrap_or(0),
                        "-ge" => a.parse::<i64>().unwrap_or(0) >= b.parse::<i64>().unwrap_or(0),
                        _ => false,
                    },
                    _ => false,
                };
                if ok {
                    StageExecBytesOutcome::ok_empty()
                } else {
                    StageExecBytesOutcome::err(1, String::new())
                }
            })(),
            "yes" => {
                let s = args.first().map(String::as_str).unwrap_or("y");
                let mut buf = String::new();
                for _ in 0..256 {
                    buf.push_str(s);
                    buf.push('\n');
                }
                StageExecBytesOutcome::ok_text(buf)
            }
            "clear" => StageExecBytesOutcome::ok_empty(),
            _ => return None,
        };
        let _ = self.cow_state_sha256();
        if out.exit_code != 0
            && out.stderr.is_empty()
            && prog != "false"
            && prog != "test"
            && prog != "["
        {
            out.stderr = format!("{prog}: operation failed\n");
        }
        Some(out)
    }

    fn exec_coreutil_archive(
        &mut self,
        prog: &str,
        args: &[String],
        cwd: &str,
        stdin: &[u8],
    ) -> Option<StageExecBytesOutcome> {
        fn write_tar_octal(buf: &mut [u8], val: u64) {
            let len = buf.len();
            if len == 0 {
                return;
            }
            let digits = len.saturating_sub(1);
            let s = format!("{val:0digits$o}");
            let sb = s.as_bytes();
            let take = sb.len().min(digits);
            let start = digits - take;
            for b in &mut buf[..digits] {
                *b = b'0';
            }
            buf[start..digits].copy_from_slice(&sb[sb.len() - take..]);
            buf[len - 1] = 0;
        }

        fn tar_build_header(
            name: &str,
            mode: u32,
            size: u64,
            typeflag: u8,
            linkname: &str,
        ) -> [u8; 512] {
            let mut hdr = [0u8; 512];
            let full_bytes = name.as_bytes();
            let (pb, nb) = if full_bytes.len() > 100 {
                if let Some(pos) = name.rfind('/') {
                    let p = &full_bytes[..pos];
                    let n = &full_bytes[pos + 1..];
                    if p.len() <= 155 && n.len() <= 100 {
                        (p, n)
                    } else {
                        (&[][..], &full_bytes[..100])
                    }
                } else {
                    (&[][..], &full_bytes[..100])
                }
            } else {
                (&[][..], full_bytes)
            };
            hdr[..nb.len().min(100)].copy_from_slice(&nb[..nb.len().min(100)]);
            write_tar_octal(&mut hdr[100..108], (mode & 0o7777) as u64);
            write_tar_octal(&mut hdr[108..116], 1000);
            write_tar_octal(&mut hdr[116..124], 1000);
            write_tar_octal(&mut hdr[124..136], size);
            write_tar_octal(&mut hdr[136..148], 1_794_332_000);
            for b in &mut hdr[148..156] {
                *b = b' ';
            }
            hdr[156] = typeflag;
            let lb = linkname.as_bytes();
            hdr[157..157 + lb.len().min(100)].copy_from_slice(&lb[..lb.len().min(100)]);
            hdr[257..263].copy_from_slice(b"ustar\0");
            hdr[263..265].copy_from_slice(b"00");
            hdr[265..272].copy_from_slice(b"lexlean");
            hdr[297..304].copy_from_slice(b"lexlean");
            hdr[345..345 + pb.len().min(155)].copy_from_slice(&pb[..pb.len().min(155)]);

            let sum: u32 = hdr.iter().map(|&b| b as u32).sum();
            let chk = format!("{sum:06o}\0 ");
            hdr[148..156].copy_from_slice(chk.as_bytes());
            hdr
        }

        struct TarEntry {
            name: String,
            mode: u32,
            typeflag: u8,
            linkname: String,
            data: Vec<u8>,
        }

        fn parse_tar_octal(field: &[u8]) -> Result<u64, &'static str> {
            let s = String::from_utf8_lossy(field);
            let trimmed = s.trim_matches(|c: char| c == '\0' || c == ' ');
            if trimmed.is_empty() {
                return Ok(0);
            }
            u64::from_str_radix(trimmed, 8).map_err(|_| "invalid tar octal field")
        }

        fn tar_parse_entries(bytes: &[u8]) -> Result<Vec<TarEntry>, &'static str> {
            if bytes.is_empty() || !bytes.len().is_multiple_of(512) {
                return Err("invalid tar archive length");
            }
            let mut entries = Vec::new();
            let mut off = 0usize;
            while off + 512 <= bytes.len() {
                let hdr = &bytes[off..off + 512];
                if hdr.iter().all(|&b| b == 0) {
                    break;
                }
                let expected_sum = parse_tar_octal(&hdr[148..156])?;
                let mut actual_sum = 0u64;
                for (idx, &b) in hdr.iter().enumerate() {
                    actual_sum += if (148..156).contains(&idx) {
                        b' ' as u64
                    } else {
                        b as u64
                    };
                }
                if actual_sum != expected_sum {
                    return Err("invalid tar header checksum");
                }
                let name_end = hdr[..100].iter().position(|&b| b == 0).unwrap_or(100);
                let name_part = String::from_utf8_lossy(&hdr[..name_end]);
                let pref_end = hdr[345..500].iter().position(|&b| b == 0).unwrap_or(155);
                let pref_part = String::from_utf8_lossy(&hdr[345..345 + pref_end]);
                let full_name = if pref_part.is_empty() {
                    name_part.into_owned()
                } else {
                    format!("{pref_part}/{name_part}")
                };
                let mode = parse_tar_octal(&hdr[100..108])? as u32;
                let size = parse_tar_octal(&hdr[124..136])? as usize;
                let typeflag = if hdr[156] == 0 { b'0' } else { hdr[156] };
                let link_end = hdr[157..257].iter().position(|&b| b == 0).unwrap_or(100);
                let linkname = String::from_utf8_lossy(&hdr[157..157 + link_end]).into_owned();
                off += 512;
                let padded = size.div_ceil(512) * 512;
                if off.saturating_add(padded) > bytes.len() {
                    return Err("truncated tar entry payload");
                }
                let data = bytes[off..off + size].to_vec();
                off += padded;
                entries.push(TarEntry {
                    name: full_name,
                    mode,
                    typeflag,
                    linkname,
                    data,
                });
            }
            Ok(entries)
        }

        match prog {
            "tar" => Some((|| -> StageExecBytesOutcome {
                let mut mode: Option<char> = None;
                let mut archive_path: Option<String> = None;
                let mut gzip = false;
                let mut verbose = false;
                let mut chdir: Option<String> = None;
                let mut operands: Vec<String> = Vec::new();
                let mut i = 0usize;
                while i < args.len() {
                    let a = &args[i];
                    if a == "-C" && i + 1 < args.len() {
                        chdir = Some(args[i + 1].clone());
                        i += 2;
                        continue;
                    } else if a == "-f" && i + 1 < args.len() {
                        archive_path = Some(args[i + 1].clone());
                        i += 2;
                        continue;
                    } else if a.starts_with('-') || (i == 0 && mode.is_none()) {
                        let chars_part = a.strip_prefix('-').unwrap_or(a);
                        let mut need_f_next = false;
                        for ch in chars_part.chars() {
                            match ch {
                                'c' | 'x' | 't' => mode = Some(ch),
                                'z' => gzip = true,
                                'v' => verbose = true,
                                'p' => {}
                                'f' => need_f_next = true,
                                _ => {
                                    return StageExecBytesOutcome::err(
                                        2,
                                        format!("tar: invalid option -- '{ch}'\n"),
                                    )
                                }
                            }
                        }
                        if need_f_next {
                            if i + 1 >= args.len() {
                                return StageExecBytesOutcome::err(
                                    2,
                                    "tar: option requires an argument -- 'f'\n".to_string(),
                                );
                            }
                            archive_path = Some(args[i + 1].clone());
                            i += 2;
                            continue;
                        }
                    } else {
                        operands.push(a.clone());
                    }
                    i += 1;
                }
                let base_dir = match chdir {
                    Some(ref d) => Self::normalize_guest_path(cwd, d),
                    None => cwd.to_string(),
                };
                let Some(m) = mode else {
                    return StageExecBytesOutcome::err(
                        2,
                        "tar: must specify one of -c, -x, or -t\n".to_string(),
                    );
                };
                match m {
                    'c' => {
                        if operands.is_empty() {
                            return StageExecBytesOutcome::err(
                                2,
                                "tar: Cowardly refusing to create an empty archive\n".to_string(),
                            );
                        }
                        let mut tar_buf = Vec::new();
                        let mut v_out = String::new();
                        for op in &operands {
                            let norm = Self::normalize_guest_path(&base_dir, op);
                            if self.dirs.contains(&norm) && !self.inodes.contains_key(&norm) {
                                let rel = op.trim_end_matches('/');
                                let hdr = tar_build_header(&format!("{rel}/"), 0o755, 0, b'5', "");
                                tar_buf.extend_from_slice(&hdr);
                                if verbose {
                                    v_out.push_str(&format!("{rel}/\n"));
                                }
                                let pref = if norm == "/" {
                                    "/".to_string()
                                } else {
                                    format!("{norm}/")
                                };
                                let sub_files: Vec<String> = self
                                    .inodes
                                    .keys()
                                    .filter(|k| k.starts_with(&pref))
                                    .cloned()
                                    .collect();
                                for sf in sub_files {
                                    let sub_rel = sf.strip_prefix(&pref).unwrap_or(&sf);
                                    let entry_name = if rel == "." {
                                        sub_rel.to_string()
                                    } else {
                                        format!("{rel}/{sub_rel}")
                                    };
                                    if let Some(bytes) = self.read_guest_file(&sf) {
                                        let mode =
                                            self.inodes.get(&sf).map(|x| x.mode).unwrap_or(0o644);
                                        tar_buf.extend_from_slice(&tar_build_header(
                                            &entry_name,
                                            mode,
                                            bytes.len() as u64,
                                            b'0',
                                            "",
                                        ));
                                        tar_buf.extend_from_slice(&bytes);
                                        let rem = bytes.len() % 512;
                                        if rem != 0 {
                                            tar_buf.resize(tar_buf.len() + (512 - rem), 0);
                                        }
                                        if verbose {
                                            v_out.push_str(&format!("{entry_name}\n"));
                                        }
                                    }
                                }
                            } else if let Some(bytes) = self.read_guest_file(&norm) {
                                let mode = self.inodes.get(&norm).map(|x| x.mode).unwrap_or(0o644);
                                tar_buf.extend_from_slice(&tar_build_header(
                                    op,
                                    mode,
                                    bytes.len() as u64,
                                    b'0',
                                    "",
                                ));
                                tar_buf.extend_from_slice(&bytes);
                                let rem = bytes.len() % 512;
                                if rem != 0 {
                                    tar_buf.resize(tar_buf.len() + (512 - rem), 0);
                                }
                                if verbose {
                                    v_out.push_str(&format!("{op}\n"));
                                }
                            } else {
                                return StageExecBytesOutcome::err(
                                    2,
                                    format!("tar: {op}: Cannot stat: No such file or directory\n"),
                                );
                            }
                        }
                        tar_buf.resize(tar_buf.len() + 1024, 0);
                        let final_bytes = if gzip {
                            gzip_compress_bytes(&tar_buf)
                        } else {
                            tar_buf
                        };
                        if let Some(ref af) = archive_path {
                            if af == "-" {
                                return StageExecBytesOutcome::ok_bytes(final_bytes);
                            }
                            if let Err(msg) =
                                self.write_redirected_bytes(cwd, af, &final_bytes, false, None)
                            {
                                return StageExecBytesOutcome::err(2, format!("{msg}\n"));
                            }
                            StageExecBytesOutcome::ok_text(v_out)
                        } else {
                            StageExecBytesOutcome::ok_bytes(final_bytes)
                        }
                    }
                    't' | 'x' => {
                        let raw_bytes = match archive_path.as_deref() {
                            None | Some("-") => stdin.to_vec(),
                            Some(af) => {
                                let norm = Self::normalize_guest_path(cwd, af);
                                match self.read_guest_file(&norm) {
                                    Some(b) => b,
                                    None => {
                                        return StageExecBytesOutcome::err(
                                            2,
                                            format!(
                                            "tar: {af}: Cannot open: No such file or directory\n"
                                        ),
                                        )
                                    }
                                }
                            }
                        };
                        let is_gz = gzip
                            || (raw_bytes.len() >= 2
                                && raw_bytes[0] == 0x1f
                                && raw_bytes[1] == 0x8b);
                        let tar_bytes = if is_gz {
                            match gzip_decompress_bytes(&raw_bytes) {
                                Ok(b) => b,
                                Err(e) => {
                                    return StageExecBytesOutcome::err(2, format!("tar: {e}\n"))
                                }
                            }
                        } else {
                            raw_bytes
                        };
                        let entries = match tar_parse_entries(&tar_bytes) {
                            Ok(e) => e,
                            Err(e) => return StageExecBytesOutcome::err(2, format!("tar: {e}\n")),
                        };
                        let mut out = String::new();
                        for ent in entries {
                            if m == 't' || verbose {
                                out.push_str(&ent.name);
                                out.push('\n');
                            }
                            if m == 'x' {
                                let clean_name =
                                    ent.name.trim_start_matches("./").trim_start_matches('/');
                                if clean_name.is_empty() || clean_name.split('/').any(|s| s == "..")
                                {
                                    return StageExecBytesOutcome::err(
                                        2,
                                        format!("tar: unsafe path '{}' rejected\n", ent.name),
                                    );
                                }
                                let dst = Self::normalize_guest_path(&base_dir, clean_name);
                                if Self::is_protected_guest_path(&dst) {
                                    return StageExecBytesOutcome::err(
                                        2,
                                        format!("mvm: refused extract to protected path '{dst}'\n"),
                                    );
                                }
                                if ent.typeflag == b'5' {
                                    self.ensure_parent_dirs(&dst);
                                    self.dirs.insert(dst);
                                } else if ent.typeflag == b'2' {
                                    if let Err(msg) = self.write_symlink_inode(&dst, &ent.linkname) {
                                        return StageExecBytesOutcome::err(2, format!("{msg}\n"));
                                    }
                                } else {
                                    if let Err(msg) = self.write_redirected_bytes(
                                        "/",
                                        &dst,
                                        &ent.data,
                                        false,
                                        Some(0o100000 | (ent.mode & 0o777)),
                                    ) {
                                        return StageExecBytesOutcome::err(2, format!("{msg}\n"));
                                    }
                                }
                            }
                        }
                        StageExecBytesOutcome::ok_text(out)
                    }
                    _ => StageExecBytesOutcome::err(2, "tar: unsupported mode\n".to_string()),
                }
            })()),
            "gzip" | "gunzip" => Some((|| -> StageExecBytesOutcome {
                let mut decompress = prog == "gunzip";
                let mut to_stdout = false;
                let mut keep = false;
                let mut test_mode = false;
                let mut files = Vec::new();
                for a in args {
                    if a != "-" && a.starts_with('-') && a.len() > 1 {
                        for ch in a[1..].chars() {
                            match ch {
                                'd' => decompress = true,
                                'c' => to_stdout = true,
                                'k' => keep = true,
                                't' => {
                                    decompress = true;
                                    test_mode = true;
                                }
                                'f' | '1'..='9' => {}
                                _ => {
                                    return StageExecBytesOutcome::err(
                                        1,
                                        format!("gzip: invalid option -- '{ch}'\n"),
                                    )
                                }
                            }
                        }
                    } else {
                        files.push(a.clone());
                    }
                }
                let sources = match self.read_input_sources(&files, cwd, stdin, "gzip") {
                    Ok(s) => s,
                    Err(e) => return StageExecBytesOutcome::err(1, e),
                };
                let mut out = Vec::new();
                for (name, bytes) in sources {
                    let transformed = if decompress {
                        match gzip_decompress_bytes(&bytes) {
                            Ok(d) => d,
                            Err(e) => {
                                return StageExecBytesOutcome::err(
                                    1,
                                    format!("gzip: {name}: {e}\n"),
                                )
                            }
                        }
                    } else {
                        gzip_compress_bytes(&bytes)
                    };
                    if test_mode {
                        continue;
                    }
                    if to_stdout || name == "-" {
                        out.extend_from_slice(&transformed);
                    } else {
                        let norm = Self::normalize_guest_path(cwd, &name);
                        if !keep && Self::is_protected_guest_path(&norm) {
                            return StageExecBytesOutcome::err(
                                1,
                                format!(
                                    "mvm: refused mutation of protected canonical path '{norm}'\n"
                                ),
                            );
                        }
                        let dst = if decompress {
                            norm.strip_suffix(".gz")
                                .map(str::to_string)
                                .unwrap_or_else(|| format!("{norm}.out"))
                        } else {
                            format!("{norm}.gz")
                        };
                        if let Err(msg) =
                            self.write_redirected_bytes("/", &dst, &transformed, false, None)
                        {
                            return StageExecBytesOutcome::err(1, format!("{msg}\n"));
                        }
                        if !keep && !Self::is_protected_guest_path(&norm) {
                            self.remove_inode_reclaiming(&norm);
                        }
                    }
                }
                StageExecBytesOutcome::ok_bytes(out)
            })()),
            "base64" => Some((|| -> StageExecBytesOutcome {
                let mut decode = false;
                let mut wrap: usize = 76;
                let mut files = Vec::new();
                let mut i = 0usize;
                while i < args.len() {
                    let a = &args[i];
                    if a == "-d" || a == "--decode" {
                        decode = true;
                    } else if a == "-w" && i + 1 < args.len() {
                        wrap = args[i + 1].parse().unwrap_or(76);
                        i += 2;
                        continue;
                    } else if let Some(rest) =
                        a.strip_prefix("-w").or_else(|| a.strip_prefix("--wrap="))
                    {
                        wrap = rest.parse().unwrap_or(76);
                    } else if a == "-i" || a == "--ignore-garbage" {
                        continue;
                    } else if a.starts_with('-') && a != "-" {
                        return StageExecBytesOutcome::err(
                            1,
                            format!("base64: invalid option '{a}'\n"),
                        );
                    } else {
                        files.push(a.clone());
                    }
                    i += 1;
                }
                let sources = match self.read_input_sources(&files, cwd, stdin, "base64") {
                    Ok(s) => s,
                    Err(e) => return StageExecBytesOutcome::err(1, e),
                };
                let mut input = Vec::new();
                for (_, b) in sources {
                    input.extend_from_slice(&b);
                }
                if decode {
                    let s = String::from_utf8_lossy(&input);
                    match decode_base64_bytes(&s) {
                        Ok(raw) => StageExecBytesOutcome::ok_bytes(raw),
                        Err(_) => {
                            StageExecBytesOutcome::err(1, "base64: invalid input\n".to_string())
                        }
                    }
                } else {
                    let encoded = encode_base64_bytes(&input);
                    if wrap == 0 || encoded.is_empty() {
                        let mut out = encoded;
                        if !out.is_empty() {
                            out.push('\n');
                        }
                        StageExecBytesOutcome::ok_text(out)
                    } else {
                        let mut out = String::new();
                        for chunk in encoded.as_bytes().chunks(wrap) {
                            out.push_str(&String::from_utf8_lossy(chunk));
                            out.push('\n');
                        }
                        StageExecBytesOutcome::ok_text(out)
                    }
                }
            })()),
            "sha256sum" => Some((|| -> StageExecBytesOutcome {
                let files: Vec<String> = args
                    .iter()
                    .filter(|a| *a == "-" || !a.starts_with('-'))
                    .cloned()
                    .collect();
                let sources = match self.read_input_sources(&files, cwd, stdin, "sha256sum") {
                    Ok(s) => s,
                    Err(e) => return StageExecBytesOutcome::err(1, e),
                };
                let mut out = String::new();
                for (name, b) in sources {
                    out.push_str(&format!("{}  {}\n", sha256_hex_bytes(&b), name));
                }
                StageExecBytesOutcome::ok_text(out)
            })()),
            "df" => Some({
                let human = args.iter().any(|a| a == "-h" || a == "--human-readable");
                let used_bytes = (self.cow_sectors.len() as u64) * (MVM_COW_SECTOR_BYTES as u64);
                let total_bytes: u64 = 64 * 1024 * 1024;
                let avail_bytes = total_bytes.saturating_sub(used_bytes);
                let pct = ((used_bytes.saturating_mul(100)) / total_bytes).min(100);
                let fmt_sz = |b: u64| -> String {
                    if !human {
                        return (b / 1024).to_string();
                    }
                    if b >= 1024 * 1024 {
                        format!("{}M", b / (1024 * 1024))
                    } else if b >= 1024 {
                        format!("{}K", b / 1024)
                    } else {
                        format!("{b}B")
                    }
                };
                let mut out = if human {
                    "Filesystem      Size  Used Avail Use% Mounted on\n".to_string()
                } else {
                    "Filesystem     1K-blocks    Used Available Use% Mounted on\n".to_string()
                };
                out.push_str(&format!(
                    "{:<14} {:>5} {:>5} {:>5} {:>3}% /\n",
                    "mvm-rootfs",
                    fmt_sz(total_bytes),
                    fmt_sz(used_bytes),
                    fmt_sz(avail_bytes),
                    pct
                ));
                out.push_str(&format!(
                    "{:<14} {:>5} {:>5} {:>5} {:>3}% /workspace\n",
                    "mvm-cowfs",
                    fmt_sz(total_bytes),
                    fmt_sz(used_bytes),
                    fmt_sz(avail_bytes),
                    pct
                ));
                StageExecBytesOutcome::ok_text(out)
            }),
            "free" => Some({
                let mut unit = 'k';
                for a in args {
                    match a.as_str() {
                        "-m" | "--mega" => unit = 'm',
                        "-b" | "--bytes" => unit = 'b',
                        "-h" | "--human" => unit = 'h',
                        "-k" | "--kilo" => unit = 'k',
                        _ => {}
                    }
                }
                let total_bytes: u64 = 16 * 1024 * 1024;
                let active_cow = (self.cow_sectors.len() as u64) * (MVM_COW_SECTOR_BYTES as u64);
                let ram_live = (self.last_guest_ram.len().min(GUEST_RAM_BYTES)) as u64;
                let used_bytes = (active_cow + ram_live).clamp(512 * 1024, 14 * 1024 * 1024);
                let free_bytes = total_bytes.saturating_sub(used_bytes);
                let buf_bytes = active_cow.min(free_bytes);
                let avail_bytes = free_bytes;
                let fmt_mem = |b: u64| -> String {
                    match unit {
                        'b' => b.to_string(),
                        'm' => (b / (1024 * 1024)).to_string(),
                        'h' => {
                            if b >= 1024 * 1024 {
                                format!("{}Mi", b / (1024 * 1024))
                            } else {
                                format!("{}Ki", b / 1024)
                            }
                        }
                        _ => (b / 1024).to_string(),
                    }
                };
                let mut out =
                    "               total        used        free      shared  buff/cache   available\n"
                        .to_string();
                out.push_str(&format!(
                    "Mem:     {:>11} {:>11} {:>11} {:>11} {:>11} {:>11}\n",
                    fmt_mem(total_bytes),
                    fmt_mem(used_bytes),
                    fmt_mem(free_bytes),
                    fmt_mem(0),
                    fmt_mem(buf_bytes),
                    fmt_mem(avail_bytes)
                ));
                StageExecBytesOutcome::ok_text(out)
            }),
            "xxd" => Some((|| -> StageExecBytesOutcome {
                let mut plain = false;
                let mut revert = false;
                let mut limit: Option<usize> = None;
                let mut cols: Option<usize> = None;
                let mut files: Vec<String> = Vec::new();
                let mut i = 0usize;
                while i < args.len() {
                    let a = &args[i];
                    if a == "-p" || a == "-ps" || a == "--plain" {
                        plain = true;
                    } else if a == "-r" || a == "--revert" {
                        revert = true;
                    } else if a == "-l" && i + 1 < args.len() {
                        limit = args[i + 1].parse::<usize>().ok();
                        i += 2;
                        continue;
                    } else if a == "-c" && i + 1 < args.len() {
                        cols = args[i + 1].parse::<usize>().ok().filter(|&v| v > 0);
                        i += 2;
                        continue;
                    } else if a != "-" && a.starts_with('-') {
                        for ch in a[1..].chars() {
                            match ch {
                                'p' => plain = true,
                                'r' => revert = true,
                                _ => {
                                    return StageExecBytesOutcome::err(
                                        1,
                                        format!("xxd: invalid option -- '{ch}'\n"),
                                    )
                                }
                            }
                        }
                    } else {
                        files.push(a.clone());
                    }
                    i += 1;
                }
                let sources = match self.read_input_sources(&files, cwd, stdin, "xxd") {
                    Ok(s) => s,
                    Err(e) => return StageExecBytesOutcome::err(1, e),
                };
                if revert {
                    let mut out_bytes = Vec::new();
                    for (_, b) in sources {
                        let text = String::from_utf8_lossy(&b);
                        let mut hex_digits = Vec::new();
                        if plain {
                            for ch in text.bytes() {
                                if ch.is_ascii_hexdigit() {
                                    hex_digits.push(ch);
                                }
                            }
                        } else {
                            for line in text.lines() {
                                let after_off =
                                    line.split_once(':').map(|(_, r)| r).unwrap_or(line);
                                let hex_section = after_off.split("  ").next().unwrap_or(after_off);
                                for ch in hex_section.bytes() {
                                    if ch.is_ascii_hexdigit() {
                                        hex_digits.push(ch);
                                    }
                                }
                            }
                        }
                        for pair in hex_digits.chunks_exact(2) {
                            let s = std::str::from_utf8(pair).unwrap_or("00");
                            if let Ok(byte) = u8::from_str_radix(s, 16) {
                                out_bytes.push(byte);
                            }
                        }
                    }
                    return StageExecBytesOutcome::ok_bytes(out_bytes);
                }
                let width = cols.unwrap_or(if plain { 30 } else { 16 });
                let mut out = String::new();
                for (_, b) in sources {
                    let slice = match limit {
                        Some(n) => &b[..n.min(b.len())],
                        None => b.as_slice(),
                    };
                    if plain {
                        for chunk in slice.chunks(width) {
                            for byte in chunk {
                                out.push_str(&format!("{byte:02x}"));
                            }
                            out.push('\n');
                        }
                    } else {
                        for (ci, chunk) in slice.chunks(width).enumerate() {
                            out.push_str(&format!("{:08x}: ", ci * width));
                            for j in 0..width {
                                if j < chunk.len() {
                                    out.push_str(&format!("{:02x}", chunk[j]));
                                } else {
                                    out.push_str("  ");
                                }
                                if j % 2 == 1 {
                                    out.push(' ');
                                }
                            }
                            out.push(' ');
                            for &byte in chunk {
                                if (0x20..=0x7e).contains(&byte) {
                                    out.push(byte as char);
                                } else {
                                    out.push('.');
                                }
                            }
                            out.push('\n');
                        }
                    }
                }
                StageExecBytesOutcome::ok_text(out)
            })()),
            _ => None,
        }
    }

    fn exec_coreutil_text_lang(
        &mut self,
        prog: &str,
        args: &[String],
        cwd: &str,
        stdin: &[u8],
    ) -> Option<StageExecBytesOutcome> {
        #[derive(Debug, Clone)]
        enum RxClassItem {
            Single(char),
            Range(char, char),
            Digit,
            Word,
            Space,
        }

        #[derive(Debug, Clone)]
        enum RxNode {
            Literal(char),
            Any,
            AnchorStart,
            AnchorEnd,
            WordBoundary,
            Class {
                negated: bool,
                items: Vec<RxClassItem>,
            },
            Group {
                id: usize,
                branches: Vec<Vec<RxQuantified>>,
            },
            Backref(usize),
        }

        #[derive(Debug, Clone)]
        struct RxQuantified {
            node: RxNode,
            min: usize,
            max: Option<usize>,
        }

        struct RxPattern {
            branches: Vec<Vec<RxQuantified>>,
            case_insensitive: bool,
            group_count: usize,
        }

        impl RxPattern {
            fn compile(pat: &str, extended: bool, case_insensitive: bool) -> Result<Self, String> {
                let chars: Vec<char> = pat.chars().collect();
                let mut pos = 0usize;
                let mut next_group = 1usize;
                let branches =
                    Self::parse_branches(&chars, &mut pos, extended, false, &mut next_group)?;
                if pos != chars.len() {
                    return Err(format!("unexpected trailing regex character at pos {pos}"));
                }
                Ok(Self {
                    branches,
                    case_insensitive,
                    group_count: next_group,
                })
            }

            fn parse_branches(
                chars: &[char],
                pos: &mut usize,
                extended: bool,
                in_group: bool,
                next_group: &mut usize,
            ) -> Result<Vec<Vec<RxQuantified>>, String> {
                let mut branches = Vec::new();
                let mut cur = Vec::new();
                while *pos < chars.len() {
                    let c = chars[*pos];
                    if in_group {
                        if extended && c == ')' {
                            *pos += 1;
                            branches.push(cur);
                            return Ok(branches);
                        }
                        if !extended
                            && c == '\\'
                            && *pos + 1 < chars.len()
                            && chars[*pos + 1] == ')'
                        {
                            *pos += 2;
                            branches.push(cur);
                            return Ok(branches);
                        }
                    }
                    if extended && c == '|' {
                        *pos += 1;
                        branches.push(cur);
                        cur = Vec::new();
                        continue;
                    }
                    if !extended && c == '\\' && *pos + 1 < chars.len() && chars[*pos + 1] == '|' {
                        *pos += 2;
                        branches.push(cur);
                        cur = Vec::new();
                        continue;
                    }
                    let atom = Self::parse_atom(chars, pos, extended, next_group)?;
                    let (min, max) = Self::parse_quantifier(chars, pos, extended);
                    cur.push(RxQuantified {
                        node: atom,
                        min,
                        max,
                    });
                }
                if in_group {
                    return Err("unclosed regex group".to_string());
                }
                branches.push(cur);
                Ok(branches)
            }

            fn parse_atom(
                chars: &[char],
                pos: &mut usize,
                extended: bool,
                next_group: &mut usize,
            ) -> Result<RxNode, String> {
                let c = chars[*pos];
                if extended && c == '(' {
                    *pos += 1;
                    let gid = *next_group;
                    *next_group += 1;
                    let branches = Self::parse_branches(chars, pos, extended, true, next_group)?;
                    return Ok(RxNode::Group { id: gid, branches });
                }
                if !extended && c == '\\' && *pos + 1 < chars.len() && chars[*pos + 1] == '(' {
                    *pos += 2;
                    let gid = *next_group;
                    *next_group += 1;
                    let branches = Self::parse_branches(chars, pos, extended, true, next_group)?;
                    return Ok(RxNode::Group { id: gid, branches });
                }
                if c == '^' {
                    *pos += 1;
                    return Ok(RxNode::AnchorStart);
                }
                if c == '$' {
                    *pos += 1;
                    return Ok(RxNode::AnchorEnd);
                }
                if c == '.' {
                    *pos += 1;
                    return Ok(RxNode::Any);
                }
                if c == '[' {
                    return Self::parse_class(chars, pos);
                }
                if c == '\\' && *pos + 1 < chars.len() {
                    let esc = chars[*pos + 1];
                    *pos += 2;
                    return Ok(match esc {
                        'd' => RxNode::Class {
                            negated: false,
                            items: vec![RxClassItem::Digit],
                        },
                        'D' => RxNode::Class {
                            negated: true,
                            items: vec![RxClassItem::Digit],
                        },
                        'w' => RxNode::Class {
                            negated: false,
                            items: vec![RxClassItem::Word],
                        },
                        'W' => RxNode::Class {
                            negated: true,
                            items: vec![RxClassItem::Word],
                        },
                        's' => RxNode::Class {
                            negated: false,
                            items: vec![RxClassItem::Space],
                        },
                        'S' => RxNode::Class {
                            negated: true,
                            items: vec![RxClassItem::Space],
                        },
                        'b' => RxNode::WordBoundary,
                        'n' => RxNode::Literal('\n'),
                        't' => RxNode::Literal('\t'),
                        '1'..='9' => RxNode::Backref((esc as u8 - b'0') as usize),
                        other => RxNode::Literal(other),
                    });
                }
                *pos += 1;
                Ok(RxNode::Literal(c))
            }

            fn parse_class(chars: &[char], pos: &mut usize) -> Result<RxNode, String> {
                *pos += 1;
                let negated = if *pos < chars.len() && chars[*pos] == '^' {
                    *pos += 1;
                    true
                } else {
                    false
                };
                let mut items = Vec::new();
                let mut first = true;
                while *pos < chars.len() {
                    let c = chars[*pos];
                    if c == ']' && !first {
                        *pos += 1;
                        return Ok(RxNode::Class { negated, items });
                    }
                    first = false;
                    let ch = if c == '\\' && *pos + 1 < chars.len() {
                        let esc = chars[*pos + 1];
                        *pos += 2;
                        match esc {
                            'd' => {
                                items.push(RxClassItem::Digit);
                                continue;
                            }
                            'w' => {
                                items.push(RxClassItem::Word);
                                continue;
                            }
                            's' => {
                                items.push(RxClassItem::Space);
                                continue;
                            }
                            'n' => '\n',
                            't' => '\t',
                            o => o,
                        }
                    } else {
                        *pos += 1;
                        c
                    };
                    if *pos + 1 < chars.len() && chars[*pos] == '-' && chars[*pos + 1] != ']' {
                        *pos += 1;
                        let end_ch = if chars[*pos] == '\\' && *pos + 1 < chars.len() {
                            let e = chars[*pos + 1];
                            *pos += 2;
                            e
                        } else {
                            let e = chars[*pos];
                            *pos += 1;
                            e
                        };
                        items.push(RxClassItem::Range(ch, end_ch));
                    } else {
                        items.push(RxClassItem::Single(ch));
                    }
                }
                Err("unclosed character class".to_string())
            }

            fn parse_quantifier(
                chars: &[char],
                pos: &mut usize,
                extended: bool,
            ) -> (usize, Option<usize>) {
                if *pos >= chars.len() {
                    return (1, Some(1));
                }
                let c = chars[*pos];
                if c == '*' {
                    *pos += 1;
                    if *pos < chars.len() && chars[*pos] == '?' {
                        *pos += 1;
                    }
                    return (0, None);
                }
                if extended && c == '+' {
                    *pos += 1;
                    if *pos < chars.len() && chars[*pos] == '?' {
                        *pos += 1;
                    }
                    return (1, None);
                }
                if extended && c == '?' {
                    *pos += 1;
                    return (0, Some(1));
                }
                if extended && c == '{' {
                    if let Some(close_rel) = chars[*pos + 1..].iter().position(|&x| x == '}') {
                        let inner: String = chars[*pos + 1..*pos + 1 + close_rel].iter().collect();
                        if let Some((a, b)) = inner.split_once(',') {
                            if let Ok(min_v) = a.trim().parse::<usize>() {
                                let max_v = if b.trim().is_empty() {
                                    None
                                } else {
                                    b.trim().parse::<usize>().ok()
                                };
                                *pos += 2 + close_rel;
                                return (min_v, max_v);
                            }
                        } else if let Ok(exact) = inner.trim().parse::<usize>() {
                            *pos += 2 + close_rel;
                            return (exact, Some(exact));
                        }
                    }
                }
                (1, Some(1))
            }

            fn char_matches_class(&self, ch: char, negated: bool, items: &[RxClassItem]) -> bool {
                let cmp = if self.case_insensitive {
                    ch.to_ascii_lowercase()
                } else {
                    ch
                };
                let mut hit = false;
                for it in items {
                    match it {
                        RxClassItem::Single(s) => {
                            let sc = if self.case_insensitive {
                                s.to_ascii_lowercase()
                            } else {
                                *s
                            };
                            if cmp == sc {
                                hit = true;
                                break;
                            }
                        }
                        RxClassItem::Range(a, b) => {
                            let (ra, rb) = if self.case_insensitive {
                                (a.to_ascii_lowercase(), b.to_ascii_lowercase())
                            } else {
                                (*a, *b)
                            };
                            if cmp >= ra && cmp <= rb {
                                hit = true;
                                break;
                            }
                        }
                        RxClassItem::Digit if ch.is_ascii_digit() => {
                            hit = true;
                            break;
                        }
                        RxClassItem::Word if ch.is_ascii_alphanumeric() || ch == '_' => {
                            hit = true;
                            break;
                        }
                        RxClassItem::Space if ch.is_whitespace() => {
                            hit = true;
                            break;
                        }
                        _ => {}
                    }
                }
                hit != negated
            }

            fn find_first(&self, text: &str) -> Option<(usize, usize, Vec<String>)> {
                let chars: Vec<char> = text.chars().collect();
                let mut byte_offsets = Vec::with_capacity(chars.len() + 1);
                let mut b = 0usize;
                for &c in &chars {
                    byte_offsets.push(b);
                    b += c.len_utf8();
                }
                byte_offsets.push(b);

                for start in 0..=chars.len() {
                    let mut caps = vec![None; self.group_count];
                    let mut budget = 50_000usize;
                    if let Some(end) =
                        self.match_branches(&self.branches, &chars, start, &mut caps, &mut budget)
                    {
                        let cap_strings: Vec<String> = caps
                            .into_iter()
                            .map(|opt| match opt {
                                Some((s, e)) => chars[s..e].iter().collect(),
                                None => String::new(),
                            })
                            .collect();
                        return Some((byte_offsets[start], byte_offsets[end], cap_strings));
                    }
                }
                None
            }

            fn find_all(&self, text: &str) -> Vec<(usize, usize, Vec<String>)> {
                let chars: Vec<char> = text.chars().collect();
                let mut byte_offsets = Vec::with_capacity(chars.len() + 1);
                let mut b = 0usize;
                for &c in &chars {
                    byte_offsets.push(b);
                    b += c.len_utf8();
                }
                byte_offsets.push(b);

                let mut matches = Vec::new();
                let mut start = 0usize;
                while start <= chars.len() {
                    let mut caps = vec![None; self.group_count];
                    let mut budget = 50_000usize;
                    if let Some(end) =
                        self.match_branches(&self.branches, &chars, start, &mut caps, &mut budget)
                    {
                        let cap_strings: Vec<String> = caps
                            .into_iter()
                            .map(|opt| match opt {
                                Some((s, e)) => chars[s..e].iter().collect(),
                                None => String::new(),
                            })
                            .collect();
                        matches.push((byte_offsets[start], byte_offsets[end], cap_strings));
                        start = if end > start { end } else { start + 1 };
                    } else {
                        start += 1;
                    }
                }
                matches
            }

            fn match_branches(
                &self,
                branches: &[Vec<RxQuantified>],
                chars: &[char],
                pos: usize,
                caps: &mut Vec<Option<(usize, usize)>>,
                budget: &mut usize,
            ) -> Option<usize> {
                for seq in branches {
                    let saved = caps.clone();
                    if let Some(end) = self.match_seq(seq, 0, chars, pos, caps, budget) {
                        return Some(end);
                    }
                    *caps = saved;
                }
                None
            }

            fn match_seq(
                &self,
                seq: &[RxQuantified],
                idx: usize,
                chars: &[char],
                pos: usize,
                caps: &mut Vec<Option<(usize, usize)>>,
                budget: &mut usize,
            ) -> Option<usize> {
                if *budget == 0 {
                    return None;
                }
                *budget -= 1;
                if idx >= seq.len() {
                    return Some(pos);
                }
                let item = &seq[idx];
                let max_reps = item.max.unwrap_or(chars.len() - pos + 1);

                // Zero-width assertions do not repeat
                if matches!(
                    item.node,
                    RxNode::AnchorStart | RxNode::AnchorEnd | RxNode::WordBoundary
                ) {
                    let ok = match item.node {
                        RxNode::AnchorStart => pos == 0,
                        RxNode::AnchorEnd => pos == chars.len(),
                        RxNode::WordBoundary => {
                            let is_w = |c: char| c.is_ascii_alphanumeric() || c == '_';
                            let prev_w = pos > 0 && is_w(chars[pos - 1]);
                            let cur_w = pos < chars.len() && is_w(chars[pos]);
                            prev_w != cur_w
                        }
                        _ => false,
                    };
                    if ok {
                        return self.match_seq(seq, idx + 1, chars, pos, caps, budget);
                    }
                    return None;
                }

                // Collect all valid greedy positions for item repetitions
                type RxCapSpan = Option<(usize, usize)>;
                let mut positions: Vec<(usize, Vec<RxCapSpan>)> = Vec::new();
                let mut cur_pos = pos;
                let mut reps = 0usize;
                if item.min == 0 {
                    positions.push((cur_pos, caps.clone()));
                }
                while reps < max_reps {
                    let saved = caps.clone();
                    let Some(next_pos) = self.match_node(&item.node, chars, cur_pos, caps, budget)
                    else {
                        *caps = saved;
                        break;
                    };
                    reps += 1;
                    if reps >= item.min {
                        positions.push((next_pos, caps.clone()));
                    }
                    if next_pos == cur_pos && item.max.is_none() {
                        break;
                    }
                    cur_pos = next_pos;
                }

                for (p, cap_snapshot) in positions.into_iter().rev() {
                    *caps = cap_snapshot;
                    if let Some(end) = self.match_seq(seq, idx + 1, chars, p, caps, budget) {
                        return Some(end);
                    }
                }
                None
            }

            fn match_node(
                &self,
                node: &RxNode,
                chars: &[char],
                pos: usize,
                caps: &mut Vec<Option<(usize, usize)>>,
                budget: &mut usize,
            ) -> Option<usize> {
                match node {
                    RxNode::Literal(expected) => {
                        let actual = *chars.get(pos)?;
                        let eq = if self.case_insensitive {
                            actual.eq_ignore_ascii_case(expected)
                        } else {
                            actual == *expected
                        };
                        eq.then_some(pos + 1)
                    }
                    RxNode::Any => {
                        let actual = *chars.get(pos)?;
                        (actual != '\n').then_some(pos + 1)
                    }
                    RxNode::Class { negated, items } => {
                        let actual = *chars.get(pos)?;
                        self.char_matches_class(actual, *negated, items)
                            .then_some(pos + 1)
                    }
                    RxNode::Group { id, branches } => {
                        let end = self.match_branches(branches, chars, pos, caps, budget)?;
                        if *id < caps.len() {
                            caps[*id] = Some((pos, end));
                        }
                        Some(end)
                    }
                    RxNode::Backref(gid) => {
                        let (s, e) = caps.get(*gid).copied().flatten()?;
                        let len = e - s;
                        if pos + len > chars.len() {
                            return None;
                        }
                        for k in 0..len {
                            let a = chars[s + k];
                            let b = chars[pos + k];
                            let eq = if self.case_insensitive {
                                a.eq_ignore_ascii_case(&b)
                            } else {
                                a == b
                            };
                            if !eq {
                                return None;
                            }
                        }
                        Some(pos + len)
                    }
                    RxNode::AnchorStart | RxNode::AnchorEnd | RxNode::WordBoundary => None,
                }
            }
        }

        fn split_inclusive_bytes(data: &[u8], delim: u8) -> Vec<&[u8]> {
            let mut out = Vec::new();
            let mut start = 0usize;
            for (idx, &b) in data.iter().enumerate() {
                if b == delim {
                    out.push(&data[start..=idx]);
                    start = idx + 1;
                }
            }
            if start < data.len() {
                out.push(&data[start..]);
            }
            out
        }

        fn expand_tr_set(spec: &str) -> Vec<u8> {
            let mut out = Vec::new();
            let bytes = spec.as_bytes();
            let mut i = 0usize;
            while i < bytes.len() {
                if let Some(rest) = spec.get(i..) {
                    if let Some((class_bytes, adv)) = match_tr_posix_class(rest) {
                        out.extend_from_slice(&class_bytes);
                        i += adv;
                        continue;
                    }
                }
                let ch = if bytes[i] == b'\\' {
                    let (seq, adv) = parse_c_escape_at(bytes, i);
                    i += adv;
                    seq.first().copied().unwrap_or(b'\\')
                } else {
                    let b = bytes[i];
                    i += 1;
                    b
                };
                if i + 1 < bytes.len() && bytes[i] == b'-' {
                    i += 1;
                    let end_ch = if bytes[i] == b'\\' {
                        let (seq, adv) = parse_c_escape_at(bytes, i);
                        i += adv;
                        seq.first().copied().unwrap_or(b'\\')
                    } else {
                        let b = bytes[i];
                        i += 1;
                        b
                    };
                    if ch <= end_ch {
                        for b in ch..=end_ch {
                            out.push(b);
                        }
                    } else {
                        out.push(ch);
                        out.push(b'-');
                        out.push(end_ch);
                    }
                } else {
                    out.push(ch);
                }
            }
            out
        }

        fn match_tr_posix_class(rest: &str) -> Option<(Vec<u8>, usize)> {
            type PosixClassMatcher = (&'static str, fn(u8) -> bool);
            let classes: [PosixClassMatcher; 8] = [
                ("[:lower:]", |b| b.is_ascii_lowercase()),
                ("[:upper:]", |b| b.is_ascii_uppercase()),
                ("[:digit:]", |b| b.is_ascii_digit()),
                ("[:alpha:]", |b| b.is_ascii_alphabetic()),
                ("[:alnum:]", |b| b.is_ascii_alphanumeric()),
                ("[:space:]", |b| b.is_ascii_whitespace()),
                ("[:blank:]", |b| b == b' ' || b == b'\t'),
                ("[:punct:]", |b| b.is_ascii_punctuation()),
            ];
            for (pat, pred) in classes {
                if rest.starts_with(pat) {
                    let vals: Vec<u8> = (0u8..=127u8).filter(|&b| pred(b)).collect();
                    return Some((vals, pat.len()));
                }
            }
            None
        }

        fn parse_cut_ranges(spec: &str) -> Result<Vec<(usize, Option<usize>)>, &'static str> {
            let mut ranges = Vec::new();
            for part in spec.split(',') {
                let p = part.trim();
                if p.is_empty() {
                    return Err("empty range item");
                }
                if let Some((a, b)) = p.split_once('-') {
                    let start = if a.is_empty() {
                        1
                    } else {
                        a.parse::<usize>().map_err(|_| "invalid range start")?
                    };
                    let end = if b.is_empty() {
                        None
                    } else {
                        Some(b.parse::<usize>().map_err(|_| "invalid range end")?)
                    };
                    if start == 0 || end.is_some_and(|e| e < start) {
                        return Err("invalid cut range bounds");
                    }
                    ranges.push((start, end));
                } else {
                    let idx = p.parse::<usize>().map_err(|_| "invalid cut index")?;
                    if idx == 0 {
                        return Err("fields and positions are numbered from 1");
                    }
                    ranges.push((idx, Some(idx)));
                }
            }
            Ok(ranges)
        }

        fn in_one_based_ranges(idx: usize, ranges: &[(usize, Option<usize>)]) -> bool {
            ranges
                .iter()
                .any(|&(s, e)| idx >= s && e.is_none_or(|max_v| idx <= max_v))
        }

        #[derive(Clone)]
        enum SedAddr {
            All,
            Line(usize),
            Last,
            Regex(RxPattern),
            Range(Box<SedAddr>, Box<SedAddr>),
        }

        impl Clone for RxPattern {
            fn clone(&self) -> Self {
                Self {
                    branches: self.branches.clone(),
                    case_insensitive: self.case_insensitive,
                    group_count: self.group_count,
                }
            }
        }

        enum SedAction {
            Delete,
            Print,
            Quit,
            LineNum,
            Transliterate(String, String),
            Substitute {
                rx: RxPattern,
                repl: String,
                global: bool,
                print: bool,
            },
        }

        struct SedCmd {
            addr: SedAddr,
            action: SedAction,
        }

        fn parse_sed_script(script: &str, extended: bool) -> Result<Vec<SedCmd>, String> {
            let mut cmds = Vec::new();
            let chars: Vec<char> = script.chars().collect();
            let mut i = 0usize;
            while i < chars.len() {
                while i < chars.len() && (chars[i].is_whitespace() || chars[i] == ';') {
                    i += 1;
                }
                if i >= chars.len() {
                    break;
                }
                let a1 = parse_sed_addr(&chars, &mut i, extended)?;
                let addr = if i < chars.len() && chars[i] == ',' {
                    i += 1;
                    let a2 = parse_sed_addr(&chars, &mut i, extended)?;
                    SedAddr::Range(Box::new(a1), Box::new(a2))
                } else {
                    a1
                };
                while i < chars.len() && chars[i].is_ascii_whitespace() {
                    i += 1;
                }
                if i >= chars.len() {
                    return Err("missing sed command".to_string());
                }
                let cmd_ch = chars[i];
                i += 1;
                let action = match cmd_ch {
                    'd' => SedAction::Delete,
                    'p' => SedAction::Print,
                    'q' => SedAction::Quit,
                    '=' => SedAction::LineNum,
                    's' | 'y' => {
                        if i >= chars.len() {
                            return Err(format!("missing delimiter after `{cmd_ch}`"));
                        }
                        let delim = chars[i];
                        i += 1;
                        let part1 = read_sed_delimited(&chars, &mut i, delim)?;
                        let part2 = read_sed_delimited(&chars, &mut i, delim)?;
                        if cmd_ch == 'y' {
                            if part1.chars().count() != part2.chars().count() {
                                return Err("sed y: strings must have equal length".to_string());
                            }
                            SedAction::Transliterate(part1, part2)
                        } else {
                            let mut global = false;
                            let mut print = false;
                            let mut case_i = false;
                            while i < chars.len() && !matches!(chars[i], ';' | '\n') {
                                match chars[i] {
                                    'g' => global = true,
                                    'p' => print = true,
                                    'i' | 'I' => case_i = true,
                                    c if c.is_ascii_whitespace() => {}
                                    other => return Err(format!("unknown sed s flag '{other}'")),
                                }
                                i += 1;
                            }
                            let rx = RxPattern::compile(&part1, extended, case_i)?;
                            SedAction::Substitute {
                                rx,
                                repl: part2,
                                global,
                                print,
                            }
                        }
                    }
                    other => return Err(format!("unsupported sed command '{other}'")),
                };
                cmds.push(SedCmd { addr, action });
            }
            Ok(cmds)
        }

        fn parse_sed_addr(
            chars: &[char],
            pos: &mut usize,
            extended: bool,
        ) -> Result<SedAddr, String> {
            while *pos < chars.len() && chars[*pos].is_ascii_whitespace() {
                *pos += 1;
            }
            if *pos >= chars.len() {
                return Ok(SedAddr::All);
            }
            if chars[*pos] == '$' {
                *pos += 1;
                return Ok(SedAddr::Last);
            }
            if chars[*pos].is_ascii_digit() {
                let start = *pos;
                while *pos < chars.len() && chars[*pos].is_ascii_digit() {
                    *pos += 1;
                }
                let s: String = chars[start..*pos].iter().collect();
                return Ok(SedAddr::Line(s.parse().unwrap_or(1)));
            }
            if chars[*pos] == '/' {
                *pos += 1;
                let pat = read_sed_delimited(chars, pos, '/')?;
                let rx = RxPattern::compile(&pat, extended, false)?;
                return Ok(SedAddr::Regex(rx));
            }
            Ok(SedAddr::All)
        }

        fn read_sed_delimited(
            chars: &[char],
            pos: &mut usize,
            delim: char,
        ) -> Result<String, String> {
            let mut out = String::new();
            while *pos < chars.len() {
                let c = chars[*pos];
                if c == '\\' && *pos + 1 < chars.len() {
                    let nxt = chars[*pos + 1];
                    if nxt == delim {
                        out.push(delim);
                    } else {
                        out.push('\\');
                        out.push(nxt);
                    }
                    *pos += 2;
                    continue;
                }
                if c == delim {
                    *pos += 1;
                    return Ok(out);
                }
                out.push(c);
                *pos += 1;
            }
            Err("unterminated sed pattern/replacement".to_string())
        }

        fn sed_addr_matches(
            addr: &SedAddr,
            line_no: usize,
            is_last: bool,
            line: &str,
            range_active: &mut bool,
        ) -> bool {
            match addr {
                SedAddr::All => true,
                SedAddr::Line(n) => line_no == *n,
                SedAddr::Last => is_last,
                SedAddr::Regex(rx) => rx.find_first(line).is_some(),
                SedAddr::Range(a, b) => {
                    if !*range_active {
                        let mut dummy = false;
                        if sed_addr_matches(a, line_no, is_last, line, &mut dummy) {
                            let closes_on_start = matches!(b.as_ref(), SedAddr::Line(end_n) if line_no >= *end_n)
                                || (matches!(b.as_ref(), SedAddr::Last) && is_last);
                            *range_active = !closes_on_start;
                            return true;
                        }
                        false
                    } else {
                        let mut dummy = false;
                        if sed_addr_matches(b, line_no, is_last, line, &mut dummy) {
                            *range_active = false;
                        }
                        true
                    }
                }
            }
        }

        fn sed_apply_sub(rx: &RxPattern, line: &str, repl: &str, global: bool) -> (String, bool) {
            let matches = if global {
                rx.find_all(line)
            } else if let Some(m) = rx.find_first(line) {
                vec![m]
            } else {
                Vec::new()
            };
            if matches.is_empty() {
                return (line.to_string(), false);
            }
            let mut out = String::new();
            let mut last = 0usize;
            for (s, e, caps) in matches {
                out.push_str(&line[last..s]);
                let whole = &line[s..e];
                let rc: Vec<char> = repl.chars().collect();
                let mut k = 0usize;
                while k < rc.len() {
                    if rc[k] == '\\' && k + 1 < rc.len() {
                        match rc[k + 1] {
                            'n' => out.push('\n'),
                            't' => out.push('\t'),
                            '0' => out.push_str(whole),
                            d @ '1'..='9' => {
                                let gid = (d as u8 - b'0') as usize;
                                if let Some(cval) = caps.get(gid) {
                                    out.push_str(cval);
                                }
                            }
                            other => out.push(other),
                        }
                        k += 2;
                    } else if rc[k] == '&' {
                        out.push_str(whole);
                        k += 1;
                    } else {
                        out.push(rc[k]);
                        k += 1;
                    }
                }
                last = e;
            }
            out.push_str(&line[last..]);
            (out, true)
        }

        fn is_binary_bytes_for_diff(bytes: &[u8]) -> bool {
            bytes.contains(&0) || std::str::from_utf8(bytes).is_err()
        }

        fn render_lcs_diff(
            a_name: &str,
            b_name: &str,
            a: &[&str],
            b: &[&str],
            unified: bool,
            ignore_case: bool,
        ) -> String {
            let eq = |x: &str, y: &str| -> bool {
                if ignore_case {
                    x.eq_ignore_ascii_case(y)
                } else {
                    x == y
                }
            };
            let n = a.len();
            let m = b.len();
            let mut ops: Vec<(char, usize, usize)> = Vec::new();
            if n.saturating_mul(m) <= 250_000 {
                let mut dp = vec![0u32; (n + 1) * (m + 1)];
                let w = m + 1;
                for i in (0..n).rev() {
                    for j in (0..m).rev() {
                        dp[i * w + j] = if eq(a[i], b[j]) {
                            dp[(i + 1) * w + (j + 1)] + 1
                        } else {
                            dp[(i + 1) * w + j].max(dp[i * w + (j + 1)])
                        };
                    }
                }
                let (mut i, mut j) = (0usize, 0usize);
                while i < n && j < m {
                    if eq(a[i], b[j]) {
                        ops.push((' ', i, j));
                        i += 1;
                        j += 1;
                    } else if dp[(i + 1) * w + j] >= dp[i * w + (j + 1)] {
                        ops.push(('-', i, j));
                        i += 1;
                    } else {
                        ops.push(('+', i, j));
                        j += 1;
                    }
                }
                while i < n {
                    ops.push(('-', i, j));
                    i += 1;
                }
                while j < m {
                    ops.push(('+', i, j));
                    j += 1;
                }
            } else {
                let mut i = 0usize;
                let mut j = 0usize;
                while i < n && j < m {
                    if eq(a[i], b[j]) {
                        ops.push((' ', i, j));
                        i += 1;
                        j += 1;
                    } else {
                        ops.push(('-', i, j));
                        ops.push(('+', i, j));
                        i += 1;
                        j += 1;
                    }
                }
                while i < n {
                    ops.push(('-', i, j));
                    i += 1;
                }
                while j < m {
                    ops.push(('+', i, j));
                    j += 1;
                }
            }

            if !unified {
                let mut out = String::new();
                for (kind, i, j) in &ops {
                    match kind {
                        '-' => out.push_str(&format!("< {}\n", a[*i])),
                        '+' => out.push_str(&format!("> {}\n", b[*j])),
                        _ => {}
                    }
                }
                return out;
            }

            let mut out = format!("--- {a_name}\n+++ {b_name}\n");
            let mut idx = 0usize;
            while idx < ops.len() {
                if ops[idx].0 == ' ' {
                    idx += 1;
                    continue;
                }
                let start = idx.saturating_sub(3);
                let mut end = idx;
                let mut last_change = idx;
                while end < ops.len() && end <= last_change + 6 {
                    if ops[end].0 != ' ' {
                        last_change = end;
                    }
                    end += 1;
                }
                end = (last_change + 4).min(ops.len());
                let a_start = ops[start].1 + 1;
                let b_start = ops[start].2 + 1;
                let a_cnt = ops[start..end].iter().filter(|o| o.0 != '+').count();
                let b_cnt = ops[start..end].iter().filter(|o| o.0 != '-').count();
                out.push_str(&format!("@@ -{a_start},{a_cnt} +{b_start},{b_cnt} @@\n"));
                for &(kind, i, j) in &ops[start..end] {
                    match kind {
                        ' ' | '-' => out.push_str(&format!("{kind}{}\n", a[i])),
                        '+' => out.push_str(&format!("+{}\n", b[j])),
                        _ => {}
                    }
                }
                idx = end;
            }
            out
        }

        // ============================================================================
        // Pure-Rust AWK Interpreter (Integer + String Associative Arrays, Zero Floats)
        // ============================================================================

        #[derive(Debug, Clone, PartialEq)]
        enum AwkTok {
            Ident(String),
            Num(i64),
            Str(String),
            Regex(String),
            Dollar,
            Semi,
            Newline,
            Comma,
            LParen,
            RParen,
            LBrace,
            RBrace,
            LBracket,
            RBracket,
            Plus,
            Minus,
            Star,
            Slash,
            Percent,
            Assign,
            PlusAssign,
            MinusAssign,
            MulAssign,
            DivAssign,
            PlusPlus,
            MinusMinus,
            Eq,
            Ne,
            Lt,
            Le,
            Gt,
            Ge,
            Match,
            NotMatch,
            And,
            Or,
            Not,
            Question,
            Colon,
        }

        fn tokenize_awk(src: &str) -> Result<Vec<AwkTok>, String> {
            let chars: Vec<char> = src.chars().collect();
            let mut toks = Vec::new();
            let mut i = 0usize;
            let can_start_regex = |prev: Option<&AwkTok>| -> bool {
                matches!(
                    prev,
                    None | Some(
                        AwkTok::Semi
                            | AwkTok::Newline
                            | AwkTok::LBrace
                            | AwkTok::RBrace
                            | AwkTok::LParen
                            | AwkTok::Comma
                            | AwkTok::Match
                            | AwkTok::NotMatch
                            | AwkTok::And
                            | AwkTok::Or
                            | AwkTok::Not
                            | AwkTok::Assign
                            | AwkTok::Question
                            | AwkTok::Colon
                    )
                )
            };
            while i < chars.len() {
                let c = chars[i];
                if c == ' ' || c == '\t' || c == '\r' {
                    i += 1;
                    continue;
                }
                if c == '#' {
                    while i < chars.len() && chars[i] != '\n' {
                        i += 1;
                    }
                    continue;
                }
                if c == '\n' {
                    toks.push(AwkTok::Newline);
                    i += 1;
                    continue;
                }
                if c == '\x22' {
                    i += 1;
                    let mut s = String::new();
                    while i < chars.len() && chars[i] != '\x22' {
                        if chars[i] == '\\' && i + 1 < chars.len() {
                            match chars[i + 1] {
                                'n' => s.push('\n'),
                                't' => s.push('\t'),
                                'r' => s.push('\r'),
                                '\\' => s.push('\\'),
                                '\x22' => s.push('\x22'),
                                o => s.push(o),
                            }
                            i += 2;
                        } else {
                            s.push(chars[i]);
                            i += 1;
                        }
                    }
                    if i >= chars.len() {
                        return Err("unterminated awk string".to_string());
                    }
                    i += 1;
                    toks.push(AwkTok::Str(s));
                    continue;
                }
                if c == '/' && can_start_regex(toks.last()) {
                    i += 1;
                    let mut pat = String::new();
                    while i < chars.len() && chars[i] != '/' {
                        if chars[i] == '\\' && i + 1 < chars.len() {
                            pat.push('\\');
                            pat.push(chars[i + 1]);
                            i += 2;
                        } else {
                            pat.push(chars[i]);
                            i += 1;
                        }
                    }
                    if i >= chars.len() {
                        return Err("unterminated awk regex".to_string());
                    }
                    i += 1;
                    toks.push(AwkTok::Regex(pat));
                    continue;
                }
                if c.is_ascii_digit() {
                    let start = i;
                    while i < chars.len() && chars[i].is_ascii_digit() {
                        i += 1;
                    }
                    let num_s: String = chars[start..i].iter().collect();
                    let v = num_s.parse::<i64>().unwrap_or(0);
                    toks.push(AwkTok::Num(v));
                    continue;
                }
                if c.is_ascii_alphabetic() || c == '_' {
                    let start = i;
                    while i < chars.len() && (chars[i].is_ascii_alphanumeric() || chars[i] == '_') {
                        i += 1;
                    }
                    let id: String = chars[start..i].iter().collect();
                    toks.push(AwkTok::Ident(id));
                    continue;
                }
                let two = if i + 1 < chars.len() {
                    Some((c, chars[i + 1]))
                } else {
                    None
                };
                if let Some(pair) = two {
                    let tok = match pair {
                        ('+', '+') => Some(AwkTok::PlusPlus),
                        ('-', '-') => Some(AwkTok::MinusMinus),
                        ('+', '=') => Some(AwkTok::PlusAssign),
                        ('-', '=') => Some(AwkTok::MinusAssign),
                        ('*', '=') => Some(AwkTok::MulAssign),
                        ('/', '=') => Some(AwkTok::DivAssign),
                        ('=', '=') => Some(AwkTok::Eq),
                        ('!', '=') => Some(AwkTok::Ne),
                        ('<', '=') => Some(AwkTok::Le),
                        ('>', '=') => Some(AwkTok::Ge),
                        ('!', '~') => Some(AwkTok::NotMatch),
                        ('&', '&') => Some(AwkTok::And),
                        ('|', '|') => Some(AwkTok::Or),
                        _ => None,
                    };
                    if let Some(t) = tok {
                        toks.push(t);
                        i += 2;
                        continue;
                    }
                }
                let one = match c {
                    '$' => AwkTok::Dollar,
                    ';' => AwkTok::Semi,
                    ',' => AwkTok::Comma,
                    '(' => AwkTok::LParen,
                    ')' => AwkTok::RParen,
                    '{' => AwkTok::LBrace,
                    '}' => AwkTok::RBrace,
                    '[' => AwkTok::LBracket,
                    ']' => AwkTok::RBracket,
                    '+' => AwkTok::Plus,
                    '-' => AwkTok::Minus,
                    '*' => AwkTok::Star,
                    '/' => AwkTok::Slash,
                    '%' => AwkTok::Percent,
                    '=' => AwkTok::Assign,
                    '<' => AwkTok::Lt,
                    '>' => AwkTok::Gt,
                    '~' => AwkTok::Match,
                    '!' => AwkTok::Not,
                    '?' => AwkTok::Question,
                    ':' => AwkTok::Colon,
                    other => return Err(format!("unexpected awk token '{other}'")),
                };
                toks.push(one);
                i += 1;
            }
            Ok(toks)
        }

        #[derive(Debug, Clone)]
        enum AwkExpr {
            Num(i64),
            Str(String),
            Regex(String),
            Var(String),
            Index(String, Box<AwkExpr>),
            Field(Box<AwkExpr>),
            UnaryNeg(Box<AwkExpr>),
            Not(Box<AwkExpr>),
            IncDec {
                target: Box<AwkExpr>,
                inc: bool,
                prefix: bool,
            },
            Bin {
                op: AwkBinOp,
                lhs: Box<AwkExpr>,
                rhs: Box<AwkExpr>,
            },
            Assign {
                target: Box<AwkExpr>,
                op: Option<AwkBinOp>,
                val: Box<AwkExpr>,
            },
            Ternary(Box<AwkExpr>, Box<AwkExpr>, Box<AwkExpr>),
            Call(String, Vec<AwkExpr>),
        }

        #[derive(Debug, Clone, Copy)]
        enum AwkBinOp {
            Add,
            Sub,
            Mul,
            Div,
            Mod,
            Concat,
            Eq,
            Ne,
            Lt,
            Le,
            Gt,
            Ge,
            Match,
            NotMatch,
            And,
            Or,
        }

        #[derive(Debug, Clone)]
        enum AwkStmt {
            Expr(AwkExpr),
            Print(Vec<AwkExpr>),
            Printf(Vec<AwkExpr>),
            If(AwkExpr, Vec<AwkStmt>, Vec<AwkStmt>),
            While(AwkExpr, Vec<AwkStmt>),
            For(
                Option<AwkExpr>,
                Option<AwkExpr>,
                Option<AwkExpr>,
                Vec<AwkStmt>,
            ),
            ForIn(String, String, Vec<AwkStmt>),
            Delete(String, AwkExpr),
            Next,
            Break,
            Continue,
            Exit(Option<AwkExpr>),
        }

        enum AwkRulePat {
            Begin,
            End,
            Always,
            Expr(AwkExpr),
        }

        struct AwkParser<'a> {
            toks: &'a [AwkTok],
            pos: usize,
        }

        impl<'a> AwkParser<'a> {
            fn peek(&self) -> Option<&AwkTok> {
                self.toks.get(self.pos)
            }

            fn skip_seps(&mut self) {
                while matches!(self.peek(), Some(AwkTok::Semi | AwkTok::Newline)) {
                    self.pos += 1;
                }
            }

            fn skip_newlines(&mut self) {
                while matches!(self.peek(), Some(AwkTok::Newline)) {
                    self.pos += 1;
                }
            }

            fn parse_program(&mut self) -> Result<Vec<(AwkRulePat, Vec<AwkStmt>)>, String> {
                let mut rules = Vec::new();
                loop {
                    self.skip_seps();
                    let Some(tok) = self.peek().cloned() else {
                        break;
                    };
                    let pat = match tok {
                        AwkTok::Ident(ref s) if s == "BEGIN" => {
                            self.pos += 1;
                            AwkRulePat::Begin
                        }
                        AwkTok::Ident(ref s) if s == "END" => {
                            self.pos += 1;
                            AwkRulePat::End
                        }
                        AwkTok::LBrace => AwkRulePat::Always,
                        _ => {
                            let e = self.parse_expr()?;
                            AwkRulePat::Expr(e)
                        }
                    };
                    self.skip_newlines();
                    let stmts = if matches!(self.peek(), Some(AwkTok::LBrace)) {
                        self.parse_block()?
                    } else {
                        vec![AwkStmt::Print(Vec::new())]
                    };
                    rules.push((pat, stmts));
                }
                Ok(rules)
            }

            fn parse_block(&mut self) -> Result<Vec<AwkStmt>, String> {
                if !matches!(self.peek(), Some(AwkTok::LBrace)) {
                    return Err("expected `{` in awk block".to_string());
                }
                self.pos += 1;
                let mut stmts = Vec::new();
                loop {
                    self.skip_seps();
                    if matches!(self.peek(), Some(AwkTok::RBrace)) {
                        self.pos += 1;
                        return Ok(stmts);
                    }
                    if self.peek().is_none() {
                        return Err("unclosed `{` in awk".to_string());
                    }
                    stmts.push(self.parse_stmt()?);
                }
            }

            fn parse_stmt(&mut self) -> Result<AwkStmt, String> {
                self.skip_newlines();
                let Some(tok) = self.peek().cloned() else {
                    return Err("unexpected end of awk statement".to_string());
                };
                if matches!(tok, AwkTok::LBrace) {
                    return Ok(AwkStmt::If(
                        AwkExpr::Num(1),
                        self.parse_block()?,
                        Vec::new(),
                    ));
                }
                if let AwkTok::Ident(ref kw) = tok {
                    match kw.as_str() {
                        "print" => {
                            self.pos += 1;
                            let mut args = Vec::new();
                            if !matches!(
                                self.peek(),
                                None | Some(AwkTok::Semi | AwkTok::Newline | AwkTok::RBrace)
                            ) {
                                args.push(self.parse_expr()?);
                                while matches!(self.peek(), Some(AwkTok::Comma)) {
                                    self.pos += 1;
                                    self.skip_newlines();
                                    args.push(self.parse_expr()?);
                                }
                            }
                            return Ok(AwkStmt::Print(args));
                        }
                        "printf" => {
                            self.pos += 1;
                            let mut args = Vec::new();
                            if !matches!(
                                self.peek(),
                                None | Some(AwkTok::Semi | AwkTok::Newline | AwkTok::RBrace)
                            ) {
                                args.push(self.parse_expr()?);
                                while matches!(self.peek(), Some(AwkTok::Comma)) {
                                    self.pos += 1;
                                    self.skip_newlines();
                                    args.push(self.parse_expr()?);
                                }
                            }
                            return Ok(AwkStmt::Printf(args));
                        }
                        "if" => {
                            self.pos += 1;
                            if !matches!(self.peek(), Some(AwkTok::LParen)) {
                                return Err("awk if expects `(`".to_string());
                            }
                            self.pos += 1;
                            let cond = self.parse_expr()?;
                            if !matches!(self.peek(), Some(AwkTok::RParen)) {
                                return Err("awk if expects `)`".to_string());
                            }
                            self.pos += 1;
                            self.skip_newlines();
                            let then_b = if matches!(self.peek(), Some(AwkTok::LBrace)) {
                                self.parse_block()?
                            } else {
                                vec![self.parse_stmt()?]
                            };
                            self.skip_seps();
                            let else_b = if matches!(self.peek(), Some(AwkTok::Ident(s)) if s == "else")
                            {
                                self.pos += 1;
                                self.skip_newlines();
                                if matches!(self.peek(), Some(AwkTok::LBrace)) {
                                    self.parse_block()?
                                } else {
                                    vec![self.parse_stmt()?]
                                }
                            } else {
                                Vec::new()
                            };
                            return Ok(AwkStmt::If(cond, then_b, else_b));
                        }
                        "while" => {
                            self.pos += 1;
                            if !matches!(self.peek(), Some(AwkTok::LParen)) {
                                return Err("awk while expects `(`".to_string());
                            }
                            self.pos += 1;
                            let cond = self.parse_expr()?;
                            if !matches!(self.peek(), Some(AwkTok::RParen)) {
                                return Err("awk while expects `)`".to_string());
                            }
                            self.pos += 1;
                            self.skip_newlines();
                            let body = if matches!(self.peek(), Some(AwkTok::LBrace)) {
                                self.parse_block()?
                            } else {
                                vec![self.parse_stmt()?]
                            };
                            return Ok(AwkStmt::While(cond, body));
                        }
                        "for" => {
                            self.pos += 1;
                            if !matches!(self.peek(), Some(AwkTok::LParen)) {
                                return Err("awk for expects `(`".to_string());
                            }
                            self.pos += 1;
                            if let (
                                Some(AwkTok::Ident(k)),
                                Some(AwkTok::Ident(in_kw)),
                                Some(AwkTok::Ident(arr)),
                                Some(AwkTok::RParen),
                            ) = (
                                self.toks.get(self.pos).cloned(),
                                self.toks.get(self.pos + 1).cloned(),
                                self.toks.get(self.pos + 2).cloned(),
                                self.toks.get(self.pos + 3).cloned(),
                            ) {
                                if in_kw == "in" {
                                    self.pos += 4;
                                    self.skip_newlines();
                                    let body = if matches!(self.peek(), Some(AwkTok::LBrace)) {
                                        self.parse_block()?
                                    } else {
                                        vec![self.parse_stmt()?]
                                    };
                                    return Ok(AwkStmt::ForIn(k, arr, body));
                                }
                            }
                            let init = if matches!(self.peek(), Some(AwkTok::Semi)) {
                                None
                            } else {
                                Some(self.parse_expr()?)
                            };
                            if matches!(self.peek(), Some(AwkTok::Semi)) {
                                self.pos += 1;
                            }
                            let cond = if matches!(self.peek(), Some(AwkTok::Semi)) {
                                None
                            } else {
                                Some(self.parse_expr()?)
                            };
                            if matches!(self.peek(), Some(AwkTok::Semi)) {
                                self.pos += 1;
                            }
                            let step = if matches!(self.peek(), Some(AwkTok::RParen)) {
                                None
                            } else {
                                Some(self.parse_expr()?)
                            };
                            if !matches!(self.peek(), Some(AwkTok::RParen)) {
                                return Err("awk for expects `)`".to_string());
                            }
                            self.pos += 1;
                            self.skip_newlines();
                            let body = if matches!(self.peek(), Some(AwkTok::LBrace)) {
                                self.parse_block()?
                            } else {
                                vec![self.parse_stmt()?]
                            };
                            return Ok(AwkStmt::For(init, cond, step, body));
                        }
                        "delete" => {
                            self.pos += 1;
                            let Some(AwkTok::Ident(arr)) = self.peek().cloned() else {
                                return Err("awk delete expects array".to_string());
                            };
                            self.pos += 1;
                            if !matches!(self.peek(), Some(AwkTok::LBracket)) {
                                return Err("awk delete expects `[key]`".to_string());
                            }
                            self.pos += 1;
                            let key = self.parse_expr()?;
                            if !matches!(self.peek(), Some(AwkTok::RBracket)) {
                                return Err("awk delete expects `]`".to_string());
                            }
                            self.pos += 1;
                            return Ok(AwkStmt::Delete(arr, key));
                        }
                        "next" => {
                            self.pos += 1;
                            return Ok(AwkStmt::Next);
                        }
                        "break" => {
                            self.pos += 1;
                            return Ok(AwkStmt::Break);
                        }
                        "continue" => {
                            self.pos += 1;
                            return Ok(AwkStmt::Continue);
                        }
                        "exit" => {
                            self.pos += 1;
                            let code = if !matches!(
                                self.peek(),
                                None | Some(AwkTok::Semi | AwkTok::Newline | AwkTok::RBrace)
                            ) {
                                Some(self.parse_expr()?)
                            } else {
                                None
                            };
                            return Ok(AwkStmt::Exit(code));
                        }
                        _ => {}
                    }
                }
                Ok(AwkStmt::Expr(self.parse_expr()?))
            }

            fn parse_expr(&mut self) -> Result<AwkExpr, String> {
                let lhs = self.parse_ternary()?;
                let op_opt = match self.peek() {
                    Some(AwkTok::Assign) => Some(None),
                    Some(AwkTok::PlusAssign) => Some(Some(AwkBinOp::Add)),
                    Some(AwkTok::MinusAssign) => Some(Some(AwkBinOp::Sub)),
                    Some(AwkTok::MulAssign) => Some(Some(AwkBinOp::Mul)),
                    Some(AwkTok::DivAssign) => Some(Some(AwkBinOp::Div)),
                    _ => None,
                };
                if let Some(op) = op_opt {
                    self.pos += 1;
                    let rhs = self.parse_expr()?;
                    return Ok(AwkExpr::Assign {
                        target: Box::new(lhs),
                        op,
                        val: Box::new(rhs),
                    });
                }
                Ok(lhs)
            }

            fn parse_ternary(&mut self) -> Result<AwkExpr, String> {
                let c = self.parse_or()?;
                if matches!(self.peek(), Some(AwkTok::Question)) {
                    self.pos += 1;
                    let t = self.parse_expr()?;
                    if !matches!(self.peek(), Some(AwkTok::Colon)) {
                        return Err("awk ternary expects `:`".to_string());
                    }
                    self.pos += 1;
                    let e = self.parse_ternary()?;
                    return Ok(AwkExpr::Ternary(Box::new(c), Box::new(t), Box::new(e)));
                }
                Ok(c)
            }

            fn parse_or(&mut self) -> Result<AwkExpr, String> {
                let mut lhs = self.parse_and()?;
                while matches!(self.peek(), Some(AwkTok::Or)) {
                    self.pos += 1;
                    let rhs = self.parse_and()?;
                    lhs = AwkExpr::Bin {
                        op: AwkBinOp::Or,
                        lhs: Box::new(lhs),
                        rhs: Box::new(rhs),
                    };
                }
                Ok(lhs)
            }

            fn parse_and(&mut self) -> Result<AwkExpr, String> {
                let mut lhs = self.parse_match()?;
                while matches!(self.peek(), Some(AwkTok::And)) {
                    self.pos += 1;
                    let rhs = self.parse_match()?;
                    lhs = AwkExpr::Bin {
                        op: AwkBinOp::And,
                        lhs: Box::new(lhs),
                        rhs: Box::new(rhs),
                    };
                }
                Ok(lhs)
            }

            fn parse_match(&mut self) -> Result<AwkExpr, String> {
                let mut lhs = self.parse_cmp()?;
                while let Some(op) = match self.peek() {
                    Some(AwkTok::Match) => Some(AwkBinOp::Match),
                    Some(AwkTok::NotMatch) => Some(AwkBinOp::NotMatch),
                    _ => None,
                } {
                    self.pos += 1;
                    let rhs = self.parse_cmp()?;
                    lhs = AwkExpr::Bin {
                        op,
                        lhs: Box::new(lhs),
                        rhs: Box::new(rhs),
                    };
                }
                Ok(lhs)
            }

            fn parse_cmp(&mut self) -> Result<AwkExpr, String> {
                let mut lhs = self.parse_concat()?;
                while let Some(op) = match self.peek() {
                    Some(AwkTok::Eq) => Some(AwkBinOp::Eq),
                    Some(AwkTok::Ne) => Some(AwkBinOp::Ne),
                    Some(AwkTok::Lt) => Some(AwkBinOp::Lt),
                    Some(AwkTok::Le) => Some(AwkBinOp::Le),
                    Some(AwkTok::Gt) => Some(AwkBinOp::Gt),
                    Some(AwkTok::Ge) => Some(AwkBinOp::Ge),
                    _ => None,
                } {
                    self.pos += 1;
                    let rhs = self.parse_concat()?;
                    lhs = AwkExpr::Bin {
                        op,
                        lhs: Box::new(lhs),
                        rhs: Box::new(rhs),
                    };
                }
                Ok(lhs)
            }

            fn parse_concat(&mut self) -> Result<AwkExpr, String> {
                let mut lhs = self.parse_add()?;
                while matches!(
                    self.peek(),
                    Some(
                        AwkTok::Ident(_)
                            | AwkTok::Num(_)
                            | AwkTok::Str(_)
                            | AwkTok::Dollar
                            | AwkTok::LParen
                    )
                ) {
                    let rhs = self.parse_add()?;
                    lhs = AwkExpr::Bin {
                        op: AwkBinOp::Concat,
                        lhs: Box::new(lhs),
                        rhs: Box::new(rhs),
                    };
                }
                Ok(lhs)
            }

            fn parse_add(&mut self) -> Result<AwkExpr, String> {
                let mut lhs = self.parse_mul()?;
                while let Some(op) = match self.peek() {
                    Some(AwkTok::Plus) => Some(AwkBinOp::Add),
                    Some(AwkTok::Minus) => Some(AwkBinOp::Sub),
                    _ => None,
                } {
                    self.pos += 1;
                    let rhs = self.parse_mul()?;
                    lhs = AwkExpr::Bin {
                        op,
                        lhs: Box::new(lhs),
                        rhs: Box::new(rhs),
                    };
                }
                Ok(lhs)
            }

            fn parse_mul(&mut self) -> Result<AwkExpr, String> {
                let mut lhs = self.parse_unary()?;
                while let Some(op) = match self.peek() {
                    Some(AwkTok::Star) => Some(AwkBinOp::Mul),
                    Some(AwkTok::Slash) => Some(AwkBinOp::Div),
                    Some(AwkTok::Percent) => Some(AwkBinOp::Mod),
                    _ => None,
                } {
                    self.pos += 1;
                    let rhs = self.parse_unary()?;
                    lhs = AwkExpr::Bin {
                        op,
                        lhs: Box::new(lhs),
                        rhs: Box::new(rhs),
                    };
                }
                Ok(lhs)
            }

            fn parse_unary(&mut self) -> Result<AwkExpr, String> {
                match self.peek().cloned() {
                    Some(AwkTok::Minus) => {
                        self.pos += 1;
                        Ok(AwkExpr::UnaryNeg(Box::new(self.parse_unary()?)))
                    }
                    Some(AwkTok::Plus) => {
                        self.pos += 1;
                        self.parse_unary()
                    }
                    Some(AwkTok::Not) => {
                        self.pos += 1;
                        Ok(AwkExpr::Not(Box::new(self.parse_unary()?)))
                    }
                    Some(AwkTok::Dollar) => {
                        self.pos += 1;
                        Ok(AwkExpr::Field(Box::new(self.parse_unary()?)))
                    }
                    Some(AwkTok::PlusPlus) => {
                        self.pos += 1;
                        Ok(AwkExpr::IncDec {
                            target: Box::new(self.parse_unary()?),
                            inc: true,
                            prefix: true,
                        })
                    }
                    Some(AwkTok::MinusMinus) => {
                        self.pos += 1;
                        Ok(AwkExpr::IncDec {
                            target: Box::new(self.parse_unary()?),
                            inc: false,
                            prefix: true,
                        })
                    }
                    _ => self.parse_postfix(),
                }
            }

            fn parse_postfix(&mut self) -> Result<AwkExpr, String> {
                let mut expr = self.parse_primary()?;
                loop {
                    match self.peek() {
                        Some(AwkTok::PlusPlus) => {
                            self.pos += 1;
                            expr = AwkExpr::IncDec {
                                target: Box::new(expr),
                                inc: true,
                                prefix: false,
                            };
                        }
                        Some(AwkTok::MinusMinus) => {
                            self.pos += 1;
                            expr = AwkExpr::IncDec {
                                target: Box::new(expr),
                                inc: false,
                                prefix: false,
                            };
                        }
                        _ => break,
                    }
                }
                Ok(expr)
            }

            fn parse_primary(&mut self) -> Result<AwkExpr, String> {
                let Some(tok) = self.peek().cloned() else {
                    return Err("unexpected end of awk expression".to_string());
                };
                match tok {
                    AwkTok::Num(n) => {
                        self.pos += 1;
                        Ok(AwkExpr::Num(n))
                    }
                    AwkTok::Str(s) => {
                        self.pos += 1;
                        Ok(AwkExpr::Str(s))
                    }
                    AwkTok::Regex(r) => {
                        self.pos += 1;
                        Ok(AwkExpr::Regex(r))
                    }
                    AwkTok::LParen => {
                        self.pos += 1;
                        let e = self.parse_expr()?;
                        if !matches!(self.peek(), Some(AwkTok::RParen)) {
                            return Err("awk expected `)`".to_string());
                        }
                        self.pos += 1;
                        Ok(e)
                    }
                    AwkTok::Ident(name) => {
                        self.pos += 1;
                        if matches!(self.peek(), Some(AwkTok::LParen)) {
                            self.pos += 1;
                            let mut args = Vec::new();
                            if !matches!(self.peek(), Some(AwkTok::RParen)) {
                                args.push(self.parse_expr()?);
                                while matches!(self.peek(), Some(AwkTok::Comma)) {
                                    self.pos += 1;
                                    args.push(self.parse_expr()?);
                                }
                            }
                            if !matches!(self.peek(), Some(AwkTok::RParen)) {
                                return Err("awk call expected `)`".to_string());
                            }
                            self.pos += 1;
                            Ok(AwkExpr::Call(name, args))
                        } else if matches!(self.peek(), Some(AwkTok::LBracket)) {
                            self.pos += 1;
                            let key = self.parse_expr()?;
                            if !matches!(self.peek(), Some(AwkTok::RBracket)) {
                                return Err("awk array index expected `]`".to_string());
                            }
                            self.pos += 1;
                            Ok(AwkExpr::Index(name, Box::new(key)))
                        } else {
                            Ok(AwkExpr::Var(name))
                        }
                    }
                    other => Err(format!("unexpected awk expression token {other:?}")),
                }
            }
        }

        enum AwkFlow {
            Normal,
            Next,
            Break,
            Continue,
            Exit(i32),
        }

        struct AwkVm {
            vars: BTreeMap<String, String>,
            arrays: BTreeMap<String, BTreeMap<String, String>>,
            fields: Vec<String>,
            out: String,
            steps: usize,
        }

        impl AwkVm {
            fn get_var(&self, name: &str) -> String {
                self.vars.get(name).cloned().unwrap_or_default()
            }

            fn set_var(&mut self, name: &str, val: String) {
                self.vars.insert(name.to_string(), val);
            }

            fn set_line(&mut self, line: &str, nr: usize, fnr: usize, fname: &str) {
                self.set_var("NR", nr.to_string());
                self.set_var("FNR", fnr.to_string());
                self.set_var("FILENAME", fname.to_string());
                let fs = self.get_var("FS");
                let cols: Vec<String> = if fs == " " || fs.is_empty() {
                    line.split_whitespace().map(str::to_string).collect()
                } else {
                    line.split(&fs).map(str::to_string).collect()
                };
                self.set_var("NF", cols.len().to_string());
                self.fields = Vec::with_capacity(cols.len() + 1);
                self.fields.push(line.to_string());
                self.fields.extend(cols);
            }

            fn to_num(s: &str) -> i64 {
                parse_leading_i64(s)
            }

            fn is_truthy(s: &str) -> bool {
                if s.is_empty() {
                    return false;
                }
                if let Ok(n) = s.trim().parse::<i64>() {
                    return n != 0;
                }
                true
            }

            fn eval_expr(&mut self, expr: &AwkExpr) -> Result<String, String> {
                self.steps += 1;
                if self.steps > 1_000_000 {
                    return Err("awk step budget exceeded".to_string());
                }
                match expr {
                    AwkExpr::Num(n) => Ok(n.to_string()),
                    AwkExpr::Str(s) => Ok(s.clone()),
                    AwkExpr::Regex(pat) => {
                        let line = self.fields.first().map(String::as_str).unwrap_or("");
                        let rx = RxPattern::compile(pat, true, false)?;
                        Ok(if rx.find_first(line).is_some() {
                            "1".to_string()
                        } else {
                            "0".to_string()
                        })
                    }
                    AwkExpr::Var(name) => Ok(self.get_var(name)),
                    AwkExpr::Index(arr, key_e) => {
                        let k = self.eval_expr(key_e)?;
                        Ok(self
                            .arrays
                            .get(arr)
                            .and_then(|m| m.get(&k))
                            .cloned()
                            .unwrap_or_default())
                    }
                    AwkExpr::Field(idx_e) => {
                        let idx_s = self.eval_expr(idx_e)?;
                        let idx = Self::to_num(&idx_s);
                        if idx < 0 {
                            return Err(format!("negative awk field index {idx}"));
                        }
                        Ok(self.fields.get(idx as usize).cloned().unwrap_or_default())
                    }
                    AwkExpr::UnaryNeg(e) => {
                        let v = Self::to_num(&self.eval_expr(e)?);
                        Ok(v.wrapping_neg().to_string())
                    }
                    AwkExpr::Not(e) => {
                        let v = self.eval_expr(e)?;
                        Ok(if Self::is_truthy(&v) {
                            "0".to_string()
                        } else {
                            "1".to_string()
                        })
                    }
                    AwkExpr::IncDec {
                        target,
                        inc,
                        prefix,
                    } => {
                        let cur = Self::to_num(&self.eval_expr(target)?);
                        let next = if *inc {
                            cur.saturating_add(1)
                        } else {
                            cur.saturating_sub(1)
                        };
                        self.write_target(target, next.to_string())?;
                        Ok(if *prefix { next } else { cur }.to_string())
                    }
                    AwkExpr::Assign { target, op, val } => {
                        let rhs = self.eval_expr(val)?;
                        let final_s = if let Some(bop) = op {
                            let lhs = self.eval_expr(target)?;
                            self.eval_bin(*bop, &lhs, &rhs)?
                        } else {
                            rhs
                        };
                        self.write_target(target, final_s.clone())?;
                        Ok(final_s)
                    }
                    AwkExpr::Ternary(c, t, e) => {
                        let cv = self.eval_expr(c)?;
                        if Self::is_truthy(&cv) {
                            self.eval_expr(t)
                        } else {
                            self.eval_expr(e)
                        }
                    }
                    AwkExpr::Bin { op, lhs, rhs } => {
                        match op {
                            AwkBinOp::And => {
                                let lv = self.eval_expr(lhs)?;
                                if !Self::is_truthy(&lv) {
                                    return Ok("0".to_string());
                                }
                                let rv = self.eval_expr(rhs)?;
                                return Ok(if Self::is_truthy(&rv) { "1" } else { "0" }.to_string());
                            }
                            AwkBinOp::Or => {
                                let lv = self.eval_expr(lhs)?;
                                if Self::is_truthy(&lv) {
                                    return Ok("1".to_string());
                                }
                                let rv = self.eval_expr(rhs)?;
                                return Ok(if Self::is_truthy(&rv) { "1" } else { "0" }.to_string());
                            }
                            AwkBinOp::Match | AwkBinOp::NotMatch => {
                                let lv = self.eval_expr(lhs)?;
                                let pat = match &**rhs {
                                    AwkExpr::Regex(r) => r.clone(),
                                    _ => self.eval_expr(rhs)?,
                                };
                                let rx = RxPattern::compile(&pat, true, false)?;
                                let matched = rx.find_first(&lv).is_some();
                                let res = if matches!(op, AwkBinOp::Match) {
                                    matched
                                } else {
                                    !matched
                                };
                                return Ok(if res { "1" } else { "0" }.to_string());
                            }
                            _ => {}
                        }
                        let lv = self.eval_expr(lhs)?;
                        let rv = self.eval_expr(rhs)?;
                        self.eval_bin(*op, &lv, &rv)
                    }
                    AwkExpr::Call(name, args) => self.eval_call(name, args),
                }
            }

            fn write_target(&mut self, target: &AwkExpr, val: String) -> Result<(), String> {
                match target {
                    AwkExpr::Var(name) => {
                        self.set_var(name, val);
                        Ok(())
                    }
                    AwkExpr::Index(arr, key_e) => {
                        let k = self.eval_expr(key_e)?;
                        self.arrays.entry(arr.clone()).or_default().insert(k, val);
                        Ok(())
                    }
                    AwkExpr::Field(idx_e) => {
                        let idx = Self::to_num(&self.eval_expr(idx_e)?).max(0) as usize;
                        while self.fields.len() <= idx {
                            self.fields.push(String::new());
                        }
                        self.fields[idx] = val;
                        if idx > 0 {
                            let ofs = self.get_var("OFS");
                            let sep = if ofs.is_empty() { " " } else { ofs.as_str() };
                            self.fields[0] = self.fields[1..].join(sep);
                            self.set_var("NF", (self.fields.len() - 1).to_string());
                        }
                        Ok(())
                    }
                    _ => Err("invalid awk assignment target".to_string()),
                }
            }

            fn eval_bin(&self, op: AwkBinOp, l: &str, r: &str) -> Result<String, String> {
                let ln = Self::to_num(l);
                let rn = Self::to_num(r);
                let both_num = l.trim().parse::<i64>().is_ok() && r.trim().parse::<i64>().is_ok();
                Ok(match op {
                    AwkBinOp::Add => ln.saturating_add(rn).to_string(),
                    AwkBinOp::Sub => ln.saturating_sub(rn).to_string(),
                    AwkBinOp::Mul => ln.saturating_mul(rn).to_string(),
                    AwkBinOp::Div => {
                        if rn == 0 {
                            return Err("awk: division by zero".to_string());
                        }
                        ln.wrapping_div(rn).to_string()
                    }
                    AwkBinOp::Mod => {
                        if rn == 0 {
                            return Err("awk: modulo by zero".to_string());
                        }
                        ln.wrapping_rem(rn).to_string()
                    }
                    AwkBinOp::Concat => format!("{l}{r}"),
                    AwkBinOp::Eq => if (both_num && ln == rn) || (!both_num && l == r) {
                        "1"
                    } else {
                        "0"
                    }
                    .to_string(),
                    AwkBinOp::Ne => if (both_num && ln != rn) || (!both_num && l != r) {
                        "1"
                    } else {
                        "0"
                    }
                    .to_string(),
                    AwkBinOp::Lt => if (both_num && ln < rn) || (!both_num && l < r) {
                        "1"
                    } else {
                        "0"
                    }
                    .to_string(),
                    AwkBinOp::Le => if (both_num && ln <= rn) || (!both_num && l <= r) {
                        "1"
                    } else {
                        "0"
                    }
                    .to_string(),
                    AwkBinOp::Gt => if (both_num && ln > rn) || (!both_num && l > r) {
                        "1"
                    } else {
                        "0"
                    }
                    .to_string(),
                    AwkBinOp::Ge => if (both_num && ln >= rn) || (!both_num && l >= r) {
                        "1"
                    } else {
                        "0"
                    }
                    .to_string(),
                    _ => "0".to_string(),
                })
            }

            fn eval_call(&mut self, name: &str, args: &[AwkExpr]) -> Result<String, String> {
                match name {
                    "length" => {
                        let s = if let Some(a) = args.first() {
                            self.eval_expr(a)?
                        } else {
                            self.fields.first().cloned().unwrap_or_default()
                        };
                        Ok(s.chars().count().to_string())
                    }
                    "tolower" => {
                        let s = args
                            .first()
                            .map(|a| self.eval_expr(a))
                            .transpose()?
                            .unwrap_or_default();
                        Ok(s.to_ascii_lowercase())
                    }
                    "toupper" => {
                        let s = args
                            .first()
                            .map(|a| self.eval_expr(a))
                            .transpose()?
                            .unwrap_or_default();
                        Ok(s.to_ascii_uppercase())
                    }
                    "substr" => {
                        let s = args
                            .first()
                            .map(|a| self.eval_expr(a))
                            .transpose()?
                            .unwrap_or_default();
                        let start_1 = args
                            .get(1)
                            .map(|a| self.eval_expr(a))
                            .transpose()?
                            .map(|v| Self::to_num(&v))
                            .unwrap_or(1)
                            .max(1) as usize;
                        let chars: Vec<char> = s.chars().collect();
                        let s_idx = (start_1 - 1).min(chars.len());
                        let len = args
                            .get(2)
                            .map(|a| self.eval_expr(a))
                            .transpose()?
                            .map(|v| Self::to_num(&v).max(0) as usize)
                            .unwrap_or(chars.len());
                        Ok(chars[s_idx..chars.len().min(s_idx + len)].iter().collect())
                    }
                    "index" => {
                        let hay = args
                            .first()
                            .map(|a| self.eval_expr(a))
                            .transpose()?
                            .unwrap_or_default();
                        let needle = args
                            .get(1)
                            .map(|a| self.eval_expr(a))
                            .transpose()?
                            .unwrap_or_default();
                        let pos = hay
                            .find(&needle)
                            .map(|b| hay[..b].chars().count() + 1)
                            .unwrap_or(0);
                        Ok(pos.to_string())
                    }
                    "int" => {
                        let s = args
                            .first()
                            .map(|a| self.eval_expr(a))
                            .transpose()?
                            .unwrap_or_default();
                        Ok(Self::to_num(&s).to_string())
                    }
                    "split" => {
                        let s = args
                            .first()
                            .map(|a| self.eval_expr(a))
                            .transpose()?
                            .unwrap_or_default();
                        let arr_name = match args.get(1) {
                            Some(AwkExpr::Var(v)) => v.clone(),
                            _ => return Err("awk split expects array name".to_string()),
                        };
                        let sep = if let Some(se) = args.get(2) {
                            self.eval_expr(se)?
                        } else {
                            self.get_var("FS")
                        };
                        let parts: Vec<&str> = if sep == " " || sep.is_empty() {
                            s.split_whitespace().collect()
                        } else {
                            s.split(&sep).collect()
                        };
                        let map = self.arrays.entry(arr_name).or_default();
                        map.clear();
                        for (i, p) in parts.iter().enumerate() {
                            map.insert((i + 1).to_string(), (*p).to_string());
                        }
                        Ok(parts.len().to_string())
                    }
                    "sub" | "gsub" => {
                        let pat = match args.first() {
                            Some(AwkExpr::Regex(r)) => r.clone(),
                            Some(e) => self.eval_expr(e)?,
                            None => String::new(),
                        };
                        let repl = args
                            .get(1)
                            .map(|e| self.eval_expr(e))
                            .transpose()?
                            .unwrap_or_default();
                        let target_expr = args
                            .get(2)
                            .cloned()
                            .unwrap_or_else(|| AwkExpr::Field(Box::new(AwkExpr::Num(0))));
                        let target_s = self.eval_expr(&target_expr)?;
                        let rx = RxPattern::compile(&pat, true, false)?;
                        let (next_s, matched) =
                            sed_apply_sub(&rx, &target_s, &repl, name == "gsub");
                        self.write_target(&target_expr, next_s)?;
                        Ok(if matched { "1" } else { "0" }.to_string())
                    }
                    "sprintf" => {
                        let mut vals = Vec::with_capacity(args.len());
                        for a in args {
                            vals.push(self.eval_expr(a)?);
                        }
                        Ok(format_awk_printf(&vals))
                    }
                    other => Err(format!("unknown awk function `{other}`")),
                }
            }

            fn exec_block(&mut self, stmts: &[AwkStmt]) -> Result<AwkFlow, String> {
                for s in stmts {
                    match self.exec_stmt(s)? {
                        AwkFlow::Normal => {}
                        other => return Ok(other),
                    }
                }
                Ok(AwkFlow::Normal)
            }

            fn exec_stmt(&mut self, stmt: &AwkStmt) -> Result<AwkFlow, String> {
                match stmt {
                    AwkStmt::Expr(e) => {
                        let _ = self.eval_expr(e)?;
                        Ok(AwkFlow::Normal)
                    }
                    AwkStmt::Print(args) => {
                        let ofs = self.get_var("OFS");
                        let ors = self.get_var("ORS");
                        if args.is_empty() {
                            let line = self.fields.first().map(String::as_str).unwrap_or("");
                            self.out.push_str(line);
                        } else {
                            let mut vals = Vec::with_capacity(args.len());
                            for a in args {
                                vals.push(self.eval_expr(a)?);
                            }
                            self.out.push_str(&vals.join(&ofs));
                        }
                        self.out.push_str(&ors);
                        Ok(AwkFlow::Normal)
                    }
                    AwkStmt::Printf(args) => {
                        let mut vals = Vec::with_capacity(args.len());
                        for a in args {
                            vals.push(self.eval_expr(a)?);
                        }
                        self.out.push_str(&format_awk_printf(&vals));
                        Ok(AwkFlow::Normal)
                    }
                    AwkStmt::If(c, tb, eb) => {
                        let cv = self.eval_expr(c)?;
                        if Self::is_truthy(&cv) {
                            self.exec_block(tb)
                        } else {
                            self.exec_block(eb)
                        }
                    }
                    AwkStmt::While(c, body) => {
                        let mut iters = 0usize;
                        while iters < 200_000 {
                            iters += 1;
                            let cv = self.eval_expr(c)?;
                            if !Self::is_truthy(&cv) {
                                break;
                            }
                            match self.exec_block(body)? {
                                AwkFlow::Normal | AwkFlow::Continue => {}
                                AwkFlow::Break => break,
                                other => return Ok(other),
                            }
                        }
                        Ok(AwkFlow::Normal)
                    }
                    AwkStmt::For(init, cond, step, body) => {
                        if let Some(ie) = init {
                            let _ = self.eval_expr(ie)?;
                        }
                        let mut iters = 0usize;
                        while iters < 200_000 {
                            iters += 1;
                            if let Some(ce) = cond {
                                let cv = self.eval_expr(ce)?;
                                if !Self::is_truthy(&cv) {
                                    break;
                                }
                            }
                            match self.exec_block(body)? {
                                AwkFlow::Normal | AwkFlow::Continue => {}
                                AwkFlow::Break => break,
                                other => return Ok(other),
                            }
                            if let Some(se) = step {
                                let _ = self.eval_expr(se)?;
                            }
                        }
                        Ok(AwkFlow::Normal)
                    }
                    AwkStmt::ForIn(kvar, arr, body) => {
                        let keys: Vec<String> = self
                            .arrays
                            .get(arr)
                            .map(|m| m.keys().cloned().collect())
                            .unwrap_or_default();
                        for k in keys {
                            self.set_var(kvar, k);
                            match self.exec_block(body)? {
                                AwkFlow::Normal | AwkFlow::Continue => {}
                                AwkFlow::Break => break,
                                other => return Ok(other),
                            }
                        }
                        Ok(AwkFlow::Normal)
                    }
                    AwkStmt::Delete(arr, key_e) => {
                        let k = self.eval_expr(key_e)?;
                        if let Some(m) = self.arrays.get_mut(arr) {
                            m.remove(&k);
                        }
                        Ok(AwkFlow::Normal)
                    }
                    AwkStmt::Next => Ok(AwkFlow::Next),
                    AwkStmt::Break => Ok(AwkFlow::Break),
                    AwkStmt::Continue => Ok(AwkFlow::Continue),
                    AwkStmt::Exit(code_e) => {
                        let code = if let Some(ce) = code_e {
                            Self::to_num(&self.eval_expr(ce)?) as i32
                        } else {
                            0
                        };
                        Ok(AwkFlow::Exit(code))
                    }
                }
            }
        }

        fn format_awk_printf(vals: &[String]) -> String {
            let Some(fmt) = vals.first() else {
                return String::new();
            };
            let mut out = String::new();
            let fc: Vec<char> = fmt.chars().collect();
            let mut i = 0usize;
            let mut aidx = 1usize;
            while i < fc.len() {
                if fc[i] == '%' && i + 1 < fc.len() {
                    if fc[i + 1] == '%' {
                        out.push('%');
                        i += 2;
                        continue;
                    }
                    let mut j = i + 1;
                    let mut left = false;
                    let mut zero = false;
                    while j < fc.len() && matches!(fc[j], '-' | '0') {
                        if fc[j] == '-' {
                            left = true;
                        } else {
                            zero = true;
                        }
                        j += 1;
                    }
                    let ws = j;
                    while j < fc.len() && fc[j].is_ascii_digit() {
                        j += 1;
                    }
                    let width: usize = fc[ws..j].iter().collect::<String>().parse().unwrap_or(0);
                    if j < fc.len() {
                        let val = vals.get(aidx).map(String::as_str).unwrap_or("");
                        aidx += 1;
                        let s = match fc[j] {
                            's' => pad_formatted_field(val, width, left, ' '),
                            'd' | 'i' => pad_formatted_field(
                                &parse_leading_i64(val).to_string(),
                                width,
                                left,
                                if zero && !left { '0' } else { ' ' },
                            ),
                            'x' => pad_formatted_field(
                                &format!("{:x}", parse_leading_i64(val) as u64),
                                width,
                                left,
                                ' ',
                            ),
                            other => format!("%{other}"),
                        };
                        out.push_str(&s);
                        i = j + 1;
                        continue;
                    }
                }
                out.push(fc[i]);
                i += 1;
            }
            out
        }

        fn run_awk_program(
            script: &str,
            fs_opt: Option<&str>,
            init_vars: BTreeMap<String, String>,
            sources: &[(String, Vec<u8>)],
        ) -> StageExecBytesOutcome {
            let toks = match tokenize_awk(script) {
                Ok(t) => t,
                Err(e) => return StageExecBytesOutcome::err(2, format!("awk: {e}\n")),
            };
            let mut parser = AwkParser {
                toks: &toks,
                pos: 0,
            };
            let rules = match parser.parse_program() {
                Ok(r) => r,
                Err(e) => return StageExecBytesOutcome::err(2, format!("awk: {e}\n")),
            };

            let mut vars = BTreeMap::new();
            vars.insert("FS".to_string(), fs_opt.unwrap_or(" ").to_string());
            vars.insert("OFS".to_string(), " ".to_string());
            vars.insert("ORS".to_string(), "\n".to_string());
            vars.insert("NR".to_string(), "0".to_string());
            vars.insert("FNR".to_string(), "0".to_string());
            vars.insert("NF".to_string(), "0".to_string());
            for (k, v) in init_vars {
                vars.insert(k, v);
            }

            let mut vm = AwkVm {
                vars,
                arrays: BTreeMap::new(),
                fields: vec![String::new()],
                out: String::new(),
                steps: 0,
            };

            let mut exit_code_override: Option<i32> = None;
            for (pat, block) in &rules {
                if matches!(pat, AwkRulePat::Begin) {
                    match vm.exec_block(block) {
                        Ok(AwkFlow::Exit(c)) => {
                            exit_code_override = Some(c);
                            break;
                        }
                        Ok(_) => {}
                        Err(e) => return StageExecBytesOutcome::err(2, format!("{e}\n")),
                    }
                }
            }

            let has_line_rules = rules
                .iter()
                .any(|(p, _)| matches!(p, AwkRulePat::Always | AwkRulePat::Expr(_)));
            let has_end = rules.iter().any(|(p, _)| matches!(p, AwkRulePat::End));

            if exit_code_override.is_none() && (has_line_rules || has_end) {
                let mut nr = 0usize;
                'outer: for (fname, bytes) in sources {
                    let text = String::from_utf8_lossy(bytes);
                    let mut fnr = 0usize;
                    for line in text.lines() {
                        nr += 1;
                        fnr += 1;
                        vm.set_line(line, nr, fnr, fname);
                        for (pat, block) in &rules {
                            let run_it = match pat {
                                AwkRulePat::Begin | AwkRulePat::End => false,
                                AwkRulePat::Always => true,
                                AwkRulePat::Expr(e) => match vm.eval_expr(e) {
                                    Ok(v) => AwkVm::is_truthy(&v),
                                    Err(err) => {
                                        return StageExecBytesOutcome::err(2, format!("{err}\n"))
                                    }
                                },
                            };
                            if run_it {
                                match vm.exec_block(block) {
                                    Ok(AwkFlow::Next) => break,
                                    Ok(AwkFlow::Exit(c)) => {
                                        exit_code_override = Some(c);
                                        break 'outer;
                                    }
                                    Ok(_) => {}
                                    Err(e) => {
                                        return StageExecBytesOutcome::err(2, format!("{e}\n"))
                                    }
                                }
                            }
                        }
                    }
                }
            }

            if exit_code_override.is_none() {
                for (pat, block) in &rules {
                    if matches!(pat, AwkRulePat::End) {
                        match vm.exec_block(block) {
                            Ok(AwkFlow::Exit(c)) => {
                                exit_code_override = Some(c);
                                break;
                            }
                            Ok(_) => {}
                            Err(e) => return StageExecBytesOutcome::err(2, format!("{e}\n")),
                        }
                    }
                }
            }

            StageExecBytesOutcome {
                exit_code: exit_code_override.unwrap_or(0),
                stdout: vm.out.into_bytes(),
                stderr: String::new(),
                instructions_retired: 0,
                vsock_frames: Vec::new(),
                compiled_elf_machine: None,
            }
        }

        // ============================================================================
        // Pure-Rust Integer `bc` Calculator & `jq` JSON Query Engine
        // ============================================================================

        #[derive(Debug, Clone, PartialEq)]
        enum BcTok {
            Ident(String),
            Num(i64),
            Semi,
            Newline,
            LParen,
            RParen,
            Plus,
            Minus,
            Star,
            Slash,
            Percent,
            Caret,
            Assign,
            Eq,
            Ne,
            Lt,
            Le,
            Gt,
            Ge,
            And,
            Or,
            Not,
        }

        fn run_bc_program(script: &str) -> StageExecBytesOutcome {
            let chars: Vec<char> = script.chars().collect();
            let mut toks = Vec::new();
            let mut i = 0usize;
            while i < chars.len() {
                let c = chars[i];
                if c == ' ' || c == '\t' || c == '\r' {
                    i += 1;
                    continue;
                }
                if c == '#' {
                    while i < chars.len() && chars[i] != '\n' {
                        i += 1;
                    }
                    continue;
                }
                if c == '\n' {
                    toks.push(BcTok::Newline);
                    i += 1;
                    continue;
                }
                if c.is_ascii_digit() {
                    let s = i;
                    while i < chars.len() && chars[i].is_ascii_digit() {
                        i += 1;
                    }
                    let num_s: String = chars[s..i].iter().collect();
                    let Ok(v) = num_s.parse::<i64>() else {
                        return StageExecBytesOutcome::err(1, "bc: integer overflow\n".to_string());
                    };
                    toks.push(BcTok::Num(v));
                    continue;
                }
                if c.is_ascii_alphabetic() || c == '_' {
                    let s = i;
                    while i < chars.len() && (chars[i].is_ascii_alphanumeric() || chars[i] == '_') {
                        i += 1;
                    }
                    let id: String = chars[s..i].iter().collect();
                    toks.push(BcTok::Ident(id));
                    continue;
                }
                let two = if i + 1 < chars.len() {
                    Some((c, chars[i + 1]))
                } else {
                    None
                };
                if let Some(p) = two {
                    let bt = match p {
                        ('=', '=') => Some(BcTok::Eq),
                        ('!', '=') => Some(BcTok::Ne),
                        ('<', '=') => Some(BcTok::Le),
                        ('>', '=') => Some(BcTok::Ge),
                        ('&', '&') => Some(BcTok::And),
                        ('|', '|') => Some(BcTok::Or),
                        _ => None,
                    };
                    if let Some(t) = bt {
                        toks.push(t);
                        i += 2;
                        continue;
                    }
                }
                let one = match c {
                    ';' => BcTok::Semi,
                    '(' => BcTok::LParen,
                    ')' => BcTok::RParen,
                    '+' => BcTok::Plus,
                    '-' => BcTok::Minus,
                    '*' => BcTok::Star,
                    '/' => BcTok::Slash,
                    '%' => BcTok::Percent,
                    '^' => BcTok::Caret,
                    '=' => BcTok::Assign,
                    '<' => BcTok::Lt,
                    '>' => BcTok::Gt,
                    '!' => BcTok::Not,
                    other => {
                        return StageExecBytesOutcome::err(
                            1,
                            format!("bc: syntax error near '{other}'\n"),
                        )
                    }
                };
                toks.push(one);
                i += 1;
            }

            struct BcEval<'a> {
                toks: &'a [BcTok],
                pos: usize,
                vars: BTreeMap<String, i64>,
            }
            impl<'a> BcEval<'a> {
                fn peek(&self) -> Option<&BcTok> {
                    self.toks.get(self.pos)
                }
                fn parse_assign(&mut self) -> Result<(i64, bool), String> {
                    if let (Some(BcTok::Ident(name)), Some(BcTok::Assign)) = (
                        self.toks.get(self.pos).cloned(),
                        self.toks.get(self.pos + 1),
                    ) {
                        self.pos += 2;
                        let (v, _) = self.parse_assign()?;
                        self.vars.insert(name, v);
                        return Ok((v, true));
                    }
                    self.parse_or().map(|v| (v, false))
                }
                fn parse_or(&mut self) -> Result<i64, String> {
                    let mut v = self.parse_and()?;
                    while matches!(self.peek(), Some(BcTok::Or)) {
                        self.pos += 1;
                        let r = self.parse_and()?;
                        v = i64::from(v != 0 || r != 0);
                    }
                    Ok(v)
                }
                fn parse_and(&mut self) -> Result<i64, String> {
                    let mut v = self.parse_cmp()?;
                    while matches!(self.peek(), Some(BcTok::And)) {
                        self.pos += 1;
                        let r = self.parse_cmp()?;
                        v = i64::from(v != 0 && r != 0);
                    }
                    Ok(v)
                }
                fn parse_cmp(&mut self) -> Result<i64, String> {
                    let mut v = self.parse_add()?;
                    while let Some(op) = self.peek().cloned() {
                        if !matches!(
                            op,
                            BcTok::Eq | BcTok::Ne | BcTok::Lt | BcTok::Le | BcTok::Gt | BcTok::Ge
                        ) {
                            break;
                        }
                        self.pos += 1;
                        let r = self.parse_add()?;
                        v = match op {
                            BcTok::Eq => i64::from(v == r),
                            BcTok::Ne => i64::from(v != r),
                            BcTok::Lt => i64::from(v < r),
                            BcTok::Le => i64::from(v <= r),
                            BcTok::Gt => i64::from(v > r),
                            BcTok::Ge => i64::from(v >= r),
                            _ => 0,
                        };
                    }
                    Ok(v)
                }
                fn parse_add(&mut self) -> Result<i64, String> {
                    let mut v = self.parse_mul()?;
                    while let Some(op) = self.peek().cloned() {
                        match op {
                            BcTok::Plus => {
                                self.pos += 1;
                                v = v.wrapping_add(self.parse_mul()?);
                            }
                            BcTok::Minus => {
                                self.pos += 1;
                                v = v.wrapping_sub(self.parse_mul()?);
                            }
                            _ => break,
                        }
                    }
                    Ok(v)
                }
                fn parse_mul(&mut self) -> Result<i64, String> {
                    let mut v = self.parse_pow()?;
                    while let Some(op) = self.peek().cloned() {
                        match op {
                            BcTok::Star => {
                                self.pos += 1;
                                v = v.wrapping_mul(self.parse_pow()?);
                            }
                            BcTok::Slash => {
                                self.pos += 1;
                                let r = self.parse_pow()?;
                                if r == 0 {
                                    return Err("bc: divide by zero".to_string());
                                }
                                v = v.wrapping_div(r);
                            }
                            BcTok::Percent => {
                                self.pos += 1;
                                let r = self.parse_pow()?;
                                if r == 0 {
                                    return Err("bc: modulo by zero".to_string());
                                }
                                v = v.wrapping_rem(r);
                            }
                            _ => break,
                        }
                    }
                    Ok(v)
                }
                fn parse_pow(&mut self) -> Result<i64, String> {
                    let base = self.parse_unary()?;
                    if matches!(self.peek(), Some(BcTok::Caret)) {
                        self.pos += 1;
                        let exp = self.parse_pow()?;
                        if exp < 0 {
                            return Ok(0);
                        }
                        return Ok(base.wrapping_pow(exp as u32));
                    }
                    Ok(base)
                }
                fn parse_unary(&mut self) -> Result<i64, String> {
                    match self.peek().cloned() {
                        Some(BcTok::Minus) => {
                            self.pos += 1;
                            Ok(self.parse_unary()?.wrapping_neg())
                        }
                        Some(BcTok::Plus) => {
                            self.pos += 1;
                            self.parse_unary()
                        }
                        Some(BcTok::Not) => {
                            self.pos += 1;
                            Ok(i64::from(self.parse_unary()? == 0))
                        }
                        _ => self.parse_primary(),
                    }
                }
                fn parse_primary(&mut self) -> Result<i64, String> {
                    match self.peek().cloned() {
                        Some(BcTok::Num(n)) => {
                            self.pos += 1;
                            Ok(n)
                        }
                        Some(BcTok::Ident(name)) => {
                            self.pos += 1;
                            Ok(self.vars.get(&name).copied().unwrap_or(0))
                        }
                        Some(BcTok::LParen) => {
                            self.pos += 1;
                            let (v, _) = self.parse_assign()?;
                            if !matches!(self.peek(), Some(BcTok::RParen)) {
                                return Err("bc: expected `)`".to_string());
                            }
                            self.pos += 1;
                            Ok(v)
                        }
                        other => Err(format!("bc: syntax error near {other:?}")),
                    }
                }
            }

            let mut ev = BcEval {
                toks: &toks,
                pos: 0,
                vars: BTreeMap::new(),
            };
            let mut out = String::new();
            while ev.pos < ev.toks.len() {
                while matches!(ev.peek(), Some(BcTok::Semi | BcTok::Newline)) {
                    ev.pos += 1;
                }
                if ev.pos >= ev.toks.len() {
                    break;
                }
                if matches!(ev.peek(), Some(BcTok::Ident(s)) if s == "quit") {
                    break;
                }
                match ev.parse_assign() {
                    Ok((v, was_assign)) => {
                        if !was_assign {
                            ev.vars.insert("last".to_string(), v);
                            out.push_str(&format!("{v}\n"));
                        }
                    }
                    Err(e) => return StageExecBytesOutcome::err(1, format!("{e}\n")),
                }
                if !matches!(ev.peek(), None | Some(BcTok::Semi | BcTok::Newline)) {
                    return StageExecBytesOutcome::err(
                        1,
                        "bc: unexpected trailing tokens\n".to_string(),
                    );
                }
            }
            StageExecBytesOutcome::ok_text(out)
        }

        match prog {
            "head" => Some((|| -> StageExecBytesOutcome {
                let mut count_lines: Option<i64> = Some(10);
                let mut count_bytes: Option<i64> = None;
                let mut quiet = false;
                let mut verbose = false;
                let mut files = Vec::new();
                let mut i = 0usize;
                while i < args.len() {
                    let a = &args[i];
                    if a == "-q" || a == "--quiet" {
                        quiet = true;
                    } else if a == "-v" || a == "--verbose" {
                        verbose = true;
                    } else if a == "-n" && i + 1 < args.len() {
                        match args[i + 1].parse::<i64>() {
                            Ok(n) => {
                                count_lines = Some(n);
                                count_bytes = None;
                            }
                            Err(_) => {
                                return StageExecBytesOutcome::err(
                                    1,
                                    format!("head: invalid number of lines: '{}'\n", args[i + 1]),
                                )
                            }
                        }
                        i += 2;
                        continue;
                    } else if let Some(rest) = a.strip_prefix("-n") {
                        match rest.parse::<i64>() {
                            Ok(n) => {
                                count_lines = Some(n);
                                count_bytes = None;
                            }
                            Err(_) => {
                                return StageExecBytesOutcome::err(
                                    1,
                                    format!("head: invalid number of lines: '{rest}'\n"),
                                )
                            }
                        }
                    } else if a == "-c" && i + 1 < args.len() {
                        match args[i + 1].parse::<i64>() {
                            Ok(n) => {
                                count_bytes = Some(n);
                                count_lines = None;
                            }
                            Err(_) => {
                                return StageExecBytesOutcome::err(
                                    1,
                                    format!("head: invalid number of bytes: '{}'\n", args[i + 1]),
                                )
                            }
                        }
                        i += 2;
                        continue;
                    } else if let Some(rest) = a.strip_prefix("-c") {
                        match rest.parse::<i64>() {
                            Ok(n) => {
                                count_bytes = Some(n);
                                count_lines = None;
                            }
                            Err(_) => {
                                return StageExecBytesOutcome::err(
                                    1,
                                    format!("head: invalid number of bytes: '{rest}'\n"),
                                )
                            }
                        }
                    } else if a.starts_with('-')
                        && a.len() > 1
                        && a[1..].chars().all(|c| c.is_ascii_digit())
                    {
                        count_lines = a[1..].parse::<i64>().ok();
                        count_bytes = None;
                    } else {
                        files.push(a.clone());
                    }
                    i += 1;
                }
                let sources = match self.read_input_sources(&files, cwd, stdin, "head") {
                    Ok(s) => s,
                    Err(e) => return StageExecBytesOutcome::err(1, e),
                };
                let show_hdr = (sources.len() > 1 && !quiet) || verbose;
                let mut out = Vec::new();
                for (idx, (name, bytes)) in sources.iter().enumerate() {
                    if show_hdr {
                        if idx > 0 {
                            out.push(b'\n');
                        }
                        out.extend_from_slice(format!("==> {name} <==\n").as_bytes());
                    }
                    if let Some(cb) = count_bytes {
                        if cb >= 0 {
                            let take = (cb as usize).min(bytes.len());
                            out.extend_from_slice(&bytes[..take]);
                        } else {
                            let drop_n = (-cb) as usize;
                            let keep = bytes.len().saturating_sub(drop_n);
                            out.extend_from_slice(&bytes[..keep]);
                        }
                    } else if let Some(cl) = count_lines {
                        let slices = split_inclusive_bytes(bytes, b'\n');
                        let take = if cl >= 0 {
                            (cl as usize).min(slices.len())
                        } else {
                            slices.len().saturating_sub((-cl) as usize)
                        };
                        for s in &slices[..take] {
                            out.extend_from_slice(s);
                        }
                    }
                }
                StageExecBytesOutcome::ok_bytes(out)
            })()),
            "tail" => Some((|| -> StageExecBytesOutcome {
                let mut line_spec = "-10".to_string();
                let mut is_bytes = false;
                let mut quiet = false;
                let mut verbose = false;
                let mut files = Vec::new();
                let mut i = 0usize;
                while i < args.len() {
                    let a = &args[i];
                    if a == "-q" {
                        quiet = true;
                    } else if a == "-v" {
                        verbose = true;
                    } else if a == "-n" && i + 1 < args.len() {
                        line_spec = args[i + 1].clone();
                        is_bytes = false;
                        i += 2;
                        continue;
                    } else if let Some(rest) = a.strip_prefix("-n") {
                        line_spec = rest.to_string();
                        is_bytes = false;
                    } else if a == "-c" && i + 1 < args.len() {
                        line_spec = args[i + 1].clone();
                        is_bytes = true;
                        i += 2;
                        continue;
                    } else if let Some(rest) = a.strip_prefix("-c") {
                        line_spec = rest.to_string();
                        is_bytes = true;
                    } else if a.starts_with('-')
                        && a.len() > 1
                        && a[1..].chars().all(|c| c.is_ascii_digit())
                    {
                        line_spec = a.clone();
                        is_bytes = false;
                    } else {
                        files.push(a.clone());
                    }
                    i += 1;
                }
                let from_start = line_spec.starts_with('+');
                let num_val: usize = match line_spec.trim_start_matches(['+', '-']).parse() {
                    Ok(v) => v,
                    Err(_) => {
                        return StageExecBytesOutcome::err(
                            1,
                            format!("tail: invalid number: '{line_spec}'\n"),
                        )
                    }
                };
                let sources = match self.read_input_sources(&files, cwd, stdin, "tail") {
                    Ok(s) => s,
                    Err(e) => return StageExecBytesOutcome::err(1, e),
                };
                let show_hdr = (sources.len() > 1 && !quiet) || verbose;
                let mut out = Vec::new();
                for (idx, (name, bytes)) in sources.iter().enumerate() {
                    if show_hdr {
                        if idx > 0 {
                            out.push(b'\n');
                        }
                        out.extend_from_slice(format!("==> {name} <==\n").as_bytes());
                    }
                    if is_bytes {
                        let start = if from_start {
                            num_val.saturating_sub(1).min(bytes.len())
                        } else {
                            bytes.len().saturating_sub(num_val)
                        };
                        out.extend_from_slice(&bytes[start..]);
                    } else {
                        let slices = split_inclusive_bytes(bytes, b'\n');
                        let start = if from_start {
                            num_val.saturating_sub(1).min(slices.len())
                        } else {
                            slices.len().saturating_sub(num_val)
                        };
                        for s in &slices[start..] {
                            out.extend_from_slice(s);
                        }
                    }
                }
                StageExecBytesOutcome::ok_bytes(out)
            })()),
            "wc" => Some((|| -> StageExecBytesOutcome {
                let mut show_l = false;
                let mut show_w = false;
                let mut show_c = false;
                let mut show_m = false;
                let mut files = Vec::new();
                for a in args {
                    if a != "-" && a.starts_with('-') && a.len() > 1 {
                        for ch in a[1..].chars() {
                            match ch {
                                'l' => show_l = true,
                                'w' => show_w = true,
                                'c' => show_c = true,
                                'm' => show_m = true,
                                _ => {
                                    return StageExecBytesOutcome::err(
                                        1,
                                        format!("wc: invalid option -- '{ch}'\n"),
                                    )
                                }
                            }
                        }
                    } else {
                        files.push(a.clone());
                    }
                }
                if !show_l && !show_w && !show_c && !show_m {
                    show_l = true;
                    show_w = true;
                    show_c = true;
                }
                let sources = match self.read_input_sources(&files, cwd, stdin, "wc") {
                    Ok(s) => s,
                    Err(e) => return StageExecBytesOutcome::err(1, e),
                };
                let mut tot_l = 0usize;
                let mut tot_w = 0usize;
                let mut tot_c = 0usize;
                let mut out = String::new();
                let has_names = !files.is_empty();
                for (name, bytes) in &sources {
                    let lines = bytes.iter().filter(|&&b| b == b'\n').count();
                    let mut words = 0usize;
                    let mut in_word = false;
                    for &b in bytes {
                        if b.is_ascii_whitespace() {
                            in_word = false;
                        } else if !in_word {
                            in_word = true;
                            words += 1;
                        }
                    }
                    let chars_or_bytes = if show_m && !show_c {
                        String::from_utf8_lossy(bytes).chars().count()
                    } else {
                        bytes.len()
                    };
                    tot_l += lines;
                    tot_w += words;
                    tot_c += chars_or_bytes;

                    let mut cols = Vec::new();
                    if show_l {
                        cols.push(lines.to_string());
                    }
                    if show_w {
                        cols.push(words.to_string());
                    }
                    if show_c || show_m {
                        cols.push(chars_or_bytes.to_string());
                    }
                    if has_names {
                        out.push_str(&format!("{} {}\n", cols.join(" "), name));
                    } else {
                        out.push_str(&format!("{}\n", cols.join(" ")));
                    }
                }
                if sources.len() > 1 {
                    let mut cols = Vec::new();
                    if show_l {
                        cols.push(tot_l.to_string());
                    }
                    if show_w {
                        cols.push(tot_w.to_string());
                    }
                    if show_c || show_m {
                        cols.push(tot_c.to_string());
                    }
                    out.push_str(&format!("{} total\n", cols.join(" ")));
                }
                StageExecBytesOutcome::ok_text(out)
            })()),
            "tr" => Some((|| -> StageExecBytesOutcome {
                let mut delete = false;
                let mut squeeze = false;
                let mut complement = false;
                let mut sets = Vec::new();
                for a in args {
                    if a.starts_with('-') && a.len() > 1 {
                        for ch in a[1..].chars() {
                            match ch {
                                'd' => delete = true,
                                's' => squeeze = true,
                                'c' | 'C' => complement = true,
                                _ => {
                                    return StageExecBytesOutcome::err(
                                        1,
                                        format!("tr: invalid option -- '{ch}'\n"),
                                    )
                                }
                            }
                        }
                    } else {
                        sets.push(a.as_str());
                    }
                }
                if sets.is_empty() || (!delete && sets.len() < 2 && !squeeze) {
                    return StageExecBytesOutcome::err(1, "tr: missing operand\n".to_string());
                }
                let mut set1 = expand_tr_set(sets[0]);
                if complement {
                    let mut comp = Vec::new();
                    for b in 0u8..=255u8 {
                        if !set1.contains(&b) {
                            comp.push(b);
                        }
                    }
                    set1 = comp;
                }
                let set2 = sets.get(1).map(|s| expand_tr_set(s)).unwrap_or_default();

                let mut out = Vec::with_capacity(stdin.len());
                let mut last_squeezed: Option<u8> = None;
                for &b in stdin {
                    let pos = set1.iter().position(|&x| x == b);
                    if delete && pos.is_some() {
                        continue;
                    }
                    let mapped = if !delete && !set2.is_empty() {
                        if let Some(p) = pos {
                            set2[p.min(set2.len() - 1)]
                        } else {
                            b
                        }
                    } else {
                        b
                    };
                    let squeeze_target = if (delete && squeeze) || (squeeze && !set2.is_empty()) {
                        set2.contains(&mapped)
                    } else if squeeze {
                        set1.contains(&mapped)
                    } else {
                        false
                    };
                    if squeeze_target && last_squeezed == Some(mapped) {
                        continue;
                    }
                    last_squeezed = if squeeze_target { Some(mapped) } else { None };
                    out.push(mapped);
                }
                StageExecBytesOutcome::ok_bytes(out)
            })()),
            "cut" => Some((|| -> StageExecBytesOutcome {
                let mut delim = '\t';
                let mut fields_spec: Option<String> = None;
                let mut chars_spec: Option<String> = None;
                let mut suppress_no_delim = false;
                let mut files = Vec::new();
                let mut i = 0usize;
                while i < args.len() {
                    let a = &args[i];
                    if a == "-s" {
                        suppress_no_delim = true;
                    } else if a == "-d" && i + 1 < args.len() {
                        delim = args[i + 1].chars().next().unwrap_or('\t');
                        i += 2;
                        continue;
                    } else if let Some(rest) = a.strip_prefix("-d") {
                        delim = rest.chars().next().unwrap_or('\t');
                    } else if a == "-f" && i + 1 < args.len() {
                        fields_spec = Some(args[i + 1].clone());
                        i += 2;
                        continue;
                    } else if let Some(rest) = a.strip_prefix("-f") {
                        fields_spec = Some(rest.to_string());
                    } else if (a == "-c" || a == "-b") && i + 1 < args.len() {
                        chars_spec = Some(args[i + 1].clone());
                        i += 2;
                        continue;
                    } else if let Some(rest) = a.strip_prefix("-c").or_else(|| a.strip_prefix("-b"))
                    {
                        chars_spec = Some(rest.to_string());
                    } else if a.starts_with('-') && a != "-" {
                        return StageExecBytesOutcome::err(
                            1,
                            format!("cut: invalid option '{a}'\n"),
                        );
                    } else {
                        files.push(a.clone());
                    }
                    i += 1;
                }
                let ranges = if let Some(ref fs) = fields_spec {
                    match parse_cut_ranges(fs) {
                        Ok(r) => r,
                        Err(e) => return StageExecBytesOutcome::err(1, format!("cut: {e}\n")),
                    }
                } else if let Some(ref cs) = chars_spec {
                    match parse_cut_ranges(cs) {
                        Ok(r) => r,
                        Err(e) => return StageExecBytesOutcome::err(1, format!("cut: {e}\n")),
                    }
                } else {
                    return StageExecBytesOutcome::err(
                        1,
                        "cut: you must specify a list of bytes, characters, or fields\n"
                            .to_string(),
                    );
                };
                let sources = match self.read_input_sources(&files, cwd, stdin, "cut") {
                    Ok(s) => s,
                    Err(e) => return StageExecBytesOutcome::err(1, e),
                };
                let mut out = String::new();
                for (_, b) in sources {
                    let text = String::from_utf8_lossy(&b);
                    for line in text.lines() {
                        if fields_spec.is_some() {
                            if !line.contains(delim) {
                                if !suppress_no_delim {
                                    out.push_str(line);
                                    out.push('\n');
                                }
                                continue;
                            }
                            let cols: Vec<&str> = line.split(delim).collect();
                            let mut selected = Vec::new();
                            for (idx, col) in cols.iter().enumerate() {
                                if in_one_based_ranges(idx + 1, &ranges) {
                                    selected.push(*col);
                                }
                            }
                            let mut d_buf = [0u8; 4];
                            let d_str = delim.encode_utf8(&mut d_buf);
                            out.push_str(&selected.join(d_str));
                            out.push('\n');
                        } else {
                            let mut line_out = String::new();
                            for (idx, ch) in line.chars().enumerate() {
                                if in_one_based_ranges(idx + 1, &ranges) {
                                    line_out.push(ch);
                                }
                            }
                            out.push_str(&line_out);
                            out.push('\n');
                        }
                    }
                }
                StageExecBytesOutcome::ok_text(out)
            })()),
            "sort" => Some((|| -> StageExecBytesOutcome {
                let mut reverse = false;
                let mut numeric = false;
                let mut unique = false;
                let mut ignore_case = false;
                let mut field_sep: Option<char> = None;
                let mut key_col: Option<usize> = None;
                let mut files = Vec::new();
                let mut i = 0usize;
                while i < args.len() {
                    let a = &args[i];
                    if a == "-t" && i + 1 < args.len() {
                        field_sep = args[i + 1].chars().next();
                        i += 2;
                        continue;
                    } else if a == "-k" && i + 1 < args.len() {
                        key_col = args[i + 1]
                            .split([',', '.'])
                            .next()
                            .and_then(|s| s.parse().ok());
                        i += 2;
                        continue;
                    } else if a != "-" && a.starts_with('-') && a.len() > 1 {
                        for ch in a[1..].chars() {
                            match ch {
                                'r' => reverse = true,
                                'n' => numeric = true,
                                'u' => unique = true,
                                'f' => ignore_case = true,
                                _ => {
                                    return StageExecBytesOutcome::err(
                                        1,
                                        format!("sort: invalid option -- '{ch}'\n"),
                                    )
                                }
                            }
                        }
                    } else {
                        files.push(a.clone());
                    }
                    i += 1;
                }
                let sources = match self.read_input_sources(&files, cwd, stdin, "sort") {
                    Ok(s) => s,
                    Err(e) => return StageExecBytesOutcome::err(1, e),
                };
                let mut lines: Vec<String> = Vec::new();
                for (_, b) in sources {
                    let text = String::from_utf8_lossy(&b);
                    for l in text.lines() {
                        lines.push(l.to_string());
                    }
                }
                let key_of = |line: &str| -> String {
                    let raw = match key_col {
                        Some(k) if k >= 1 => {
                            if let Some(sep) = field_sep {
                                line.split(sep).nth(k - 1).unwrap_or("").to_string()
                            } else {
                                line.split_whitespace().nth(k - 1).unwrap_or("").to_string()
                            }
                        }
                        _ => line.to_string(),
                    };
                    if ignore_case {
                        raw.to_ascii_lowercase()
                    } else {
                        raw
                    }
                };
                lines.sort_by(|a, b| {
                    let ka = key_of(a);
                    let kb = key_of(b);
                    let ord = if numeric {
                        let na = parse_leading_i64(&ka);
                        let nb = parse_leading_i64(&kb);
                        na.cmp(&nb).then_with(|| ka.cmp(&kb))
                    } else {
                        ka.cmp(&kb)
                    };
                    if reverse {
                        ord.reverse()
                    } else {
                        ord
                    }
                });
                if unique {
                    let mut deduped: Vec<String> = Vec::with_capacity(lines.len());
                    for l in lines {
                        let k = key_of(&l);
                        if deduped.last().is_none_or(|prev| key_of(prev) != k) {
                            deduped.push(l);
                        }
                    }
                    lines = deduped;
                }
                let mut out = lines.join("\n");
                if !out.is_empty() {
                    out.push('\n');
                }
                StageExecBytesOutcome::ok_text(out)
            })()),
            "uniq" => Some((|| -> StageExecBytesOutcome {
                let mut count = false;
                let mut only_dups = false;
                let mut only_unique = false;
                let mut ignore_case = false;
                let mut files = Vec::new();
                for a in args {
                    if a != "-" && a.starts_with('-') && a.len() > 1 {
                        for ch in a[1..].chars() {
                            match ch {
                                'c' => count = true,
                                'd' => only_dups = true,
                                'u' => only_unique = true,
                                'i' => ignore_case = true,
                                _ => {
                                    return StageExecBytesOutcome::err(
                                        1,
                                        format!("uniq: invalid option -- '{ch}'\n"),
                                    )
                                }
                            }
                        }
                    } else {
                        files.push(a.clone());
                    }
                }
                let in_list = if files.is_empty() {
                    Vec::new()
                } else {
                    vec![files[0].clone()]
                };
                let sources = match self.read_input_sources(&in_list, cwd, stdin, "uniq") {
                    Ok(s) => s,
                    Err(e) => return StageExecBytesOutcome::err(1, e),
                };
                let text = String::from_utf8_lossy(&sources[0].1);
                let mut groups: Vec<(String, usize)> = Vec::new();
                for line in text.lines() {
                    let matches_last = groups.last().is_some_and(|(prev, _)| {
                        if ignore_case {
                            prev.eq_ignore_ascii_case(line)
                        } else {
                            prev == line
                        }
                    });
                    if matches_last {
                        if let Some(last) = groups.last_mut() {
                            last.1 += 1;
                        }
                    } else {
                        groups.push((line.to_string(), 1));
                    }
                }
                let mut out = String::new();
                for (line, cnt) in groups {
                    if only_dups && cnt <= 1 {
                        continue;
                    }
                    if only_unique && cnt > 1 {
                        continue;
                    }
                    if count {
                        out.push_str(&format!("{cnt:>7} {line}\n"));
                    } else {
                        out.push_str(&format!("{line}\n"));
                    }
                }
                if let Some(out_file) = files.get(1) {
                    if let Err(msg) =
                        self.write_redirected_bytes(cwd, out_file, out.as_bytes(), false, None)
                    {
                        return StageExecBytesOutcome::err(1, format!("{msg}\n"));
                    }
                    return StageExecBytesOutcome::ok_empty();
                }
                StageExecBytesOutcome::ok_text(out)
            })()),
            "grep" | "rg" => Some((|| -> StageExecBytesOutcome {
                let is_rg = prog == "rg";
                let mut extended = is_rg;
                let mut fixed = false;
                let mut ignore_case = false;
                let mut invert = false;
                let mut show_line_num = is_rg;
                let mut count_only = false;
                let mut files_with_matches = false;
                let mut quiet = false;
                let mut only_matching = false;
                let mut word_match = false;
                let mut recursive = is_rg;
                let mut force_no_fname = false;
                let mut force_fname = false;
                let mut include_hidden = false;
                let mut patterns: Vec<String> = Vec::new();
                let mut operands: Vec<String> = Vec::new();

                let mut i = 0usize;
                while i < args.len() {
                    let a = &args[i];
                    if (a == "-e" || a == "--regexp") && i + 1 < args.len() {
                        patterns.push(args[i + 1].clone());
                        i += 2;
                        continue;
                    } else if is_rg && (a == "-g" || a == "--glob") && i + 1 < args.len() {
                        i += 2;
                        continue;
                    } else if a == "--" {
                        operands.extend(args[i + 1..].iter().cloned());
                        break;
                    } else if is_rg && a.starts_with("--") {
                        match a.as_str() {
                            "--no-line-number" => show_line_num = false,
                            "--line-number" => show_line_num = true,
                            "--fixed-strings" => fixed = true,
                            "--ignore-case" => ignore_case = true,
                            "--invert-match" => invert = true,
                            "--files-with-matches" => files_with_matches = true,
                            "--count" => count_only = true,
                            "--quiet" => quiet = true,
                            "--only-matching" => only_matching = true,
                            "--word-regexp" => word_match = true,
                            "--no-filename" => force_no_fname = true,
                            "--with-filename" => force_fname = true,
                            "--hidden" | "--unrestricted" => include_hidden = true,
                            "--no-heading" | "--color=never" => {}
                            _ => {
                                return StageExecBytesOutcome::err(
                                    2,
                                    format!("rg: unrecognized flag '{a}'\n"),
                                )
                            }
                        }
                    } else if a != "-" && a.starts_with('-') && a.len() > 1 {
                        for ch in a[1..].chars() {
                            match ch {
                                'E' => extended = true,
                                'F' => fixed = true,
                                'i' => ignore_case = true,
                                'v' => invert = true,
                                'n' => show_line_num = true,
                                'N' if is_rg => show_line_num = false,
                                'c' => count_only = true,
                                'l' => files_with_matches = true,
                                'q' => quiet = true,
                                'o' => only_matching = true,
                                'w' => word_match = true,
                                'r' | 'R' => recursive = true,
                                'h' => force_no_fname = true,
                                'H' => force_fname = true,
                                'u' if is_rg => include_hidden = true,
                                'S' if is_rg => {}
                                _ => {
                                    return StageExecBytesOutcome::err(
                                        2,
                                        format!("{prog}: invalid option -- '{ch}'\n"),
                                    )
                                }
                            }
                        }
                    } else {
                        operands.push(a.clone());
                    }
                    i += 1;
                }
                if patterns.is_empty() {
                    if operands.is_empty() {
                        return StageExecBytesOutcome::err(2, format!("{prog}: missing pattern\n"));
                    }
                    patterns.push(operands.remove(0));
                }
                if is_rg && operands.is_empty() && stdin.is_empty() {
                    operands.push(".".to_string());
                }

                let compiled: Vec<Option<RxPattern>> = if fixed {
                    Vec::new()
                } else {
                    let mut v = Vec::with_capacity(patterns.len());
                    for p in &patterns {
                        let eff_pat = if word_match {
                            format!("\\b({p})\\b")
                        } else {
                            p.clone()
                        };
                        match RxPattern::compile(&eff_pat, extended || word_match, ignore_case) {
                            Ok(rx) => v.push(Some(rx)),
                            Err(e) => {
                                return StageExecBytesOutcome::err(2, format!("{prog}: {e}\n"))
                            }
                        }
                    }
                    v
                };

                let mut file_list: Vec<(String, Vec<u8>)> = Vec::new();
                let mut searched_dir = false;
                if operands.is_empty() {
                    file_list.push(("(standard input)".to_string(), stdin.to_vec()));
                } else {
                    for op in &operands {
                        if op == "-" {
                            file_list.push(("(standard input)".to_string(), stdin.to_vec()));
                            continue;
                        }
                        let norm = Self::normalize_guest_path(cwd, op);
                        if self.dirs.contains(&norm) {
                            if recursive {
                                searched_dir = true;
                                let prefix = if norm == "/" {
                                    "/".to_string()
                                } else {
                                    format!("{norm}/")
                                };
                                for p in self.inodes.keys() {
                                    if p.starts_with(&prefix) {
                                        if is_rg && !include_hidden && p.contains("/.") {
                                            continue;
                                        }
                                        if let Some(b) = self.read_guest_file(p) {
                                            if is_rg
                                                && (b.starts_with(b"\x7fELF") || b.contains(&0))
                                            {
                                                continue;
                                            }
                                            file_list.push((p.clone(), b));
                                        }
                                    }
                                }
                            } else {
                                return StageExecBytesOutcome::err(
                                    2,
                                    format!("{prog}: {op}: Is a directory\n"),
                                );
                            }
                        } else if let Some(b) = self.read_guest_file(&norm) {
                            file_list.push((op.clone(), b));
                        } else {
                            return StageExecBytesOutcome::err(
                                2,
                                format!("{prog}: {op}: No such file or directory\n"),
                            );
                        }
                    }
                }

                let show_fname = force_fname
                    || (!force_no_fname && (file_list.len() > 1 || (is_rg && searched_dir)));
                let mut out = String::new();
                let mut any_match = false;
                for (fname, bytes) in &file_list {
                    let text = String::from_utf8_lossy(bytes);
                    let mut match_count = 0usize;
                    for (lidx, line) in text.lines().enumerate() {
                        let line_matches = if fixed {
                            patterns.iter().any(|p| {
                                if ignore_case {
                                    line.to_ascii_lowercase().contains(&p.to_ascii_lowercase())
                                } else {
                                    line.contains(p.as_str())
                                }
                            })
                        } else {
                            compiled
                                .iter()
                                .flatten()
                                .any(|rx| rx.find_first(line).is_some())
                        };
                        let selected = line_matches != invert;
                        if selected {
                            any_match = true;
                            match_count += 1;
                            if quiet {
                                return StageExecBytesOutcome::ok_empty();
                            }
                            if files_with_matches {
                                out.push_str(&format!("{fname}\n"));
                                break;
                            }
                            if !count_only {
                                if only_matching && !fixed && !invert {
                                    for rx in compiled.iter().flatten() {
                                        for (s, e, _) in rx.find_all(line) {
                                            if show_fname {
                                                out.push_str(&format!("{fname}:"));
                                            }
                                            if show_line_num {
                                                out.push_str(&format!("{}:", lidx + 1));
                                            }
                                            out.push_str(&line[s..e]);
                                            out.push('\n');
                                        }
                                    }
                                } else {
                                    if show_fname {
                                        out.push_str(&format!("{fname}:"));
                                    }
                                    if show_line_num {
                                        out.push_str(&format!("{}:", lidx + 1));
                                    }
                                    out.push_str(line);
                                    out.push('\n');
                                }
                            }
                        }
                    }
                    if count_only {
                        if show_fname {
                            out.push_str(&format!("{fname}:"));
                        }
                        out.push_str(&format!("{match_count}\n"));
                    }
                }
                StageExecBytesOutcome {
                    exit_code: if any_match { 0 } else { 1 },
                    stdout: out.into_bytes(),
                    stderr: String::new(),
                    instructions_retired: 0,
                    vsock_frames: Vec::new(),
                    compiled_elf_machine: None,
                }
            })()),
            "patch" => Some((|| -> StageExecBytesOutcome {
                let mut strip_p: usize = 0;
                let mut patch_file: Option<String> = None;
                let mut patch_dir = cwd.to_string();
                let mut dry_run = false;
                let mut silent = false;
                let mut pos_args: Vec<String> = Vec::new();
                let mut i = 0usize;
                while i < args.len() {
                    let a = &args[i];
                    if a == "-p" && i + 1 < args.len() {
                        strip_p = args[i + 1].parse::<usize>().unwrap_or(0);
                        i += 2;
                        continue;
                    } else if let Some(n) = a.strip_prefix("-p") {
                        strip_p = n.parse::<usize>().unwrap_or(0);
                    } else if (a == "-i" || a == "--input") && i + 1 < args.len() {
                        patch_file = Some(args[i + 1].clone());
                        i += 2;
                        continue;
                    } else if (a == "-d" || a == "--directory") && i + 1 < args.len() {
                        patch_dir = Self::normalize_guest_path(&patch_dir, &args[i + 1]);
                        i += 2;
                        continue;
                    } else if a == "--dry-run" {
                        dry_run = true;
                    } else if a == "-s" || a == "--silent" || a == "--quiet" {
                        silent = true;
                    } else if a == "-u" || a == "--unified" || a == "-N" || a == "--forward" {
                    } else if a.starts_with('-') && a != "-" {
                        return StageExecBytesOutcome::err(
                            2,
                            format!("patch: unsupported option '{a}'\n"),
                        );
                    } else {
                        pos_args.push(a.clone());
                    }
                    i += 1;
                }

                let explicit_target = pos_args.first().cloned();
                if patch_file.is_none() && pos_args.len() >= 2 {
                    patch_file = Some(pos_args[1].clone());
                }
                let patch_bytes = if let Some(ref pf) = patch_file {
                    if pf == "-" {
                        stdin.to_vec()
                    } else {
                        let norm = Self::normalize_guest_path(cwd, pf);
                        let Some(b) = self.read_guest_file(&norm) else {
                            return StageExecBytesOutcome::err(
                                2,
                                format!("patch: **** Can't open patch file {pf} : No such file or directory\n"),
                            );
                        };
                        b
                    }
                } else {
                    stdin.to_vec()
                };
                let patch_text = String::from_utf8_lossy(&patch_bytes);
                let plines: Vec<&str> = patch_text.lines().collect();

                fn strip_patch_path(raw: &str, strip_p: usize) -> String {
                    let clean = raw.split('\t').next().unwrap_or(raw).trim();
                    if clean == "/dev/null" {
                        return "/dev/null".to_string();
                    }
                    let mut parts: Vec<&str> = clean.split('/').filter(|s| !s.is_empty()).collect();
                    if strip_p > 0 && parts.len() > strip_p {
                        parts.drain(0..strip_p);
                    }
                    if clean.starts_with('/') && strip_p == 0 {
                        format!("/{}", parts.join("/"))
                    } else {
                        parts.join("/")
                    }
                }

                fn parse_hunk_range(spec: &str) -> (usize, usize) {
                    let body = &spec[1..];
                    if let Some((s, c)) = body.split_once(',') {
                        (
                            s.parse::<usize>().unwrap_or(1),
                            c.parse::<usize>().unwrap_or(1),
                        )
                    } else {
                        (body.parse::<usize>().unwrap_or(1), 1)
                    }
                }

                let mut out_msg = String::new();
                let mut li = 0usize;
                let mut applied_files = 0usize;
                while li < plines.len() {
                    if !plines[li].starts_with("--- ") {
                        li += 1;
                        continue;
                    }
                    let old_raw = &plines[li][4..];
                    if li + 1 >= plines.len() || !plines[li + 1].starts_with("+++ ") {
                        return StageExecBytesOutcome::err(
                            2,
                            "patch: malformed unified diff header\n".to_string(),
                        );
                    }
                    let new_raw = &plines[li + 1][4..];
                    li += 2;

                    let old_path = strip_patch_path(old_raw, strip_p);
                    let new_path = strip_patch_path(new_raw, strip_p);
                    let target_rel = if let Some(ref et) = explicit_target {
                        et.clone()
                    } else if old_path != "/dev/null"
                        && new_path != "/dev/null"
                        && self
                            .read_guest_file(&Self::normalize_guest_path(&patch_dir, &old_path))
                            .is_some()
                    {
                        old_path.clone()
                    } else if new_path != "/dev/null" {
                        new_path.clone()
                    } else {
                        old_path.clone()
                    };
                    let norm_target = Self::normalize_guest_path(&patch_dir, &target_rel);

                    let mut orig_lines: Vec<String> = if old_path == "/dev/null" {
                        Vec::new()
                    } else if let Some(b) = self.read_guest_file(&norm_target) {
                        String::from_utf8_lossy(&b)
                            .lines()
                            .map(|s| s.to_string())
                            .collect()
                    } else if explicit_target.is_some() {
                        Vec::new()
                    } else {
                        return StageExecBytesOutcome::err(
                            1,
                            format!("patch: can't find file to patch '{target_rel}'\n"),
                        );
                    };
                    let had_trailing_nl = self
                        .read_guest_file(&norm_target)
                        .is_none_or(|b| b.is_empty() || b.ends_with(b"\n"));

                    let mut net_delta: isize = 0;
                    let mut saw_hunk = false;
                    while li < plines.len() && !plines[li].starts_with("--- ") {
                        if !plines[li].starts_with("@@ ") {
                            li += 1;
                            continue;
                        }
                        saw_hunk = true;
                        let hdr_end = plines[li][3..]
                            .find(" @@")
                            .map(|k| 3 + k)
                            .unwrap_or(plines[li].len());
                        let header_parts: Vec<&str> =
                            plines[li][3..hdr_end].split_whitespace().collect();
                        let (old_start, _) = header_parts
                            .first()
                            .filter(|s| s.starts_with('-'))
                            .map(|s| parse_hunk_range(s))
                            .unwrap_or((1, 1));
                        li += 1;

                        let mut h_old: Vec<String> = Vec::new();
                        let mut h_new: Vec<String> = Vec::new();
                        while li < plines.len()
                            && !plines[li].starts_with("@@ ")
                            && !plines[li].starts_with("--- ")
                        {
                            let hl = plines[li];
                            if let Some(rest) = hl.strip_prefix(' ') {
                                h_old.push(rest.to_string());
                                h_new.push(rest.to_string());
                            } else if let Some(rest) = hl.strip_prefix('-') {
                                h_old.push(rest.to_string());
                            } else if let Some(rest) = hl.strip_prefix('+') {
                                h_new.push(rest.to_string());
                            } else if hl.starts_with("\\ No newline") {
                            } else {
                                break;
                            }
                            li += 1;
                        }

                        let base_pos = if old_start == 0 {
                            0usize
                        } else {
                            ((old_start as isize - 1 + net_delta).max(0) as usize)
                                .min(orig_lines.len())
                        };
                        let mut found_pos: Option<usize> = None;
                        for offset in [0isize, -1, 1, -2, 2, -3, 3] {
                            let cand = (base_pos as isize + offset).max(0) as usize;
                            if cand + h_old.len() <= orig_lines.len()
                                && orig_lines[cand..cand + h_old.len()] == h_old[..]
                            {
                                found_pos = Some(cand);
                                break;
                            }
                        }
                        let Some(apply_at) = found_pos else {
                            return StageExecBytesOutcome::err(
                                1,
                                format!(
                                    "patch: hunk FAILED for '{target_rel}' at line {old_start}\n"
                                ),
                            );
                        };
                        let old_len = h_old.len();
                        let new_len = h_new.len();
                        orig_lines.splice(apply_at..apply_at + old_len, h_new);
                        net_delta += (new_len as isize) - (old_len as isize);
                    }
                    if !saw_hunk {
                        continue;
                    }
                    applied_files += 1;
                    if !dry_run {
                        if new_path == "/dev/null" {
                            if let Err(msg) = self.remove_guest_path(&norm_target, false) {
                                return StageExecBytesOutcome::err(1, format!("{msg}\n"));
                            }
                        } else {
                            let mut rendered = orig_lines.join("\n");
                            if !orig_lines.is_empty() && had_trailing_nl {
                                rendered.push('\n');
                            }
                            if let Err(msg) = self.write_redirected_bytes(
                                "/",
                                &norm_target,
                                rendered.as_bytes(),
                                false,
                                None,
                            ) {
                                return StageExecBytesOutcome::err(1, format!("{msg}\n"));
                            }
                        }
                    }
                    if !silent {
                        out_msg.push_str(&format!("patching file {target_rel}\n"));
                    }
                }

                if applied_files == 0 {
                    return StageExecBytesOutcome::err(
                        2,
                        "patch: **** Only garbage was found in the patch input.\n".to_string(),
                    );
                }
                StageExecBytesOutcome::ok_text(out_msg)
            })()),
            "sed" => Some((|| -> StageExecBytesOutcome {
                let mut quiet = false;
                let mut extended = false;
                let mut in_place = false;
                let mut scripts: Vec<String> = Vec::new();
                let mut operands: Vec<String> = Vec::new();
                let mut i = 0usize;
                while i < args.len() {
                    let a = &args[i];
                    if a == "-n" {
                        quiet = true;
                    } else if a == "-E" || a == "-r" {
                        extended = true;
                    } else if a == "-i" || a.starts_with("-i") {
                        in_place = true;
                    } else if a == "-e" && i + 1 < args.len() {
                        scripts.push(args[i + 1].clone());
                        i += 2;
                        continue;
                    } else if a.starts_with('-') && a != "-" {
                        return StageExecBytesOutcome::err(
                            1,
                            format!("sed: invalid option '{a}'\n"),
                        );
                    } else {
                        operands.push(a.clone());
                    }
                    i += 1;
                }
                if scripts.is_empty() {
                    if operands.is_empty() {
                        return StageExecBytesOutcome::err(1, "sed: missing script\n".to_string());
                    }
                    scripts.push(operands.remove(0));
                }
                let full_script = scripts.join("\n");
                let cmds = match parse_sed_script(&full_script, extended) {
                    Ok(c) => c,
                    Err(e) => return StageExecBytesOutcome::err(1, format!("sed: {e}\n")),
                };
                let sources = match self.read_input_sources(&operands, cwd, stdin, "sed") {
                    Ok(s) => s,
                    Err(e) => return StageExecBytesOutcome::err(2, e),
                };
                let mut out_all = String::new();
                for (fname, bytes) in sources {
                    let text = String::from_utf8_lossy(&bytes);
                    let lines: Vec<&str> = text.lines().collect();
                    let trailing_nl = bytes.ends_with(b"\n");
                    let mut file_out = String::new();
                    let mut range_active = vec![false; cmds.len()];

                    for (idx, raw_line) in lines.iter().enumerate() {
                        let line_no = idx + 1;
                        let is_last = line_no == lines.len();
                        let mut cur = (*raw_line).to_string();
                        let mut deleted = false;
                        let mut quit = false;
                        for (cidx, cmd) in cmds.iter().enumerate() {
                            if !sed_addr_matches(
                                &cmd.addr,
                                line_no,
                                is_last,
                                &cur,
                                &mut range_active[cidx],
                            ) {
                                continue;
                            }
                            match &cmd.action {
                                SedAction::Delete => {
                                    deleted = true;
                                    break;
                                }
                                SedAction::Print => {
                                    file_out.push_str(&cur);
                                    file_out.push('\n');
                                }
                                SedAction::Quit => {
                                    quit = true;
                                    break;
                                }
                                SedAction::LineNum => {
                                    file_out.push_str(&format!("{line_no}\n"));
                                }
                                SedAction::Transliterate(src, dst) => {
                                    cur = cur
                                        .chars()
                                        .map(|ch| {
                                            src.chars()
                                                .position(|s| s == ch)
                                                .and_then(|p| dst.chars().nth(p))
                                                .unwrap_or(ch)
                                        })
                                        .collect();
                                }
                                SedAction::Substitute {
                                    rx,
                                    repl,
                                    global,
                                    print,
                                } => {
                                    let (next_s, matched) = sed_apply_sub(rx, &cur, repl, *global);
                                    cur = next_s;
                                    if matched && *print {
                                        file_out.push_str(&cur);
                                        file_out.push('\n');
                                    }
                                }
                            }
                        }
                        if !deleted && !quiet {
                            file_out.push_str(&cur);
                            if idx + 1 < lines.len() || trailing_nl {
                                file_out.push('\n');
                            }
                        }
                        if quit {
                            break;
                        }
                    }
                    if in_place && fname != "-" {
                        if let Err(msg) = self.write_redirected_bytes(
                            cwd,
                            &fname,
                            file_out.as_bytes(),
                            false,
                            None,
                        ) {
                            return StageExecBytesOutcome::err(1, format!("{msg}\n"));
                        }
                    } else {
                        out_all.push_str(&file_out);
                    }
                }
                StageExecBytesOutcome::ok_text(out_all)
            })()),
            "awk" => Some((|| -> StageExecBytesOutcome {
                let mut fs: Option<String> = None;
                let mut vars: BTreeMap<String, String> = BTreeMap::new();
                let mut prog_src: Option<String> = None;
                let mut files: Vec<String> = Vec::new();
                let mut i = 0usize;
                while i < args.len() {
                    let a = &args[i];
                    if a == "-F" && i + 1 < args.len() {
                        fs = Some(args[i + 1].clone());
                        i += 2;
                        continue;
                    } else if let Some(rest) = a.strip_prefix("-F") {
                        fs = Some(rest.to_string());
                    } else if a == "-v" && i + 1 < args.len() {
                        if let Some((k, v)) = args[i + 1].split_once('=') {
                            vars.insert(k.trim().to_string(), v.to_string());
                        }
                        i += 2;
                        continue;
                    } else if a == "-f" && i + 1 < args.len() {
                        let norm = Self::normalize_guest_path(cwd, &args[i + 1]);
                        let Some(b) = self.read_guest_file(&norm) else {
                            return StageExecBytesOutcome::err(
                                2,
                                format!("awk: fatal: cannot open file '{}'\n", args[i + 1]),
                            );
                        };
                        prog_src = Some(String::from_utf8_lossy(&b).into_owned());
                        i += 2;
                        continue;
                    } else if a != "-" && a.starts_with('-') {
                        return StageExecBytesOutcome::err(
                            2,
                            format!("awk: invalid option '{a}'\n"),
                        );
                    } else if prog_src.is_none() {
                        prog_src = Some(a.clone());
                    } else {
                        files.push(a.clone());
                    }
                    i += 1;
                }
                let Some(script) = prog_src else {
                    return StageExecBytesOutcome::err(2, "awk: missing program\n".to_string());
                };
                let sources = match self.read_input_sources(&files, cwd, stdin, "awk") {
                    Ok(s) => s,
                    Err(e) => return StageExecBytesOutcome::err(2, e),
                };
                run_awk_program(&script, fs.as_deref(), vars, &sources)
            })()),
            "find" => Some((|| -> StageExecBytesOutcome {
                let mut roots: Vec<String> = Vec::new();
                let mut idx = 0usize;
                while idx < args.len() && !args[idx].starts_with('-') && args[idx] != "!" {
                    roots.push(args[idx].clone());
                    idx += 1;
                }
                if roots.is_empty() {
                    roots.push(".".to_string());
                }
                let mut name_pats: Vec<(bool, String)> = Vec::new();
                let mut type_filter: Option<char> = None;
                let mut maxdepth: Option<usize> = None;
                let mut mindepth: usize = 0;
                let mut only_empty = false;
                while idx < args.len() {
                    match args[idx].as_str() {
                        "-name" if idx + 1 < args.len() => {
                            name_pats.push((false, args[idx + 1].clone()));
                            idx += 2;
                        }
                        "-iname" if idx + 1 < args.len() => {
                            name_pats.push((true, args[idx + 1].clone()));
                            idx += 2;
                        }
                        "-type" if idx + 1 < args.len() => {
                            type_filter = args[idx + 1].chars().next();
                            idx += 2;
                        }
                        "-maxdepth" if idx + 1 < args.len() => {
                            maxdepth = args[idx + 1].parse().ok();
                            idx += 2;
                        }
                        "-mindepth" if idx + 1 < args.len() => {
                            mindepth = args[idx + 1].parse().unwrap_or(0);
                            idx += 2;
                        }
                        "-empty" => {
                            only_empty = true;
                            idx += 1;
                        }
                        "-print" => idx += 1,
                        other => {
                            return StageExecBytesOutcome::err(
                                1,
                                format!("find: unsupported predicate '{other}'\n"),
                            )
                        }
                    }
                }

                let mut out = String::new();
                for raw_root in roots {
                    let norm_root = Self::normalize_guest_path(cwd, &raw_root);
                    let is_dir = self.dirs.contains(&norm_root);
                    let is_file = self.inodes.contains_key(&norm_root);
                    if !is_dir && !is_file {
                        return StageExecBytesOutcome::err(
                            1,
                            format!("find: '{raw_root}': No such file or directory\n"),
                        );
                    }
                    let mut candidates: BTreeSet<String> = BTreeSet::new();
                    candidates.insert(norm_root.clone());
                    if is_dir {
                        let prefix = if norm_root == "/" {
                            "/".to_string()
                        } else {
                            format!("{norm_root}/")
                        };
                        for d in &self.dirs {
                            if d.starts_with(&prefix) {
                                candidates.insert(d.clone());
                            }
                        }
                        for f in self.inodes.keys() {
                            if f.starts_with(&prefix) {
                                candidates.insert(f.clone());
                            }
                        }
                    }
                    for cand in candidates {
                        let rel_to_root = if cand == norm_root {
                            ""
                        } else if norm_root == "/" {
                            cand.strip_prefix('/').unwrap_or(&cand)
                        } else {
                            cand.strip_prefix(&format!("{norm_root}/")).unwrap_or(&cand)
                        };
                        let depth = if rel_to_root.is_empty() {
                            0
                        } else {
                            rel_to_root.split('/').count()
                        };
                        if depth < mindepth || maxdepth.is_some_and(|m| depth > m) {
                            continue;
                        }
                        let inode_opt = self.inodes.get(&cand);
                        let ctype = match inode_opt {
                            Some(ino) => match ino.kind {
                                InodeKind::Symlink { .. } => 'l',
                                InodeKind::Regular { .. } => 'f',
                            },
                            None => 'd',
                        };
                        if let Some(tf) = type_filter {
                            if ctype != tf {
                                continue;
                            }
                        }
                        if only_empty {
                            let is_empty_item = match inode_opt {
                                Some(ino) => match ino.kind {
                                    InodeKind::Regular { byte_len, .. } => byte_len == 0,
                                    InodeKind::Symlink { .. } => false,
                                },
                                None => {
                                    let pref = format!("{cand}/");
                                    !self.inodes.keys().any(|k| k.starts_with(&pref))
                                        && !self.dirs.iter().any(|d| d.starts_with(&pref))
                                }
                            };
                            if !is_empty_item {
                                continue;
                            }
                        }
                        let base = cand.rsplit('/').next().unwrap_or(&cand);
                        let mut name_ok = true;
                        for (ci, pat) in &name_pats {
                            let hit = if *ci {
                                glob_match(&pat.to_ascii_lowercase(), &base.to_ascii_lowercase())
                            } else {
                                glob_match(pat, base)
                            };
                            if !hit {
                                name_ok = false;
                                break;
                            }
                        }
                        if name_ok {
                            let display = if raw_root == "." {
                                if cand == norm_root {
                                    ".".to_string()
                                } else {
                                    format!("./{rel_to_root}")
                                }
                            } else if raw_root.ends_with('/') && raw_root != "/" {
                                if cand == norm_root {
                                    raw_root.clone()
                                } else {
                                    format!("{raw_root}{rel_to_root}")
                                }
                            } else if cand == norm_root {
                                raw_root.clone()
                            } else if raw_root == "/" {
                                format!("/{rel_to_root}")
                            } else {
                                format!("{raw_root}/{rel_to_root}")
                            };
                            out.push_str(&display);
                            out.push('\n');
                        }
                    }
                }
                StageExecBytesOutcome::ok_text(out)
            })()),
            "seq" => Some((|| -> StageExecBytesOutcome {
                let nums: Result<Vec<i64>, _> = args.iter().map(|s| s.parse::<i64>()).collect();
                let Ok(vals) = nums else {
                    return StageExecBytesOutcome::err(
                        1,
                        "seq: invalid integer argument\n".to_string(),
                    );
                };
                let (start, step, end) = match vals.as_slice() {
                    [e] => (1, 1, *e),
                    [s, e] => (*s, if *s <= *e { 1 } else { -1 }, *e),
                    [s, inc, e] => (*s, *inc, *e),
                    _ => {
                        return StageExecBytesOutcome::err(1, "seq: missing operand\n".to_string())
                    }
                };
                if step == 0 {
                    return StageExecBytesOutcome::err(1, "seq: zero increment\n".to_string());
                }
                let mut out = String::new();
                let mut cur = start;
                let mut count = 0usize;
                while ((step > 0 && cur <= end) || (step < 0 && cur >= end)) && count < 10_000 {
                    out.push_str(&format!("{cur}\n"));
                    cur += step;
                    count += 1;
                }
                StageExecBytesOutcome::ok_text(out)
            })()),
            "diff" => Some((|| -> StageExecBytesOutcome {
                let mut unified = false;
                let mut brief = false;
                let mut ignore_case = false;
                let mut files = Vec::new();
                for a in args {
                    if a != "-" && a.starts_with('-') && a.len() > 1 {
                        for ch in a[1..].chars() {
                            match ch {
                                'u' => unified = true,
                                'q' => brief = true,
                                'i' => ignore_case = true,
                                'r' | 'w' | 'N' => {}
                                _ => {
                                    return StageExecBytesOutcome::err(
                                        2,
                                        format!("diff: invalid option -- '{ch}'\n"),
                                    )
                                }
                            }
                        }
                    } else {
                        files.push(a.clone());
                    }
                }
                if files.len() < 2 {
                    return StageExecBytesOutcome::err(2, "diff: missing operand\n".to_string());
                }
                let read_one = |p: &str| -> Result<Vec<u8>, String> {
                    if p == "-" {
                        Ok(stdin.to_vec())
                    } else {
                        let norm = Self::normalize_guest_path(cwd, p);
                        self.read_guest_file(&norm)
                            .ok_or_else(|| format!("diff: {p}: No such file or directory\n"))
                    }
                };
                let a_bytes = match read_one(&files[0]) {
                    Ok(b) => b,
                    Err(e) => return StageExecBytesOutcome::err(2, e),
                };
                let b_bytes = match read_one(&files[1]) {
                    Ok(b) => b,
                    Err(e) => return StageExecBytesOutcome::err(2, e),
                };
                if a_bytes == b_bytes {
                    return StageExecBytesOutcome::ok_empty();
                }
                if is_binary_bytes_for_diff(&a_bytes) || is_binary_bytes_for_diff(&b_bytes) {
                    return StageExecBytesOutcome {
                        exit_code: 1,
                        stdout: format!("Binary files {} and {} differ\n", files[0], files[1])
                            .into_bytes(),
                        stderr: String::new(),
                        instructions_retired: 0,
                        vsock_frames: Vec::new(),
                        compiled_elf_machine: None,
                    };
                }
                let a_text = String::from_utf8_lossy(&a_bytes);
                let b_text = String::from_utf8_lossy(&b_bytes);
                let a_lines: Vec<&str> = a_text.lines().collect();
                let b_lines: Vec<&str> = b_text.lines().collect();
                let eq_line = |x: &str, y: &str| -> bool {
                    if ignore_case {
                        x.eq_ignore_ascii_case(y)
                    } else {
                        x == y
                    }
                };
                if a_lines.len() == b_lines.len()
                    && a_lines
                        .iter()
                        .zip(b_lines.iter())
                        .all(|(x, y)| eq_line(x, y))
                {
                    return StageExecBytesOutcome::ok_empty();
                }
                if brief {
                    return StageExecBytesOutcome {
                        exit_code: 1,
                        stdout: format!("Files {} and {} differ\n", files[0], files[1])
                            .into_bytes(),
                        stderr: String::new(),
                        instructions_retired: 0,
                        vsock_frames: Vec::new(),
                        compiled_elf_machine: None,
                    };
                }
                let diff_str = render_lcs_diff(
                    &files[0],
                    &files[1],
                    &a_lines,
                    &b_lines,
                    unified,
                    ignore_case,
                );
                StageExecBytesOutcome {
                    exit_code: 1,
                    stdout: diff_str.into_bytes(),
                    stderr: String::new(),
                    instructions_retired: 0,
                    vsock_frames: Vec::new(),
                    compiled_elf_machine: None,
                }
            })()),
            "bc" => Some((|| -> StageExecBytesOutcome {
                let mut files = Vec::new();
                for a in args {
                    if a == "-l" || a == "-q" {
                        continue;
                    } else if a.starts_with('-') && a != "-" {
                        return StageExecBytesOutcome::err(
                            1,
                            format!("bc: invalid option '{a}'\n"),
                        );
                    } else {
                        files.push(a.clone());
                    }
                }
                let sources = match self.read_input_sources(&files, cwd, stdin, "bc") {
                    Ok(s) => s,
                    Err(e) => return StageExecBytesOutcome::err(1, e),
                };
                let mut combined = String::new();
                for (_, b) in sources {
                    combined.push_str(&String::from_utf8_lossy(&b));
                    combined.push('\n');
                }
                run_bc_program(&combined)
            })()),
            "jq" => Some((|| -> StageExecBytesOutcome {
                let mut raw_output = false;
                let mut compact = false;
                let mut exit_status = false;
                let mut sort_keys = false;
                let mut slurp = false;
                let mut null_input = false;
                let mut env_args: BTreeMap<String, serde_json::Value> = BTreeMap::new();
                let mut filter_str: Option<String> = None;
                let mut files: Vec<String> = Vec::new();
                let mut i = 0usize;
                while i < args.len() {
                    let a = &args[i];
                    if a == "--arg" && i + 2 < args.len() {
                        env_args.insert(
                            args[i + 1].clone(),
                            serde_json::Value::String(args[i + 2].clone()),
                        );
                        i += 3;
                        continue;
                    } else if a == "--argjson" && i + 2 < args.len() {
                        let val: serde_json::Value = match serde_json::from_str(&args[i + 2]) {
                            Ok(v) => v,
                            Err(e) => {
                                return StageExecBytesOutcome::err(
                                    2,
                                    format!("jq: invalid JSON for --argjson: {e}\n"),
                                )
                            }
                        };
                        env_args.insert(args[i + 1].clone(), val);
                        i += 3;
                        continue;
                    } else if a == "--raw-output" {
                        raw_output = true;
                    } else if a == "--compact-output" {
                        compact = true;
                    } else if a == "--exit-status" {
                        exit_status = true;
                    } else if a == "--sort-keys" {
                        sort_keys = true;
                    } else if a == "--slurp" {
                        slurp = true;
                    } else if a == "--null-input" {
                        null_input = true;
                    } else if a != "-" && a.starts_with('-') && a.len() > 1 {
                        for ch in a[1..].chars() {
                            match ch {
                                'r' => raw_output = true,
                                'c' => compact = true,
                                'e' => exit_status = true,
                                'S' => sort_keys = true,
                                's' => slurp = true,
                                'n' => null_input = true,
                                _ => {
                                    return StageExecBytesOutcome::err(
                                        2,
                                        format!("jq: invalid option -- '{ch}'\n"),
                                    )
                                }
                            }
                        }
                    } else if filter_str.is_none() {
                        filter_str = Some(a.clone());
                    } else {
                        files.push(a.clone());
                    }
                    i += 1;
                }
                let filter = filter_str.unwrap_or_else(|| ".".to_string());
                let compiled = match parse_jq_filter(&filter) {
                    Ok(f) => f,
                    Err(e) => return StageExecBytesOutcome::err(3, format!("jq: error: {e}\n")),
                };

                let inputs: Vec<serde_json::Value> = if null_input {
                    vec![serde_json::Value::Null]
                } else {
                    let sources = match self.read_input_sources(&files, cwd, stdin, "jq") {
                        Ok(s) => s,
                        Err(e) => return StageExecBytesOutcome::err(2, e),
                    };
                    let mut vals = Vec::new();
                    for (_, b) in sources {
                        let text = match std::str::from_utf8(&b) {
                            Ok(t) => t,
                            Err(_) => {
                                return StageExecBytesOutcome::err(
                                    5,
                                    "jq: parse error: Invalid UTF-8 in JSON input\n".to_string(),
                                )
                            }
                        };
                        let stream = serde_json::Deserializer::from_str(text)
                            .into_iter::<serde_json::Value>();
                        for item in stream {
                            match item {
                                Ok(v) => vals.push(v),
                                Err(e) => {
                                    return StageExecBytesOutcome::err(
                                        5,
                                        format!("jq: parse error: {e}\n"),
                                    )
                                }
                            }
                        }
                    }
                    if slurp {
                        vec![serde_json::Value::Array(vals)]
                    } else {
                        vals
                    }
                };

                let mut out = String::new();
                let mut last_truthy = false;
                let mut any_output = false;
                for inp in &inputs {
                    let results = match eval_jq_filter(&compiled, inp, &env_args) {
                        Ok(r) => r,
                        Err(e) => {
                            return StageExecBytesOutcome::err(5, format!("jq: error: {e}\n"))
                        }
                    };
                    for val in results {
                        any_output = true;
                        last_truthy = !matches!(
                            val,
                            serde_json::Value::Null | serde_json::Value::Bool(false)
                        );
                        let rendered_val = if sort_keys {
                            sort_json_keys_recursive(&val)
                        } else {
                            val
                        };
                        if raw_output {
                            if let Some(s) = rendered_val.as_str() {
                                out.push_str(s);
                                out.push('\n');
                                continue;
                            }
                        }
                        let s = if compact {
                            serde_json::to_string(&rendered_val).unwrap_or_default()
                        } else {
                            serde_json::to_string_pretty(&rendered_val).unwrap_or_default()
                        };
                        out.push_str(&s);
                        out.push('\n');
                    }
                }
                let exit_code = if exit_status && (!any_output || !last_truthy) {
                    1
                } else {
                    0
                };
                StageExecBytesOutcome {
                    exit_code,
                    stdout: out.into_bytes(),
                    stderr: String::new(),
                    instructions_retired: 0,
                    vsock_frames: Vec::new(),
                    compiled_elf_machine: None,
                }
            })()),
            _ => None,
        }
    }

    /// Formal R4.MvmLinuxContainer.
    fn read_input_sources(
        &self,
        files: &[String],
        cwd: &str,
        stdin: &[u8],
        cmd_name: &str,
    ) -> Result<Vec<(String, Vec<u8>)>, String> {
        if files.is_empty() {
            return Ok(vec![("-".to_string(), stdin.to_vec())]);
        }
        let mut out = Vec::with_capacity(files.len());
        for f in files {
            if f == "-" {
                out.push(("-".to_string(), stdin.to_vec()));
            } else {
                let norm = Self::normalize_guest_path(cwd, f);
                if self.dirs.contains(&norm) && !self.inodes.contains_key(&norm) {
                    return Err(format!("{cmd_name}: {f}: Is a directory\n"));
                }
                match self.read_guest_file(&norm) {
                    Some(b) => out.push((f.clone(), b)),
                    None => {
                        return Err(format!(
                            "{cmd_name}: cannot open '{f}' for reading: No such file or directory\n"
                        ))
                    }
                }
            }
        }
        Ok(out)
    }
}

pub(super) fn is_native_coreutil_or_builtin(name: &str) -> bool {
    matches!(
        name,
        "cd" | "export"
            | "unset"
            | "true"
            | "false"
            | "pwd"
            | "echo"
            | "printf"
            | "env"
            | "which"
            | "ps"
            | "uname"
            | "date"
            | "cat"
            | "ls"
            | "mkdir"
            | "rmdir"
            | "rm"
            | "head"
            | "tail"
            | "wc"
            | "tr"
            | "cut"
            | "sort"
            | "uniq"
            | "grep"
            | "rg"
            | "sed"
            | "awk"
            | "find"
            | "xargs"
            | "patch"
            | "make"
            | "sh"
            | "df"
            | "free"
            | "xxd"
            | "cp"
            | "mv"
            | "touch"
            | "chmod"
            | "ln"
            | "tar"
            | "gzip"
            | "gunzip"
            | "base64"
            | "diff"
            | "bc"
            | "jq"
            | "tee"
            | "dirname"
            | "basename"
            | "readlink"
            | "realpath"
            | "stat"
            | "seq"
            | "sha256sum"
            | "test"
            | "["
            | "cc"
            | "gcc"
            | "git"
            | "pkg"
            | "node"
            | "npm"
            | "npx"
            | "python"
            | "python3"
            | "pip"
            | "pip3"
            | "wasm"
            | "sqlite3"
    )
}

pub(super) fn parse_c_escape_at(bytes: &[u8], pos: usize) -> (Vec<u8>, usize) {
    if pos + 1 >= bytes.len() {
        return (vec![b'\\'], 1);
    }
    match bytes[pos + 1] {
        b'n' => (vec![b'\n'], 2),
        b't' => (vec![b'\t'], 2),
        b'r' => (vec![b'\r'], 2),
        b'\\' => (vec![b'\\'], 2),
        b'"' => (vec![b'"'], 2),
        b'\'' => (vec![b'\''], 2),
        b'a' => (vec![0x07], 2),
        b'b' => (vec![0x08], 2),
        b'f' => (vec![0x0c], 2),
        b'v' => (vec![0x0b], 2),
        b'x' => {
            let mut end = pos + 2;
            while end < bytes.len() && end < pos + 4 && bytes[end].is_ascii_hexdigit() {
                end += 1;
            }
            if end > pos + 2 {
                let hex = String::from_utf8_lossy(&bytes[pos + 2..end]);
                if let Ok(v) = u8::from_str_radix(&hex, 16) {
                    return (vec![v], end - pos);
                }
            }
            (vec![b'\\', b'x'], 2)
        }
        b'0'..=b'7' => {
            let mut end = pos + 1;
            while end < bytes.len() && end < pos + 4 && (b'0'..=b'7').contains(&bytes[end]) {
                end += 1;
            }
            let oct = String::from_utf8_lossy(&bytes[pos + 1..end]);
            let v = u8::from_str_radix(&oct, 8).unwrap_or(0);
            (vec![v], end - pos)
        }
        other => (vec![b'\\', other], 2),
    }
}

/// Formal R4.MvmLinuxContainer.
pub(super) fn pad_formatted_field(
    s: &str,
    width: usize,
    left_align: bool,
    pad_char: char,
) -> String {
    let len = s.chars().count();
    if len >= width {
        return s.to_string();
    }
    let pad: String = std::iter::repeat_n(pad_char, width - len).collect();
    if left_align {
        format!("{s}{pad}")
    } else {
        format!("{pad}{s}")
    }
}

pub(super) fn parse_printf_i64(s: &str) -> i64 {
    let t = s.trim();
    if let Some(hex) = t.strip_prefix("0x").or_else(|| t.strip_prefix("0X")) {
        i64::from_str_radix(hex, 16).unwrap_or(0)
    } else {
        t.parse::<i64>().unwrap_or(0)
    }
}

pub(super) fn parse_leading_i64(s: &str) -> i64 {
    let t = s.trim_start();
    let mut end = 0usize;
    for (idx, ch) in t.char_indices() {
        if (idx == 0 && (ch == '-' || ch == '+')) || ch.is_ascii_digit() {
            end = idx + ch.len_utf8();
        } else {
            break;
        }
    }
    t[..end].parse::<i64>().unwrap_or(0)
}

#[derive(Debug, Clone)]
pub(super) enum JqFilter {
    Identity,
    Var(String),
    Literal(serde_json::Value),
    Field(Box<JqFilter>, String, bool),
    Index(Box<JqFilter>, usize, bool),
    Slice(Box<JqFilter>, Option<usize>, Option<usize>, bool),
    Iter(Box<JqFilter>, bool),
    Pipe(Box<JqFilter>, Box<JqFilter>),
    Comma(Box<JqFilter>, Box<JqFilter>),
    Array(Box<JqFilter>),
    Object(Vec<(String, JqFilter)>),
    Builtin(String),
    Has(Box<JqFilter>),
    Select(Box<JqFilter>),
    Map(Box<JqFilter>),
    Cmp(String, Box<JqFilter>, Box<JqFilter>),
    And(Box<JqFilter>, Box<JqFilter>),
    Or(Box<JqFilter>, Box<JqFilter>),
}

pub(super) fn parse_jq_filter(raw: &str) -> Result<JqFilter, String> {
    struct JqScan;
    impl JqScan {
        fn matching_paren_end(s: &str, open_idx: usize) -> Option<usize> {
            let bytes = s.as_bytes();
            let mut depth = 0i32;
            let mut in_str = false;
            let mut i = open_idx;
            while i < bytes.len() {
                let b = bytes[i];
                if in_str {
                    if b == b'\\' {
                        i += 2;
                        continue;
                    }
                    if b == 0x22 {
                        in_str = false;
                    }
                } else {
                    match b {
                        0x22 => in_str = true,
                        b'(' | b'[' | b'{' => depth += 1,
                        b')' | b']' | b'}' => {
                            depth -= 1;
                            if depth == 0 {
                                return Some(i);
                            }
                        }
                        _ => {}
                    }
                }
                i += 1;
            }
            None
        }

        fn split_top(s: &str, delim: char) -> Vec<String> {
            let mut out = Vec::new();
            let mut depth = 0i32;
            let mut in_str = false;
            let mut start = 0usize;
            let bytes = s.as_bytes();
            let mut i = 0usize;
            while i < bytes.len() {
                let b = bytes[i];
                if in_str {
                    if b == b'\\' {
                        i += 2;
                        continue;
                    }
                    if b == 0x22 {
                        in_str = false;
                    }
                } else {
                    match b {
                        0x22 => in_str = true,
                        b'(' | b'[' | b'{' => depth += 1,
                        b')' | b']' | b'}' => depth -= 1,
                        _ if depth == 0 && b == delim as u8 => {
                            out.push(s[start..i].trim().to_string());
                            start = i + 1;
                        }
                        _ => {}
                    }
                }
                i += 1;
            }
            out.push(s[start..].trim().to_string());
            out
        }

        fn split_top_colon(s: &str) -> Option<(&str, &str)> {
            let bytes = s.as_bytes();
            let mut depth = 0i32;
            let mut in_str = false;
            let mut i = 0usize;
            while i < bytes.len() {
                let b = bytes[i];
                if in_str {
                    if b == b'\\' {
                        i += 2;
                        continue;
                    }
                    if b == 0x22 {
                        in_str = false;
                    }
                } else {
                    match b {
                        0x22 => in_str = true,
                        b'(' | b'[' | b'{' => depth += 1,
                        b')' | b']' | b'}' => depth -= 1,
                        b':' if depth == 0 => return Some((&s[..i], &s[i + 1..])),
                        _ => {}
                    }
                }
                i += 1;
            }
            None
        }

        fn split_word(s: &str, kw: &str) -> Option<(String, String)> {
            let bytes = s.as_bytes();
            let mut depth = 0i32;
            let mut in_str = false;
            let mut i = 0usize;
            while i < bytes.len() {
                let b = bytes[i];
                if in_str {
                    if b == b'\\' {
                        i += 2;
                        continue;
                    }
                    if b == 0x22 {
                        in_str = false;
                    }
                } else {
                    match b {
                        0x22 => in_str = true,
                        b'(' | b'[' | b'{' => depth += 1,
                        b')' | b']' | b'}' => depth -= 1,
                        _ if depth == 0
                            && s.is_char_boundary(i)
                            && s.as_bytes()[i..].starts_with(kw.as_bytes())
                            && s.is_char_boundary(i + kw.len()) =>
                        {
                            return Some((
                                s[..i].trim().to_string(),
                                s[i + kw.len()..].trim().to_string(),
                            ));
                        }
                        _ => {}
                    }
                }
                i += 1;
            }
            None
        }
    }

    let s = raw.trim();
    if s.is_empty() {
        return Ok(JqFilter::Identity);
    }
    let parts = JqScan::split_top(s, '|');
    if parts.len() > 1 {
        let mut cur = parse_jq_filter(&parts[0])?;
        for p in &parts[1..] {
            cur = JqFilter::Pipe(Box::new(cur), Box::new(parse_jq_filter(p)?));
        }
        return Ok(cur);
    }
    let commas = JqScan::split_top(s, ',');
    if commas.len() > 1 {
        let mut cur = parse_jq_filter(&commas[0])?;
        for p in &commas[1..] {
            cur = JqFilter::Comma(Box::new(cur), Box::new(parse_jq_filter(p)?));
        }
        return Ok(cur);
    }
    for kw in [" or ", " and "] {
        if let Some((l, r)) = JqScan::split_word(s, kw) {
            let lf = Box::new(parse_jq_filter(&l)?);
            let rf = Box::new(parse_jq_filter(&r)?);
            return Ok(if kw == " or " {
                JqFilter::Or(lf, rf)
            } else {
                JqFilter::And(lf, rf)
            });
        }
    }
    for op in ["==", "!=", "<=", ">=", "<", ">"] {
        if let Some((l, r)) = JqScan::split_word(s, op) {
            return Ok(JqFilter::Cmp(
                op.to_string(),
                Box::new(parse_jq_filter(&l)?),
                Box::new(parse_jq_filter(&r)?),
            ));
        }
    }
    if s.starts_with('(')
        && s.ends_with(')')
        && JqScan::matching_paren_end(s, 0) == Some(s.len() - 1)
    {
        return parse_jq_filter(&s[1..s.len() - 1]);
    }
    if s.starts_with('[')
        && s.ends_with(']')
        && JqScan::matching_paren_end(s, 0) == Some(s.len() - 1)
    {
        return Ok(JqFilter::Array(Box::new(parse_jq_filter(
            &s[1..s.len() - 1],
        )?)));
    }
    if s.starts_with('{')
        && s.ends_with('}')
        && JqScan::matching_paren_end(s, 0) == Some(s.len() - 1)
    {
        let inner = s[1..s.len() - 1].trim();
        if inner.is_empty() {
            return Ok(JqFilter::Object(Vec::new()));
        }
        let mut entries = Vec::new();
        for item in JqScan::split_top(inner, ',') {
            if let Some((k, v)) = JqScan::split_top_colon(&item) {
                let key = k.trim().trim_matches('\x22').to_string();
                entries.push((key, parse_jq_filter(v.trim())?));
            } else {
                let key = item.trim().trim_matches('\x22').to_string();
                entries.push((
                    key.clone(),
                    JqFilter::Field(Box::new(JqFilter::Identity), key, false),
                ));
            }
        }
        return Ok(JqFilter::Object(entries));
    }
    for (prefix, wrap) in [
        ("select(", JqFilter::Select as fn(Box<JqFilter>) -> JqFilter),
        ("map(", JqFilter::Map as fn(Box<JqFilter>) -> JqFilter),
        ("has(", JqFilter::Has as fn(Box<JqFilter>) -> JqFilter),
    ] {
        if s.starts_with(prefix)
            && s.ends_with(')')
            && JqScan::matching_paren_end(s, prefix.len() - 1) == Some(s.len() - 1)
        {
            return Ok(wrap(Box::new(parse_jq_filter(
                &s[prefix.len()..s.len() - 1],
            )?)));
        }
    }
    if let Some(vname) = s.strip_prefix('$') {
        return Ok(JqFilter::Var(vname.to_string()));
    }
    if matches!(
        s,
        "keys"
            | "keys_unsorted"
            | "length"
            | "type"
            | "values"
            | "empty"
            | "not"
            | "add"
            | "sort"
            | "reverse"
            | "unique"
            | "first"
            | "last"
            | "tonumber"
            | "tostring"
    ) {
        return Ok(JqFilter::Builtin(s.to_string()));
    }
    if let Ok(lit) = serde_json::from_str::<serde_json::Value>(s) {
        return Ok(JqFilter::Literal(lit));
    }
    if !s.starts_with('.') {
        return Err(format!("invalid jq filter `{s}`"));
    }

    // Parse `.a.b[0].c[]?` path chain
    let mut cur = JqFilter::Identity;
    let bytes = s.as_bytes();
    let mut i = 0usize;
    while i < bytes.len() {
        if bytes[i] == b'.' {
            i += 1;
            if i == bytes.len() {
                break;
            }
            if bytes[i] == b'[' {
                continue;
            }
            if bytes[i] == 0x22 {
                i += 1;
                let st = i;
                while i < bytes.len() && bytes[i] != 0x22 {
                    i += 1;
                }
                let key = s[st..i].to_string();
                if i < bytes.len() {
                    i += 1;
                }
                let opt = if i < bytes.len() && bytes[i] == b'?' {
                    i += 1;
                    true
                } else {
                    false
                };
                cur = JqFilter::Field(Box::new(cur), key, opt);
            } else {
                let st = i;
                while i < bytes.len()
                    && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_' || bytes[i] == b'-')
                {
                    i += 1;
                }
                if i == st {
                    return Err(format!("invalid jq field path `{s}`"));
                }
                let key = s[st..i].to_string();
                let opt = if i < bytes.len() && bytes[i] == b'?' {
                    i += 1;
                    true
                } else {
                    false
                };
                cur = JqFilter::Field(Box::new(cur), key, opt);
            }
        } else if bytes[i] == b'[' {
            let Some(close) = JqScan::matching_paren_end(s, i) else {
                return Err(format!("unclosed `[` in `{s}`"));
            };
            let inside = s[i + 1..close].trim();
            i = close + 1;
            let opt = if i < bytes.len() && bytes[i] == b'?' {
                i += 1;
                true
            } else {
                false
            };
            if inside.is_empty() {
                cur = JqFilter::Iter(Box::new(cur), opt);
            } else if inside.starts_with('\x22') && inside.ends_with('\x22') && inside.len() >= 2 {
                let key = inside[1..inside.len() - 1].to_string();
                cur = JqFilter::Field(Box::new(cur), key, opt);
            } else if let Some((a, b)) = inside.split_once(':') {
                let st_i = (!a.trim().is_empty())
                    .then(|| a.trim().parse::<usize>())
                    .transpose()
                    .map_err(|_| format!("invalid slice start in `{s}`"))?;
                let en_i = (!b.trim().is_empty())
                    .then(|| b.trim().parse::<usize>())
                    .transpose()
                    .map_err(|_| format!("invalid slice end in `{s}`"))?;
                cur = JqFilter::Slice(Box::new(cur), st_i, en_i, opt);
            } else if let Ok(idx) = inside.parse::<usize>() {
                cur = JqFilter::Index(Box::new(cur), idx, opt);
            } else {
                return Err(format!("invalid jq bracket expression `[{inside}]`"));
            }
        } else {
            return Err(format!("unexpected character in jq filter `{s}`"));
        }
    }
    Ok(cur)
}

pub(super) fn eval_jq_filter(
    f: &JqFilter,
    val: &serde_json::Value,
    vars: &BTreeMap<String, serde_json::Value>,
) -> Result<Vec<serde_json::Value>, String> {
    match f {
        JqFilter::Identity => Ok(vec![val.clone()]),
        JqFilter::Var(name) => Ok(vec![vars
            .get(name)
            .cloned()
            .unwrap_or(serde_json::Value::Null)]),
        JqFilter::Literal(v) => Ok(vec![v.clone()]),
        JqFilter::Field(base, key, opt) => {
            let bases = eval_jq_filter(base, val, vars)?;
            let mut out = Vec::new();
            for b in bases {
                match b {
                    serde_json::Value::Object(map) => {
                        out.push(map.get(key).cloned().unwrap_or(serde_json::Value::Null));
                    }
                    serde_json::Value::Null => out.push(serde_json::Value::Null),
                    _ if *opt => {}
                    _ => return Err(format!("Cannot index non-object with string \"{key}\"")),
                }
            }
            Ok(out)
        }
        JqFilter::Index(base, idx, opt) => {
            let bases = eval_jq_filter(base, val, vars)?;
            let mut out = Vec::new();
            for b in bases {
                match b {
                    serde_json::Value::Array(arr) => {
                        out.push(arr.get(*idx).cloned().unwrap_or(serde_json::Value::Null));
                    }
                    serde_json::Value::Null => out.push(serde_json::Value::Null),
                    _ if *opt => {}
                    _ => return Err(format!("Cannot index non-array with number {idx}")),
                }
            }
            Ok(out)
        }
        JqFilter::Slice(base, st, en, opt) => {
            let bases = eval_jq_filter(base, val, vars)?;
            let mut out = Vec::new();
            for b in bases {
                match b {
                    serde_json::Value::Array(arr) => {
                        let lo = st.unwrap_or(0).min(arr.len());
                        let hi = en.unwrap_or(arr.len()).min(arr.len()).max(lo);
                        out.push(serde_json::Value::Array(arr[lo..hi].to_vec()));
                    }
                    serde_json::Value::String(s) => {
                        let chars: Vec<char> = s.chars().collect();
                        let lo = st.unwrap_or(0).min(chars.len());
                        let hi = en.unwrap_or(chars.len()).min(chars.len()).max(lo);
                        out.push(serde_json::Value::String(chars[lo..hi].iter().collect()));
                    }
                    _ if *opt => {}
                    _ => return Err("Cannot slice non-array/string".to_string()),
                }
            }
            Ok(out)
        }
        JqFilter::Iter(base, opt) => {
            let bases = eval_jq_filter(base, val, vars)?;
            let mut out = Vec::new();
            for b in bases {
                match b {
                    serde_json::Value::Array(arr) => out.extend(arr),
                    serde_json::Value::Object(map) => {
                        for (_, v) in map {
                            out.push(v);
                        }
                    }
                    _ if *opt => {}
                    _ => return Err("Cannot iterate over scalar".to_string()),
                }
            }
            Ok(out)
        }
        JqFilter::Pipe(l, r) => {
            let mids = eval_jq_filter(l, val, vars)?;
            let mut out = Vec::new();
            for m in mids {
                out.extend(eval_jq_filter(r, &m, vars)?);
            }
            Ok(out)
        }
        JqFilter::Comma(l, r) => {
            let mut out = eval_jq_filter(l, val, vars)?;
            out.extend(eval_jq_filter(r, val, vars)?);
            Ok(out)
        }
        JqFilter::Array(inner) => {
            let vals = eval_jq_filter(inner, val, vars)?;
            Ok(vec![serde_json::Value::Array(vals)])
        }
        JqFilter::Object(entries) => {
            let mut map = serde_json::Map::new();
            for (k, vf) in entries {
                let vals = eval_jq_filter(vf, val, vars)?;
                if let Some(first) = vals.into_iter().next() {
                    map.insert(k.clone(), first);
                }
            }
            Ok(vec![serde_json::Value::Object(map)])
        }
        JqFilter::Has(kf) => {
            let kvals = eval_jq_filter(kf, val, vars)?;
            let mut out = Vec::new();
            for kv in kvals {
                let res = match (val, &kv) {
                    (serde_json::Value::Object(m), serde_json::Value::String(s)) => {
                        m.contains_key(s)
                    }
                    (serde_json::Value::Array(a), serde_json::Value::Number(n)) => {
                        n.as_u64().is_some_and(|i| (i as usize) < a.len())
                    }
                    _ => false,
                };
                out.push(serde_json::Value::Bool(res));
            }
            Ok(out)
        }
        JqFilter::Select(cond) => {
            let cv = eval_jq_filter(cond, val, vars)?;
            if cv.first().is_some_and(|v| {
                !matches!(v, serde_json::Value::Null | serde_json::Value::Bool(false))
            }) {
                Ok(vec![val.clone()])
            } else {
                Ok(Vec::new())
            }
        }
        JqFilter::Map(inner) => match val {
            serde_json::Value::Array(arr) => {
                let mut res = Vec::new();
                for item in arr {
                    res.extend(eval_jq_filter(inner, item, vars)?);
                }
                Ok(vec![serde_json::Value::Array(res)])
            }
            _ => Err("Cannot map over non-array".to_string()),
        },
        JqFilter::Cmp(op, l, r) => {
            let lv = eval_jq_filter(l, val, vars)?
                .into_iter()
                .next()
                .unwrap_or(serde_json::Value::Null);
            let rv = eval_jq_filter(r, val, vars)?
                .into_iter()
                .next()
                .unwrap_or(serde_json::Value::Null);
            let ok = match op.as_str() {
                "==" => lv == rv,
                "!=" => lv != rv,
                "<" => jq_cmp_ord(&lv, &rv) == Some(std::cmp::Ordering::Less),
                "<=" => matches!(
                    jq_cmp_ord(&lv, &rv),
                    Some(std::cmp::Ordering::Less | std::cmp::Ordering::Equal)
                ),
                ">" => jq_cmp_ord(&lv, &rv) == Some(std::cmp::Ordering::Greater),
                ">=" => matches!(
                    jq_cmp_ord(&lv, &rv),
                    Some(std::cmp::Ordering::Greater | std::cmp::Ordering::Equal)
                ),
                _ => false,
            };
            Ok(vec![serde_json::Value::Bool(ok)])
        }
        JqFilter::And(l, r) => {
            let lv = eval_jq_filter(l, val, vars)?;
            let lt = lv.first().is_some_and(|v| {
                !matches!(v, serde_json::Value::Null | serde_json::Value::Bool(false))
            });
            if !lt {
                return Ok(vec![serde_json::Value::Bool(false)]);
            }
            let rv = eval_jq_filter(r, val, vars)?;
            let rt = rv.first().is_some_and(|v| {
                !matches!(v, serde_json::Value::Null | serde_json::Value::Bool(false))
            });
            Ok(vec![serde_json::Value::Bool(rt)])
        }
        JqFilter::Or(l, r) => {
            let lv = eval_jq_filter(l, val, vars)?;
            let lt = lv.first().is_some_and(|v| {
                !matches!(v, serde_json::Value::Null | serde_json::Value::Bool(false))
            });
            if lt {
                return Ok(vec![serde_json::Value::Bool(true)]);
            }
            let rv = eval_jq_filter(r, val, vars)?;
            let rt = rv.first().is_some_and(|v| {
                !matches!(v, serde_json::Value::Null | serde_json::Value::Bool(false))
            });
            Ok(vec![serde_json::Value::Bool(rt)])
        }
        JqFilter::Builtin(b) => match b.as_str() {
            "empty" => Ok(Vec::new()),
            "not" => Ok(vec![serde_json::Value::Bool(matches!(
                val,
                serde_json::Value::Null | serde_json::Value::Bool(false)
            ))]),
            "type" => Ok(vec![serde_json::Value::String(
                match val {
                    serde_json::Value::Null => "null",
                    serde_json::Value::Bool(_) => "boolean",
                    serde_json::Value::Number(_) => "number",
                    serde_json::Value::String(_) => "string",
                    serde_json::Value::Array(_) => "array",
                    serde_json::Value::Object(_) => "object",
                }
                .to_string(),
            )]),
            "length" => {
                let len = match val {
                    serde_json::Value::Null => 0,
                    serde_json::Value::String(s) => s.chars().count(),
                    serde_json::Value::Array(a) => a.len(),
                    serde_json::Value::Object(m) => m.len(),
                    _ => return Err("length undefined for number/bool".to_string()),
                };
                Ok(vec![serde_json::Value::Number((len as u64).into())])
            }
            "keys" | "keys_unsorted" => match val {
                serde_json::Value::Object(m) => {
                    let mut ks: Vec<String> = m.keys().cloned().collect();
                    if b == "keys" {
                        ks.sort();
                    }
                    Ok(vec![serde_json::Value::Array(
                        ks.into_iter().map(serde_json::Value::String).collect(),
                    )])
                }
                serde_json::Value::Array(a) => Ok(vec![serde_json::Value::Array(
                    (0..a.len())
                        .map(|i| serde_json::Value::Number((i as u64).into()))
                        .collect(),
                )]),
                _ => Err("keys requires object or array".to_string()),
            },
            "values" => match val {
                serde_json::Value::Null => Ok(Vec::new()),
                other => Ok(vec![other.clone()]),
            },
            "sort" => match val {
                serde_json::Value::Array(a) => {
                    let mut sorted = a.clone();
                    sorted.sort_by(|x, y| jq_cmp_ord(x, y).unwrap_or(std::cmp::Ordering::Equal));
                    Ok(vec![serde_json::Value::Array(sorted)])
                }
                _ => Err("sort requires array".to_string()),
            },
            "reverse" => match val {
                serde_json::Value::Array(a) => {
                    let mut rev = a.clone();
                    rev.reverse();
                    Ok(vec![serde_json::Value::Array(rev)])
                }
                _ => Err("reverse requires array".to_string()),
            },
            "unique" => match val {
                serde_json::Value::Array(a) => {
                    let mut out_a = Vec::new();
                    for item in a {
                        if !out_a.contains(item) {
                            out_a.push(item.clone());
                        }
                    }
                    Ok(vec![serde_json::Value::Array(out_a)])
                }
                _ => Err("unique requires array".to_string()),
            },
            "first" => match val {
                serde_json::Value::Array(a) => {
                    Ok(vec![a.first().cloned().unwrap_or(serde_json::Value::Null)])
                }
                _ => Err("first requires array".to_string()),
            },
            "last" => match val {
                serde_json::Value::Array(a) => {
                    Ok(vec![a.last().cloned().unwrap_or(serde_json::Value::Null)])
                }
                _ => Err("last requires array".to_string()),
            },
            "add" => match val {
                serde_json::Value::Array(a) => {
                    if a.is_empty() {
                        return Ok(vec![serde_json::Value::Null]);
                    }
                    if a.iter().all(|v| v.is_i64()) {
                        let sum: i64 = a.iter().filter_map(|v| v.as_i64()).sum();
                        return Ok(vec![serde_json::Value::Number(sum.into())]);
                    }
                    if a.iter().all(|v| v.is_string()) {
                        let joined: String = a.iter().filter_map(|v| v.as_str()).collect();
                        return Ok(vec![serde_json::Value::String(joined)]);
                    }
                    Err("unsupported types for add".to_string())
                }
                _ => Err("add requires array".to_string()),
            },
            "tonumber" => match val {
                serde_json::Value::Number(_) => Ok(vec![val.clone()]),
                serde_json::Value::String(s) => s
                    .trim()
                    .parse::<i64>()
                    .map(|n| vec![serde_json::Value::Number(n.into())])
                    .map_err(|_| format!("cannot parse `{s}` as number")),
                _ => Err("cannot convert to number".to_string()),
            },
            "tostring" => Ok(vec![serde_json::Value::String(match val {
                serde_json::Value::String(s) => s.clone(),
                other => serde_json::to_string(other).unwrap_or_default(),
            })]),
            _ => Err(format!("unsupported jq builtin `{b}`")),
        },
    }
}

pub(super) fn jq_cmp_ord(
    a: &serde_json::Value,
    b: &serde_json::Value,
) -> Option<std::cmp::Ordering> {
    match (a, b) {
        (serde_json::Value::Number(x), serde_json::Value::Number(y)) => {
            Some(x.as_i64()?.cmp(&y.as_i64()?))
        }
        (serde_json::Value::String(x), serde_json::Value::String(y)) => Some(x.cmp(y)),
        (serde_json::Value::Bool(x), serde_json::Value::Bool(y)) => Some(x.cmp(y)),
        _ => None,
    }
}

pub(super) fn sort_json_keys_recursive(v: &serde_json::Value) -> serde_json::Value {
    match v {
        serde_json::Value::Array(a) => {
            serde_json::Value::Array(a.iter().map(sort_json_keys_recursive).collect())
        }
        serde_json::Value::Object(m) => {
            let mut btree: BTreeMap<String, serde_json::Value> = BTreeMap::new();
            for (k, val) in m {
                btree.insert(k.clone(), sort_json_keys_recursive(val));
            }
            let mut out = serde_json::Map::new();
            for (k, val) in btree {
                out.insert(k, val);
            }
            serde_json::Value::Object(out)
        }
        other => other.clone(),
    }
}

/// Formal R4.MvmLinuxContainer.
/// Expand a 16-bit RV64C compressed instruction into its exact 32-bit RV64I equivalent.
pub(super) fn expand_rv64c(c: u16) -> Result<u32, MvmError> {
    let c32 = c as u32;
    let op = c32 & 0b11;
    let funct3 = (c32 >> 13) & 0x7;
    let r_prime = |bits: u32| 8 + (bits & 0x7);

    let sign_ext = |val: u32, bits: u32| -> i32 {
        let shift = 32 - bits;
        ((val << shift) as i32) >> shift
    };
    let enc_i = |imm: i32, rs1: u32, f3: u32, rd: u32, opc: u32| -> u32 {
        (((imm as u32) & 0xfff) << 20) | (rs1 << 15) | (f3 << 12) | (rd << 7) | opc
    };
    let enc_s = |imm: i32, rs2: u32, rs1: u32, f3: u32, opc: u32| -> u32 {
        let u = (imm as u32) & 0xfff;
        ((u >> 5) << 25) | (rs2 << 20) | (rs1 << 15) | (f3 << 12) | ((u & 0x1f) << 7) | opc
    };
    let enc_r = |f7: u32, rs2: u32, rs1: u32, f3: u32, rd: u32, opc: u32| -> u32 {
        (f7 << 25) | (rs2 << 20) | (rs1 << 15) | (f3 << 12) | (rd << 7) | opc
    };
    let enc_b = |off: i32, rs2: u32, rs1: u32, f3: u32| -> u32 {
        let u = off as u32;
        (((u >> 12) & 1) << 31)
            | (((u >> 5) & 0x3f) << 25)
            | (rs2 << 20)
            | (rs1 << 15)
            | (f3 << 12)
            | (((u >> 1) & 0xf) << 8)
            | (((u >> 11) & 1) << 7)
            | 0x63
    };
    let enc_j = |off: i32, rd: u32| -> u32 {
        let u = off as u32;
        (((u >> 20) & 1) << 31)
            | (((u >> 1) & 0x3ff) << 21)
            | (((u >> 11) & 1) << 20)
            | (((u >> 12) & 0xff) << 12)
            | (rd << 7)
            | 0x6f
    };

    match (op, funct3) {
        (0b00, 0b000) => {
            // C.ADDI4SPN
            let rd = r_prime(c32 >> 2);
            let imm = (((c32 >> 5) & 1) << 3)
                | (((c32 >> 6) & 1) << 2)
                | (((c32 >> 7) & 0xf) << 6)
                | (((c32 >> 11) & 0x3) << 4);
            if imm == 0 {
                return Err(MvmError::CpuTrap(
                    "illegal C instruction 0x0000".to_string(),
                ));
            }
            Ok(enc_i(imm as i32, 2, 0, rd, 0x13))
        }
        (0b00, 0b010) => {
            // C.LW
            let rd = r_prime(c32 >> 2);
            let rs1 = r_prime(c32 >> 7);
            let imm =
                (((c32 >> 5) & 1) << 6) | (((c32 >> 6) & 1) << 2) | (((c32 >> 10) & 0x7) << 3);
            Ok(enc_i(imm as i32, rs1, 2, rd, 0x03))
        }
        (0b00, 0b011) => {
            // C.LD
            let rd = r_prime(c32 >> 2);
            let rs1 = r_prime(c32 >> 7);
            let imm = (((c32 >> 5) & 0x3) << 6) | (((c32 >> 10) & 0x7) << 3);
            Ok(enc_i(imm as i32, rs1, 3, rd, 0x03))
        }
        (0b00, 0b110) => {
            // C.SW
            let rs2 = r_prime(c32 >> 2);
            let rs1 = r_prime(c32 >> 7);
            let imm =
                (((c32 >> 5) & 1) << 6) | (((c32 >> 6) & 1) << 2) | (((c32 >> 10) & 0x7) << 3);
            Ok(enc_s(imm as i32, rs2, rs1, 2, 0x23))
        }
        (0b00, 0b111) => {
            // C.SD
            let rs2 = r_prime(c32 >> 2);
            let rs1 = r_prime(c32 >> 7);
            let imm = (((c32 >> 5) & 0x3) << 6) | (((c32 >> 10) & 0x7) << 3);
            Ok(enc_s(imm as i32, rs2, rs1, 3, 0x23))
        }
        (0b01, 0b000) => {
            // C.NOP / C.ADDI
            let rd = (c32 >> 7) & 0x1f;
            let raw = (((c32 >> 12) & 1) << 5) | ((c32 >> 2) & 0x1f);
            let imm = sign_ext(raw, 6);
            Ok(enc_i(imm, rd, 0, rd, 0x13))
        }
        (0b01, 0b001) => {
            // C.ADDIW
            let rd = (c32 >> 7) & 0x1f;
            let raw = (((c32 >> 12) & 1) << 5) | ((c32 >> 2) & 0x1f);
            let imm = sign_ext(raw, 6);
            Ok(enc_i(imm, rd, 0, rd, 0x1b))
        }
        (0b01, 0b010) => {
            // C.LI
            let rd = (c32 >> 7) & 0x1f;
            let raw = (((c32 >> 12) & 1) << 5) | ((c32 >> 2) & 0x1f);
            let imm = sign_ext(raw, 6);
            Ok(enc_i(imm, 0, 0, rd, 0x13))
        }
        (0b01, 0b011) => {
            // C.ADDI16SP / C.LUI
            let rd = (c32 >> 7) & 0x1f;
            if rd == 2 {
                let raw = (((c32 >> 12) & 1) << 9)
                    | (((c32 >> 3) & 0x3) << 7)
                    | (((c32 >> 5) & 1) << 6)
                    | (((c32 >> 2) & 1) << 5)
                    | (((c32 >> 6) & 1) << 4);
                let imm = sign_ext(raw, 10);
                Ok(enc_i(imm, 2, 0, 2, 0x13))
            } else {
                let raw = (((c32 >> 12) & 1) << 5) | ((c32 >> 2) & 0x1f);
                let imm = sign_ext(raw, 6);
                Ok((((imm as u32) & 0xfffff) << 12) | (rd << 7) | 0x37)
            }
        }
        (0b01, 0b100) => {
            let rd = r_prime(c32 >> 7);
            let sub_op = (c32 >> 10) & 0x3;
            let shamt = (((c32 >> 12) & 1) << 5) | ((c32 >> 2) & 0x1f);
            match sub_op {
                0 => Ok(enc_i(shamt as i32, rd, 5, rd, 0x13)), // C.SRLI
                1 => Ok(enc_i((shamt | (1 << 10)) as i32, rd, 5, rd, 0x13)), // C.SRAI
                2 => {
                    let imm = sign_ext(shamt, 6);
                    Ok(enc_i(imm, rd, 7, rd, 0x13)) // C.ANDI
                }
                3 => {
                    let rs2 = r_prime(c32 >> 2);
                    let f = (((c32 >> 12) & 1) << 2) | ((c32 >> 5) & 0x3);
                    match f {
                        0b000 => Ok(enc_r(0x20, rs2, rd, 0, rd, 0x33)), // C.SUB
                        0b001 => Ok(enc_r(0x00, rs2, rd, 4, rd, 0x33)), // C.XOR
                        0b010 => Ok(enc_r(0x00, rs2, rd, 6, rd, 0x33)), // C.OR
                        0b011 => Ok(enc_r(0x00, rs2, rd, 7, rd, 0x33)), // C.AND
                        0b100 => Ok(enc_r(0x20, rs2, rd, 0, rd, 0x3b)), // C.SUBW
                        0b101 => Ok(enc_r(0x00, rs2, rd, 0, rd, 0x3b)), // C.ADDW
                        _ => Err(MvmError::CpuTrap("bad C.ALU".to_string())),
                    }
                }
                _ => unreachable!(),
            }
        }
        (0b01, 0b101) => {
            // C.J
            let raw = (((c32 >> 12) & 1) << 11)
                | (((c32 >> 8) & 1) << 10)
                | (((c32 >> 9) & 0x3) << 8)
                | (((c32 >> 6) & 1) << 7)
                | (((c32 >> 7) & 1) << 6)
                | (((c32 >> 2) & 1) << 5)
                | (((c32 >> 11) & 1) << 4)
                | (((c32 >> 3) & 0x7) << 1);
            Ok(enc_j(sign_ext(raw, 12), 0))
        }
        (0b01, 0b110) | (0b01, 0b111) => {
            // C.BEQZ / C.BNEZ
            let rs1 = r_prime(c32 >> 7);
            let raw = (((c32 >> 12) & 1) << 8)
                | (((c32 >> 5) & 0x3) << 6)
                | (((c32 >> 2) & 1) << 5)
                | (((c32 >> 10) & 0x3) << 3)
                | (((c32 >> 3) & 0x3) << 1);
            let f3 = if funct3 == 0b110 { 0 } else { 1 };
            Ok(enc_b(sign_ext(raw, 9), 0, rs1, f3))
        }
        (0b10, 0b000) => {
            // C.SLLI
            let rd = (c32 >> 7) & 0x1f;
            let shamt = (((c32 >> 12) & 1) << 5) | ((c32 >> 2) & 0x1f);
            Ok(enc_i(shamt as i32, rd, 1, rd, 0x13))
        }
        (0b10, 0b010) => {
            // C.LWSP
            let rd = (c32 >> 7) & 0x1f;
            let imm =
                (((c32 >> 2) & 0x3) << 6) | (((c32 >> 12) & 1) << 5) | (((c32 >> 4) & 0x7) << 2);
            Ok(enc_i(imm as i32, 2, 2, rd, 0x03))
        }
        (0b10, 0b011) => {
            // C.LDSP
            let rd = (c32 >> 7) & 0x1f;
            let imm =
                (((c32 >> 2) & 0x7) << 6) | (((c32 >> 12) & 1) << 5) | (((c32 >> 5) & 0x3) << 3);
            Ok(enc_i(imm as i32, 2, 3, rd, 0x03))
        }
        (0b10, 0b100) => {
            let bit12 = (c32 >> 12) & 1;
            let rd = (c32 >> 7) & 0x1f;
            let rs2 = (c32 >> 2) & 0x1f;
            match (bit12, rs2) {
                (0, 0) => Ok(enc_i(0, rd, 0, 0, 0x67)),      // C.JR
                (0, _) => Ok(enc_r(0, rs2, 0, 0, rd, 0x33)), // C.MV
                (1, 0) => {
                    if rd == 0 {
                        Ok(0x0010_0073) // C.EBREAK
                    } else {
                        Ok(enc_i(0, rd, 0, 1, 0x67)) // C.JALR
                    }
                }
                (1, _) => Ok(enc_r(0, rs2, rd, 0, rd, 0x33)), // C.ADD
                _ => unreachable!(),
            }
        }
        (0b10, 0b110) => {
            // C.SWSP
            let rs2 = (c32 >> 2) & 0x1f;
            let imm = (((c32 >> 7) & 0x3) << 6) | (((c32 >> 9) & 0xf) << 2);
            Ok(enc_s(imm as i32, rs2, 2, 2, 0x23))
        }
        (0b10, 0b111) => {
            // C.SDSP
            let rs2 = (c32 >> 2) & 0x1f;
            let imm = (((c32 >> 7) & 0x7) << 6) | (((c32 >> 10) & 0x7) << 3);
            Ok(enc_s(imm as i32, rs2, 2, 3, 0x23))
        }
        _ => Err(MvmError::CpuTrap(format!(
            "unsupported RV64C instruction 0x{c:04x}"
        ))),
    }
}
