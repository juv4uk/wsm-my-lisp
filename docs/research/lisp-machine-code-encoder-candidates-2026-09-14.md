# Research: Lisp-native machine-code encoder donors

Date: 2026-09-14
Branch: `research/lisp-machine-code-encoders`
Base: `wsm-my-lisp/main@f43e94f01b2e75e1386426859aa87c266173de01`

## Question

Can `wsm-my-lisp` avoid writing an x86-64 assembler/encoder from scratch by borrowing or adapting an existing Lisp-native implementation while preserving the architectural rule:

```text
Lisp owns behavior and compiler logic.
Machine-code encoding is a Lisp capability.
C production runtime = 0.
Rust production runtime = 0.
External assembler is optional verification tooling, not semantic authority.
```

This is a research branch only. No production path is changed here.

## Initial verdict

**Yes. We should not start from zero.**

The strongest current donor is **Mezzano's Lisp x86-64 LAP assembler**, not because we should import all of Mezzano, but because its encoder already contains exactly the difficult x86-64 mechanics we would otherwise have to rediscover: register classes/numbers, REX, ModR/M, SIB, displacement handling, labels/fixups/relocations, variable-size instruction settling, and instruction-definition dispatch.

Recommended direction:

```text
my-lisp / CML machine IR
          |
          v
minimal WSM encoder API in Lisp
          |
          +-- adapted encoding nucleus from Mezzano (MIT)
          |
          +-- design cross-checks from SBCL
          |
          +-- tiny DSL lessons from uLisp RISC-V
          |
          v
raw x86-64 machine bytes
```

Do **not** vendor an entire general-purpose assembler until a smaller extraction is proven insufficient.

## Candidate matrix

| Candidate | Target | License | Direct code reuse | Design reuse | Verdict |
|---|---|---|---:|---:|---|
| Mezzano `compiler/lap.lisp` + `lap-x86.lisp` | x86-64 | MIT | **High** | **High** | Primary donor |
| SBCL `src/compiler/assem.lisp` + `x86-64/insts.lisp` | x86-64 | public-domain / FreeBSD-style with retained notices | Medium | **Very high** | Oracle + selective donor |
| David Johnson-Davies `lisp-riscv-assembler` | RISC-V | MIT | Low for x86-64 | High | DSL/bit-packing model; future RISC-V |
| `cl-asm` | many, includes 8086 but not x86-64 | MIT | Medium for generic IR/core | High | IR/front-end architecture donor, not x86-64 opcode donor |
| Movitz | x86 (historical, primarily 32-bit) | BSD-style | Medium | Medium | Secondary historical donor |
| Yalo | x86-64 | GPL-2.0 | Avoid until license policy is explicit | Medium | Design-only for now |
| iced-x86 | x86/x64, non-Lisp implementation | MIT | **No production dependency** | Oracle only | Byte-encoding verification oracle |

## 1. Mezzano — primary candidate

Upstream commit inspected:

`froggey/Mezzano@c5cab761d07c1115965a357b1da67a547a91e2b8`

Relevant files:

- `compiler/lap.lisp` — generic byte emitter, labels, symbol table, fixups, relocations, pass/settling machinery.
- `compiler/lap-x86.lisp` — x86/x86-64 registers, REX, ModR/M, SIB, addressing modes and instruction definitions.
- `COPYING` — MIT.

High-value pieces visible directly in `lap-x86.lisp`:

```text
reg-class
reg-number
encode-register
encode-modrm
encode-sib
encode-rex
emit-rex
encode-disp32
emit-modrm-address
define-instruction
```

High-value pieces visible in `lap.lisp`:

```text
emit
label / make-label
fixups
relocations
perform-relocation
variable-sized instruction settling
instruction dispatch
```

### What should be borrowed

A **small attributed adaptation**, not the whole subsystem:

1. x86-64 register-number model;
2. REX byte encoding;
3. ModR/M encoding;
4. SIB encoding;
5. 8/32-bit displacement selection;
6. RIP-relative/rel32 mechanics;
7. minimal label/fixup model;
8. instruction-definition pattern if it maps cleanly to `my-lisp` macros.

### What should NOT be borrowed blindly

Mezzano-specific coupling:

```text
mezzano.compiler:x86-64-target
mezzano.debug hooks
mezzano.extensions hooks
Mezzano GC/debug metadata
Mezzano dynamic environment assumptions
full instruction registry before WSM needs it
```

The target is not "port Mezzano LAP". The target is "extract the smallest proven x86-64 encoding nucleus and re-express it in our Lisp semantics/API".

## 2. SBCL — strongest design/oracle reference

Upstream commit inspected:

`sbcl/sbcl@148216723d956e93211b24c3d984c5fce80833e9`

Relevant files:

- `src/compiler/assem.lisp`
- `src/compiler/x86-64/insts.lisp`
- `COPYING`

SBCL has a scheduling assembler written in Lisp and a large x86-64 instruction-description layer. Useful concepts include:

```text
statement/instruction objects
sections/segments
labels and backpatches
define-instruction-format
define-bitfield-emitter
emit-byte / emit-word / emit-dword / emit-qword
explicit operand/address encoders
```

License status is unusually permissive: CMUCL ancestry is public domain; SBCL changes are public domain where possible or FreeBSD-style otherwise, with specified notices retained.

### Why not copy SBCL wholesale

Its assembler is highly integrated with SBCL's compiler, VOP scheduler, TN representation, GC metadata, object system and backend conventions. It is excellent as a correctness/design oracle, but a full extraction would likely import more complexity than WSM currently needs.

Best use:

- compare tricky x86-64 encodings against SBCL;
- selectively adapt tiny generic encoding ideas where simpler than Mezzano;
- study its instruction-format table model before designing our long-term table.

## 3. uLisp RISC-V assembler — proof that the Lisp DSL can stay tiny

Upstream commit inspected:

`technoblogy/lisp-riscv-assembler@10c375bbd81cc673d71076ca058255469729688d`

License: MIT, David Johnson-Davies.

The assembler is only a small Lisp program and uses a compact `emit` function to pack instruction fields. That exact code is RISC-V-specific and therefore not a drop-in x86-64 solution, but the architecture is highly relevant:

```text
Lisp form -> register decode -> bit-field pack -> machine word
```

This is evidence that our machine layer does not need textual `.s` as an architectural boundary.

Future value: strong candidate for a second backend when WSM experiments with RISC-V/FPGA targets.

## 4. cl-asm — modular IR lessons

Upstream commit inspected:

`Phibrizo/cl-asm@3e18542b76ad40c9bc3c0824db314c749758adda`

License: MIT.

Its documented architecture cleanly separates:

```text
frontend -> IR -> backend -> byte vector
```

with an IR roughly shaped as labels, instructions, directives and operands. It also has explicit two-pass symbol resolution.

This is attractive for the **shape** of a future CML machine IR, but the current project does not provide an x86-64 backend. Therefore it should not displace Mezzano as the primary encoding donor.

Important lesson: WSM should keep the Lisp-facing machine vocabulary separate from the target encoder, so x86-64 does not become hardwired into Canon semantics.

## 5. Other references

### Movitz

Historical Common Lisp x86 development platform with `asm.lisp`, `asm-x86.lisp`, and permissive BSD-style terms. Worth mining for small patterns, especially historical Lisp-native compiler/assembler integration, but it is less directly aligned with modern x86-64 than Mezzano.

### Yalo

Bare-metal x86-64 Lisp OS with a Common Lisp assembler, but GPL-2.0. Until WSM has an explicit license-compatibility decision, treat Yalo as **design/reference only**, not a code donor.

### iced-x86

Very broad and heavily tested x86/x64 encoder under MIT, but not a Lisp implementation and would violate the production-language direction if linked as a runtime/compiler dependency. Excellent independent oracle for tests only.

## License discipline

`wsm-my-lisp` currently uses the custom `ВОЛЬНІСТЬ` text, not a standard SPDX license. Directly imported upstream code must therefore preserve its upstream notices and terms explicitly.

For any future implementation spike:

```text
MIT/BSD/public-domain donor code:
  preserve copyright/license notices
  identify adapted source and exact upstream SHA
  keep a THIRD_PARTY_NOTICES or equivalent attribution record

GPL donor code:
  do not copy into WSM until compatibility/distribution policy is explicitly decided
```

This note is engineering research, not legal advice.

## Recommended first implementation experiment (not performed on this research branch yet)

Do not import hundreds of instructions. Prove the donor strategy with the smallest x86-64 slice:

```text
1. byte emitter
2. register numbers
3. REX
4. ModR/M
5. SIB
6. rel32/fixup
7. 3-5 instructions sufficient for one current CML witness
```

Candidate instructions should be chosen from an existing CML-generated fixture, not invented in isolation.

Success test:

```text
same Lisp machine-IR input
       |
       +--> WSM Lisp encoder -> bytes A
       |
       +--> GNU as/objdump or llvm-mc oracle -> bytes B
       |
       +--> optional iced-x86 oracle -> bytes C

A == B == C
```

Then execute the generated bytes through the existing witness path and compare the returned WSM value against the `my-lisp` oracle.

## Recommendation

**Primary:** adapt a small MIT-licensed Mezzano x86-64 encoding nucleus.

**Secondary:** use SBCL as the deep correctness/design reference, not as the initial bulk import.

**Architecture:** preserve a tiny target-independent CML machine IR inspired by the separation seen in cl-asm.

**Philosophy:** learn from uLisp's minimal direct emitter — machine instructions should become Lisp data/forms that encode directly to bytes, not textual assembly that requires a second language boundary.

If the first extraction works, the desired production path becomes:

```text
my-lisp source
    -> Canon / semantic IDs
    -> CML
    -> machine IR
    -> Lisp-native x86-64 encoder
    -> machine bytes
    -> CPU
```

GNU `as`, LLVM MC, objdump and iced-x86 remain external verification instruments only.
