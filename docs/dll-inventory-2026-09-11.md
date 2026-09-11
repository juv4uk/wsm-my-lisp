# dll/ inventory — P0 #15 (2026-09-11)

**Рішення власника:** `wsm-my-lisp` = Lisp-first + asm substrate + bounded C.  
**Rust у runtime цього repo — не напрямок росту.** Host/embed Rust → Cyberpunk track.

Цей документ — inventory, не rewrite. Кожен компонент класифіковано.

## Authority boundary (fail-closed intent)

```text
Lisp owns program / self-hosting logic
        ↓
CML / offline compiler (external)
        ↓
asm nucleus (this repo)
        ↓
optional bounded C mechanism only
        ↓
native x86_64
```

**Заборонено додавати в core path цього repo як нову semantic authority:**

- reader / eval / apply / cond / quote / lambda / def / closure semantics у Rust
- Lisp value identities, truth/error policy, heap/session semantics як «мова»
- optimizer semantics у Rust

`dll/` на 2026-09-11 **вже містить** Rust reader/eval — це host-embed шлях для Cyberpunk, не self-hosting core. Нові Lisp capabilities туди **заморожені** (лише bugfix / security / compatibility для передачі).

Self-hosting core path = `asm/` + `harness/` + `external/my-lisp` meta-eval → CML → nucleus.  
CI: `.github/workflows/self-hosting-authority.yml` (не включає `dll/`).

## Inventory table

| Path | Role today | Classification | Destination | Replacement |
|------|------------|----------------|-------------|-------------|
| `dll/src/eval.rs` | Rust evaluator (quote/atom/eq/cons/car/cdr/cond/app, host primitives) | **cyberpunk-host-only** | migrate to `my-lisp-cyberpunk` or dedicated host-runtime repo | self-host: meta-eval on asm nucleus; host: keep until parity in destination |
| `dll/src/reader.rs` | Rust S-expression reader | **cyberpunk-host-only** | same | host needs reader until Lisp-side reader exists on target |
| `dll/src/printer.rs` | Word → string for FFI | **cyberpunk-host-only** | same | host diagnostics |
| `dll/src/word.rs` | Tag/word helpers over `wsm-os-target` | **cyberpunk-host-only** (also mirrors ABI) | same; numbers stay from `wsm-target-contract` | no semantic invention allowed |
| `dll/src/ffi.rs` | C ABI: session, bind, eval_string, game_handle wrap | **cyberpunk-host-only** | **must** stay available to Cyberpunk adapter | destination owns after migration |
| `dll/src/lib.rs` | links `nucleus-win64.s`, `wsm_fail_win64` | **mixed**: asm link = mechanism; session surface = host | split conceptually: nucleus stays here; session surface → host repo | asm stays in `wsm-my-lisp` |
| `dll/build.rs` | build glue for cdylib | **mechanism / host** | follow ffi | — |
| `dll/src/bin/bench.rs` | performance harness | **test/bootstrap** | host or local only | optional |
| `dll/src/bin/oom_trigger.rs` | OOM path witness | **test/bootstrap** | stay or host tests | — |
| `dll/tests/*` | parity / oom | **test/bootstrap** | migrate with host surface | — |
| `asm/nucleus.s` | SysV 5 primitives | **self-hosting core** | **this repo** | — |
| `asm/nucleus-win64.s` | Win64 5 primitives + arena reset | **mechanism for host + shared ABI** | nucleus definition stays here; used by dll | — |
| `asm/entry-*.s` | CML-linked witnesses | **self-hosting core** | this repo | — |
| `harness/*` | executed SysV witnesses | **self-hosting core** | this repo | — |
| `external/my-lisp` | semantic oracle + meta-eval.my | **oracle (external)** | pin only | never fork semantics |

## Freeze policy (dll/)

Until migration completes:

1. **Allowed in `dll/`:** bugfix, security, Windows ABI compatibility, Cyberpunk adapter breakage fixes, documentation.
2. **Forbidden in `dll/`:** new language capabilities (lambda/def/closure/meta-eval/reasoning/exact-number features beyond what is needed to keep current Cyberpunk vertical slice green).
3. New capabilities → Stage2/Stage3 self-hosting line (`meta-eval.my` → CML → `asm/nucleus.s`) or, if host-only, implemented **after** destination repo exists.

## Migration map (atomic, no silent break)

```text
Phase A (done in this doc)
  inventory + freeze + README boundary

Phase B (next)
  destination: my-lisp-cyberpunk (or wsm-host-runtime)
  copy/move dll surface with parity tests green in both places

Phase C
  my-lisp-cyberpunk adapter depends on destination only
  remove dual maintenance

Phase D
  wsm-my-lisp keeps asm/harness only for self-hosting;
  dll/ removed or reduced to thin re-export deprecation window
```

**Do not** delete `dll/` until Cyberpunk load path has a green alternate.

## C policy (reminder)

Allowed: ABI glue, OS entry, bounded allocator mechanism, byte I/O, loader shim, diagnostics.  
Forbidden: C as alternate eval/apply/cond/lambda language or truth policy.

## Next Stage2 gate (chosen, not executed in this PR)

Continue existing Stage2: next nontrivial `meta-eval.my` gate through **Lisp → CML → asm nucleus**, without linking Rust `dll/eval.rs`.

See `docs/ROADMAP.md` and harness witnesses already on main.

## Related

- Issue #15 (P0)
- Issue #10 self-hosting compiler roadmap
- `my-lisp-cyberpunk` consumes `wsm_my_lisp_cyberpunk_dll.dll` today
