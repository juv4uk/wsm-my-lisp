//! Minimal reader: text -> cons-structure, for one-shot console commands
//! like `(teleport player 100 200 50)`. Deliberately narrow for this
//! MVP slice (per the owner/my-lisp-cyberpunk go-ahead 2026-09-10):
//! symbols, fixnums (including negative), and parenthesized lists only.
//!
//! Explicitly NOT supported yet:
//!   - quote syntax (`'x`) -- not needed for one-shot commands
//!   - dotted-pair literals in input
//!   - string literals -- my-lisp confirmed (2026-09-10) strings are a
//!     distinct, immutable UTF-8 type, never the same as Symbol, but
//!     word.rs's wsm-os-target::Tag still has no String variant to encode
//!     one into. Still genuinely open (cross-repo ABI decision, not
//!     something to invent here) -- see eval.rs's module doc for the full
//!     status.

use crate::word::{encode_fixnum, SymbolTable, WORD_NIL};
use crate::wsm_cons;

#[derive(Debug, PartialEq)]
pub enum ReadError {
    UnexpectedEof,
    UnexpectedCloseParen,
    TrailingInput(String),
}

enum Token {
    Open,
    Close,
    Atom(String),
}

fn tokenize(source: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut chars = source.chars().peekable();
    while let Some(&c) = chars.peek() {
        match c {
            c if c.is_whitespace() => {
                chars.next();
            }
            '(' => {
                chars.next();
                tokens.push(Token::Open);
            }
            ')' => {
                chars.next();
                tokens.push(Token::Close);
            }
            _ => {
                let mut atom = String::new();
                while let Some(&c) = chars.peek() {
                    if c.is_whitespace() || c == '(' || c == ')' {
                        break;
                    }
                    atom.push(c);
                    chars.next();
                }
                tokens.push(Token::Atom(atom));
            }
        }
    }
    tokens
}

fn atom_to_word(atom: &str, symbols: &mut SymbolTable) -> u64 {
    if atom == "()" {
        return WORD_NIL;
    }
    if let Ok(value) = atom.parse::<i64>() {
        return encode_fixnum(value);
    }
    symbols.intern(atom)
}

/// Parses exactly one form from `source` and interns its symbols into
/// `symbols`. Errors on trailing input after the first form (a one-shot
/// console command is exactly one form) and on unbalanced parens.
pub fn read_one(source: &str, symbols: &mut SymbolTable) -> Result<u64, ReadError> {
    let tokens = tokenize(source);
    let mut pos = 0;
    let word = parse_form(&tokens, &mut pos, symbols)?;
    if pos != tokens.len() {
        let rest: Vec<String> = tokens[pos..]
            .iter()
            .map(|t| match t {
                Token::Open => "(".to_string(),
                Token::Close => ")".to_string(),
                Token::Atom(a) => a.clone(),
            })
            .collect();
        return Err(ReadError::TrailingInput(rest.join(" ")));
    }
    Ok(word)
}

fn parse_form(tokens: &[Token], pos: &mut usize, symbols: &mut SymbolTable) -> Result<u64, ReadError> {
    match tokens.get(*pos) {
        None => Err(ReadError::UnexpectedEof),
        Some(Token::Close) => Err(ReadError::UnexpectedCloseParen),
        Some(Token::Atom(a)) => {
            let word = atom_to_word(a, symbols);
            *pos += 1;
            Ok(word)
        }
        Some(Token::Open) => {
            *pos += 1;
            parse_list(tokens, pos, symbols)
        }
    }
}

fn parse_list(tokens: &[Token], pos: &mut usize, symbols: &mut SymbolTable) -> Result<u64, ReadError> {
    match tokens.get(*pos) {
        None => Err(ReadError::UnexpectedEof),
        Some(Token::Close) => {
            *pos += 1;
            Ok(WORD_NIL)
        }
        Some(_) => {
            let head = parse_form(tokens, pos, symbols)?;
            let tail = parse_list(tokens, pos, symbols)?;
            Ok(unsafe { wsm_cons(core::ptr::null_mut(), head, tail) })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{wsm_car, wsm_cdr};
    use crate::word::decode_fixnum;

    #[test]
    fn reads_bare_fixnum() {
        let mut symbols = SymbolTable::new();
        let word = read_one("42", &mut symbols).unwrap();
        assert_eq!(decode_fixnum(word), 42);
    }

    #[test]
    fn reads_negative_fixnum() {
        let mut symbols = SymbolTable::new();
        let word = read_one("-10", &mut symbols).unwrap();
        assert_eq!(decode_fixnum(word), -10);
    }

    #[test]
    fn reads_empty_call() {
        let mut symbols = SymbolTable::new();
        let word = read_one("(save-game)", &mut symbols).unwrap();
        let head = unsafe { wsm_car(core::ptr::null_mut(), word) };
        let tail = unsafe { wsm_cdr(core::ptr::null_mut(), word) };
        assert_eq!(symbols.name_of(head), Some("save-game"));
        assert_eq!(tail, WORD_NIL);
    }

    #[test]
    fn reads_teleport_call() {
        let mut symbols = SymbolTable::new();
        let word = read_one("(teleport player 100 200 50)", &mut symbols).unwrap();
        // Walk the list: (teleport . (player . (100 . (200 . (50 . ())))))
        let head = unsafe { wsm_car(core::ptr::null_mut(), word) };
        assert_eq!(symbols.name_of(head), Some("teleport"));
        let rest1 = unsafe { wsm_cdr(core::ptr::null_mut(), word) };
        let arg1 = unsafe { wsm_car(core::ptr::null_mut(), rest1) };
        assert_eq!(symbols.name_of(arg1), Some("player"));
        let rest2 = unsafe { wsm_cdr(core::ptr::null_mut(), rest1) };
        let arg2 = unsafe { wsm_car(core::ptr::null_mut(), rest2) };
        assert_eq!(decode_fixnum(arg2), 100);
    }

    #[test]
    fn reads_ukrainian_identifiers() {
        // The project's default working language is Ukrainian (lib/surface/uk.my
        // in my-lisp, 100% Ukrainian Surface Coverage) -- the reader's own
        // tokenizer already iterates `char`s (Unicode scalar values via
        // Rust's UTF-8-aware `chars()`), so Cyrillic identifiers were never
        // a special case to add, just something to actually verify.
        let mut symbols = SymbolTable::new();
        let word = read_one("(телепортуй гравець 100 200 50)", &mut symbols).unwrap();
        let head = unsafe { wsm_car(core::ptr::null_mut(), word) };
        assert_eq!(symbols.name_of(head), Some("телепортуй"));
        let rest = unsafe { wsm_cdr(core::ptr::null_mut(), word) };
        let arg1 = unsafe { wsm_car(core::ptr::null_mut(), rest) };
        assert_eq!(symbols.name_of(arg1), Some("гравець"));
    }

    #[test]
    fn rejects_trailing_input() {
        let mut symbols = SymbolTable::new();
        let err = read_one("42 43", &mut symbols).unwrap_err();
        assert!(matches!(err, ReadError::TrailingInput(_)));
    }

    #[test]
    fn rejects_unbalanced_close_paren() {
        let mut symbols = SymbolTable::new();
        let err = read_one(")", &mut symbols).unwrap_err();
        assert_eq!(err, ReadError::UnexpectedCloseParen);
    }
}
