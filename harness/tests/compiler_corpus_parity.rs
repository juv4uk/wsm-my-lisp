//! Mechanical drift guard for wsm-my-lisp#4's remaining acceptance
//! criterion: "deliberate mutation of one result/error/provenance
//! field is caught." `docs/compiler-oracle-corpus-parity-2026-09-11.md`
//! is a hand-maintained table describing all 17 `(compiler-corpus . t)`
//! fixtures in `external/my-lisp/tests/fixtures/conformance.my` -- this
//! test is the mechanical half that document's own "Parity gate note"
//! section named as not yet done: if my-lisp changes any tagged
//! fixture's `expr`/`expected`/`error` (or adds/removes one), this
//! fails closed instead of the parity doc silently going stale.
//!
//! Deliberately NOT a full S-expression parser (that belongs in
//! `build.rs`-style generators consuming this file for real dispatch,
//! e.g. the Canon-spelling projection in `my-lisp-cyberpunk/host-runtime`)
//! -- `conformance.my`'s own fixture format is one flat alist per line,
//! so line-based extraction of the two fields this test cares about is
//! sufficient and simpler than a general reader.

const CONFORMANCE: &str = include_str!("../../external/my-lisp/tests/fixtures/conformance.my");

/// Extracts the quoted string value of `(key . "value")` from one line,
/// or `None` if that key isn't present on the line.
fn field(line: &str, key: &str) -> Option<String> {
    let marker = format!("({key} . \"");
    let start = line.find(&marker)? + marker.len();
    let rest = &line[start..];
    let end = rest.find('"')?;
    Some(rest[..end].to_string())
}

#[derive(Debug, PartialEq)]
enum Outcome {
    Expected(String),
    Error(String),
}

fn tagged_fixtures() -> Vec<(String, Outcome)> {
    CONFORMANCE
        .lines()
        .filter(|line| line.contains("(compiler-corpus . t)"))
        .map(|line| {
            let expr = field(line, "expr").expect("compiler-corpus fixture must have expr");
            let outcome = if let Some(expected) = field(line, "expected") {
                Outcome::Expected(expected)
            } else if let Some(error) = field(line, "error") {
                Outcome::Error(error)
            } else {
                panic!("compiler-corpus fixture has neither expected nor error: {line}");
            };
            (expr, outcome)
        })
        .collect()
}

/// The exact 17 fixtures documented in
/// docs/compiler-oracle-corpus-parity-2026-09-11.md, as of my-lisp
/// commit 6d71151 (the pinned external/my-lisp submodule commit).
/// Update BOTH this list and that document together if my-lisp adds,
/// removes, or changes a `compiler-corpus` fixture -- that is the
/// point of this test failing: it forces the parity doc to stay honest
/// rather than silently drifting from the real corpus.
fn documented_fixtures() -> Vec<(&'static str, Outcome)> {
    vec![
        ("(quote radio)", Outcome::Expected("radio".into())),
        ("(atom (quote radio))", Outcome::Expected("t".into())),
        (
            "(eq (quote radio) (quote radio))",
            Outcome::Expected("t".into()),
        ),
        (
            "(car (quote (radio antenna)))",
            Outcome::Expected("radio".into()),
        ),
        (
            "(cdr (quote (radio antenna)))",
            Outcome::Expected("(antenna)".into()),
        ),
        (
            "(cons (quote radio) (quote (antenna)))",
            Outcome::Expected("(radio antenna)".into()),
        ),
        (
            "(cond (() (quote wrong)) (t (quote right)))",
            Outcome::Expected("right".into()),
        ),
        ("(/ 5 6 8 7)", Outcome::Expected("5/336".into())),
        (
            "(eq (lambda (x) x) (lambda (x) x))",
            Outcome::Expected("()".into()),
        ),
        ("(defmacro foo)", Outcome::Error("Arity".into())),
        (
            "(def count-down (lambda (n) (cond ((eq n 0) (quote done)) (t (count-down (- n 1)))))) (count-down 100000)",
            Outcome::Expected("done".into()),
        ),
        (
            "((lambda (a b . rest) rest) 1 2 3 4 5)",
            Outcome::Expected("(3 4 5)".into()),
        ),
        (
            "((lambda args args) 1 2 3)",
            Outcome::Expected("(1 2 3)".into()),
        ),
        (
            "((lambda (a b . rest) a) 1)",
            Outcome::Error("Arity".into()),
        ),
        (
            "(let ((second (lambda (x) (quote shadowed)))) (second (quote (1 2 3))))",
            Outcome::Expected("shadowed".into()),
        ),
        (
            "(let ((car (lambda (x) (quote shadowed)))) (car (quote (1 2))))",
            Outcome::Error("InvalidForm".into()),
        ),
        (
            "(defmacro my-list items (cons (quote quote) (cons items (quote ())))) (my-list 1 2 3)",
            Outcome::Expected("(1 2 3)".into()),
        ),
    ]
    .into_iter()
    .map(|(expr, outcome)| (expr, outcome))
    .collect()
}

#[test]
fn compiler_corpus_matches_documented_parity_table() {
    let actual = tagged_fixtures();
    let documented = documented_fixtures();

    assert_eq!(
        actual.len(),
        documented.len(),
        "conformance.my's compiler-corpus fixture count changed ({} found, {} documented) -- \
         update docs/compiler-oracle-corpus-parity-2026-09-11.md AND this test's \
         documented_fixtures() together",
        actual.len(),
        documented.len()
    );

    for (expr, outcome) in &documented {
        let found = actual
            .iter()
            .find(|(actual_expr, _)| actual_expr == expr)
            .unwrap_or_else(|| {
                panic!(
                    "documented fixture not found in conformance.my (removed or expr text \
                     changed): {expr}"
                )
            });
        assert_eq!(
            &found.1, outcome,
            "compiler-corpus fixture's outcome drifted from the documented parity table: {expr}"
        );
    }
}
