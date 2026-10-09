# Gold ideas: Lisp all the way to machine code

Date: 2026-09-14
Branch: `research/lisp-machine-code-encoders`

## Owner direction

This note supersedes the earlier implementation recommendation to directly adapt donor code.

The new rule is stricter:

> Do not collect foreign assembler code. Harvest only ideas that are strong enough to simplify or fundamentally improve the WSM architecture. Re-express those ideas independently under the project's own `ВОЛЬНІСТЬ` direction.

The upstream projects below are therefore **design references and verification oracles**, not code donors.

## Selection bar

An idea enters this document only if it does at least one of these:

1. removes an entire architectural layer;
2. turns duplicated machine knowledge into one source of truth;
3. makes correctness mechanically testable;
4. makes the compiler substantially easier to understand or self-host;
5. cleanly separates Lisp semantics from target-machine mechanism.

Anything merely convenient, conventional, or feature-rich is excluded.

---

# GOLD 1 — One canonical machine-instruction object, many projections

### Source of the idea

LLVM MC layer (`MCInst`, `MCOperand`, `MCStreamer`).

LLVM describes `MCInst` as the common currency of its machine-code layer. The same small instruction representation is consumed by the instruction encoder and printer and is produced by the assembly parser and disassembler.

### The idea

WSM should not have separate semantic representations for:

```text
text assembly
encoder input
disassembler output
validator input
CML low-level output
```

There should be exactly one target-level Lisp data shape, conceptually:

```lisp
(machine-inst opcode operands attributes)
```

For example:

```lisp
(machine-inst 'mov
  (list (reg 'rax) (reg 'rbx))
  '())
```

Then:

```text
                    +-> machine bytes
                    |
MachineInst --------+-> human-readable asm
                    |
                    +-> validator
                    |
                    +-> disassembler canonical result
                    |
                    +-> trace/debug view
```

Text `.s` becomes a **projection**, not an architectural boundary.

### Why this is exceptional for WSM

It matches the existing Canon philosophy almost perfectly:

```text
semantic identity is not spelling
machine instruction identity is not assembly text
```

Human assembly syntax becomes analogous to EN/UK/UKR language surfaces: a presentation of a deeper identity.

### WSM implication

CML should lower to machine instruction data, never to assembly text as its semantic endpoint.

Possible boundary:

```text
my-lisp semantic ID
        -> CML lowering
        -> MachineInst
        -> encoder
        -> bytes
```

**Priority: P0.** This should shape everything below it.

---

# GOLD 2 — Describe the ISA once; generate encoder, validator, printer and decoder views

### Sources of the idea

- LLVM TableGen
- SBCL `define-instruction-format` / `define-bitfield-emitter`
- Intel XED's generated instruction machinery and shared operand model

LLVM explicitly explains that manually specifying all instruction properties is unmaintainable and error-prone; TableGen factors common properties into declarative records and generates multiple backend artifacts from them.

SBCL similarly describes instruction layouts declaratively with instruction-format and bitfield-emitter definitions across several architectures.

Intel XED demonstrates the scale of the problem on modern x86 and derives encoder/decoder machinery from structured instruction knowledge rather than treating each encoding as unrelated handwritten code.

### The idea

Do **not** hand-write an ever-growing forest such as:

```text
encode-mov-rr
encode-mov-ri
encode-mov-rm
encode-add-rr
encode-add-ri
...
```

Instead define machine knowledge as Lisp data/macros once.

Conceptually:

```lisp
(define-machine-format modrm-r64-r64
  :prefix rex-w
  :opcode byte
  :operands (dst-r64 src-r64)
  :modrm (reg src-r64 rm dst-r64))

(define-machine-instruction mov-r64-r64
  :mnemonic mov
  :format modrm-r64-r64
  :opcode #x89)
```

From that one declaration WSM can derive, where practical:

```text
encoder
operand validator
pretty printer
canonical disassembler matcher
documentation
test vectors
feature/CPU requirements
```

### Why this is exceptional for WSM

This is **Canon + function table applied to the CPU**.

The same principle that says:

```text
one semantic ID -> many human spellings
```

can say:

```text
one machine declaration -> many mechanical projections
```

It prevents encoder, disassembler and documentation from slowly disagreeing.

### WSM implication

Do not create a second ad-hoc "assembler language database" next to the semantic registry. Create a small target-machine registry whose authority is structured Lisp data.

**Priority: P0.** This is likely more important than borrowing any existing encoder implementation.

---

# GOLD 3 — Nanopass CML: many tiny proofs instead of one compiler monolith

### Sources of the idea

Chez Scheme + Nanopass Framework.

Chez Scheme's compiler is largely Scheme, passes through many explicit intermediate languages, and emits machine code directly rather than relying on a system assembler. The Nanopass framework is an embedded DSL centered on explicit language definitions and explicit passes.

### The idea

Do not make CML a giant transformation:

```text
Lisp AST -------------------------------> x86 bytes
```

Make the path a sequence of tiny named transformations:

```text
Canon Lisp
  -> normalized core
  -> closed lexical form
  -> explicit control form
  -> primitive/semantic-ID form
  -> target-neutral machine ops
  -> x86 selected ops
  -> allocated registers
  -> MachineInst
  -> bytes
```

Each pass owns one idea and has a precise input/output contract.

### Why this is exceptional for WSM

This aligns with the project's scientific discipline:

```text
small claim
small witness
small failure surface
```

A failed transformation can be localized to one pass instead of becoming "the compiler emitted wrong bytes".

It also makes self-hosting more realistic: Lisp can gradually take ownership pass by pass.

### WSM implication

CML should prefer numerous small Lisp passes over a clever large backend. Every pass should be independently executable on a fixture and ideally serializable/inspectable.

**Priority: P0/P1.** This is the strongest compiler-structure idea found in the survey.

---

# GOLD 4 — Macro-instructions are Lisp expansions, not new CPU semantics

### Source of the idea

Mezzano LAP `define-macro-instruction`, together with Lisp's ordinary macro model.

### The idea

Useful low-level operations often do not correspond one-to-one with a real CPU instruction. Do not pollute the encoder with semantic pseudo-instructions.

Instead:

```lisp
(machine-macro load-immediate64 (dst value)
  ...expand to real MachineInst forms...)
```

The encoder understands only real encodable instruction forms.

Pseudo-instructions, convenience operations, calling-sequence helpers, prologue/epilogue builders, etc. remain Lisp transformations above the encoder.

### Why this is exceptional for WSM

It protects the boundary:

```text
Lisp owns policy and composition
encoder owns physical encoding facts
```

The assembler/encoder therefore cannot quietly become a second runtime or a second semantic language.

### WSM implication

Raw target operations should have two layers:

```text
machine macros / pseudo ops   <- Lisp, extensible
             |
             v
real MachineInst              <- target facts only
             |
             v
bytes
```

**Priority: P1.** Adopt as a boundary rule before adding many machine operations.

---

# GOLD 5 — Checked and trusted encoders from the same declaration

### Source of the idea

Intel XED `enc2` exposes checked and unchecked forms of generated encoders. The checked form validates operand ranges/classes before using the fast path; the unchecked form assumes valid input.

### The idea

Development and trusted compiler paths do not need the same safety/performance tradeoff.

From the same machine declaration, WSM can conceptually expose:

```lisp
(encode/checked inst)
(encode/trusted inst)
```

During development, fuzzing and self-host bring-up:

```text
CML -> validator -> encoder
```

After a preceding pass has mechanically proven the instruction shape:

```text
validated MachineInst -> trusted encoder
```

### Why this is exceptional for WSM

It avoids the false choice between:

```text
safe but permanently expensive
```

and

```text
fast but impossible to diagnose
```

Both paths can be generated from the same machine facts, so they cannot drift independently.

### WSM implication

Do not optimize this yet, but design the machine declaration format so validation and encoding are separable projections.

**Priority: P2 design constraint, not an immediate implementation task.**

---

# GOLD 6 — Encode/decode round-trip as an executable machine contract

### Sources of the idea

LLVM uses one machine representation across encoder/disassembler tooling. Intel XED's encoder and decoder share common operand-level concepts and machine-state information.

### The idea

Eventually WSM should be able to prove:

```text
MachineInst A
   |
 encode
   v
 bytes
   |
 decode
   v
MachineInst B

canonical(A) == canonical(B)
```

For aliases or multiple legal encodings, comparison is semantic/canonical rather than byte-text spelling equality.

During bootstrap, external tools remain independent oracles:

```text
WSM encode bytes
    == LLVM/GNU/XED expected bytes
```

But the long-term internal invariant is stronger:

```text
encode -> decode -> same canonical instruction identity
```

### Why this is exceptional for WSM

The machine layer becomes self-checking rather than merely trusted.

That is closely aligned with the project's existing practice of turning claims into executable witnesses.

### WSM implication

When designing the instruction declaration table, preserve enough information that a future decoder can be derived or independently implemented against the same canonical MachineInst model.

**Priority: P1/P2.** Do not build a full disassembler now; preserve the possibility structurally.

---

# Engineering mechanism worth keeping, but not a governing idea

## Branch relaxation / variable-size settling

Mezzano has explicit machinery for variable-sized instructions and settling; SBCL has chooser/backpatch concepts. x86 branches and immediates make some equivalent mechanism unavoidable.

This is important engineering, but it should stay below the architecture line:

```text
MachineInst / labels
    -> layout
    -> choose valid shortest encoding where policy permits
    -> fixups/relocations
    -> bytes
```

Do not let relocation/layout logic leak upward into Lisp language semantics.

---

# What was deliberately rejected

## Reject: import a general-purpose assembler

It adds a subsystem before proving WSM needs one.

## Reject: copy Mezzano/SBCL encoder code because it already works

The owner direction is now idea-first. Their value is as evidence and as a source of patterns; WSM should express the selected patterns independently.

## Reject: assembly text as compiler IR

Text is for humans and interoperability. It should not own instruction identity.

## Reject: one handwritten encoder function per instruction variant

Modern x86 makes this a maintenance trap. Instruction facts should be data-driven where possible.

## Reject: make `(asm ...)` a second uncontrolled language

Any raw machine surface should construct the same canonical MachineInst objects used by CML. There must not be a privileged textual escape path with separate semantics.

---

# The architecture suggested by the gold ideas

```text
my-lisp source
      |
      v
Canon / semantic IDs
      |
      v
CML nanopasses
      |
      v
target-neutral machine operations
      |
      v
x86 selection / register allocation
      |
      v
Canonical MachineInst objects
      |
      +--------------------+
      |                    |
      v                    v
validator             asm/debug printer
      |
      v
Lisp-native encoder
      |
      v
layout / fixups / relaxation
      |
      v
machine bytes
      |
      v
CPU
```

And beside it, from the **same target-machine declarations**:

```text
machine declarations
   |      |       |        |
   v      v       v        v
encode  validate  print   future decode
```

This is the key result of the research so far.

---

# Relationship to Canon + function table

The most interesting synthesis is that the project's existing language architecture can repeat one level lower.

Language level:

```text
semantic ID
  -> EN / UK / UKR / SA surfaces
  -> one meaning
```

Machine level:

```text
machine instruction identity/record
  -> encoder
  -> printer
  -> validator
  -> decoder
  -> one physical operation/encoding family
```

The repeated rule is:

> Identity lives in structured data; spellings and byte renderings are projections.

That may be the deepest idea worth carrying forward from this research.

---

# Recommended next research question

Do **not** implement an x86 encoder yet.

First inspect current CML and answer one architectural question with evidence:

> What is the smallest target-neutral data shape that can sit between CML's existing lowering and a future canonical `MachineInst` without importing x86 knowledge into Canon?

Only after that boundary is clear should an encoder spike begin.

---

# References inspected

- LLVM MC layer / CodeGenerator documentation — `MCInst`, `MCOperand`, `MCStreamer`, direct `.s` or `.o` projections.
- LLVM TableGen documentation — declarative target records and generated backend artifacts.
- `sbcl/sbcl@148216723d956e93211b24c3d984c5fce80833e9` — instruction formats, bitfield emitters, VOP patterns.
- `froggey/Mezzano@c5cab761d07c1115965a357b1da67a547a91e2b8` — LAP, macro-instructions, fixups, variable-sized assembly.
- `cisco/ChezScheme@8d105909c0afe899b6983e3371e5a7163c7e1826` — direct machine-code compiler, architecture-specific backends; Nanopass ecosystem.
- `intelxed/xed@0bcb6237345c5066726dcc08b3d87928df3b5b26` — encoder/decoder model and checked/unchecked ENC2 pattern.
- Nanopass Framework — embedded Scheme DSL for explicit intermediate languages and passes.

No upstream source code is imported by this research note.