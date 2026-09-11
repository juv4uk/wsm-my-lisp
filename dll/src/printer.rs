//! Printer: cons-structure -> text, the inverse of reader.rs, for showing
//! evaluation results back in the game's console. Independent of the
//! (now-resolved, see eval.rs's module doc) cond/bindings questions --
//! printing a value never depended on those. String printing depends on
//! the still-TENTATIVE TAG_BOXED (word.rs's module doc), so it's marked
//! as such below too.
//!
//! Same narrow scope as reader.rs: Fixnum, Nil (`()`), Symbol (including
//! `t`), String, and proper lists `(a b c)`. Dotted pairs print as
//! `(a . b)` since the underlying representation can express them even
//! though reader.rs doesn't parse that syntax on input yet.

use crate::word::{tag_of, BoxedKind, BoxedTable, SymbolTable, TAG_BOXED, TAG_CONS, TAG_FIXNUM, TAG_NIL, TAG_SYMBOL};
use crate::{wsm_car, wsm_cdr};
use std::fmt::Write as _;

pub fn value_to_string(word: u64, symbols: &SymbolTable, strings: &BoxedTable) -> String {
    let mut out = String::new();
    write_value(word, symbols, strings, &mut out);
    out
}

fn write_value(word: u64, symbols: &SymbolTable, strings: &BoxedTable, out: &mut String) {
    match tag_of(word) {
        TAG_NIL => out.push_str("()"),
        TAG_FIXNUM => {
            let value = crate::word::decode_fixnum(word);
            let _ = write!(out, "{value}");
        }
        TAG_SYMBOL => match symbols.name_of(word) {
            Some(name) => out.push_str(name),
            None => {
                let _ = write!(out, "#<unknown-symbol:{:#x}>", word);
            }
        },
        TAG_BOXED => match strings.kind_of(word) {
            // Quoting is intentionally naive (wrap in `"`, no internal-quote
            // escaping on output) -- matches reader.rs's own minimal
            // escaping, not a general string-literal printer.
            Some(BoxedKind::Str) => {
                out.push('"');
                out.push_str(strings.get_string(word).expect("kind_of said Str"));
                out.push('"');
            }
            // Deliberately does NOT print the underlying pointer -- doing
            // so would leak a host address into Lisp-visible text, the
            // exact thing GameHandle's opaqueness is meant to prevent
            // (see word.rs's BoxedValue::GameHandle doc).
            Some(BoxedKind::GameHandle) => out.push_str("#<game-handle>"),
            // "N/D", matching my-lisp's own Rational print format exactly
            // (confirmed against conformance.my: `(/ 5 6 8 7)` prints
            // "5/336") -- byte-identical to their oracle because
            // BoxedTable::add_rational already reduces at construction,
            // not just here at print time.
            Some(BoxedKind::Rational) => {
                let (n, d) = strings.get_rational(word).expect("kind_of said Rational");
                let _ = write!(out, "{n}/{d}");
            }
            None => {
                let _ = write!(out, "#<unknown-boxed:{:#x}>", word);
            }
        },
        TAG_CONS => write_list(word, symbols, strings, out),
        other => {
            let _ = write!(out, "#<unprintable-tag:{other}>");
        }
    }
}

fn write_list(word: u64, symbols: &SymbolTable, strings: &BoxedTable, out: &mut String) {
    out.push('(');
    let mut current = word;
    let mut first = true;
    loop {
        match tag_of(current) {
            TAG_NIL => break, // proper-list terminator, not printed
            TAG_CONS => {
                if !first {
                    out.push(' ');
                }
                first = false;
                let head = unsafe { wsm_car(core::ptr::null_mut(), current) };
                write_value(head, symbols, strings, out);
                current = unsafe { wsm_cdr(core::ptr::null_mut(), current) };
            }
            _ => {
                // Improper (dotted) tail.
                if !first {
                    out.push(' ');
                }
                out.push_str(". ");
                write_value(current, symbols, strings, out);
                break;
            }
        }
    }
    out.push(')');
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::reader::read_one;

    #[test]
    fn prints_fixnum() {
        let mut symbols = SymbolTable::new();
        let mut strings = BoxedTable::new();
        let word = read_one("42", &mut symbols, &mut strings).unwrap();
        assert_eq!(value_to_string(word, &symbols, &strings), "42");
    }

    #[test]
    fn prints_negative_fixnum() {
        let mut symbols = SymbolTable::new();
        let mut strings = BoxedTable::new();
        let word = read_one("-10", &mut symbols, &mut strings).unwrap();
        assert_eq!(value_to_string(word, &symbols, &strings), "-10");
    }

    #[test]
    fn prints_nil() {
        let mut symbols = SymbolTable::new();
        let mut strings = BoxedTable::new();
        let word = read_one("()", &mut symbols, &mut strings).unwrap();
        assert_eq!(value_to_string(word, &symbols, &strings), "()");
    }

    #[test]
    fn prints_symbol_and_t() {
        let mut symbols = SymbolTable::new();
        let mut strings = BoxedTable::new();
        let sym = read_one("player", &mut symbols, &mut strings).unwrap();
        assert_eq!(value_to_string(sym, &symbols, &strings), "player");
        let t = read_one("t", &mut symbols, &mut strings).unwrap();
        assert_eq!(value_to_string(t, &symbols, &strings), "t");
    }

    #[test]
    fn prints_list_round_trip() {
        let mut symbols = SymbolTable::new();
        let mut strings = BoxedTable::new();
        let word = read_one("(teleport player 100 200 50)", &mut symbols, &mut strings).unwrap();
        assert_eq!(value_to_string(word, &symbols, &strings), "(teleport player 100 200 50)");
    }

    #[test]
    fn prints_dotted_pair() {
        let symbols = SymbolTable::new();
        let strings = BoxedTable::new();
        let pair = unsafe {
            crate::wsm_cons(core::ptr::null_mut(), crate::word::encode_fixnum(1), crate::word::encode_fixnum(2))
        };
        assert_eq!(value_to_string(pair, &symbols, &strings), "(1 . 2)");
    }

    #[test]
    fn prints_string_literal_with_quotes() {
        let mut symbols = SymbolTable::new();
        let mut strings = BoxedTable::new();
        let word = read_one(r#"(дай-зброю "пістолет" 5)"#, &mut symbols, &mut strings).unwrap();
        assert_eq!(value_to_string(word, &symbols, &strings), r#"(дай-зброю "пістолет" 5)"#);
    }

    #[test]
    fn prints_game_handle_without_leaking_the_pointer() {
        let symbols = SymbolTable::new();
        let mut strings = BoxedTable::new();
        let word = strings.add_game_handle(0xdead_beef_usize as *mut core::ffi::c_void);
        let printed = value_to_string(word, &symbols, &strings);
        assert_eq!(printed, "#<game-handle>");
        assert!(!printed.contains("deadbeef") && !printed.contains("dead_beef"));
    }
}
