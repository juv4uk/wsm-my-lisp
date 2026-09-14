# Machine Contract Audit — MACHINE-CONTRACT-1 (wsm-my-lisp#23)

Date: 2026-09-14
Pinned my-lisp: `8ffffce9` (origin/main, contains machine-lowering-boundary.lisp + memory-layout-contract.lisp + lib/machine/**)
Verification gate: `scripts/check-machine-contracts.lisp` (Lisp-first, CI-invoked via `my-lisp`)
Audit type: fail-closed; unknown/missing = RED

## Principle

**WSM executes the contract. WSM does not rewrite the contract to itself.**

## Contract Source of Truth

| Contract file | Upstream path (pinned) | Role |
|---|---|---|
| `machine-lowering-boundary.lisp` | `external/my-lisp/machine-lowering-boundary.lisp` | Authority direction: my-lisp owns semantic IDs and observation; lowering is one-way semantic-to-machine; semantic IDs from ISA/opcode/asm forbidden; machine text/bytes are projection-only |
| `memory-layout-contract.lisp` | `external/my-lisp/memory-layout-contract.lisp` | Shared memory layout: nan-boxing-64 value representation, 4-bit tag in bits 31-28, heap representations for string/rational/closure |
| `semantic-x86-64.lisp` | `external/my-lisp/lib/machine/lowering/semantic-x86-64.lisp` | Semantic-to-x86-64 lowering profile: maps Lisp-owned semantic IDs (0002-1075) to admitted x86 operations |
| `x86-64.lisp` | `external/my-lisp/lib/machine/encoding/x86-64.lisp` | x86-64 instruction encoder: Lisp-authored, own encoding authority |

## Local Machine Fact Audit

### Admitted Slice — Semantic IDs (projection from Lisp contract)

| Local use | Upstream semantic ID | Upstream source | Status | Notes |
|---|---|---|---|---|
| cons (allocate + store pair) | 0004 | semantic-x86-64.lisp | **projection** | `(0004 runtime "allocate+STORE-pair")` — Lisp-owned identity, not born from ISA |
| car (load pair head) | 0005 | semantic-x86-64.lisp | **projection** | `(0005 direct "LOAD-pair-head")` |
| cdr (load pair tail) | 0006 | semantic-x86-64.lisp | **projection** | `(0006 direct "LOAD-pair-tail")` |
| tag-test / atom | 0002 | semantic-x86-64.lisp | **projection** | `(0002 sequence "tag-test: TEST/AND/CMP")` |
| cmp / conditional | 0003, 0007 | semantic-x86-64.lisp | **projection** | boundary operations |

### asm/nucleus.s — Bootstrap Mechanism (not semantic truth)

| Local fact | Value | Upstream source | Status | Notes |
|---|---|---|---|---|
| `.equ TAG_CONS` | 0 | wsm-target-contract (target-contract.wsm) | **bootstrap mirror** | WSM freestanding target ABI; NOT a projection of memory-layout-contract nan-boxing (cons=1 there). Mechanism, not semantic identity. |
| `.equ TAG_NIL` | 1 | wsm-target-contract | **bootstrap mirror** | Same: different from memory-layout-contract nil=3 |
| `.equ TAG_SYMBOL` | 4 | wsm-target-contract | **bootstrap mirror** | Different from memory-layout-contract symbol=2 |
| `.equ TAG_CLOSURE` | 5 | wsm-target-contract | **bootstrap mirror** | Different from memory-layout-contract closure=8 |
| `.equ TAG_MASK` | 7 | wsm-target-contract | **bootstrap mirror** | 3-bit mask; memory-layout-contract uses 4-bit in bits 31-28 |
| `.equ TAG_TRUE` | — | — | **obsolete / absent** | Must not exist; guarded by CI |
| `.equ SYM_T_ID` | SYMBOL_ID_MAX | wsm-target-contract / language-contract | **projection** | Canonical `t` identity matches language-contract |
| `.equ CLOSURE_*_OFFSET` | 0, 8 | wsm-target-contract | **bootstrap mirror** | Closure ABI is target-level mechanism |
| ERR_* codes | 1, 2, 4 | wsm-target-contract | **bootstrap mirror** | Runtime error codes: target mechanism |

### CI Workflow — Contract Gate

| Check | Fast CI | Deep CI |
|---|---|---|
| Pin `external/my-lisp` matches `8ffffce9` | ✅ | ✅ |
| `machine-lowering-boundary.lisp` present + schema /2 | ✅ | ✅ |
| `memory-layout-contract.lisp` present + version (1 0) | ✅ | ✅ |
| `lib/machine/lowering/semantic-x86-64.lisp` present | ✅ | ✅ |
| `lib/machine/encoding/x86-64.lisp` present | ✅ | ✅ |
| Authority direction: semantic-authority=my-lisp, lowering=semantic→machine, semantic-id-from-isa=forbidden | ✅ | ✅ |
| Admitted-slice semantic IDs (0002/0004/0005/0006) findable | ✅ | ✅ |
| No `.equ TAG_TRUE` manufactured in nucleus.s | ✅ | ✅ |
| Full projection parity (all witnesses pass) | — | ✅ |

## Architectural Tension: Value Layout Representation

The WSM freestanding target (wsm-target-contract, 3-bit low-bits tags) and my-lisp's `memory-layout-contract.lisp` (nan-boxing-64, 4-bit tags in bits 31-28) define **different physical value representations**. This is not drift: they serve different architectural roles.

- **memory-layout-contract** describes the host runtime value layout (my-lisp, CML, fpga-lisp).
- **wsm-target-contract** describes the WSM freestanding self-hosting target ABI — a deliberately independent, bounded mechanism for physical execution proof.

What WSM does consume from the Lisp contracts (projected):
- Semantic identity allocation (meaning of 0002-1075) — **authoritative from Lisp**
- One-way lowering direction (semantic → machine) — **authoritative from Lisp**
- No ISA/opcode labels creating semantic IDs — **authoritative from Lisp**

What WSM owns as its own bootstrap mechanism (not a claim of language truth):
- Tag encoding (3-bit low-bits) — **local-only mechanism** for target execution proof
- Pair/closure ABI offsets — **local-only mechanism** for target slice
- Error code mapping — **local-only mechanism** for runtime

Per issue #24 (STRUCTURAL-LISP-PARITY-1): full value-layout parity between Lisp-native and WSM target is the next step after this contract gate.

## Falsification

This audit is **NOT** the claim that the representation difference is correct or permanent. It is an honest status record. The CI drift gate catches:
- Contract file deletion/version change → RED
- Missing semantic IDs in the admitted slice → RED
- New `TAG_TRUE` or local truth manufacturing → RED
- Upstream contract drift without WSM replay → RED
