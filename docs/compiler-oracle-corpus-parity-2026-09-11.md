# Compiler oracle corpus parity — wsm-my-lisp#4 (2026-09-11)

Submodule `external/my-lisp` bumped `ccacc68` → `6d71151` (my-lisp's own
`#67` corpus-freeze commit) so this repo consumes the same
`(compiler-corpus . t)`-tagged fixture set my-lisp actually froze,
rather than an earlier commit that predates that tagging. This document
is the three-column observation table #4's acceptance criteria ask
for, built by reading `wsm-my-lisp/harness/src/harness_*.rs` directly
against `external/my-lisp/tests/fixtures/conformance.my`'s 17 tagged
records — not assumed or copied from an earlier audit.

## Columns

- **my-lisp oracle**: the fixture's own `expr`/`expected` (or `error`)
  from `conformance.my`, unchanged, my-lisp's own authority.
- **WSM/meta**: whether `external/my-lisp/lib/meta-eval.my` (the
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
  compiling real `meta-eval.my` source — see `wsm-my-lisp#15`/`#6`.)
- **unsupported** — outside current nucleus capability (rational
  arithmetic, macros) — not a bug, a scope boundary.

## The table

All 17 `compiler-corpus`-tagged records, verified by `grep -n
'compiler-corpus' external/my-lisp/tests/fixtures/conformance.my` at
this commit — no row omitted or collapsed.

| # | expr | expected/error | WSM/meta | compiled/native target | status |
|---|---|---|---|---|---|
| 1 | `(quote radio)` | `radio` | yes | none | pending |
| 2 | `(atom (quote radio))` | `t` | yes | `harness-atom` tests `(atom (quote ()))`, different literal | related |
| 3 | `(eq (quote radio) (quote radio))` | `t` | yes | `harness-eq` tests fixnum `(eq 41 41)`, not Symbol `eq` | related |
| 4 | `(car (quote (radio antenna)))` | `radio` | yes | `harness-cons` calls `wsm_car` on `(cons (quote A) (quote B))`'s result, different literals/shape | related |
| 5 | `(cdr (quote (radio antenna)))` | `(antenna)` | yes | `harness-cons` calls `wsm_cdr` similarly, different literals/shape | related |
| 6 | `(cons (quote radio) (quote (antenna)))` | `(radio antenna)` | yes | `harness-cons` proves `(cons (quote A) (quote B))` → `(A . B)` — different structure (atom+list here vs atom+atom there) | related |
| 7 | `(cond (() (quote wrong)) (t (quote right)))` | `right` | yes | none — this is exactly the fixture `cml` is compiling from real `meta-eval.my` source right now | pending |
| 8 | `(/ 5 6 8 7)` | `5/336` | yes | none — no rational-arithmetic asm primitive exists in `asm/nucleus.s` | unsupported |
| 9 | `(eq (lambda (x) x) (lambda (x) x))` | `()` | yes | `harness-closure` proves closure identity via direct `wsm_closure_new` ABI calls, not through evaluated `(lambda ...)` sugar | related |
| 10 | `(defmacro foo)` | error `Arity` | yes | none — no macro support in the asm nucleus | unsupported |
| 11 | `(def count-down (lambda (n) (cond ((eq n 0) (quote done)) (t (count-down (- n 1)))))) (count-down 100000)` | `done` | yes | `harness-countdown` — same shape (bounded self-tail-recursion, 100k), CML-generated entry, linked only against `asm/nucleus.s` | **confirmed** |
| 12 | `((lambda (a b . rest) rest) 1 2 3 4 5)` | `(3 4 5)` | yes | none — dotted/variadic lambda-list binding | pending |
| 13 | `((lambda args args) 1 2 3)` | `(1 2 3)` | yes | none — bare-symbol lambda-list binding | pending |
| 14 | `((lambda (a b . rest) a) 1)` | error `Arity` | yes | none — variadic lambda still enforcing fixed-param arity | pending |
| 15 | `(let ((second (lambda (x) (quote shadowed)))) (second (quote (1 2 3))))` | `shadowed` | yes | none — ordinary-binding shadowing through `let`'s lambda-lowering | pending |
| 16 | `(let ((car (lambda (x) (quote shadowed)))) (car (quote (1 2))))` | error `InvalidForm` | yes | none — Canon 0+7 spellings unshadowable (Contract 6.0) | pending |
| 17 | `(defmacro my-list items (cons (quote quote) (cons items (quote ())))) (my-list 1 2 3)` | `(1 2 3)` | yes | none — no macro support in the asm nucleus | unsupported |

Earlier draft of this table under-counted at 12 rows, silently dropping
5 real fixtures (12–16 above) — caught and corrected before commit,
noted here rather than erased, since #4's acceptance specifically
requires unsupported/uncovered fixtures to be named, not quietly
absent.

## Honest summary

**1 of these fixtures (`count-down`/`countdown`) has a byte-exact
compiled/native witness today.** Everything else is either `related`
(same primitive proven, different literal — real evidence of
mechanism, not of the specific fixture) or `pending`/`unsupported`
(explicitly named, per #4's acceptance, rather than silently assumed
passing). This matches the state found in an earlier informal
cross-reference; this document makes it a checked-in artifact instead
of session-local knowledge.

## What would move a `related`/`pending` row to `confirmed`

Not proposed here as new work — this is descriptive, not a work
commitment — but for a future session picking this up: the honest gap
between `related` and `confirmed` for rows 2/3/4/5/6/9 is almost always
"the harness needs a CML-generated entry stub for *this exact
expression's* literals" (a mechanical CML-invocation step, not a new
asm primitive). Rows 1/7/12-16 (`pending`) need a witness written from
scratch, not just re-targeted literals — row 7 (`cond`) is the one
`cml` is actively compiling from real `meta-eval.my` source right now.
Rows 8/10/17 (`unsupported`) need new nucleus capability (rational
arithmetic, macros) and are out of current scope per
`docs/AUTHORITY.md`, not next steps.

## Parity gate note (#4 acceptance: "deliberate mutation of one
result/error/provenance field is caught")

No automated gate exists yet enforcing this table stays honest as
`conformance.my` changes upstream — that would require the
build-time-generated-projection mechanism this repo's
`docs/canon-dispatch-migration-plan.md` already names as the
standing discipline (never hand-copy a fact the registry/fixture file
owns). A future increment could generate this table's `status` column
mechanically from a `(wsm-native . (...))`-shaped key on each fixture
record, per `docs/shared-oracle-corpus-fixture-format-proposal.md`'s
original proposal — deferred here, not attempted in this document,
since it needs my-lisp's agreement on the exact key shape (my-lisp
chose `compiler-corpus . t` as a boolean marker for #67, not yet the
richer per-consumer status object that proposal sketched).
