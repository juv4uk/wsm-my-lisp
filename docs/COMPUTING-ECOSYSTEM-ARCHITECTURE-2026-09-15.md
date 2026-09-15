# Computing Ecosystem — Architecture, Dependencies and Flow (2026-09-15)

![My Computing Ecosystem — Architecture, Dependencies and Flow](architecture/computing-ecosystem-architecture.png)

Owner-authored architecture diagram of the full stack, from Lisp semantics
down to hardware. `wsm-my-lisp` is the physical/self-hosting proof
consumer: it demonstrates that `my-lisp`'s Lisp-owned semantics and
machine contracts (Canon, function table, Machine Contract) physically
reach target/self-hosting execution on the owner's own x86-64 hardware,
growing independence from my-lisp's Rust bootstrap capability-by-capability.
Lisp is the logic/behavior/machine-facing representation; ASM is the
irreducible target/bootstrap/ABI mechanism — no C/Rust evaluator or
runtime in the canonical production target unless separately re-ratified.

Full stack, top to bottom: User/Developer → **my-lisp** (Language &
Semantics, owns Canon/meaning) → **CML** (Compiler for My Lisp) → Target
Runtime/OS (this repo demonstrates the target/self-hosting proof side of
this layer) → Hardware.

This diagram is a snapshot of intent, not itself an authority document —
`juv4uk/ecosystem#7` (the living cross-repo map) remains the authoritative,
updated-in-place source of truth about current ownership and boundaries.
