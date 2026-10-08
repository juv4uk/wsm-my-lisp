# Compiler oracle corpus parity — wsm-my-lisp#4 (2026-09-11)

Submodule `external/sens` bumped `ccacc68` → `a5ade0b` (my-lisp's own
current semantic-projection commit) so this repo consumes the current
`(compiler-corpus . t)`-tagged fixture set, rather than an earlier commit
that predates that tagging. This document
is the three-column observation table #4's acceptance criteria ask
for, built by reading `wsm-my-lisp/harness/src/harness_*.rs` directly
against `external/sens/tests/fixtures/conformance.lisp`'s 17 tagged
records — not assumed or copied from an earlier audit.

## Columns

- **my-lisp oracle**: the fixture's own `expr`/`expected` (or `error`)
  from `conformance.lisp`, unchanged, my-lisp's own authority.
- **WSM/meta**: whether `external/sens/lib/meta-eval.lisp` (the
  self-hosted evaluator source) is expected to cover this shape at all
  — all 17 are ordinary evaluation, so this column is `yes` throughout;
  it exists structurally per #4's ask, not because any fixture here is
  actually out of meta-eval's scope.
- **compiled/native target**: the actual `wsm-my-lisp/harness/`
  witness, if any, checked against the *exact* expression and value —
  not "a harness exists nearby."

## Status legend (per #4's acceptance: unsupported must be explicit, not silently dropped or faked)

- **confirmed** — a harness executes this *exact* expression and its
  printed/decoded result matches this fixture's `expected`/`error`.
- **related** — a harness exercises the same target primitive
  (`wsm_cons`/`wsm_car`/etc.) but on different literal values or a
  different expression shape, so it is evidence the mechanism works,
  not evidence this specific fixture passes.
- **pending** — in scope for the current Stage2/asm nucleus, no harness
  written yet. (`cond` is the fixture actively being worked by `cml`,
  compiling real `meta-eval.lisp` source — see `wsm-my-lisp#15`/`#6`.)
- **unsupported** — outside current nucleus capability (rational
  arithmetic, macros) — not a bug, a scope boundary.

## The table

All 17 `compiler-corpus`-tagged records, verified by `grep -n
'compiler-corpus' external/sens/tests/fixtures/conformance.lisp` at
this commit — no row omitted or collapsed.

| # | expr | expected/error | WSM/meta | compiled/native target | status |
|---|---|---|---|---|---|
| 1 | `(quote radio)` | `radio` | yes | `harness-quote` (added 2026-09-12) — real `cml x86-asm` CLI compile of a file containing this exact source; linked against this repo's `asm/nucleus.s`, executed, returns `radio` | **confirmed** |
| 2 | `(00000010 (quote radio))` | `(1)` | yes | current exact-D1 ATOM corpus row; `harness-atom` is historical/compatibility evidence only until #78 migrates the native result carrier | related |
| 3 | `(00000011 (quote radio) (quote radio))` | `(1)` | yes | current exact-D1 EQ corpus row; the historical `harness-eq-symbol` proves target word identity only, not the new PredicateBit result carrier | related |
| 4 | `(car (quote (radio antenna)))` | `radio` | yes | `harness-cons` calls `wsm_car` on `(cons (quote A) (quote B))`'s result, different literals/shape | related |
| 5 | `(cdr (quote (radio antenna)))` | `(antenna)` | yes | `harness-cons` calls `wsm_cdr` similarly, different literals/shape | related |
| 6 | `(cons (quote radio) (quote (antenna)))` | `(radio antenna)` | yes | `harness-cons` proves `(cons (quote A) (quote B))` → `(A . B)` — different structure (atom+list here vs atom+atom there) | related |
| 7 | `(cond (() (quote wrong)) (t (quote right)))` | `right` | yes | `harness-cond` (added 2026-09-12) — real cml front-end compile (`parser::parse` → `lower::lower_program` → `x86_freestanding`), not hand-built IR; linked against this repo's `asm/nucleus.s`, executed, returns `right` | **confirmed** |
| 8 | `(/ 5 6 8 7)` | `5/336` | yes | none — no rational-arithmetic asm primitive exists in `asm/nucleus.s` | unsupported |
| 9 | `(00000011 (lambda (x) x) (lambda (x) x))` | `(0)` | yes | current exact-D1 EQ corpus row; `harness-closure` proves closure identity via direct `wsm_closure_new` ABI calls, while PredicateBit projection remains part of #78/#618 | related |
| 10 | `(defmacro foo)` | error `Arity` | yes | none — no macro support in the asm nucleus | unsupported |
| 11 | `(def count-down (lambda (n) (cond ((00000011 n 0) (quote done)) (t (count-down (- n 1)))))) (count-down 100000)` | `done` | yes | `harness-countdown` — same bounded self-tail-recursion shape; current corpus uses the binary EQ identity directly | **confirmed** |
| 12 | `((lambda (a b . rest) rest) 1 2 3 4 5)` | `(3 4 5)` | yes | `cml` commit `1104657` (`tests/x86_top_level_let_test.rs::row12_corpus_fixture_exact_witness`) proves variadic lambda application with right-to-left `wsm_cons` folding, returning `(3 4 5)` linked against `asm/nucleus.s` | **confirmed** |
| 13 | `((lambda args args) 1 2 3)` | `(1 2 3)` | yes | `cml` commit `1104657` (`tests/x86_top_level_let_test.rs::row13_corpus_fixture_exact_witness`) proves all-rest lambda application with right-to-left `wsm_cons` folding, returning `(1 2 3)` linked against `asm/nucleus.s` | **confirmed** |
| 14 | `((lambda (a b . rest) a) 1)` | error `Arity` | yes | `cml` commit `1104657` (`tests/x86_freestanding_test.rs::variadic_lambda_under_arity_is_rejected`) proves fail-closed compile-time rejection with `InvalidArity { expected: 2, actual: 1 }` | **confirmed** |
| 15 | `(let ((second (lambda (x) (quote shadowed)))) (second (quote (1 2 3))))` | `shadowed` | yes | `cml` commit `6ce577f` (`tests/x86_top_level_let_test.rs`) proves top-level lexical `let` closure application within its body, linked against `asm/nucleus.s`, returning `shadowed` | **confirmed** |
| 16 | `(let ((car (lambda (x) (quote shadowed)))) (car (quote (1 2))))` | error `InvalidForm` | yes | none — Canon 0+7 spellings unshadowable (Contract 6.0) | pending |
| 17 | `(defmacro my-list items (cons (quote quote) (cons items (quote ())))) (my-list 1 2 3)` | `(1 2 3)` | yes | none — no macro support in the asm nucleus | unsupported |

Earlier draft of this table under-counted at 12 rows, silently dropping
5 real fixtures (12–16 above) — caught and corrected before commit,
noted here rather than erased, since #4's acceptance specifically
requires unsupported/uncovered fixtures to be named, not quietly
absent.

## Honest summary

**7 of these fixtures (`quote` [row 1], `cond` [row 7],
`count-down`/`countdown` [row 11], variadic rest list [row 12],
all-rest list [row 13], variadic under-arity error [row 14], and
`let` closure application [row 15]) have a byte-exact compiled/native
witness today.** `quote` and `cond` closed via `wsm-os-lisp`/`cml`
discoveries on 2026-09-12 — `cml`'s CLI already compiled a bare
`(quote radio)` file, and `wsm-os-lisp` had already added bounded
`Ir::Cond`/`Ir::Quote` support to `cml`'s `x86_freestanding` backend
for its own M5A milestone; neither needed the general first-class
application support that's still genuinely missing for the *real*
`my-eval-cond` function inside `meta-eval.lisp`. Row 15 closed via
`cml` commit `6ce577f`, and rows 12–14 closed via `cml` commit
`1104657` (admitting direct variadic and all-rest lambda application
with `wsm_cons` list packing and fail-closed arity checking).
Everything else is either `related` (same primitive proven, different
literal — real evidence of mechanism, not of the specific fixture) or
`pending`/`unsupported` (explicitly named, per #4's acceptance, rather
than silently assumed passing).

## What would move a `related`/`pending` row to `confirmed`

Not proposed here as new work — this is descriptive, not a work
commitment — but for a future session picking this up: the honest gap
between `related` and `confirmed` for rows 2/3/4/5/6/9 is almost always
"the harness needs a CML-generated entry stub for *this exact
expression's* literals" (a mechanical CML-invocation step, not a new
asm primitive) — rows 1 and 7 went through exactly this process and
turned out to already work, and rows 12–15 turned out to already work
too once `cml` admitted general fixed/variadic lambda application.
Row 16 (`pending`) is the one real remaining gap in this category:
`let`-shadowing of a Canon 0+7 name (`car`) rejected as `InvalidForm`
— needs a witness proving the compiler actually fails closed on this,
not just that the oracle expects it to. Rows 8/10/17 (`unsupported`)
need new nucleus capability (rational arithmetic, macros) and are out
of current scope per `docs/AUTHORITY.md`, not next steps.

## Parity gate note (#4 acceptance: "deliberate mutation of one
result/error/provenance field is caught")

**Partially closed, 2026-09-12**:
`harness/tests/compiler_corpus_parity.rs` mechanically checks that
every `(compiler-corpus . t)` fixture in `conformance.lisp` still has the
exact `expr`/`expected`/`error` this table's rows document — it
fails closed (wrong count, missing fixture, or changed outcome) if
my-lisp mutates a tagged fixture without this table being updated to
match. Run explicitly in the fast PR-gate workflow
(`self-hosting-authority.yml`), not just the nightly deep gate, so
drift is caught on every PR.

**Closed 2026-10-08**: the Rust gate no longer carries its own third hand-copied
`documented_fixtures()` list. It parses these 17 Markdown data rows directly
and compares them against the pinned SENS `(compiler-corpus . t)` records.
The remaining duplication is intentional and visible: SENS owns the executable
corpus; this document owns the human-readable consumer-status table. A change
to either side without the other now fails the PR gate.
