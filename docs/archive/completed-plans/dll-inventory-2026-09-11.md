> **ARCHIVED (2026-09-12), completed-plan.** `dll/` no longer exists in
> this repo — Phase D of the migration this document tracks was
> completed at commit `1e1549a` (2026-09-11). This is a non-normative
> historical record of how the migration happened, not current
> guidance. Current authority: [`docs/AUTHORITY.md`](../../AUTHORITY.md).
> Superseded-by: `docs/AUTHORITY.md`'s `dll/` row.

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

## Full inventory (issue #15 §2: file/component → role → destination → replacement)

Verified by enumerating `dll/`'s actual tracked files, not written from
memory. Classification uses issue #15's own three categories.

**Finding, not assumed going in**: every file below is `cyberpunk-host-only`.
None is `mechanism-needed-by-self-hosting` -- the self-hosting path
(`external/my-lisp/lib/meta-eval.my` → CML → `asm/nucleus.s`, proven via
`harness/`) is an entirely separate execution path that has never
depended on any code in `dll/`. Nothing here needs a Lisp/asm/C rewrite
to keep self-hosting working; migration is a lift-and-shift, not a
reimplementation.

| File | Role | Destination | Replacement here |
|---|---|---|---|
| `src/lib.rs` | cdylib entry point; links `asm/nucleus-win64.s`; `wsm_fail_win64` OOM handler | `cyberpunk-host-only` → `host-runtime/src/lib.rs` | none (mirror only, per Phase B/C) |
| `src/eval.rs` | Rust evaluator: `cond`/`quote` special forms (all Canon surface spellings, generated from `semantic-registry.wsm`), symbol lookup, host-primitive dispatch | `cyberpunk-host-only` -- explicitly named in issue #15 §1's forbidden list ("reader/evaluator/apply/cond/quote/... semantics") | none; self-hosting's own `cond`/`quote` path is `meta-eval.my`'s `my-eval-cond`/`my-canon-quote-name?`, proven independently via `harness/`, not this file |
| `src/reader.rs` | text → cons-structure parser (symbols, fixnums, strings) | `cyberpunk-host-only` → `host-runtime/src/reader.rs` | none |
| `src/printer.rs` | cons-structure → text (inverse of reader.rs) | `cyberpunk-host-only` → `host-runtime/src/printer.rs` | none |
| `src/ffi.rs` | `extern "C"` session API (`wsm_session_init/free`, `wsm_register_primitive`, `wsm_bind`, `wsm_eval_string`, `wsm_wrap_game_handle`/`wsm_wrap_rational` and their unwrap counterparts, `wsm_free_string`) -- the RED4ext-adapter-facing surface | `cyberpunk-host-only` → `host-runtime/src/ffi.rs` | none |
| `src/word.rs` | Word/tag re-exports from `wsm_os_target` (already the real source), `SymbolTable`, `BoxedTable`/`BoxedValue` (Str/GameHandle/Rational) -- host-session bookkeeping, not nucleus semantics (the nucleus's own tag encoding lives in `asm/` + the `wsm_os_target` crate, already properly separated) | `cyberpunk-host-only` → `host-runtime/src/word.rs` | none |
| `build.rs` | Generates `canon_spellings.rs` from `external/my-lisp/lib/surface/semantic-registry.wsm` at build time, so `eval.rs` doesn't hand-copy Canon surface spellings | `cyberpunk-host-only` -- travels with `eval.rs` | none; a self-hosted `cond`/`quote` never needs this, since Canon recognition there is `meta-eval.my`'s own `my-canon-*-name?` reading the same registry as Lisp data, not a Rust build step |
| `src/bin/bench.rs` | Manual `std::time::Instant` timing harness for `wsm_eval_string`/FFI overhead/`BoxedTable` growth | `test/bootstrap-only` → `host-runtime/src/bin/bench.rs` (Cyberpunk performance concern, not a self-hosting question) | delete from this repo after Phase D; no self-hosting equivalent needed |
| `src/bin/oom_trigger.rs` | Deliberately exhausts the arena to exercise `wsm_fail_win64`'s exit path (subprocess target for `tests/oom_path.rs`) | `test/bootstrap-only` → already synced to `host-runtime/src/bin/oom_trigger.rs` | delete from this repo after Phase D; `harness/`'s own witnesses don't need an OOM path test (SysV `nucleus.s`'s arena-exhaustion behavior is exercised, if ever, by a `harness/` binary of its own, not this one) |
| `tests/my_lisp_fixture_parity.rs` | Consolidated parity harness against `my-lisp`'s `docs/cyberpunk-host-dispatch-fixtures.md` | `test/bootstrap-only` → `host-runtime/tests/my_lisp_fixture_parity.rs` | delete from this repo after Phase D; self-hosting's own parity witnesses are `harness/`'s binaries (`harness-atom`, `harness-closure`, etc.), a separate, already-existing mechanism |
| `tests/oom_path.rs` | Subprocess test asserting `oom_trigger.rs`'s exit code/stderr | `test/bootstrap-only` → already synced to `host-runtime/tests/oom_path.rs` | delete from this repo after Phase D |
| `Cargo.toml`, `.cargo/config.toml` | Crate manifest; `RUST_TEST_THREADS=1` (arena is a single unsynchronized global, not thread-safe) | `cyberpunk-host-only` → `host-runtime/Cargo.toml` (path-adapted) | none |
| `README.md` | Embed contract docs (ownership, threading, panic-safety, performance) | `cyberpunk-host-only` -- content already migrated to `host-runtime/README.md`; this copy is now the frozen-mirror notice | none |

## Migration map (issue #15 §6)

```text
wsm-my-lisp/dll (Rust host runtime, frozen mirror)
            ↓  (already exists: host-runtime/scripts/sync-from-wsm-my-lisp.sh,
            ↓   pinned to wsm-my-lisp@4381e93 per host-runtime/VENDOR.md)
my-lisp-cyberpunk/host-runtime (destination, Phase B/C)
```

Phase C (adapter builds/loads only from `host-runtime`) and Phase D
(delete or stub this `dll/`) are `my-lisp-cyberpunk`'s own execution,
not this repo's -- per the same host/self-hosting authority boundary
this whole document exists to state. This repo's remaining
responsibility is only: (a) keep this mirror's freeze honest (bugfix/
security/compat only, enforced by `scripts/check-lisp-first-authority.sh`),
(b) not block `my-lisp-cyberpunk`'s own migration timeline.

## Inventory (mirror)

Same modules as before (`eval`, `reader`, `ffi`, …) — classification
**cyberpunk-host-only**; destination is the cyberpunk repo. See the full
table above for the file-by-file breakdown issue #15 §2 asks for.

## Migration

| Phase | Status |
|-------|--------|
| A inventory + freeze + guard | done |
| B destination tree in cyberpunk | done (sync script + ownership) |
| C adapter builds only from host-runtime | done — confirmed via `my-lisp-cyberpunk@91d4ae7` (55/55 tests green in `host-runtime`, sync script deleted, sources committed there) and independently verified here: `adapter/CMakeLists.txt` has no build dependency on `wsm-my-lisp` (loads the DLL at runtime via a relative path, agnostic to build source) |
| D delete this dll/ from wsm-my-lisp | **done, 2026-09-11** — `dll/` removed (14 tracked files), `.github/workflows/dll-tests.yml` removed, `scripts/check-lisp-first-authority.sh` updated to assert `dll/` stays absent |

**Owner directive (2026-09-11, standing) — fulfilled**: the owner said
once migration was properly complete, delete `dll/` from this repo
without asking again. Phase C was verified (not just taken on the
`host-runtime` repo's own word — its `adapter/` build config was read
directly), and `dll/` was deleted in this same session. Rust host-embed
code no longer exists anywhere in `wsm-my-lisp`; the sole home for it is
`my-lisp-cyberpunk/host-runtime` going forward.
