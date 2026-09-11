# Authority · Авторитет

**2026-09-11 · P0 #15**

## English

| Layer | Owns |
|-------|------|
| `my-lisp` language-contract | Meaning of the language |
| `wsm-target-contract` | Shared ABI numbers (words, tags, calling convention) |
| `wsm-my-lisp` **core** (`asm/`, `harness/`, Stage* self-host) | Proving conformant execution on target without Rust evaluator |
| `dll/` | **Temporary** Windows host embed for Cyberpunk — **not** semantic authority; **frozen** for new Lisp capabilities |
| CML | Compiler mechanism, not semantics |
| `my-lisp-cyberpunk` | Product surface (RED4ext); destination for host Rust runtime |

Principle: if logic can live in Lisp, put it in Lisp. Asm only when the machine must be touched. Bounded C only when asm is absurdly painful. Rust host work belongs on the Cyberpunk track, not as the next self-hosting layer here.

## Українською

| Шар | Володіє |
|-----|---------|
| language-contract | Значення мови |
| wsm-target-contract | Спільні ABI-числа |
| core цього repo | Доказ виконання на target без Rust-evaluator |
| `dll/` | Тимчасовий Windows embed для Cyberpunk; **заморожений** для нових Lisp capabilities |
| CML | Механізм компілятора |
| my-lisp-cyberpunk | Продуктова поверхня; місце для host Rust |

Детальний inventory: [`dll-inventory-2026-09-11.md`](dll-inventory-2026-09-11.md).
