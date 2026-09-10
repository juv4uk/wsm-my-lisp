# Game injection plan

Status update #2 (2026-09-10, later same day): the RED4ext plugin skeleton
this repo briefly carried at `plugin/` has been REMOVED from here. Owner
issue [wsm-my-lisp#1](https://github.com/juv4uk/wsm-my-lisp/issues/1)
states this repo's boundary explicitly: `wsm-my-lisp` stays a host-neutral
WSM/self-hosted runtime, and RED4ext/Cyberpunk-specific code belongs in
`my-lisp-cyberpunk`, not here. That repo's `adapter/` (with its own
`deps/red4ext.sdk` submodule) is now the canonical location for the
plugin -- it already has a working first vertical slice (see its
`docs/vertical-slice.md`: a `запиши-лог` host primitive round-tripping
Lisp -> host -> Lisp, log-only, no save/inventory/player-state access).
The `plugin/` directory this doc originally described was a real,
verified-buildable duplicate of that same idea, written before the
adapter/runtime boundary was made explicit -- kept only as history below,
not as a currently-accurate description of what's in this repo.

Status update #1 (2026-09-10, earlier the same day, kept for history): the
plan below was originally written as research-only, blocked on an
explicit owner go-ahead. That go-ahead was given (confirmed directly with
the user, not just relayed) and the plugin skeleton described in §(b) was
built and verified in THIS repo at the time -- CMake+MSVC built it
cleanly, and it exported exactly the 3 functions RED4ext's loader
requires (`Main`/`Query`/`Supports`, confirmed via `dumpbin -exports`).
It was never loaded into a real running game process from here (no
Cyberpunk 2077 installation was available in this environment) before
being superseded by the extraction described in update #2 above. The
original research is kept for context, with corrections flagged inline
where the actual vendored SDK differed from what was originally found on
the web -- that correction is still accurate and still relevant to
`my-lisp-cyberpunk/adapter`, which inherited the same fix.

**Important correction, found only by actually vendoring and building
against the real SDK, not by re-reading docs.red4ext.com harder**: the
`RED4ext::PluginHandle`/`RED4EXT_SEMVER`/`RED4EXT_RUNTIME_INDEPENDENT`-style
names below (§a, from docs.red4ext.com and an older example repo) do NOT
match the SDK version this project vendored (commit `ad72777`, cloned
2026-09-10). That version namespaces everything under `RED4ext::v1::` and
uses `RED4EXT_V1_`-prefixed macros instead (`RED4ext::v1::PluginHandle`,
`RED4EXT_V1_SEMVER`, `RED4EXT_V1_RUNTIME_VERSION_INDEPENDENT`, etc.) --
confirmed against the SDK's own vendored headers and its own
`examples/execute_functions/Main.cpp`, not against the web docs, which
are stale relative to this SDK version. `plugin/src/Main.cpp` uses the
correct current names; treat this document's §(a) code sample below as
historical illustration, not literal current API.

This is also still blocked on `dll/`'s wider RTTI/host-primitive-registration
integration for anything beyond "prove the DLL loads" -- a real mod script
still can't be meaningfully evaluated inside the game yet. Nothing in
§(a)/(b) below depended on that, though: this was always purely "how does
a DLL get into the process and prove it's there," not "what does the DLL
do once loaded."

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

## (b) Minimal RED4ext plugin skeleton -- MOVED to `my-lisp-cyberpunk/adapter/`

This section describes what `plugin/` in THIS repo used to contain,
before the update #2 extraction above. It is no longer present here --
see `my-lisp-cyberpunk/adapter/` for the current, maintained version
(same design, now living in the repo boundary it belongs in):

1. `plugin/` is a separate C++20 CMake project (RED4ext plugins are C++
   against RED4ext.SDK's C++ headers, not something `dll/`'s Rust crate
   can directly satisfy). RED4ext.SDK is vendored as a git submodule at
   `plugin/deps/red4ext.sdk` (same pattern as `external/my-lisp`).
2. `plugin/src/Main.cpp`'s `Main(..., EMainReason::Load, ...)`:
   - Resolves its own directory via `GetModuleHandleExW`/
     `GetModuleFileNameW` (not the working directory, which RED4ext does
     not guarantee), then `LoadLibraryW`s `wsm_my_lisp_cyberpunk_dll.dll`
     from that same directory.
   - `GetProcAddress`es `wsm_session_init` and calls it.
   - Logs success/failure via the real `RED4ext::v1::Logger` API
     (`aSdk->logger->InfoF`/`ErrorF`) -- the exact API §(a) above flagged
     as "not yet looked up" is now used for real, confirmed against the
     vendored SDK's `include/RED4ext/Api/v1/Logger.hpp` and `Sdk.hpp`.
   - No real game-facing behavior yet -- this only proves the Rust DLL
     loaded and `wsm_session_init` is callable in-process.
3. `Main(..., EMainReason::Unload, ...)`: calls `wsm_session_free` on the
   session handle (if init succeeded), then `FreeLibrary`s the module.
4. `Query`/`Supports` export the plugin's identity and declare
   `RED4EXT_V1_RUNTIME_VERSION_INDEPENDENT` (this skeleton doesn't touch
   game RTTI/state, so it isn't pinned to one game version) and
   `RED4EXT_API_VERSION_1`.

**Verified, not assumed**: `cmake -G "Visual Studio 17 2022" -A x64` then
`cmake --build . --config Release` succeeds end to end (RED4ext.SDK's ~40
source files compile, then `Main.cpp`), producing
`wsm-my-lisp-cyberpunk-plugin.dll`. `dumpbin -exports` on that DLL shows
exactly `Main`, `Query`, `Supports` -- the 3 symbols RED4ext's loader
requires, correctly exported, not mangled.

**NOT yet verified** (no Cyberpunk 2077 installation available in this
environment): actually placing this DLL + `wsm_my_lisp_cyberpunk_dll.dll`
in a real `<game_directory>/red4ext/plugins/` and confirming RED4ext's
real loader accepts and loads it, that `wsm_session_init` really returns
a valid session pointer inside the actual game process (not just a test
harness), and the exact log output location (RED4ext's own log file,
path unconfirmed). Also still open, carried over unchanged from before
this update:

- Whether calling the Rust `cdylib`'s exports from inside the real game
  process needs anything beyond what `dll/src/ffi.rs` already provides.
  `ffi.rs` does have `catch_unwind` on its exported functions now (see
  commit `affc570`) -- but see that commit's own confirmed limit: a panic
  *inside* a host-registered callback still aborts the process outright,
  `catch_unwind` doesn't reach that case. Not exercised against a real
  RED4ext-hosted callback yet, only Rust-side unit tests.

**Threading rule for any adapter calling into `wsm_my_lisp_cyberpunk_dll.dll`**:
single-threaded per session, no exceptions -- `dll/README.md`'s "Embed
contract" section has the full, confirmed reasoning (the win64 nucleus's
arena is one unsynchronized global bump allocator). An adapter must call
every `wsm_*` function for a given session from one thread; RED4ext's own
`Main`/callback dispatch model needs to be checked against this before any
future capability registers a primitive that might be invoked from a
different thread than the one that called `wsm_session_init`.

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
