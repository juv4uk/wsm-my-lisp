# Authority · Авторитет

**2026-09-11 · P0 #15** — **architecturally corrected 2026-09-13 (owner directive)**

## English

| Layer | Owns |
|-------|------|
| `my-lisp` language-contract | Meaning of the language |
| `wsm-target-contract` | Shared ABI numbers (words, tags, calling convention) |
| `wsm-my-lisp` **core** (`asm/`, `harness/`, Stage* self-host) | Proving conformant execution on target without Rust evaluator |
| `dll/` (deleted, Phase D, 2026-09-11) | Was a temporary Windows host embed for Cyberpunk; migrated to `my-lisp-cyberpunk/host-runtime` and removed here once that tree confirmed Phase C complete |
| CML | Compiler mechanism, not semantics |
| `my-lisp-cyberpunk` | Product surface (RED4ext); destination for host Rust runtime |

Principle: **Lisp owns all logic. Assembler owns only irreducible machine mechanism.**

There is no third production implementation language.

```text
If logic can live in Lisp → Lisp.
If an operation is literally irreducible machine mechanism → assembler.
```

ABI is **SysV AMD64 calling convention / target ABI / machine ABI** — not "C ABI". C is not part of the implementation.

## Українською

| Шар | Володіє |
|-----|---------|
| language-contract | Значення мови |
| wsm-target-contract | Спільні ABI-числа |
| core цього repo | Доказ виконання на target без Rust-evaluator |
| `dll/` (видалено, Phase D, 2026-09-11) | Був тимчасовим Windows embed для Cyberpunk; мігровано в `my-lisp-cyberpunk/host-runtime`, видалено звідси після підтвердження Phase C |
| CML | Механізм компілятора |
| my-lisp-cyberpunk | Продуктова поверхня; місце для host Rust |

Принцип: **Lisp володіє логікою. Assembler — лише незвідним машинним механізмом.**

Немає третьої production мови реалізації.

```text
Якщо логіка живе в Lisp → Lisp.
Якщо операція — незвідний машинний механізм → assembler.
```

ABI — це **SysV AMD64 calling convention / target ABI / machine ABI** — не "C ABI". C не є частиною реалізації.

Детальний inventory (архів, dll/ уже видалено): [`archive/completed-plans/dll-inventory-2026-09-11.md`](archive/completed-plans/dll-inventory-2026-09-11.md).
