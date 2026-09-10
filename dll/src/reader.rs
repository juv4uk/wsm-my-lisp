//! Minimal reader: text -> cons-structure, for one-shot console commands
//! like `(teleport player 100 200 50)`. Deliberately narrow for this
//! MVP slice (per the owner/my-lisp-cyberpunk go-ahead 2026-09-10):
//! symbols, fixnums (including negative), string literals, and
//! parenthesized lists.
//!
//! Explicitly NOT supported yet:
//!   - quote syntax (`'x`) -- not needed for one-shot commands
//!   - dotted-pair literals in input
//!
//! String literals (`"..."`) are TENTATIVE -- see word.rs's module doc
//! for the full status (TAG_STRING isn't yet reserved in
//! wsm-target-contract). Escaping is minimal: `\"` and `\\` only, no
//! `\n`/unicode escapes -- none of my-lisp's fixtures needed them.

use crate::word::{encode_fixnum, StringTable, SymbolTable, WORD_NIL};
use crate::wsm_cons;

#[derive(Debug, PartialEq)]
pub enum ReadError {
    UnexpectedEof,
    UnexpectedCloseParen,
    UnterminatedString,
    TrailingInput(String),
}

enum Token {
    Open,
    Close,
    Atom(String),
    Str(String),
}

fn tokenize(source: &str) -> Result<Vec<Token>, ReadError> {
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
            '"' => {
                chars.next();
                let mut value = String::new();
                loop {
                    match chars.next() {
                        None => return Err(ReadError::UnterminatedString),
                        Some('"') => break,
                        Some('\\') => match chars.next() {
                            Some('"') => value.push('"'),
                            Some('\\') => value.push('\\'),
                            Some(other) => {
                                // Unrecognized escape: keep both characters
                                // literally rather than silently dropping
                                // the backslash or guessing an interpretation
                                // my-lisp's fixtures never exercised.
                                value.push('\\');
                                value.push(other);
                            }
                            None => return Err(ReadError::UnterminatedString),
                        },
                        Some(other) => value.push(other),
                    }
                }
                tokens.push(Token::Str(value));
            }
            _ => {
                let mut atom = String::new();
                while let Some(&c) = chars.peek() {
                    if c.is_whitespace() || c == '(' || c == ')' || c == '"' {
                        break;
                    }
                    atom.push(c);
                    chars.next();
                }
                tokens.push(Token::Atom(atom));
            }
        }
    }
    Ok(tokens)
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

/// Parses exactly one form from `source`, interning symbols into `symbols`
/// and adding string literals to `strings`. Errors on trailing input after
/// the first form (a one-shot console command is exactly one form),
/// unbalanced parens, and unterminated string literals.
pub fn read_one(source: &str, symbols: &mut SymbolTable, strings: &mut StringTable) -> Result<u64, ReadError> {
    let tokens = tokenize(source)?;
    let mut pos = 0;
    let word = parse_form(&tokens, &mut pos, symbols, strings)?;
    if pos != tokens.len() {
        let rest: Vec<String> = tokens[pos..]
            .iter()
            .map(|t| match t {
                Token::Open => "(".to_string(),
                Token::Close => ")".to_string(),
                Token::Atom(a) => a.clone(),
                Token::Str(s) => format!("{s:?}"),
            })
            .collect();
        return Err(ReadError::TrailingInput(rest.join(" ")));
    }
    Ok(word)
}

fn parse_form(
    tokens: &[Token],
    pos: &mut usize,
    symbols: &mut SymbolTable,
    strings: &mut StringTable,
) -> Result<u64, ReadError> {
    match tokens.get(*pos) {
        None => Err(ReadError::UnexpectedEof),
        Some(Token::Close) => Err(ReadError::UnexpectedCloseParen),
        Some(Token::Atom(a)) => {
            let word = atom_to_word(a, symbols);
            *pos += 1;
            Ok(word)
        }
        Some(Token::Str(s)) => {
            let word = strings.add(s.clone());
            *pos += 1;
            Ok(word)
        }
        Some(Token::Open) => {
            *pos += 1;
            parse_list(tokens, pos, symbols, strings)
        }
    }
}

fn parse_list(
    tokens: &[Token],
    pos: &mut usize,
    symbols: &mut SymbolTable,
    strings: &mut StringTable,
) -> Result<u64, ReadError> {
    match tokens.get(*pos) {
        None => Err(ReadError::UnexpectedEof),
        Some(Token::Close) => {
            *pos += 1;
            Ok(WORD_NIL)
        }
        Some(_) => {
            let head = parse_form(tokens, pos, symbols, strings)?;
            let tail = parse_list(tokens, pos, symbols, strings)?;
            Ok(unsafe { wsm_cons(core::ptr::null_mut(), head, tail) })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::word::decode_fixnum;
    use crate::{wsm_car, wsm_cdr};

    #[test]
    fn reads_bare_fixnum() {
        let mut symbols = SymbolTable::new();
        let mut strings = StringTable::new();
        let word = read_one("42", &mut symbols, &mut strings).unwrap();
        assert_eq!(decode_fixnum(word), 42);
    }

    #[test]
    fn reads_negative_fixnum() {
        let mut symbols = SymbolTable::new();
        let mut strings = StringTable::new();
        let word = read_one("-10", &mut symbols, &mut strings).unwrap();
        assert_eq!(decode_fixnum(word), -10);
    }

    #[test]
    fn reads_empty_call() {
        let mut symbols = SymbolTable::new();
        let mut strings = StringTable::new();
        let word = read_one("(save-game)", &mut symbols, &mut strings).unwrap();
        let head = unsafe { wsm_car(core::ptr::null_mut(), word) };
        let tail = unsafe { wsm_cdr(core::ptr::null_mut(), word) };
        assert_eq!(symbols.name_of(head), Some("save-game"));
        assert_eq!(tail, WORD_NIL);
    }

    #[test]
    fn reads_teleport_call() {
        let mut symbols = SymbolTable::new();
        let mut strings = StringTable::new();
        let word = read_one("(teleport player 100 200 50)", &mut symbols, &mut strings).unwrap();
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
        let mut strings = StringTable::new();
        let word = read_one("(телепортуй гравець 100 200 50)", &mut symbols, &mut strings).unwrap();
        let head = unsafe { wsm_car(core::ptr::null_mut(), word) };
        assert_eq!(symbols.name_of(head), Some("телепортуй"));
        let rest = unsafe { wsm_cdr(core::ptr::null_mut(), word) };
        let arg1 = unsafe { wsm_car(core::ptr::null_mut(), rest) };
        assert_eq!(symbols.name_of(arg1), Some("гравець"));
    }

    #[test]
    fn reads_string_literal_argument() {
        // From my-lisp's docs/cyberpunk-host-dispatch-fixtures.md §1:
        // `(дай-зброю "пістолет" 5)`.
        let mut symbols = SymbolTable::new();
        let mut strings = StringTable::new();
        let word = read_one(r#"(дай-зброю "пістолет" 5)"#, &mut symbols, &mut strings).unwrap();
        let rest = unsafe { wsm_cdr(core::ptr::null_mut(), word) };
        let arg1 = unsafe { wsm_car(core::ptr::null_mut(), rest) };
        assert_eq!(strings.get(arg1), Some("пістолет"));
    }

    #[test]
    fn two_equal_string_literals_are_independent_allocations() {
        // my-lisp confirmed strings are NOT interned, unlike symbols --
        // two textually-equal literals must not collapse to the same word.
        let mut symbols = SymbolTable::new();
        let mut strings = StringTable::new();
        let word = read_one(r#"(f "x" "x")"#, &mut symbols, &mut strings).unwrap();
        let rest = unsafe { wsm_cdr(core::ptr::null_mut(), word) };
        let arg1 = unsafe { wsm_car(core::ptr::null_mut(), rest) };
        let rest2 = unsafe { wsm_cdr(core::ptr::null_mut(), rest) };
        let arg2 = unsafe { wsm_car(core::ptr::null_mut(), rest2) };
        assert_ne!(arg1, arg2, "two equal string literals must not share a word");
        assert_eq!(strings.get(arg1), Some("x"));
        assert_eq!(strings.get(arg2), Some("x"));
    }

    #[test]
    fn rejects_unterminated_string() {
        let mut symbols = SymbolTable::new();
        let mut strings = StringTable::new();
        let err = read_one(r#"(f "unterminated)"#, &mut symbols, &mut strings).unwrap_err();
        assert_eq!(err, ReadError::UnterminatedString);
    }

    #[test]
    fn rejects_trailing_input() {
        let mut symbols = SymbolTable::new();
        let mut strings = StringTable::new();
        let err = read_one("42 43", &mut symbols, &mut strings).unwrap_err();
        assert!(matches!(err, ReadError::TrailingInput(_)));
    }

    #[test]
    fn rejects_unbalanced_close_paren() {
        let mut symbols = SymbolTable::new();
        let mut strings = StringTable::new();
        let err = read_one(")", &mut symbols, &mut strings).unwrap_err();
        assert_eq!(err, ReadError::UnexpectedCloseParen);
    }
}
