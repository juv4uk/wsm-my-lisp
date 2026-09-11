//! Generates `$OUT_DIR/canon_spellings.rs`: the accepted surface
//! spellings for Canon special forms (currently `quote`=`0001`,
//! `cond`=`0007`), parsed directly from my-lisp's own
//! `lib/surface/semantic-registry.wsm` at build time -- NOT hand-copied
//! into eval.rs as a `matches!` list.
//!
//! Why this exists: eval.rs originally hardcoded
//! `matches!(name, "quote" | "як-є" | "svarūpa" | "'")` after finding a
//! real bug (my-lisp-cyberpunk's `перший-зріз.мій` uses the `uk`
//! spelling `як-є`, which the hardcoded English-only check didn't
//! recognize). That fix was correct, but the owner's own follow-up
//! review named the fix itself as a new instance of exactly the
//! copy-paste-drift problem wsm-my-lisp#4 exists to prevent ("Не
//! допустити ручного дублювання expected values" -- the same acceptance
//! criterion #4 states for oracle fixtures applies here too: these
//! spellings are a semantic fact my-lisp owns, not wsm-my-lisp's to
//! fork by hand). This file is the fix: a generated, checked projection
//! instead of a hardcoded list eval.rs's own comment would otherwise
//! have to remember to update by hand.
//!
//! Scope deliberately narrow (per owner instruction: "не додавати ще 20
//! matches! варіантів" -- don't grow this into a general registry
//! importer): only the two Canon ids eval.rs actually special-cases
//! today. Adding a third later means adding one more `emit_entry` call
//! below, not new parsing logic -- the parser itself is already generic
//! over "which numeric id."

use std::env;
use std::fmt::Write as _;
use std::fs;
use std::path::Path;

const REGISTRY_PATH: &str = "../external/my-lisp/lib/surface/semantic-registry.wsm";

/// Minimal untyped S-expression -- just enough to walk
/// `semantic-registry.wsm`'s own shape (a big list of `(id (lang word
/// status) ...)` entries), not a general Lisp reader. Deliberately
/// simpler than `dll/src/reader.rs` (no fixnum/string distinction
/// needed here -- everything is either a sub-list or an opaque token).
enum Sexpr {
    Atom(String),
    List(Vec<Sexpr>),
}

fn tokenize(source: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut chars = source.chars().peekable();
    while let Some(&c) = chars.peek() {
        match c {
            c if c.is_whitespace() => {
                chars.next();
            }
            ';' => {
                // Line comment, same convention as semantic-registry.wsm's
                // own header lines.
                while let Some(&c) = chars.peek() {
                    if c == '\n' {
                        break;
                    }
                    chars.next();
                }
            }
            '(' | ')' => {
                chars.next();
                tokens.push(c.to_string());
            }
            _ => {
                let mut atom = String::new();
                while let Some(&c) = chars.peek() {
                    if c.is_whitespace() || c == '(' || c == ')' || c == ';' {
                        break;
                    }
                    atom.push(c);
                    chars.next();
                }
                tokens.push(atom);
            }
        }
    }
    tokens
}

fn parse(tokens: &[String], pos: &mut usize) -> Sexpr {
    match tokens.get(*pos).map(|s| s.as_str()) {
        Some("(") => {
            *pos += 1;
            let mut items = Vec::new();
            while tokens.get(*pos).map(|s| s.as_str()) != Some(")") {
                if *pos >= tokens.len() {
                    panic!("semantic-registry.wsm: unbalanced parentheses while parsing");
                }
                items.push(parse(tokens, pos));
            }
            *pos += 1; // consume ")"
            Sexpr::List(items)
        }
        Some(")") => panic!("semantic-registry.wsm: unexpected ')'"),
        Some(atom) => {
            *pos += 1;
            Sexpr::Atom(atom.to_string())
        }
        None => panic!("semantic-registry.wsm: unexpected end of input"),
    }
}

/// Finds the entry `(id (lang word status) ...)` for `target_id` inside
/// the registry's top-level form and returns every `word` whose `status`
/// is not `missing` (a `—` placeholder word always carries `missing`, so
/// filtering on status alone is sufficient -- no separate `—` check
/// needed). Panics (fails the build) if `target_id` isn't found at all,
/// per this ecosystem's fail-closed convention -- a silently-empty
/// spelling list would be worse than a build error.
fn spellings_for_id(root: &Sexpr, target_id: &str) -> Vec<String> {
    let Sexpr::List(top) = root else {
        panic!("semantic-registry.wsm: expected the whole file to be one list");
    };
    // top[0] is the "sr/1" schema-version atom; entries follow.
    for entry in &top[1..] {
        let Sexpr::List(fields) = entry else { continue };
        let Some(Sexpr::Atom(id)) = fields.first() else { continue };
        if id != target_id {
            continue;
        }
        let mut spellings = Vec::new();
        for surface in &fields[1..] {
            let Sexpr::List(parts) = surface else { continue };
            // Each surface form is (lang word status) or (lang word status extra...),
            // e.g. defmacro's (compat defmacro-derived compatibility-only) tag --
            // only lang/word/status (first 3) matter for spelling extraction.
            let [Sexpr::Atom(_lang), Sexpr::Atom(word), Sexpr::Atom(status), ..] = parts.as_slice()
            else {
                continue;
            };
            if status != "missing" {
                spellings.push(word.clone());
            }
        }
        return spellings;
    }
    panic!("semantic-registry.wsm: no entry found for id {target_id} -- registry shape changed?");
}

fn emit_entry(out: &mut String, const_name: &str, root: &Sexpr, id: &str) {
    let spellings = spellings_for_id(root, id);
    assert!(
        !spellings.is_empty(),
        "semantic-registry.wsm: id {id} has zero non-missing spellings -- registry regressed?"
    );
    write!(out, "pub const {const_name}: &[&str] = &[").unwrap();
    for word in &spellings {
        write!(out, "{word:?}, ").unwrap();
    }
    writeln!(out, "];").unwrap();
}

fn main() {
    println!("cargo:rerun-if-changed={REGISTRY_PATH}");
    println!("cargo:rerun-if-changed=build.rs");

    let source = fs::read_to_string(REGISTRY_PATH).unwrap_or_else(|e| {
        panic!("failed to read {REGISTRY_PATH} (is the external/my-lisp submodule initialized? `git submodule update --init`): {e}")
    });
    let tokens = tokenize(&source);
    let mut pos = 0;
    let root = parse(&tokens, &mut pos);

    let mut out = String::new();
    out.push_str("// Generated by dll/build.rs from external/my-lisp/lib/surface/semantic-registry.wsm.\n");
    out.push_str("// DO NOT EDIT BY HAND -- see build.rs's own module doc for why this exists.\n");
    emit_entry(&mut out, "QUOTE_SPELLINGS", &root, "0001");
    emit_entry(&mut out, "COND_SPELLINGS", &root, "0007");

    let out_dir = env::var("OUT_DIR").unwrap();
    let dest = Path::new(&out_dir).join("canon_spellings.rs");
    fs::write(&dest, out).expect("failed to write generated canon_spellings.rs");
}
