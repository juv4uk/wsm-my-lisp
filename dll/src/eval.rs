//! Minimal evaluator for one-shot console commands: self-evaluating
//! fixnums/Nil/`t`, symbol lookup (host-registered bindings or
//! UnknownSymbol error), host-primitive dispatch, and `cond` as a special
//! form. Deliberately no lambda/closures/def -- see asm/nucleus-win64.s's
//! header for why that stays out of scope for this pass.
//!
//! Both design questions below were open when this file was first written
//! and have since been confirmed directly by my-lisp (2026-09-10, against
//! their own code, not from memory):
//!
//! - **`cond` is a true evaluator special form**, not a macro -- my-lisp's
//!   own `language-contract.my` `special-forms-boundary` states `quote
//!   cond lambda def defmacro` "are NOT callable values. They are
//!   syntactic evaluation rules," part of immutable Canon (0+7). This
//!   file's model (recognize `cond` by name before normal call dispatch,
//!   never evaluate it as an ordinary call) matches that.
//! - **Bare symbol arguments require a prior binding, same as real
//!   my-lisp's own `def`** -- my-lisp confirmed there is no
//!   "self-evaluating identifier" concept: a bare symbol always attempts
//!   lookup and raises `UnknownSymbol` if unbound, exactly like
//!   `(undefined-symbol)` already does here. `Env::bindings` is this
//!   module's stand-in for the host side doing the equivalent of
//!   `(def player <handle>)` before evaluating a command that references
//!   `player` -- a real implementation of `def`'s effect, not a deviation
//!   from it. (If some future primitive instead wanted `player` passed as
//!   a literal name-token with no lookup at all, that WOULD be a real
//!   deviation from my-lisp semantics and would need to be flagged as
//!   such explicitly -- not the case here.)
//!
//! Still genuinely open, NOT resolved by this file: **string literal
//! encoding**. my-lisp confirmed strings are a distinct, immutable UTF-8
//! type (`Rc<str>` on their side), never the same type as Symbol --
//! word.rs's wsm-os-target::Tag has no String variant, so representing
//! one needs a new tag (payload = pointer+length into the arena, or
//! similar) that doesn't exist yet. The exact tag value/layout is a
//! cross-repo ABI decision (affects cml/wsm-os-target too), not something
//! to invent unilaterally here -- reader.rs still does not parse quoted
//! strings pending that decision.

use std::collections::HashMap;

use crate::word::{is_truthy, tag_of, SymbolTable, TAG_CONS, TAG_FIXNUM, TAG_NIL, TAG_SYMBOL, WORD_NIL};
use crate::{wsm_car, wsm_cdr};

#[derive(Debug, PartialEq)]
pub enum EvalError {
    /// Mirrors my-lisp's own UnknownSymbol condition (conformance.my:
    /// `(undefined-symbol)` -> named error, not a panic/crash). Carries
    /// the offending name for a caller-facing message.
    UnknownSymbol(String),
    /// A list's head evaluated to something other than a Symbol -- no
    /// value-producing expression is callable in this MVP (no closures).
    NotCallable,
    /// `cond` reached its end with no truthy test -- real my-lisp's own
    /// behavior here (error vs. returning Nil) is unconfirmed; treated as
    /// an explicit error for now rather than silently returning Nil, so
    /// it's visible instead of masquerading as a valid Nil result.
    CondFallthrough,
    /// A registered host primitive reported failure. Carries the
    /// primitive's name and a host-supplied message, so a host-reported
    /// error surfaces as a real error instead of silently becoming Nil
    /// (that used to be this crate's behavior -- see ffi.rs's git history
    /// for why it changed).
    HostPrimitiveFailed { name: String, message: String },
}

/// Returns `Err(message)` on host-side failure -- see `EvalError::
/// HostPrimitiveFailed`. This is the internal Rust-level contract;
/// ffi.rs's `HostPrimitiveFn` (the C ABI a registered primitive is
/// actually called through) adapts its int return code into this.
pub type HostPrimitive = Box<dyn Fn(&[u64]) -> Result<u64, String>>;

#[derive(Default)]
pub struct Env {
    primitives: HashMap<String, HostPrimitive>,
    /// Host-registered constant values (see module doc's "open question").
    bindings: HashMap<String, u64>,
}

impl Env {
    pub fn new() -> Self {
        Self { primitives: HashMap::new(), bindings: HashMap::new() }
    }

    pub fn register_primitive(&mut self, name: &str, f: HostPrimitive) {
        self.primitives.insert(name.to_string(), f);
    }

    pub fn bind(&mut self, name: &str, value: u64) {
        self.bindings.insert(name.to_string(), value);
    }
}

pub fn eval(word: u64, env: &Env, symbols: &SymbolTable) -> Result<u64, EvalError> {
    match tag_of(word) {
        TAG_FIXNUM | TAG_NIL => Ok(word), // self-evaluating
        TAG_SYMBOL => eval_symbol(word, env, symbols),
        TAG_CONS => eval_list(word, env, symbols),
        _ => Ok(word), // Closure/Capability: out of scope, pass through unevaluated
    }
}

fn eval_symbol(word: u64, env: &Env, symbols: &SymbolTable) -> Result<u64, EvalError> {
    let name = symbols
        .name_of(word)
        .expect("symbol word not found in the table that produced it");
    if name == "t" {
        return Ok(word); // t self-evaluates
    }
    if let Some(&value) = env.bindings.get(name) {
        return Ok(value);
    }
    Err(EvalError::UnknownSymbol(name.to_string()))
}

fn eval_list(word: u64, env: &Env, symbols: &SymbolTable) -> Result<u64, EvalError> {
    let head = unsafe { wsm_car(core::ptr::null_mut(), word) };
    let rest = unsafe { wsm_cdr(core::ptr::null_mut(), word) };

    if tag_of(head) != TAG_SYMBOL {
        return Err(EvalError::NotCallable);
    }
    let name = symbols
        .name_of(head)
        .expect("symbol word not found in the table that produced it");

    if name == "cond" {
        return eval_cond(rest, env, symbols);
    }

    let args = eval_args(rest, env, symbols)?;
    match env.primitives.get(name) {
        Some(f) => f(&args).map_err(|message| EvalError::HostPrimitiveFailed {
            name: name.to_string(),
            message,
        }),
        None => Err(EvalError::UnknownSymbol(name.to_string())),
    }
}

fn eval_args(mut list: u64, env: &Env, symbols: &SymbolTable) -> Result<Vec<u64>, EvalError> {
    let mut args = Vec::new();
    while list != WORD_NIL {
        let item = unsafe { wsm_car(core::ptr::null_mut(), list) };
        args.push(eval(item, env, symbols)?);
        list = unsafe { wsm_cdr(core::ptr::null_mut(), list) };
    }
    Ok(args)
}

/// clauses is the cdr of `(cond (test1 body1) (test2 body2) ...)`: a list
/// of `(test . body)` conses. First truthy test wins; only its body form
/// is evaluated (matches conformance.my: later false clauses are simply
/// skipped, never evaluated).
fn eval_cond(mut clauses: u64, env: &Env, symbols: &SymbolTable) -> Result<u64, EvalError> {
    while clauses != WORD_NIL {
        let clause = unsafe { wsm_car(core::ptr::null_mut(), clauses) };
        let test = unsafe { wsm_car(core::ptr::null_mut(), clause) };
        let body = unsafe { wsm_cdr(core::ptr::null_mut(), clause) };
        let test_result = eval(test, env, symbols)?;
        if is_truthy(test_result) {
            let result_expr = unsafe { wsm_car(core::ptr::null_mut(), body) };
            return eval(result_expr, env, symbols);
        }
        clauses = unsafe { wsm_cdr(core::ptr::null_mut(), clauses) };
    }
    Err(EvalError::CondFallthrough)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::reader::read_one;
    use crate::word::{decode_fixnum, encode_fixnum, SYM_T_WORD};

    #[test]
    fn fixnum_self_evaluates() {
        let mut symbols = SymbolTable::new();
        let env = Env::new();
        let word = read_one("42", &mut symbols).unwrap();
        assert_eq!(decode_fixnum(eval(word, &env, &symbols).unwrap()), 42);
    }

    #[test]
    fn unbound_symbol_errors() {
        let mut symbols = SymbolTable::new();
        let env = Env::new();
        let word = read_one("(undefined-symbol)", &mut symbols).unwrap();
        assert_eq!(
            eval(word, &env, &symbols),
            Err(EvalError::UnknownSymbol("undefined-symbol".to_string()))
        );
    }

    #[test]
    fn dispatches_to_host_primitive_with_evaluated_args() {
        let mut symbols = SymbolTable::new();
        let mut env = Env::new();
        env.bind("player", symbols.intern("player-handle")); // placeholder binding
        let calls = std::rc::Rc::new(std::cell::RefCell::new(Vec::new()));
        let calls_clone = calls.clone();
        env.register_primitive(
            "teleport",
            Box::new(move |args: &[u64]| {
                calls_clone.borrow_mut().push(args.to_vec());
                Ok(WORD_NIL)
            }),
        );
        let word = read_one("(teleport player 100 200 50)", &mut symbols).unwrap();
        eval(word, &env, &symbols).unwrap();
        let recorded = calls.borrow();
        assert_eq!(recorded.len(), 1);
        assert_eq!(decode_fixnum(recorded[0][1]), 100);
        assert_eq!(decode_fixnum(recorded[0][2]), 200);
        assert_eq!(decode_fixnum(recorded[0][3]), 50);
    }

    #[test]
    fn host_primitive_failure_surfaces_as_eval_error() {
        let mut symbols = SymbolTable::new();
        let mut env = Env::new();
        env.register_primitive(
            "give-weapon",
            Box::new(|_args: &[u64]| Err("unknown weapon id".to_string())),
        );
        let word = read_one("(give-weapon)", &mut symbols).unwrap();
        assert_eq!(
            eval(word, &env, &symbols),
            Err(EvalError::HostPrimitiveFailed {
                name: "give-weapon".to_string(),
                message: "unknown weapon id".to_string(),
            })
        );
    }

    #[test]
    fn nested_call_argument_evaluates_before_dispatch() {
        // From my-lisp's own docs/cyberpunk-host-dispatch-fixtures.md §2
        // (produced by running their real CLI, not written from memory):
        // `(teleport player (+ x 10) 200 -10)` with player=42, x=5 evaluates
        // to args `(42 15 200 -10)` -- the `(+ x 10)` argument (5+10=15)
        // evaluates recursively before the outer primitive is invoked, not
        // passed as a literal list.
        let mut symbols = SymbolTable::new();
        let mut env = Env::new();
        env.bind("player", encode_fixnum(42));
        env.bind("x", encode_fixnum(5));
        env.register_primitive(
            "+",
            Box::new(|args: &[u64]| Ok(encode_fixnum(decode_fixnum(args[0]) + decode_fixnum(args[1])))),
        );
        let calls = std::rc::Rc::new(std::cell::RefCell::new(Vec::new()));
        let calls_clone = calls.clone();
        env.register_primitive(
            "teleport",
            Box::new(move |args: &[u64]| {
                calls_clone.borrow_mut().push(args.to_vec());
                Ok(WORD_NIL)
            }),
        );
        let word = read_one("(teleport player (+ x 10) 200 -10)", &mut symbols).unwrap();
        eval(word, &env, &symbols).unwrap();
        let recorded = calls.borrow();
        assert_eq!(
            recorded[0].iter().map(|&w| decode_fixnum(w)).collect::<Vec<_>>(),
            vec![42, 15, 200, -10]
        );
    }

    #[test]
    fn dispatches_ukrainian_identifiers() {
        // Matches my-lisp's own updated docs/cyberpunk-host-dispatch-fixtures.md
        // (commit 5a6bb90): canonical examples now use Ukrainian identifiers
        // (телепортуй/гравець) rather than English ones -- same semantics,
        // just proving the evaluator dispatches on them identically, not
        // only on ASCII symbol names.
        let mut symbols = SymbolTable::new();
        let mut env = Env::new();
        env.bind("гравець", symbols.intern("гравець-handle"));
        let calls = std::rc::Rc::new(std::cell::RefCell::new(Vec::new()));
        let calls_clone = calls.clone();
        env.register_primitive(
            "телепортуй",
            Box::new(move |args: &[u64]| {
                calls_clone.borrow_mut().push(args.to_vec());
                Ok(WORD_NIL)
            }),
        );
        let word = read_one("(телепортуй гравець 100 200 50)", &mut symbols).unwrap();
        eval(word, &env, &symbols).unwrap();
        assert_eq!(calls.borrow().len(), 1);
    }

    #[test]
    fn cond_skips_falsy_and_picks_first_truthy() {
        let mut symbols = SymbolTable::new();
        let env = Env::new();
        let word = read_one("(cond (() 1) (() 2) (t 3))", &mut symbols).unwrap();
        assert_eq!(decode_fixnum(eval(word, &env, &symbols).unwrap()), 3);
    }

    #[test]
    fn cond_treats_fixnum_zero_as_truthy() {
        let mut symbols = SymbolTable::new();
        let env = Env::new();
        let word = read_one("(cond (0 1) (t 2))", &mut symbols).unwrap();
        assert_eq!(decode_fixnum(eval(word, &env, &symbols).unwrap()), 1);
    }

    #[test]
    fn t_self_evaluates() {
        let mut symbols = SymbolTable::new();
        let env = Env::new();
        let word = read_one("t", &mut symbols).unwrap();
        assert_eq!(eval(word, &env, &symbols).unwrap(), SYM_T_WORD);
    }
}
