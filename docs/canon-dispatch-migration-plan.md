# Canon dispatch migration plan: numeric ID + generated projection, not hardcoded spellings

**2026-09-11 · owner-commissioned, relayed via `my-lisp-cyberpunk`**

This is a plan document, not code. It proposes how every consumer of
`semantic-registry.wsm` (this repo, `cml`, `my-lisp-cyberpunk`, and any
future consumer) should recognize Canon symbols going forward, and
names the concrete follow-up work each repo would need to do to get
there. Nothing here is committed to code by this document alone —
implementation is tracked as separate issues per repo, referenced below.

## The problem, stated once so it isn't rediscovered per-repo

`semantic-registry.wsm` is the single authority mapping a numeric
semantic ID (e.g. `0001`) to its equally-valid surface spellings across
languages (`en`/`uk`/`sa`/`sym`) and, separately, a *display* casing
convention some consumers layer on top (e.g. uppercasing for a
radio/device-name style surface).

Three real bugs have now been found, independently, in three different
consumers, all the same root cause — a consumer re-deriving or
hand-copying "which spellings count as X" instead of asking the
registry:

1. **wsm-my-lisp** (`dll/eval.rs`, fixed 2026-09-10 → 2026-09-11): the
   Rust evaluator originally recognized only the English ASCII spelling
   of `quote`/`cond`. A real Cyberpunk script
   (`перший-зріз.мій`) used the `uk` spelling `як-є` and failed to
   evaluate. First fix hardcoded all 4 known spellings per form via
   `matches!` — itself flagged by the owner as a new instance of the
   same copy-paste-drift problem, since the list silently goes stale
   the moment the registry changes. Real fix: `dll/build.rs` (now
   migrated to `my-lisp-cyberpunk/host-runtime/build.rs`) parses
   `semantic-registry.wsm` at build time and generates a
   `canon_spellings.rs` const-slice file, consumed via `include!`.
   Deliberately scoped to only the two Canon ids the evaluator
   special-cases (`0001` quote, `0007` cond) — not a general registry
   importer.
2. **cml** (`cml#9`, closed 2026-09-11): same shape of bug, same fix
   pattern applied independently after seeing this repo's `build.rs` —
   a sibling-checkout-relative build script generates
   `CANON_UPPER_SURFACES`/`CANON_EXACT_SURFACES` for the 7 ids (`0001`-
   `0007`) cml's own compiler special-cases, fail-closed if the
   registry is missing an expected entry.
3. **cml** (found just now, not yet its own issue): a *display-casing*
   collision — `radio` and `RADIO` both uppercase to the same string
   and were treated as identical symbols by a consumer that uppercases
   for comparison, when they are in fact distinct symbols in
   case-sensitive Symbol identity (`wsm-my-lisp`'s own `Tag::Symbol`
   carries no case-folding). This is a *different* failure mode from
   (1)/(2) — those were "which Canon spellings match a fixed special
   form," this is "two distinct ordinary symbols collapsed by a naive
   normalization step" — but it's the same underlying discipline
   failure: a consumer derived an identity/equality rule by local
   transformation (uppercase-and-compare) instead of consulting
   registry-declared identity, which for ordinary (non-Canon) symbols
   is exact string equality, full stop — there is no registry entry
   that licenses case-insensitive comparison for anything outside the
   documented Canon-spelling-equivalence relation itself.

## What "canon + table of functions" means concretely

Two genuinely different mechanisms are being asked for here, and
conflating them is exactly how bug (3) happened:

- **Canon spelling equivalence** (mechanism 1: what `build.rs`
  generates today): a *closed, registry-declared* set of strings that
  all denote the same numeric semantic ID — `quote`/`як-є`/`svarūpa`/
  `'` are one form. This equivalence is opt-in per ID (only ids a
  consumer actually special-cases need a generated table) and is
  **never** derived by a local text transform (uppercasing, stripping
  accents, case-folding) — it is always a direct lookup against
  `semantic-registry.wsm`'s own listed spellings for that ID.
- **Ordinary symbol identity** (what bug (3) violated): for any string
  that is *not* itself one of a Canon ID's registry-declared spellings,
  identity is exact string equality. `radio` and `RADIO` are two
  different symbols unless and until the registry itself declares them
  as alternate spellings of the same ID (it does not, and should not —
  case variation is not language-surface variation). No consumer should
  run its own normalization pass over ordinary symbol text for
  comparison purposes.

The generalization the owner is asking for is **not** "make every
symbol dispatch through the registry" (most symbols, like a Cyberpunk
mod's `radio` device name, aren't Canon at all and never touch
`semantic-registry.wsm`) — it is: **wherever a consumer's code today
encodes a hardcoded belief about which spellings are equivalent to
which, or which strings should compare equal after some transform,
that belief must instead come from a generated projection of the
registry, checked at build time, or from exact string equality with no
transform.** The `build.rs` pattern is the template for the former;
the fix for bug (3) is simply to delete the uppercase-and-compare step
and use exact string equality, which needs no registry involvement at
all.

## Consumer-by-consumer status and next step

| Consumer | Current state | Next step | Owner |
|---|---|---|---|
| `wsm-my-lisp` | Migrated away — Canon-spelling dispatch lived in `dll/eval.rs`, deleted at Phase D of `wsm-my-lisp#15` (2026-09-11); the pattern now lives in `my-lisp-cyberpunk/host-runtime/build.rs` | none — no Canon-spelling consumer remains in this repo | n/a |
| `my-lisp-cyberpunk` (`host-runtime`) | Has the generated `canon_spellings.rs` (2 ids: quote, cond), migrated intact from `wsm-my-lisp` | if/when `host-runtime`'s evaluator special-cases a third Canon form, extend `build.rs`'s `emit_entry` calls — parser is already generic over "which numeric id" | `my-lisp-cyberpunk` session |
| `cml` | Has its own generated `CANON_UPPER_SURFACES`/`CANON_EXACT_SURFACES` (7 ids: `0001`-`0007`), closed as `cml#9` | investigate and fix the newly-found `radio`/`RADIO` collision as its own issue — likely a straight deletion of an uppercase-normalization step in symbol comparison, not a registry change | `cml` session |
| `my-lisp` | Registry owner; no generated-projection consumer of its own noted so far (it *is* the source of truth, not a consumer) | none identified yet; flag if `my-lisp` itself is found to hardcode a Canon-spelling list anywhere in its own tooling | `my-lisp` session |

## Standing principle for future consumers

Any new repo or module that needs to recognize a Canon special form by
any of its surface spellings should:

1. Treat `semantic-registry.wsm` as the only source for "which
   spellings are equivalent" — never hand-list spellings in a
   `matches!`/`switch`/`if`-chain, even if the list looks small and
   stable today (it looked small and stable in `wsm-my-lisp/dll/eval.rs`
   too, until a real script used a spelling the list omitted).
2. Generate the projection at build time (or an equivalent
   checked-at-startup step for a non-Rust consumer) from a
   relative-path sibling checkout of `my-lisp`, fail-closed if an
   expected ID or spelling is missing — not fail-open/silently-empty.
3. Scope the generated table to only the IDs the consumer actually
   special-cases, not a general import of the whole registry — per the
   owner's explicit instruction against `wsm-my-lisp/build.rs` growing
   into "not a general registry importer."
4. Never introduce a case-folding, accent-stripping, or other
   text-normalization step as a substitute for step 1/2 — if two
   strings need to compare equal, that equivalence must already be
   registry-declared (Canon spelling) or be exact string equality
   (everything else). This is the fix for the `radio`/`RADIO` class of
   bug specifically.

## Explicitly out of scope for this document

- Rewriting `cml`'s or `host-runtime`'s existing generated tables — both
  already follow the correct pattern; this document doesn't ask either
  to change anything except cml's new case-folding bug.
- Extending `semantic-registry.wsm`'s own schema — the registry format
  is unchanged; this is a plan for *consuming* it correctly, not for
  authoring it differently.
- A shared crate/library that all consumers import instead of each
  generating their own `build.rs` — that would be a real simplification
  worth considering later (three near-identical S-expression parsers
  now exist independently), but is a separate proposal with its own
  cross-repo coordination cost, not assumed here.
