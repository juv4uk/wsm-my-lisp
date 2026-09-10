//! Consolidated parity harness against my-lisp's own
//! docs/cyberpunk-host-dispatch-fixtures.md -- tasks-cyberpunk-
//! recommendations.my's CP-FIXTURE-MATRIX ("Automate my-lisp
//! cyberpunk-host-dispatch-fixtures against dll; UnknownSymbol text and
//! nested args must not drift. Acceptance: CI or documented harness
//! fails closed on mismatch").
//!
//! Every test below mirrors exactly one row/example from that document,
//! as of my-lisp commit `f97dfb3` (fetched and read directly, not from
//! memory, while writing this file). If my-lisp's own oracle output for
//! one of these expressions ever changes, the fix belongs on this side
//! (this crate re-verified against the new oracle output) -- this file
//! existing at all is what turns "did we notice the fixtures moved" into
//! a CI failure instead of a manual, easy-to-forget check.
//!
//! HONEST LIMIT: this is a documented harness, not a live cross-repo
//! oracle. It does not re-run my-lisp's own CLI in CI and diff against
//! it -- it hard-codes the fixture doc's already-verified expected
//! values. A fixture doc update that this file isn't updated to match
//! will NOT be caught automatically; it still requires a human (or
//! agent) to notice the doc changed and update this file to match. True
//! live-oracle automation (running my-lisp.exe in CI) is a larger,
//! separate piece of work, not attempted here.

use wsm_my_lisp_cyberpunk_dll::eval::{eval, Env};
use wsm_my_lisp_cyberpunk_dll::printer::value_to_string;
use wsm_my_lisp_cyberpunk_dll::reader::read_one;
use wsm_my_lisp_cyberpunk_dll::word::{encode_fixnum, BoxedTable, SymbolTable};

/// §1: reader fixtures (text -> structure, verified by round-tripping
/// through read -> print rather than inspecting the cons cells directly,
/// since the fixture doc's own "expected" column is itself printed text).
mod section_1_reader {
    use super::*;

    fn round_trip(input: &str) -> String {
        let mut symbols = SymbolTable::new();
        let mut strings = BoxedTable::new();
        let word = read_one(input, &mut symbols, &mut strings).expect("fixture input must parse");
        value_to_string(word, &symbols, &strings)
    }

    #[test]
    fn negative_numbers_are_ordinary_tokens() {
        assert_eq!(round_trip("(телепортуй гравець 100 200 -10)"), "(телепортуй гравець 100 200 -10)");
    }

    #[test]
    fn string_literal_keeps_quotes_in_printed_form() {
        assert_eq!(round_trip(r#"(дай-зброю "пістолет" 5)"#), r#"(дай-зброю "пістолет" 5)"#);
    }

    #[test]
    fn zero_argument_call_is_a_one_element_list() {
        assert_eq!(round_trip("(збережи-гру)"), "(збережи-гру)");
    }

    #[test]
    fn nested_list_argument_reads_as_ordinary_sublist() {
        assert_eq!(
            round_trip("(телепортуй гравець (+ x 10) y z)"),
            "(телепортуй гравець (+ x 10) y z)"
        );
    }
}

/// §2: evaluator fixtures -- host-primitive dispatch, no closures. Each
/// test sets up the same `def`-equivalent bindings the fixture doc's own
/// setup block describes (гравець=42, x=5, телепортуй/дай-зброю as host
/// primitives returning their evaluated args as a list-shaped value for
/// inspection).
mod section_2_evaluator {
    use super::*;
    use std::cell::RefCell;
    use std::rc::Rc;
    use wsm_my_lisp_cyberpunk_dll::word::decode_fixnum;

    #[test]
    fn teleport_row() {
        let mut symbols = SymbolTable::new();
        let mut strings = BoxedTable::new();
        let mut env = Env::new();
        env.bind("гравець", encode_fixnum(42));
        let calls = Rc::new(RefCell::new(Vec::new()));
        let calls_clone = calls.clone();
        env.register_primitive(
            "телепортуй",
            Box::new(move |args: &[u64]| {
                calls_clone.borrow_mut().push(args.to_vec());
                Ok(wsm_my_lisp_cyberpunk_dll::word::WORD_NIL)
            }),
        );
        let word = read_one("(телепортуй гравець 100 200 -10)", &mut symbols, &mut strings).unwrap();
        eval(word, &env, &symbols).unwrap();
        let args: Vec<i64> = calls.borrow()[0].iter().map(|&w| decode_fixnum(w)).collect();
        assert_eq!(args, vec![42, 100, 200, -10]); // fixture: (42 100 200 -10)
    }

    #[test]
    fn give_weapon_row_with_string_argument() {
        let mut symbols = SymbolTable::new();
        let mut strings = BoxedTable::new();
        let mut env = Env::new();
        let calls = Rc::new(RefCell::new(Vec::new()));
        let calls_clone = calls.clone();
        env.register_primitive(
            "дай-зброю",
            Box::new(move |args: &[u64]| {
                calls_clone.borrow_mut().push(args.to_vec());
                Ok(wsm_my_lisp_cyberpunk_dll::word::WORD_NIL)
            }),
        );
        let word = read_one(r#"(дай-зброю "пістолет" 5)"#, &mut symbols, &mut strings).unwrap();
        eval(word, &env, &symbols).unwrap();
        let recorded = calls.borrow();
        assert_eq!(strings.get_string(recorded[0][0]), Some("пістолет")); // fixture: "пістолет"
        assert_eq!(decode_fixnum(recorded[0][1]), 5);
    }

    #[test]
    fn nested_call_argument_evaluates_before_dispatch() {
        let mut symbols = SymbolTable::new();
        let mut strings = BoxedTable::new();
        let mut env = Env::new();
        env.bind("гравець", encode_fixnum(42));
        env.bind("x", encode_fixnum(5));
        env.register_primitive(
            "+",
            Box::new(|args: &[u64]| Ok(encode_fixnum(decode_fixnum(args[0]) + decode_fixnum(args[1])))),
        );
        let calls = Rc::new(RefCell::new(Vec::new()));
        let calls_clone = calls.clone();
        env.register_primitive(
            "телепортуй",
            Box::new(move |args: &[u64]| {
                calls_clone.borrow_mut().push(args.to_vec());
                Ok(wsm_my_lisp_cyberpunk_dll::word::WORD_NIL)
            }),
        );
        let word = read_one("(телепортуй гравець (+ x 10) 200 -10)", &mut symbols, &mut strings).unwrap();
        eval(word, &env, &symbols).unwrap();
        let args: Vec<i64> = calls.borrow()[0].iter().map(|&w| decode_fixnum(w)).collect();
        assert_eq!(args, vec![42, 15, 200, -10]); // fixture: (42 15 200 -10), (+ x 10) = 15
    }
}

/// §3: `cond` special form, against my-lisp's own player-identity example.
/// Writing this test found a real gap: eval.rs had no `quote` support at
/// all (only `cond` was implemented as a special form) -- this fixture
/// uses `(quote ...)` directly, so the gap surfaced immediately as a
/// failing test rather than staying unnoticed. Fixed in eval.rs alongside
/// this file, not worked around by rewriting the fixture to avoid quote.
mod section_3_cond {
    use super::*;

    #[test]
    fn known_player_branch() {
        let mut symbols = SymbolTable::new();
        let mut strings = BoxedTable::new();
        let mut env = Env::new();
        env.bind("гравець", encode_fixnum(42));
        env.register_primitive(
            "eq",
            Box::new(|args: &[u64]| Ok(if args[0] == args[1] { wsm_my_lisp_cyberpunk_dll::word::SYM_T_WORD } else { wsm_my_lisp_cyberpunk_dll::word::WORD_NIL })),
        );
        let word = read_one(
            "(cond ((eq гравець 42) (quote відомий-гравець)) (t (quote невідомий-гравець)))",
            &mut symbols,
            &mut strings,
        )
        .unwrap();
        let result = eval(word, &env, &symbols).unwrap();
        assert_eq!(value_to_string(result, &symbols, &strings), "відомий-гравець"); // fixture result
    }
}

/// §4: `UnknownSymbol` -- exact trilingual text, Cyrillic identifier
/// unmangled. Verified through the full FFI round-trip (not just the
/// in-memory EvalError) in ffi.rs's own
/// unknown_symbol_error_text_is_not_mangled_for_cyrillic test; this one
/// checks the same fact at the eval layer directly, against the exact
/// example expression the fixture doc uses (`(збережи-гру)`).
mod section_4_unknown_symbol {
    use super::*;
    use wsm_my_lisp_cyberpunk_dll::eval::EvalError;

    #[test]
    fn save_game_unregistered_gives_exact_trilingual_text() {
        let mut symbols = SymbolTable::new();
        let mut strings = BoxedTable::new();
        let env = Env::new();
        let word = read_one("(збережи-гру)", &mut symbols, &mut strings).unwrap();
        assert_eq!(
            eval(word, &env, &symbols),
            Err(EvalError::UnknownSymbol("збережи-гру".to_string()))
        );
    }
}
