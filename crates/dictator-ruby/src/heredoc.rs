//! Heredoc body detection, so line rules don't mistake string content for Ruby.
//!
//! A heredoc line such as `#{table} (` is SQL with interpolation, not a
//! comment. This is a line scanner, not a parser: it finds `<<ID`, `<<-ID`,
//! `<<~ID` (bare or quoted identifier) outside string literals and comments,
//! then marks lines up to the matching terminator.

/// Per-line flags indexed like `source.split('\n')`: `true` for heredoc body
/// and terminator lines.
///
/// An opener whose terminator never appears is dropped, so a stray `<<FOO`
/// cannot blank out the rest of the file.
#[must_use]
pub fn heredoc_lines(source: &str) -> Vec<bool> {
    let lines: Vec<&str> = source
        .split('\n')
        .map(|line| line.strip_suffix('\r').unwrap_or(line))
        .collect();
    let mut mask = vec![false; lines.len()];
    let mut idx = 0;

    while idx < lines.len() {
        let openers = find_openers(lines[idx]);
        idx += 1;
        // Stacked heredocs (`foo(<<~A, <<~B)`) have their bodies back to back.
        for opener in openers {
            let Some(len) = lines[idx..].iter().position(|l| opener.terminates(l)) else {
                continue;
            };
            mask[idx..=idx + len].fill(true);
            idx += len + 1;
        }
    }

    mask
}

struct Opener<'a> {
    ident: &'a str,
    /// `<<-` / `<<~` allow an indented terminator; bare `<<` needs column 0.
    indented: bool,
}

impl Opener<'_> {
    fn terminates(&self, line: &str) -> bool {
        // Ruby rejects trailing whitespace after the terminator.
        if self.indented {
            line.trim_start() == self.ident
        } else {
            line == self.ident
        }
    }
}

fn find_openers(line: &str) -> Vec<Opener<'_>> {
    let bytes = line.as_bytes();
    let mut openers = Vec::new();
    let mut i = 0;

    while i < bytes.len() {
        match bytes[i] {
            b'#' => break,
            quote @ (b'\'' | b'"' | b'`') => {
                // An unclosed quote runs past this line; nothing left to scan.
                let Some(close) = closing_quote(bytes, i + 1, quote) else {
                    break;
                };
                i = close + 1;
            }
            b'<' if bytes.get(i + 1) == Some(&b'<') => {
                if let Some((opener, end)) = parse_opener(line, i + 2) {
                    openers.push(opener);
                    i = end;
                } else {
                    i += 2;
                }
            }
            _ => i += 1,
        }
    }

    openers
}

/// Parse what follows `<<`; returns the opener and the index just past it.
fn parse_opener(line: &str, mut i: usize) -> Option<(Opener<'_>, usize)> {
    let bytes = line.as_bytes();
    let indented = matches!(bytes.get(i), Some(b'~' | b'-'));
    if indented {
        i += 1;
    }

    match *bytes.get(i)? {
        quote @ (b'\'' | b'"' | b'`') => {
            let close = closing_quote(bytes, i + 1, quote)?;
            let ident = &line[i + 1..close];
            (!ident.is_empty()).then_some((Opener { ident, indented }, close + 1))
        }
        first => {
            // A bare `<<` needs an upper-case identifier so `a<<b` stays a shift.
            let starts_ident = if indented {
                first.is_ascii_alphabetic() || first == b'_'
            } else {
                first.is_ascii_uppercase() || first == b'_'
            };
            if !starts_ident {
                return None;
            }
            let len = bytes[i..]
                .iter()
                .take_while(|b| b.is_ascii_alphanumeric() || **b == b'_')
                .count();
            Some((
                Opener {
                    ident: &line[i..i + len],
                    indented,
                },
                i + len,
            ))
        }
    }
}

fn closing_quote(bytes: &[u8], mut i: usize, quote: u8) -> Option<usize> {
    while i < bytes.len() {
        match bytes[i] {
            b'\\' => i += 2,
            b if b == quote => return Some(i),
            _ => i += 1,
        }
    }
    None
}

#[cfg(test)]
mod tests;
