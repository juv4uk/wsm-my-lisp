; repo.my — Swarm Contract v0.1 scope declaration for wsm-my-lisp.
; Format confirmed by example against my-idea/repo.my and my-lisp's own
; sibling declarations.
;
; A declaration of scope, not an authorization grant -- authorities/
; non-authorities state what this repo is and is not the source of truth
; for, so other repos' agents don't have to re-derive it.
;
; wsm-my-lisp is the self-hosted growth target for WSM: Lisp + the target
; machine's own assembler, growing toward the owner's own x86_64 hardware
; (see wsm-os/docs/OWNER-HARDWARE-PROFILE.md), capability-by-capability,
; per the owner's 2026-09-02 bootstrap-boundary strategy.
;
; my-lisp (crates/my-lisp, Rust) remains the canonical semantic oracle and
; reference implementation for every capability until that capability
; specifically clears its own parity bar here: WSM implementation exists,
; independent semantic fixtures PASS, Rust<->WSM parity PASS, CML admission
; PASS, native target PASS (QEMU/FPGA where in scope). This repo does not
; become authoritative over language semantics by existing -- it earns
; authority one proven capability at a time, and only for the capabilities
; it has actually proven, not by declaration.
;
; The first real witness (2026-09-02, in my-lisp/cml/wsm-os, not yet moved
; here): (atom (quote ())) compiled via CML's x86_64-freestanding backend,
; actually executed on the owner's hardware profile via wsm-os-hosted,
; value matched the my-lisp oracle. This repo is where that line of work
; is meant to consolidate and grow, not a restart.

(repository
  (id wsm-my-lisp)
  (role wsm-self-hosted-growth-target)
  (exports self-hosted-wsm-nucleus lisp-authored-assembler-frontend hardware-targeted-witnesses)
  (imports language-contract meta-eval-nucleus cml-lowering fpga-lisp-assembler-precedent wsm-os-target-abi)
  (capabilities wsm lisp assembler x86_64 self-hosting)
  (authorities (none-yet -- earned capability-by-capability, see context above))
  (non-authorities language-semantics isa-design compiler-internals hardware-implementation swarm-coordination))
