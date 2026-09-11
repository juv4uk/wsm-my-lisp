# `dll/` — win64 host-embeddable WSM runtime

> **Authority note (2026-09-11, owner P0 #15)**  
> This directory is a **Cyberpunk host embed** path, not the self-hosting
> core of `wsm-my-lisp`. **New Lisp language capabilities must not be
> added here.** Allowed: bugfix, security, ABI compatibility for the
> existing vertical slice. Growth of eval/reader/closures/meta-eval
> belongs on the Lisp→asm self-hosting line or, after migration, in the
> Cyberpunk/host destination. See [`../docs/dll-inventory-2026-09-11.md`](../docs/dll-inventory-2026-09-11.md).

Host-neutral Lisp runtime for embedding this repo's WSM nucleus
(`asm/nucleus-win64.s`) into a Windows process via a stable C ABI. Built
for the my-lisp-cyberpunk embedding effort, but contains no game- or
RED4ext-specific code — this crate must build without a Cyberpunk/RED4ext
SDK. The RED4ext-facing adapter lives in `my-lisp-cyberpunk/adapter/`.

## Embed contract

### Threading: single-threaded per process

`asm/nucleus-win64.s` arena is one global unsynchronized bump allocator.
Call every `wsm_*` FFI entry from **one thread only** per process/session
lifetime. `dll/.cargo/config.toml` sets `RUST_TEST_THREADS=1` for tests;
that does not make the arena thread-safe.

### Session lifetime

- `wsm_session_init` / `wsm_session_free` — own exactly once.
- Strings from `wsm_eval_string` → `wsm_free_string` only.
- `wsm_wrap_game_handle` / unwrap — opaque tokens; caller owns real handles.

### Panics across FFI

Host callbacks must not panic across `extern "C"` (abort). See `ffi.rs`.

## Word encoding

From `wsm-os-target` / `wsm-target-contract` — not hand-copied policy.

## Testing

```sh
cargo test --target x86_64-pc-windows-msvc
```

## Performance / arena

See historical notes in git history: per-eval `wsm_arena_reset` after the
old 128-call ceiling. Limits documented in `ffi.rs`.
