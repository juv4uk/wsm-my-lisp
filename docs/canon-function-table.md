# Canon function table — wsm-my-lisp#16 (2026-09-12)

Machine-readable-in-spirit table of every canonical identity this repo
actually implements or witnesses on `asm/nucleus.s` — not a
speculative full-language table. Scope is deliberately narrow to what
this repo owns per `docs/AUTHORITY.md`: proving conformant execution
on the target without a Rust evaluator, not being semantic authority.
Canonical `id`/`surfaces` columns are read directly from
`external/my-lisp/lib/surface/semantic-registry.wsm` at the currently
pinned commit (`6d71151`) — not retyped from memory, and not owned
here (my-lisp owns that file; this table only projects it).

## Columns

- **canonical identity**: the `en` surface, used only as a stable
  human label for this table's own rows — not itself the identity.
- **semantic id**: the numeric id from `semantic-registry.wsm` — the
  actual identity.
- **surfaces**: every registered spelling for that id, verbatim.
- **asm symbol**: the exported `asm/nucleus.s` label, or `—` if this
  repo implements no asm projection for this identity at all.
- **support status**: `native` (asm primitive exists and is witnessed),
  `witnessed-only` (proven through a harness but not a standalone
  primitive — e.g. `cond`/`lambda` application, proven only as part of
  a larger compiled fixture), or `not-in-scope` (this repo does not,
  and per `AUTHORITY.md` should not, implement this — dispatch for it
  lives in `my-lisp-cyberpunk`/`cml`, not here).
- **provenance**: which commit/fixture the asm projection or witness
  traces back to.
- **executable witness**: the `harness/src/harness_*.rs` binary that
  actually runs this on real x86_64, if any.

## Table

| canonical identity | semantic id | surfaces | asm symbol | support status | provenance | executable witness |
|---|---|---|---|---|---|---|
| cons | 0004 | `en cons`, `uk сполучити`, `sa saṃyuj`, `sym :` | `wsm_cons` | native | hand-written, this repo, Stage 0 | `harness-cons` |
| car | 0005 | `en car`, `uk перше`, `sa ādi`, `sym :п` | `wsm_car` | native | hand-written, this repo, Stage 0 | `harness-cons` (calls `wsm_car` on its result) |
| cdr | 0006 | `en cdr`, `uk решта`, `sa śeṣa`, `sym :р` | `wsm_cdr` | native | hand-written, this repo, Stage 0 | `harness-cons` (calls `wsm_cdr` on its result) |
| eq | 0003 | `en eq`, `uk тотожне?`, `sa abheda`, `sym =?` | `wsm_eq` | native | hand-written, this repo, Stage 0 (Fixnum witness) / Stage 2 (Symbol witness, 2026-09-12) | `harness-eq` (Fixnum) + `harness-eq-symbol` (Symbol, closes the gap noted below) |
| atom | 0002 | `en atom`, `uk атом?`, `sa aṇu`, `sym .?` | `wsm_atom` | native | hand-written, this repo, Stage 0 | `harness-atom` |
| quote | 0001 | `en quote`, `uk як-є`, `sa svarūpa`, `sym '` | — | not-in-scope | dispatch lived in `dll/eval.rs`, deleted at Phase D (`1e1549a`); moved to `my-lisp-cyberpunk/host-runtime/build.rs` + `cml`'s compiler | none in this repo — CML resolves `quote` before lowering reaches `asm/nucleus.s`; nucleus never sees the surface spelling |
| cond | 0007 | `en cond`, `uk за-умовою`, `sa anukrama`, `sym ?:` | — | witnessed-only, not yet in this repo | same as `quote`: dispatch is CML's job, not the nucleus's; the fixture `(cond (() (quote wrong)) (t (quote right)))` is `pending` in `docs/compiler-oracle-corpus-parity-2026-09-11.md` (`cml` actively compiling it) | none yet — will be `harness-cond` once `cml` lands the compiled entry |
| lambda (application) | 0010 | `en lambda`, `uk функція` | `wsm_closure_new`, `wsm_closure_definition`, `wsm_closure_environment` | native (identity/ABI only, not general application) | hand-written, this repo, Stage 2 (`docs/ROADMAP.md`) | `harness-lambda` (pure identity fixture, no nucleus call needed) + `harness-closure` (Stage2 closure-identity ABI witness) |

## Honest gaps, named per this issue's own discipline ("невідомий ID fail-closed")

- ~~`eq` is only witnessed on `Fixnum`s, not `Symbol`s.~~ **Closed
  2026-09-12**: `harness-eq-symbol` now proves `wsm_eq` on two
  `Symbol` words (matching id vs. distinct id), mirroring the
  `compiler-corpus` fixtures `(eq (quote radio) (quote radio))` → `t`
  and `(eq (quote radio) (quote antenna))` → `()`. `wsm_eq`'s own asm
  (`asm/nucleus.s`) does a plain full-word compare with no tag-specific
  branch, so this was expected to already work — the gap was purely in
  witness coverage, not in the primitive itself, and this closes it.
- **No asm projection exists for `quote`/`cond` in this repo, by
  design** — those are CML-lowering-time forms, resolved away before
  `asm/nucleus.s` ever runs; a `wsm_cond`/`wsm_quote` primitive would
  be architecturally wrong here (`asm/nucleus.s` has no room to make a
  runtime branch decision that CML's own compiled control flow already
  encodes as jumps). Listed as `not-in-scope`, not `unsupported` —
  different reasons: `unsupported` (elsewhere in this repo's docs)
  means "the nucleus can't do this yet"; `not-in-scope` here means
  "the nucleus should never do this, another layer already does."
- **`lambda`'s row is architecture, not general application.** No
  witness here proves *arbitrary* `(f x y z)` application through a
  compiled closure call — `harness-lambda` is pure stack-only identity
  (no `wsm_*` primitive call at all, per its own doc comment) and
  `harness-closure` proves ABI/identity properties (`wsm_closure_new`/
  `_definition`/`_environment`), not invocation. General closure
  application is Stage 2's open work per `docs/ROADMAP.md`.

## Drift protection

This table is hand-maintained, not generated — a real limitation this
issue's acceptance criterion ("parity/mutation tests ловлять drift")
is not yet met. What partially substitutes today:
`harness/tests/semantic_authority.rs` fail-closes on `SYM_T_ID`
drifting from its ratified reserved value (`SYMBOL_ID_MAX`), which
covers the one canonical-`t`-identity mutation risk this repo's own
asm carries. No equivalent test yet asserts this *table's* own
correctness against `semantic-registry.wsm` mechanically — a future
increment could extend `scripts/check-lisp-first-authority.sh` (or a
small Rust test) to regenerate the surfaces column from the registry
and diff against this file, same discipline as
`docs/canon-dispatch-migration-plan.md` already names as standing
practice. Not attempted here; flagged rather than silently assumed
solved.
