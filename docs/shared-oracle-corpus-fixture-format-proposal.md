# Shared oracle corpus fixture format — proposal, ground-prep for my-lisp#67

Status: **proposal only, not implemented, not requested by my-lisp yet** --
my-lisp is actively working on #66 (a prerequisite for #67, the shared
compiler oracle corpus wsm-my-lisp#4 depends on) and explicitly asked to
hold this idea until they reach #67 rather than build it now. Written in
advance so it's ready to hand over, not to preempt their design.

## What already exists, read directly, not assumed

- `external/my-lisp/tests/fixtures/conformance.my`: one flat alist per
  fixture, one fixture per top-level form -- `(expr . "...") (expected .
  "...") (tier . N) (axioms . (...)) (role . "...")` (occasionally
  `(error . "...")` instead of `expected`, and a free-text `(note . "...")`).
  227 fixtures currently, per my-lisp's own count. This is the "my-lisp
  oracle" column #4 asks for, already in a stable, appendable format.
- `wsm-my-lisp/harness/src/harness_*.rs`: this repo's own pattern for a
  *compiled/native target* observation -- link a CML-generated `.s` entry
  file against `asm/nucleus.s`, execute it, decode the resulting Word,
  compare against a hardcoded expected value (see `harness_countdown.rs`:
  `assert_eq!(symbol, "done", ...)`). This is the "compiled/native target"
  column, but today each harness hand-codes its own single expected value
  -- there's no shared table linking a harness back to the exact
  `conformance.my` fixture it's supposed to match.

## The proposal: extend `conformance.my`'s own alist, don't fork a parallel file

#4's acceptance criterion is explicit: "one corpus reifies through both
repos without manual duplication." The simplest way to satisfy that,
given what already exists: add an *optional* key to the SAME fixture
record in `conformance.my` -- not a second file wsm-my-lisp would have to
keep in sync by hand. Something shaped like:

```lisp
((expr . "(atom (quote ()))") (expected . "t") (tier . 1) (axioms . (G2))
 (role . "constitutive")
 (wsm-native . ((status . confirmed) (harness . "harness-atom")
                (witness-commit . "wsm-my-lisp@bcd3d5d"))))
```

- `status`: one of `confirmed` (an executed harness proves this fixture
  on `asm/nucleus.s`, matching this record's own `expected`/`error`),
  `pending` (in scope, no witness yet), or `unsupported` (out of current
  scope -- e.g. macros/Advice-Taker provenance before Stage2/3 land that
  far) -- #4's acceptance explicitly requires unsupported cases to be
  named, not silently dropped or faked as passing.
- `harness`: which `wsm-my-lisp/harness/src/harness_*.rs` binary is the
  actual executable evidence, so "which code proves this fixture" is a
  lookup, not tribal knowledge.
- `witness-commit`: pins the exact wsm-my-lisp commit the witness was
  last confirmed against -- lets a drift check say "this fixture's
  `expected` changed since the witness was last run," not just "some
  witness somewhere exists."

**Why extending rather than forking**: `conformance.my` remains the one
place a fixture's oracle value lives -- my-lisp owns and edits that
column exactly as they do today. `wsm-my-lisp` would only ever add/update
its own `wsm-native` key on a record it doesn't otherwise touch, the same
way `tier`/`axioms`/`role` already coexist as "my-lisp's own
classification of the same contract" per that file's own header comment.
No second table to keep byte-identical to the first by hand -- the
"single canonical fact + additive per-consumer columns" shape the file
already uses.

## What #4's parity gate could look like, concretely, once this exists

A small check (CI or a documented script, matching #4's "fast corpus
stays PR-CI-suitable" acceptance) that:

1. Parses `conformance.my`'s alist form (already Lisp -- either read via
   my-lisp itself, or a minimal Rust s-expression reader if wsm-my-lisp
   needs to consume it without shelling out to a `my-lisp` binary).
2. For every record with `(wsm-native . ((status . confirmed) ...))`,
   confirms the named `harness` binary exists, builds, runs, and its
   printed/decoded result matches that record's own `expected` --
   **not** a separately hand-typed copy of the same string.
3. Flags (fail-closed, per #4's acceptance) any record whose `expected`
   changed without its `witness-commit` being bumped -- catches drift
   between my-lisp's own oracle and wsm-my-lisp's stale evidence, which
   is exactly the failure mode #4 exists to prevent.

Not designed further than this outline -- the actual parser/CI mechanics
are implementation, appropriately deferred until my-lisp reaches #67 and
the two sides can agree on the exact shape together, not decided
unilaterally here.
