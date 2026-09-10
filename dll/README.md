# `dll/` — win64 host-embeddable WSM runtime

Host-neutral Lisp runtime for embedding this repo's WSM nucleus
(`asm/nucleus-win64.s`) into a Windows process via a stable C ABI. Built
for the my-lisp-cyberpunk embedding effort, but contains no game- or
RED4ext-specific code — see `wsm-my-lisp#1`'s own acceptance criterion:
this crate must build without a Cyberpunk/RED4ext SDK, and it does
(`cargo build --target x86_64-pc-windows-msvc`, no non-Rust dependency
beyond the vendored asm). The RED4ext-facing adapter that loads this
DLL lives in a separate repo, `my-lisp-cyberpunk/adapter/`.

## Embed contract

A consumer linking against `wsm_my_lisp_cyberpunk_dll.dll` needs to know
these rules; they are not enforced by the type system on the C side, only
documented here and in the source.

### Threading: single-threaded per process, no exceptions

`asm/nucleus-win64.s`'s arena (`wsm_arena`/`wsm_arena_next`/
`wsm_arena_end`) is **one global, unsynchronized bump allocator** — by
design, matching `asm/nucleus.s`'s SysV original, which assumes a
single-threaded embedding. There is no lock, no per-thread arena, no
atomic bump pointer.

**Confirmed, not hypothetical**: during development, running this
crate's own test suite with Rust's default multi-threaded test runner
corrupted results in practice — two test threads racing on the same
arena pointer produced wrong values and spurious `NotCallable` errors in
unrelated tests. `dll/.cargo/config.toml` forces
`RUST_TEST_THREADS=1` to work around this for `cargo test`; that fixes
the test suite, it does **not** make the arena itself thread-safe.

**Rule for any embedder**: call every `wsm_*` function in this crate's
FFI surface (`wsm_session_init`, `wsm_register_primitive`, `wsm_bind`,
`wsm_eval_string`, `wsm_wrap_game_handle`, `wsm_unwrap_game_handle`,
`wsm_session_free`, and any host-registered `HostPrimitiveFn` callback)
from **one thread only**, for the lifetime of
a given `Session`. Calling from multiple threads concurrently, even with
external synchronization around individual calls, does not make this
safe: the arena has no memory ordering guarantees across threads at all.
If a future consumer genuinely needs multi-threaded access, that requires
either a locked arena or a per-thread/per-session arena in
`asm/nucleus-win64.s` itself — not implemented, not attempted here.

### Session lifetime and ownership

- `wsm_session_init()` returns an opaque `Session*`. The caller owns it
  and must eventually call `wsm_session_free()` on it exactly once — not
  zero times (leak) and not twice (double-free/use-after-free, undefined
  behavior).
- Every other `wsm_*` function taking a `Session*` requires a live,
  not-yet-freed session pointer from `wsm_session_init`.
- A `Session` owns its own `Env` (host primitive/binding registry),
  `SymbolTable`, and `BoxedTable` (`Str`/`GameHandle` boxed values) --
  **session-local**, not shared or image-local. This matches
  `wsm-target-contract`'s ratified scope for `Tag::Boxed` handles (see
  `docs/migration-2026-09-10-boxed-tag.md` and
  `docs/migration-2026-09-10-game-handle-boxed-kind.md` in that repo): a
  boxed handle is only meaningful within the session that created it,
  never across sessions or processes.
- A string returned by `wsm_eval_string` is heap-allocated by this crate
  (`CString::into_raw`) and must be freed with `wsm_free_string` --
  exactly once, never with a foreign `free`/`delete`.
- `wsm_wrap_game_handle(session, handle, &mut out)` stores an opaque
  host pointer (e.g. a RED4ext RTTI handle) into the session's boxed
  table and writes the resulting Word to `out`; this crate never
  dereferences `handle`. `wsm_unwrap_game_handle(session, word, &mut out)`
  recovers it -- returns 1 (not a crash) if `word` isn't a `GameHandle`
  Boxed word. The caller is solely responsible for the wrapped pointer's
  validity for as long as any Lisp value might still reference it.

### Panics across the FFI boundary

`wsm_register_primitive`, `wsm_bind`, and `wsm_eval_string` wrap their
bodies in `std::panic::catch_unwind`, so a panic in this crate's own
Rust logic turns into an ordinary error return instead of unwinding
further.

**Confirmed limit, not a gap glossed over**: this does **not** protect
against a panic *inside a host-registered `HostPrimitiveFn` callback*.
That callback type is declared plain `extern "C"` (not `extern
"C-unwind"`), and modern rustc inserts a hard `abort()` right at that
function's own boundary the instant a panic tries to cross it -- before
it ever reaches this crate's `catch_unwind`, which sits one stack frame
further up. Reproduced deliberately during development (see `ffi.rs`'s
own module doc for the exact crash signature). A real C++ host callback
can't panic in the Rust sense, so this may not be reachable in practice
from RED4ext/CET -- but a future Rust-side host callback (e.g. a
same-process test double) that panics still takes the whole process down
with it, full stop.

## Word encoding

See `word.rs`'s own module doc for the full, current tag layout. In
short: `Cons`/`Nil`/`Fixnum`/`Symbol`/`Closure`/`Capability`/`Boxed`
come from `wsm_os_target::Tag` (a pinned git dependency on
`wsm-target-contract`, not a hand-copied constant) -- `Boxed` (string
literals and opaque game handles, via `BoxedValue::Str`/`GameHandle`) is
session-local and non-interned, everything else follows that crate's own
documented semantics.

## Testing

```sh
cargo test --target x86_64-pc-windows-msvc
```

50 tests total: 49 in-process unit/integration tests (including
`tests/my_lisp_fixture_parity.rs`, a consolidated parity harness against
my-lisp's own conformance fixtures) plus one real subprocess test
(`tests/oom_path.rs`) that deliberately exhausts the 4096-byte arena and
checks the resulting crash/exit path for real, rather than asserting it
would probably work.
