//! Mechanical drift guard for wsm-my-lisp#4's remaining acceptance
//! criterion: "deliberate mutation of one result/error/provenance
//! field is caught." `docs/compiler-oracle-corpus-parity-2026-09-11.md`
//! is a hand-maintained table describing all 17 `(compiler-corpus . t)`
//! fixtures in `external/sens/tests/fixtures/conformance.lisp` -- this
//! test is the mechanical half that document's own "Parity gate note"
//! section named as not yet done: if sens changes any tagged
//! fixture's `expr`/`expected`/`error` (or adds/removes one), this
//! fails closed instead of the parity doc silently going stale.
//!
//! Deliberately NOT a full S-expression parser (that belongs in
//! `build.rs`-style generators consuming this file for real dispatch,
//! e.g. the Canon-spelling projection in `my-lisp-cyberpunk/host-runtime`)
//! -- `conformance.lisp`'s own fixture format is one flat alist per line,
//! so line-based extraction of the two fields this test cares about is
//! sufficient and simpler than a general reader.

const CONFORMANCE: &str = include_str!("../../external/sens/tests/fixtures/conformance.lisp");
const PARITY_DOC: &str = include_str!("../../docs/compiler-oracle-corpus-parity-2026-09-11.md");

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

/// Parse the 17 data rows from the parity document itself.
///
/// This deliberately removes the third hand-copied fixture list that used to
/// live in this Rust test. There are now only two authorities to compare:
/// pinned SENS corpus data and this repository's human-readable parity table.
fn documented_fixtures() -> Vec<(String, Outcome)> {
    PARITY_DOC
        .lines()
        .filter(|line| {
            let trimmed = line.trim_start();
            trimmed.starts_with("| ") &&
                trimmed.chars().nth(2).is_some_and(|ch| ch.is_ascii_digit())
        })
        .map(|line| {
            let columns: Vec<_> = line.split('|').map(str::trim).collect();
            assert!(
                columns.len() >= 4,
                "malformed compiler parity table row: {line}"
            );

            let expr = columns[2]
                .strip_prefix('`')
                .and_then(|value| value.strip_suffix('`'))
                .unwrap_or_else(|| panic!("parity expr is not code-formatted: {}", columns[2]))
                .to_string();

            let observed = columns[3];
            let outcome = if let Some(error) = observed.strip_prefix("error `") {
                Outcome::Error(
                    error
                        .strip_suffix('`')
                        .unwrap_or_else(|| panic!("malformed error cell: {observed}"))
                        .to_string(),
                )
            } else {
                Outcome::Expected(
                    observed
                        .strip_prefix('`')
                        .and_then(|value| value.strip_suffix('`'))
                        .unwrap_or_else(|| panic!("parity expected value is not code-formatted: {observed}"))
                        .to_string(),
                )
            };

            (expr, outcome)
        })
        .collect()
}

#[test]
fn compiler_corpus_matches_documented_parity_table() {
    let actual = tagged_fixtures();
    let documented = documented_fixtures();

    assert_eq!(
        actual.len(),
        documented.len(),
        "conformance.lisp's compiler-corpus fixture count changed ({} found, {} documented) -- \
         did external/sens add/remove a tagged fixture? Update docs/compiler-oracle-corpus-parity-2026-09-11.md",
        actual.len(),
        documented.len()
    );

    for (expr, outcome) in &documented {
        let found = actual
            .iter()
            .find(|(actual_expr, _)| actual_expr == expr)
            .unwrap_or_else(|| {
                panic!(
                    "documented fixture not found in conformance.lisp (removed or expr text \
                     changed): {expr}"
                )
            });
        assert_eq!(
            &found.1, outcome,
            "compiler-corpus fixture's outcome drifted from the documented parity table: {expr}"
        );
    }
}
