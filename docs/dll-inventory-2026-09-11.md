# dll/ inventory — P0 #15 (2026-09-11)

**Рішення власника:** `wsm-my-lisp` = Lisp-first + asm substrate + bounded C.  
**Rust у runtime цього repo — не напрямок росту.** Host/embed Rust → Cyberpunk track.

## Machine-checkable guard

```bash
bash scripts/check-lisp-first-authority.sh
```

Also runs on CI via `.github/workflows/self-hosting-authority.yml`.

Fails closed if:
- `eval.rs` / `reader.rs` / `printer.rs` / `ffi.rs` / `word.rs` appear **outside** `dll/`
- required authority docs or nuclei are missing
- `dll/README.md` loses freeze/Cyberpunk wording
- self-hosting workflow path-triggers on `dll/`

## Authority boundary

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

`dll/` = frozen Cyberpunk host embed (see table). Self-hosting CI does not depend on it.

## Inventory table

| Path | Role today | Classification | Destination | Replacement |
|------|------------|----------------|-------------|-------------|
| `dll/src/eval.rs` | Rust evaluator | **cyberpunk-host-only** | my-lisp-cyberpunk / host-runtime | meta-eval on asm |
| `dll/src/reader.rs` | Rust reader | **cyberpunk-host-only** | same | host until Lisp reader on target |
| `dll/src/printer.rs` | Word → string | **cyberpunk-host-only** | same | host diagnostics |
| `dll/src/word.rs` | Tag helpers via wsm-os-target | **cyberpunk-host-only** | same | ABI from target-contract |
| `dll/src/ffi.rs` | C ABI session/eval/handle | **cyberpunk-host-only** | must stay for adapter | destination owns after migration |
| `dll/src/lib.rs` | links nucleus-win64 | **mixed** | nucleus here; session → host | — |
| `asm/nucleus.s` | SysV primitives | **self-hosting core** | this repo | — |
| `asm/nucleus-win64.s` | Win64 primitives | **mechanism** | this repo | used by dll |
| `harness/*` | witnesses | **self-hosting core** | this repo | — |
| `external/my-lisp` | oracle | **external** | pin only | — |

## Freeze policy (dll/)

1. Allowed: bugfix, security, Windows ABI compatibility for current Cyberpunk slice.
2. Forbidden: new language capabilities in Rust dll/.
3. New capabilities → Stage2/3 self-hosting line or post-migration host repo.

## Migration map

```text
Phase A — inventory + freeze + guard (this doc + script) ✓
Phase B — destination repo + parity tests
Phase C — adapter depends on destination only
Phase D — dll/ removed or deprecation window in wsm-my-lisp
```

## Related

Issue #15 · `docs/AUTHORITY.md` · `scripts/check-lisp-first-authority.sh`
