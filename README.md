# wsm-my-lisp

**Lisp on Lisp.**

Self-hosted growth target for WSM: Lisp + the target machine's own
assembler, growing toward the owner's own x86_64 hardware (Intel Core
i5-6400, see `wsm-os/docs/OWNER-HARDWARE-PROFILE.md`), capability by
capability. WSM defining WSM — not Rust defining WSM — is the actual
point.

`my-lisp` remains the reference semantic oracle. This repo does **not**
become authoritative over language semantics. Authority belongs to the
language contract; each capability earns conformant status through
fixtures, parity, CML admission, and native target execution.

## Language priority (owner P0, 2026-09-11)

```text
1. Lisp     — system logic and self-hosting growth
2. Assembler — minimal substrate (ABI, primitives, hot paths when justified)
3. C         — bounded mechanical glue only (never eval/apply semantics)
4. Rust      — NOT a runtime growth direction in this repo
```

Rust host embed work (formerly `dll/`: reader, eval, FFI session) has
fully migrated to **`my-lisp-cyberpunk/host-runtime`**; `dll/` was
deleted from this repo 2026-09-11 (Phase D of the migration). Policy
and the archived migration record:
[`docs/AUTHORITY.md`](docs/AUTHORITY.md),
[`docs/archive/completed-plans/dll-inventory-2026-09-11.md`](docs/archive/completed-plans/dll-inventory-2026-09-11.md).

**Documentation entry point: [`docs/CURRENT.md`](docs/CURRENT.md).**

Self-hosting core path remains:

```text
external/my-lisp/lib/meta-eval.my
        → CML admitted lowering
        → asm/nucleus.s (+ minimal C only if mechanically required)
        → harness witnesses (no Rust evaluator)
```

## Boundary with `my-lisp/c-runtime`

These are two different things, not duplication. `my-lisp/c-runtime` is an
independent C+asm substrate for checking the language contract. This repo is
the **self-hosting destination**. Neither absorbs the other.

See `repo.my` for the full scope declaration.

## Status, 2026-09-02 (+ authority note 2026-09-11)

`external/my-lisp` is a git submodule. `lib/meta-eval.my` is the starting
point for "my Lisp in my Lisp" — not yet fully running on the asm core.

`asm/nucleus.s` — hand-written x86_64 implementation of the 5 core
primitives (`wsm_cons`, `wsm_car`, `wsm_cdr`, `wsm_eq`, `wsm_atom`), same
ABI as CML/`wsm-os-target`. Bounded 4096-byte arena, no GC.

**Parity witnesses** (`harness/`, executed):

| Witness | Result |
|---|---|
| `harness-atom` | `t` |
| `harness-cons` | `(A . B)` |
| `harness-lambda` | `7` |
| `harness-eq` | both correct (Fixnum) |
| `harness-eq-symbol` | both correct (Symbol) |
| `harness-countdown` | `done` (100000) |
| `harness-closure` | Stage2 identity witness |

**Next:** Stage2/Stage3 meta-eval on asm nucleus; `dll/` freeze + migration
per inventory doc; QEMU later.

## Статус (укр.)

**Lisp-first.** Self-hosting ядро — `asm/` + `harness/`.  
`dll/` — тимчасовий Windows embed для Cyberpunk; **без нових Lisp
capabilities у Rust**; міграція → Cyberpunk track.  
`dll/` видалено 2026-09-11 (Phase D). Деталі: `docs/archive/completed-plans/dll-inventory-2026-09-11.md`, `docs/AUTHORITY.md`.
Точка входу в документацію: `docs/CURRENT.md`.

## Ліцензія

[ВОЛЬНІСТЬ](LICENSE).
