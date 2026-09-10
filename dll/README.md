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
  caller-defined token into the session's boxed table and writes the
  resulting Word to `out`; this crate never dereferences `handle`.
  `wsm_unwrap_game_handle(session, word, &mut out)` recovers it --
  returns 1 (not a crash) if `word` isn't a `GameHandle` Boxed word.
  **`handle` must NOT be a raw refcounted engine pointer** (e.g. a
  `RED4ext::Handle<T>`'s `T*` taken from a `Handle<T>` that then goes
  out of scope) -- this crate holds no reference of its own, so that
  pointer would dangle the instant its real owner releases it. The
  caller must keep its own table of real, refcount-holding handles and
  pass an opaque token identifying a row in THAT table instead (see
  `wsm_wrap_game_handle`'s own doc comment in `ffi.rs` for the full
  reasoning). The caller remains solely responsible for the token's
  validity for as long as any Lisp value might still reference it --
  this crate cannot check or enforce that.

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

51 tests total: 50 in-process unit/integration tests (including
`tests/my_lisp_fixture_parity.rs`, a consolidated parity harness against
my-lisp's own conformance fixtures) plus one real subprocess test
(`tests/oom_path.rs`) that deliberately exhausts the 4096-byte arena and
checks the resulting crash/exit path for real, rather than asserting it
would probably work.

## Performance

`cargo run --release --bin bench --target x86_64-pc-windows-msvc -- all`
runs `src/bin/bench.rs`, a manual `std::time::Instant` timing harness
(no criterion/nightly dependency). Read that file's own header comment
for the full context on each number below:

- `wsm_eval_string("(noop)")`, full FFI round trip: ~480 ns/call.
- The same work called directly in Rust (no `CString`/`CStr`/
  `catch_unwind` marshaling): ~365 ns/call -- so the FFI boundary itself
  costs roughly ~115 ns/call (~25%) here, not the dominant cost.
- `eval()` alone on an already-parsed word (no re-parsing text each
  call): ~40 ns/call -- the reader, not the evaluator, is most of a
  fresh `wsm_eval_string` call's cost (~325 ns/call of the ~365 ns
  no-FFI figure).
- `BoxedTable::add_string`/`get_string`: confirmed O(1) as the table
  grows (measured at 100, 10,000, and 1,000,000 entries) -- lookups stay
  at ~5-7 ns/call regardless of table size, as the `Vec`-index design
  predicts, rather than assuming it.

### Arena capacity -- found broken, then fixed (both real, both worth knowing)

`asm/nucleus-win64.s`'s arena (see "Threading" above) is a single global
4096-byte (256-cell) bump allocator -- originally never reset, for the
entire lifetime of the loaded DLL, not per-session, not reclaimable. A
controlled measurement pinned the exact ceiling this caused:
`wsm_eval_string("(quote a)")` (a 2-cons-cell expression, 32 bytes)
succeeded for **exactly 128 calls** (4096 / 32 = 128, confirmed
empirically) before the 129th call hit `wsm_fail_win64` and aborted the
whole process -- a real game session issuing more than ~128-256 total
list-allocating Lisp commands across its entire lifetime would have
hard-crashed the same way, regardless of how fast the interpreter itself
ran.

**Fixed** (owner go-ahead, per-eval arena reset): `asm/nucleus-win64.s`
now exports `wsm_arena_reset`, and `ffi.rs`'s `wsm_eval_string` calls it
at the start of every top-level eval. `repeated_eval_string_calls_
survive_past_the_old_arena_ceiling` (in `ffi.rs`'s own test module) runs
500 calls -- past the old 128-call ceiling -- and passes. Read
`eval_str`'s own doc comment in `ffi.rs` for the precise safety
reasoning and its two known, currently-unexercised limits: (1) it
assumes Lisp code never needs a cons value to survive past the end of
one top-level eval (true today -- no `def`/`let`/closures exist yet;
would need revisiting if any of those are added), and (2) the arena is
still one global resource, not per-`Session` -- only one `Session`
should be mid-eval-lifetime at a time, since one session's reset
discards any other live session's in-flight cons allocations too.
