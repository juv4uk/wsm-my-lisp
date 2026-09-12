# Archive — non-normative

**Nothing in `docs/archive/` (or its subdirectories) is authoritative.**
Everything here is preserved for history/context only. On any conflict
between an archived document and current material, current material
wins unconditionally — current contracts/Canon/semantic-registry
(owned by `my-lisp`), `docs/AUTHORITY.md`, active ADRs, and
`docs/CURRENT.md` as the entry point.

No active requirement, acceptance criterion, or implementation
decision in this repo may cite a document under this directory as its
source of truth. An archived doc may be read for historical context —
why a decision was made, what was tried before, what a now-superseded
plan looked like — never as a current specification.

## Subdirectories

- `superseded/` — a document whose content has been replaced by a
  newer document; the newer one should name it via `Supersedes:`.
- `completed-plans/` — a plan or migration doc whose work finished;
  kept as the historical record of how it happened, not as ongoing
  guidance.
- `experiments/` — research/proposal documents that were never
  implemented, or were addressed to a different repo to decide.
- `historical/` — documents whose subject moved elsewhere or whose
  premise no longer applies, kept for context on how the repo got to
  its current shape.

Part of the ecosystem-wide `DOC-AUTHORITY-ARCHIVE` policy
(`juv4uk/ecosystem#5`); this repo's application of it is tracked in
`wsm-my-lisp#19`.
