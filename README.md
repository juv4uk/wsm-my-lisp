# wsm-my-lisp

**Lisp on Lisp.**

Self-hosted growth target for WSM: Lisp + the target machine's own
assembler, growing toward the owner's own x86_64 hardware (Intel Core
i5-6400, see `wsm-os/docs/OWNER-HARDWARE-PROFILE.md`), capability by
capability. WSM defining WSM — not Rust defining WSM — is the actual
point: a nucleus small enough to grow the rest of itself from inside,
the same instinct McCarthy's own eval/apply already had.

`my-lisp` (the Rust implementation) remains the canonical semantic oracle
and reference implementation for every capability until that capability
specifically proves parity here: WSM implementation exists, independent
semantic fixtures pass, Rust↔WSM parity passes, CML admission passes,
native target execution passes (QEMU/FPGA where in scope). This repo does
not become authoritative over language semantics by existing — it earns
authority one proven capability at a time.

**Boundary with `my-lisp/c-runtime` (added 2026-09-03, owner's own framing):** these are two different things, not a duplication. `my-lisp/c-runtime` is an **independent C+asm substrate** for checking the language contract itself and for learning — it proves the same `conformance.my` facts on a third, physically different implementation, alongside Rust and `fpga-lisp`. `wsm-my-lisp` (this repo) is the **self-hosting destination** — where WSM increasingly hosts itself, in WSM's own Lisp+asm, growing toward the owner's real hardware, capability by capability. A representation choice proven out in `my-lisp/c-runtime` (e.g. a tagged-`Value` encoding) is exactly the kind of thing that could later inform this repo's own `asm/nucleus.s` — but the two repos answer different questions and neither absorbs the other.

See `repo.my` for the full scope declaration.

## Status, 2026-09-02

`external/my-lisp` is a git submodule (pinned, same pattern as `my-idea`'s),
not a copy — `lib/meta-eval.my` there (already interpreting WSM with WSM's
own primitives, quote/atom/eq/car/cdr/cons/cond/lambda/application/
closures/def/defmacro) is the starting point for "my Lisp in my Lisp." It
has not yet been made to run through the asm core below — that is the next,
larger step, not done here.

`asm/nucleus.s` is a hand-written (not generated, not ported from Rust)
x86_64 GNU-asm implementation of the 5 core primitive operations, using the
same `extern "C"` ABI and word-tagging CML's generated code already expects
(`wsm-os-target::Tag`/`CALLING_CONVENTION`): `wsm_cons`, `wsm_car`,
`wsm_cdr`, `wsm_eq`, `wsm_atom`. Backed by a small fixed 4096-byte
(256-cell) bump-allocated arena, no GC, no growth — deliberately bounded,
matching `wsm-os/docs/OWNER-HARDWARE-PROFILE.md`'s own M1/M2 guidance and
this task's own scope (closures/GC are explicitly out of scope here).
Scalar x86_64 only, no AVX2/BMI2, matching the owner's documented hardware
baseline.

**Parity witnesses, all executed and checked against the oracle, not just
assembled** (`harness/`, `cargo run --bin <name>`):

| Witness | CML-generated entry | Result | Oracle |
|---|---|---|---|
| `harness-atom` | `(atom (quote ()))` | `t` | matches |
| `harness-cons` | `(cons (quote A) (quote B))` (the canonical `FIRST_FIXTURE_SOURCE`) | `(A . B)` | matches `FIRST_FIXTURE_EXPECTED` |
| `harness-lambda` | `((lambda (x) x) 7)` | `7` | matches (this fixture calls no primitive, so it doesn't exercise the nucleus) |
| `harness-eq` | direct call, no CML entry | `(eq 41 41)` → `t`, `(eq 41 42)` → `()` | both correct |

All 5 primitives now have at least one real, executed proof: `wsm_cons`/
`wsm_car`/`wsm_cdr` together via the cons fixture, `wsm_atom` via the atom
fixture, `wsm_eq` directly. Same CML-generated assembly, unchanged — only
which implementation of the 5 primitives it links against changed.

**`wsm-os-runtime`'s Rust implementation of these same 5 primitives is
untouched** (`wsm-os/crates/wsm-os-runtime/src/lib.rs:409-449`) and remains
the working reference path for `wsm-os-hosted`/`wsm-os-kernel`. This
nucleus does not replace it in place — it is a parallel, additive proof
that hand-written asm can implement the same ABI, living here. It does not
reuse `RuntimeContext`'s internal layout (a private Rust struct, not a
stable cross-repo ABI) — only the `wsm-os-target` word encoding and the
`wsm_*` function signatures are the actual shared contract, so this
nucleus keeps its own static arena instead.

**Next**, not started here: (1) make `meta-eval.my` itself run — first
against `crates/my-lisp`'s own evaluator as it already does, eventually
compiled/interpreted so it executes on this asm core; (2) closures/GC/
heap-growth, explicitly out of scope for this pass; (3) QEMU boot parity
(everything above is hosted-Linux-process only).

## Статус, 2026-09-02 (укр.)

`external/my-lisp` — git submodule (закріплений, той самий паттерн, що в
`my-idea`), не копія — `lib/meta-eval.my` там (уже інтерпретує WSM
власними примітивами) є стартовою точкою "мій лісп на моєму ліспі". Ще
не змушений виконуватись через asm-ядро нижче — це наступний, більший
крок, тут не зроблено.

`asm/nucleus.s` — ручно написана (не згенерована, не портована з Rust)
x86_64 GNU-asm реалізація 5 базових примітивів: `wsm_cons`, `wsm_car`,
`wsm_cdr`, `wsm_eq`, `wsm_atom`, з тією самою `extern "C"` ABI та
word-tagging, яку вже очікує згенерований CML-код. Малий фіксований буфер
4096 байт (256 клітин), bump-allocator, без GC, без росту — навмисно
обмежено. Лише scalar x86_64, без AVX2/BMI2.

**Parity witnesses, усі реально ЗАПУЩЕНІ й звірені з oracle, не лише
зібрані:** `harness-atom` → `t`, `harness-cons` (канонічний
`FIRST_FIXTURE_SOURCE`) → `(A . B)`, `harness-lambda` → `7`,
`harness-eq` → `(eq 41 41)`→`t`, `(eq 41 42)`→`()`. Усі 5 примітивів
мають реальний, виконаний доказ.

`wsm-os-runtime`'s Rust-реалізація цих самих 5 примітивів **недоторкана**
і лишається робочим шляхом для `wsm-os-hosted`/`wsm-os-kernel`. Це ядро
не замінює її на місці — паралельний, додатковий доказ, що ручний asm
може реалізувати той самий ABI.

**Далі**, не почато тут: (1) змусити `meta-eval.my` реально виконуватись
на цьому asm-ядрі; (2) closures/GC/ріст купи — свідомо поза межами цього
проходу; (3) QEMU boot parity (усе вище — лише hosted Linux-процес).

## Самостійна ціль WSM

**Лісп на ліспі.**

WSM повністю в Lisp + асемблер цільової машини, під власне x86_64-залізо
власника (Intel Core i5-6400). Суть не в тому, щоб позбутись Rust заради
самого факту — суть у тому, що WSM визначає WSM, а не Rust визначає WSM:
достатньо маленьке ядро, щоб вирощувати решту себе зсередини, той самий
інстинкт, який уже мав власний eval/apply Маккарті. `my-lisp` (Rust) лишається канонічним
семантичним oracle, доки кожна capability окремо не доведе паритет тут:
є WSM-реалізація, незалежні semantic fixtures проходять, Rust↔WSM parity
проходить, CML admission проходить, реальне виконання на target проходить
(QEMU/FPGA де застосовно). Цей репозиторій не стає авторитетним над
мовною семантикою самим фактом існування — авторитет здобувається по
одній доведеній capability за раз.

Повна декларація scope — у `repo.my`.

## Ліцензія

Цей твір поширюється під [ВОЛЬНІСТЮ](LICENSE) — простим словом про свободу творити, пам'ятаючи про волю іншого.
