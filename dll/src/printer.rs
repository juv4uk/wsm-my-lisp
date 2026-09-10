//! Printer: cons-structure -> text, the inverse of reader.rs, for showing
//! evaluation results back in the game's console. Independent of the 3
//! open semantics questions flagged in eval.rs (cond special-form-vs-macro,
//! bare-symbol bindings, string encoding) -- printing a value doesn't
//! require resolving any of them, so this doesn't wait on my-lisp.
//!
//! Same narrow scope as reader.rs: Fixnum, Nil (`()`), Symbol (including
//! `t`), and proper lists `(a b c)`. Dotted pairs print as `(a . b)` since
//! the underlying representation can express them even though reader.rs
//! doesn't parse that syntax on input yet.

use crate::word::{tag_of, SymbolTable, TAG_CONS, TAG_FIXNUM, TAG_NIL, TAG_SYMBOL};
use crate::{wsm_car, wsm_cdr};
use std::fmt::Write as _;

pub fn value_to_string(word: u64, symbols: &SymbolTable) -> String {
    let mut out = String::new();
    write_value(word, symbols, &mut out);
    out
}

fn write_value(word: u64, symbols: &SymbolTable, out: &mut String) {
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
        TAG_CONS => write_list(word, symbols, out),
        other => {
            let _ = write!(out, "#<unprintable-tag:{other}>");
        }
    }
}

fn write_list(word: u64, symbols: &SymbolTable, out: &mut String) {
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
                write_value(head, symbols, out);
                current = unsafe { wsm_cdr(core::ptr::null_mut(), current) };
            }
            _ => {
                // Improper (dotted) tail.
                if !first {
                    out.push(' ');
                }
                out.push_str(". ");
                write_value(current, symbols, out);
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
        let word = read_one("42", &mut symbols).unwrap();
        assert_eq!(value_to_string(word, &symbols), "42");
    }

    #[test]
    fn prints_negative_fixnum() {
        let mut symbols = SymbolTable::new();
        let word = read_one("-10", &mut symbols).unwrap();
        assert_eq!(value_to_string(word, &symbols), "-10");
    }

    #[test]
    fn prints_nil() {
        let mut symbols = SymbolTable::new();
        let word = read_one("()", &mut symbols).unwrap();
        assert_eq!(value_to_string(word, &symbols), "()");
    }

    #[test]
    fn prints_symbol_and_t() {
        let mut symbols = SymbolTable::new();
        let sym = read_one("player", &mut symbols).unwrap();
        assert_eq!(value_to_string(sym, &symbols), "player");
        let t = read_one("t", &mut symbols).unwrap();
        assert_eq!(value_to_string(t, &symbols), "t");
    }

    #[test]
    fn prints_list_round_trip() {
        let mut symbols = SymbolTable::new();
        let word = read_one("(teleport player 100 200 50)", &mut symbols).unwrap();
        assert_eq!(value_to_string(word, &symbols), "(teleport player 100 200 50)");
    }

    #[test]
    fn prints_dotted_pair() {
        let symbols = SymbolTable::new();
        let pair = unsafe { crate::wsm_cons(core::ptr::null_mut(), crate::word::encode_fixnum(1), crate::word::encode_fixnum(2)) };
        assert_eq!(value_to_string(pair, &symbols), "(1 . 2)");
    }
}
