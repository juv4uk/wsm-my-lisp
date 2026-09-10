# Game injection plan (research, not implementation)

Status: **ground-preparation only**, per owner directive relayed 2026-09-10
via the my-lisp-cyberpunk coordination session. This document researches
and records how `dll/` (the win64 nucleus + reader/evaluator/FFI crate,
see [f91ee1c](https://github.com/juv4uk/wsm-my-lisp/commit/f91ee1c) and
[955ee4e](https://github.com/juv4uk/wsm-my-lisp/commit/955ee4e)) would get
loaded into a running Cyberpunk 2077 process. **No injection code exists
yet and none is written here** -- per this repo's own "first step on a
new topic belongs to the owner, not the agent" discipline, actually
writing and loading a game-process plugin needs its own explicit
go-ahead, separate from this research.

This is also still blocked on the same 3 open semantics questions as the
rest of the cyberpunk effort (cond special-form-vs-macro, bare-symbol
bindings, string encoding) -- a real mod script can't be meaningfully
evaluated without them. Nothing here depends on those, though: this is
purely "how does a DLL get into the process and prove it's there," not
"what does the DLL do once loaded."

## (a) The standard, sanctioned path: RED4ext, not raw injection

Cyberpunk 2077's mod ecosystem has a documented, community-standard
extension point for exactly this: **RED4ext**, a plugin loader the game
itself is patched to load at startup (via CDPR's own supported modding
path, not a third-party process injector). This is the appropriate
mechanism here, not a bespoke DLL-injection tool -- reinventing process
injection would be both more fragile and outside what the game's own
modding ecosystem expects a plugin to do.

- Plugin DLLs are placed in `<game_directory>/red4ext/plugins/` (or a
  subdirectory of it); a plugin's own dependency DLLs can sit alongside
  it in the same directory.
- RED4ext's own loader (installed once by the player/modder, not part of
  this repo's concern) finds and loads every plugin DLL under that
  directory when the game starts.
- A plugin exports exactly two `extern "C"` functions RED4ext calls
  directly, per its documented ABI:

  ```cpp
  RED4EXT_C_EXPORT bool RED4EXT_CALL Main(
      RED4ext::PluginHandle aHandle,
      RED4ext::EMainReason aReason,
      const RED4ext::Sdk* aSdk)
  {
      switch (aReason) {
          case RED4ext::EMainReason::Load:   /* init here */   break;
          case RED4ext::EMainReason::Unload: /* teardown here */ break;
      }
      return true;
  }

  RED4EXT_C_EXPORT void RED4EXT_CALL Query(RED4ext::PluginInfo* aInfo)
  {
      aInfo->name = L"...";
      aInfo->author = L"...";
      aInfo->version = RED4EXT_SEMVER(1, 0, 0);
  }
  ```

  (`RED4EXT_C_EXPORT`/`RED4EXT_CALL` are RED4ext.SDK macros for
  `extern "C" __declspec(dllexport)` / `__fastcall`.) RED4ext calls
  `Main(..., EMainReason::Load, ...)` only after it has already checked
  the plugin is compatible with the running game version -- version
  mismatches are rejected before `Main` runs, not something this plugin
  needs to check itself.
- Plugins must be 64-bit, matching this repo's `dll/` crate already
  targeting `x86_64-pc-windows-msvc`.

## (b) Minimal RED4ext plugin skeleton (design only, not written)

The smallest thing that would prove the DLL loads in-process at all:

1. A separate small **C++** project (RED4ext plugins are C++ against
   RED4ext.SDK's C++ headers, not something `dll/`'s Rust crate can
   directly satisfy -- `dll/` would be loaded *by* this C++ shim, not
   *as* the RED4ext plugin itself. Two DLLs: a thin RED4ext-plugin.dll in
   C++, plus this repo's existing `wsm_my_lisp_cyberpunk_dll.dll`).
2. The C++ shim's `Main(..., EMainReason::Load, ...)`:
   - `LoadLibraryW` the Rust `wsm_my_lisp_cyberpunk_dll.dll` (placed
     alongside it in `red4ext/plugins/`).
   - `GetProcAddress` for `wsm_session_init` (already exported per
     `dll/src/ffi.rs`) and call it.
   - Log success/failure through RED4ext's own logging facility (exact
     API not yet looked up here) -- no real game-facing behavior yet,
     just proof the Rust DLL loaded and its FFI surface is callable
     in-process.
3. `Main(..., EMainReason::Unload, ...)`: call `wsm_session_free` on the
   handle, then `FreeLibrary`.
4. Build via RED4ext's own documented CMake/Premake example projects
   (`WopsS/RED4ext.Example.CMake`, `WopsS/RED4ext.Example.Premake` --
   third-party community examples referenced by RED4ext's own docs, not
   authored here).

Not yet resolved by this research, needs an actual attempt to answer:

- Whether calling a Rust `cdylib`'s exports from a `LoadLibraryW`'d
  context inside the game process needs anything beyond what `dll/src/
  ffi.rs` already provides (thread-local state, panic unwinding across
  the FFI boundary -- Rust panics unwinding into C++ is UB and must be
  caught with `catch_unwind` at every exported function; `ffi.rs`
  doesn't do this yet).
- Exact RED4ext logging API call (left as a placeholder above).

## CET (CyberEngineTweaks) coexistence

Not confirmed by this research -- the RED4ext documentation fetched here
says nothing about CET compatibility. Worth noting, not yet verified:
CET has, in recent versions, come to depend on RED4ext being present
(rather than being a fully independent injector) -- if that's still
accurate, a RED4ext plugin and CET should coexist by construction, since
CET itself already relies on the same loader this plan uses. This needs
direct verification against current CET release notes before being
treated as fact, not assumed from memory.

## Sources

- [Creating a Plugin | RED4ext](https://docs.red4ext.com/mod-developers/creating-a-plugin)
- [Creating a plugin with RedLib | RED4ext](https://docs.red4ext.com/mod-developers/creating-a-plugin-with-redlib)
- [RED4ext.Example.CMake](https://github.com/WopsS/RED4ext.Example.CMake)
- [RED4ext.Example.Premake](https://github.com/WopsS/RED4ext.Example.Premake)
- [RED4ext.Example.VisualStudio/src/Main.cpp](https://github.com/WopsS/RED4ext.Example.VisualStudio/blob/master/src/Main.cpp)
- [Red4Ext-Wiki: creating a plugin](https://github.com/CDPR-Modding-Documentation/Red4Ext-Wiki/blob/main/mod-developers/creating-a-plugin.md)
