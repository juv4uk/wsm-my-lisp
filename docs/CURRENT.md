# Current documentation entry point — wsm-my-lisp

**Start here.** On any conflict between this page (or the documents it
points to) and anything under [`docs/archive/`](archive/README.md),
current material wins unconditionally — archived documents are
historical context only, never a source of requirements.

## Authority order

1. **Language semantics** — owned entirely by `my-lisp`'s Canon /
   `external/my-lisp/lib/surface/semantic-registry.wsm`. This repo is
   never semantic authority (see [`AUTHORITY.md`](AUTHORITY.md)).
2. **[`AUTHORITY.md`](AUTHORITY.md)** — the current layer-ownership
   table for the whole ecosystem as it touches this repo (who owns
   language meaning, ABI numbers, self-hosted execution, the Cyberpunk
   host track).
3. **[`ROADMAP.md`](ROADMAP.md)** — the active self-hosting stage plan
   (Stage 0 done through Stage 7 long-horizon), and the foundational
   `()`-as-absence principle every stage is checked against.
4. **Active plans** (in progress, not yet closed):
   - [`canon-dispatch-migration-plan.md`](canon-dispatch-migration-plan.md)
     — registry-projection discipline for Canon-form dispatch across
     `wsm-my-lisp`/`cml`/`my-lisp-cyberpunk`.
   - [`compiler-oracle-corpus-parity-2026-09-11.md`](compiler-oracle-corpus-parity-2026-09-11.md)
     — the live three-column parity table for `wsm-my-lisp#4`, updated
     as fixtures move from `pending`/`related` to `confirmed`.
   - [`canon-function-table.md`](canon-function-table.md) — the
     machine-readable function table for `wsm-my-lisp#16`: every Canon
     identity this repo implements or witnesses on `asm/nucleus.s`,
     with honest gaps named.
5. **Tests/evidence** — `harness/`'s executed witness binaries are the
   proof that a given stage of `ROADMAP.md` actually holds on real
   `asm/nucleus.s` execution, not just on paper.

## What's NOT here

`docs/archive/` holds completed migrations, superseded proposals, and
research that was never implemented — see
[`docs/archive/README.md`](archive/README.md) for the four
subdirectories and why each document there was moved. None of it
drives a current requirement.

## Open GitHub issues as of 2026-09-12

Tracked directly on GitHub, not duplicated here (would drift): run
`gh issue list --state open` in this repo for the current list.
Ecosystem-wide policy issues (this repo's copy is one of several
identical per-repo issues) live in `juv4uk/ecosystem`.
