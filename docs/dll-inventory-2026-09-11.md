# dll/ inventory — P0 #15 (2026-09-11)

**Canonical host runtime home (Phase B):**  
`https://github.com/juv4uk/my-lisp-cyberpunk/tree/main/host-runtime`

This `dll/` tree is a **mirror** until Phase D.

## Machine-checkable guard

```bash
bash scripts/check-lisp-first-authority.sh
```

## Authority boundary

```text
Lisp owns self-hosting logic
        ↓
asm nucleus (this repo) + harness
        ↓
native x86_64

Host embed (reader/eval/FFI session) → my-lisp-cyberpunk/host-runtime
```

## Inventory (mirror)

Same modules as before (`eval`, `reader`, `ffi`, …) — classification
**cyberpunk-host-only**; destination is the cyberpunk repo.

## Migration

| Phase | Status |
|-------|--------|
| A inventory + freeze + guard | done |
| B destination tree in cyberpunk | done (sync script + ownership) |
| C adapter builds only from host-runtime | next |
| D delete or stub this dll/ | after C |
