# `dll/` — Windows host embed (**mirror**)

> **2026-09-11 — canonical home moved**  
> **https://github.com/juv4uk/my-lisp-cyberpunk/tree/main/host-runtime**
>
> This directory is a **compatibility mirror** until Cyberpunk adapter/CI
> build only from `my-lisp-cyberpunk/host-runtime` (migration Phase C/D).
> Prefer landing host/FFI fixes **there**. Self-hosting core of *this* repo
> remains `asm/` + `harness/` (Lisp-first, no Rust eval growth here).

See `docs/dll-inventory-2026-09-11.md` and `docs/AUTHORITY.md`.

Host-neutral Lisp runtime linking `asm/nucleus-win64.s` into a cdylib for
Cyberpunk. No RED4ext types in this crate.

## Freeze (still in force on this mirror)

No new Lisp language capabilities in Rust here. Bugfix / security /
compat only — and prefer the cyberpunk tree.

## Build

```sh
cargo test --target x86_64-pc-windows-msvc
```

Prefer building from `my-lisp-cyberpunk/host-runtime` after sync.
