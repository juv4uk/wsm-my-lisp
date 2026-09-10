# RTTI capability design — research only, no implementation

Status: **ground-preparation**, per the same discipline as
`docs/game-injection-plan.md`'s original research pass — this document
researches and records how a future read-only RTTI capability (e.g.
`game-version`, `player-position/read`) could reach a Lisp session as an
opaque value, per `my-lisp-cyberpunk#1`'s own requirement ("Lisp повинен
бачити opaque capability/value, а не довільні host pointers"). **No
capability is implemented here** — this is the step after the proven
log-only vertical slice (`запиши-лог`), not that step itself, and it
still needs its own explicit go-ahead before real code, per this repo's
"first step on a new topic belongs to the owner" discipline.

This document is split deliberately: the RED4ext-specific half (how to
call into RTTI) belongs in `my-lisp-cyberpunk`, not here, per this
repo's own host-neutral boundary (`wsm-my-lisp#1`, closed). What's
recorded here is only the **host-neutral half**: how an opaque handle
would be represented on this crate's side of the FFI boundary once one
exists. The RED4ext-specific research findings were relayed directly to
the `my-lisp-cyberpunk` coordination session rather than written into
that repo from here.

## The real finding: `Tag::Capability` already exists, but for a different shape

`wsm_os_target` (the same ratified crate `dll/` already consumes for
`Tag::Boxed`) already defines `Tag::Capability = 6`, with
`CapabilityKind` (`PciConfig`/`Mmio`/`Dma`/`Interrupt`),
`CapabilityDescriptor` (kind + instance + provenance nonce), and
`encode_capability`/`decode_capability`/`encode_capability_descriptor`.
This is NOT free real estate for an RTTI game-object handle without
checking the fit first — it was designed for **image-local**, hardware/OS
capabilities (its own doc comment: "non-zero **image-local** capability
handle"), a fundamentally different lifetime and kind-set than a
dynamically-created, per-session game object reference.

A game RTTI handle (e.g. "the player entity," "this NPC") is:

- **session-local**, not image-local — it only makes sense for the
  lifetime of one game process/session, exactly like `Tag::Boxed`'s
  ratified scope, not like `Tag::Capability`'s current image-local one.
- **not** one of the existing `CapabilityKind` variants (PciConfig/Mmio/
  Dma/Interrupt are hardware/OS concepts, not game concepts) -- reusing
  `Tag::Capability` as-is would either require a new `CapabilityKind`
  variant that doesn't fit the existing hardware-flavored set, or
  misusing an existing one.

**Open question for `wsm-target-contract`, not decided here**: does a
game/RTTI handle belong under `Tag::Capability` with a new session-local
`CapabilityKind` variant (inconsistent with that tag's current
image-local scope), or should it follow `Tag::Boxed`'s own already-
ratified session-local table pattern instead (a `BoxedValue::GameHandle`
variant, or a parallel table) -- reusing the mechanism that already fits
the right lifetime, rather than the mechanism that happens to share the
word "capability"? This crate's own inclination, not a decision: the
`Boxed` pattern is the closer fit by lifetime, and `dll/src/word.rs`'s
`BoxedTable` already has the exact "opaque session-local handle, kind
carried inside the table entry" shape this needs -- but the name
`Tag::Capability` is also more semantically honest for something the
issue explicitly wants Lisp to see as "a capability, not a raw pointer."
This tension is real and worth raising with `wsm-target-contract`
directly before any implementation, not resolved unilaterally here.

## What would NOT change on this crate's side, regardless of which tag wins

Whichever tag ends up carrying a game-object handle, the shape of the
change to `dll/` would be the same kind of thing `BoxedTable` already is:

- A new session-local table in `Session` (`ffi.rs`), parallel to
  `BoxedTable`/`SymbolTable`.
- A new `HostPrimitiveFn`-style registration path, or an extension to the
  existing one, for a primitive that returns an opaque handle word
  instead of (or alongside) a plain `u64` -- the C side would put a raw
  RTTI pointer into the table via some new FFI function (`wsm_capability_wrap`
  or similar, not designed here), get back an opaque word, and that word
  is what a Lisp expression like `(player-position гравець)` would carry
  and pass back to a later primitive call -- never dereferenceable from
  Lisp itself, matching the issue's own requirement.
- `dll/README.md`'s existing single-threaded-per-session rule already
  covers this case too: the table backing whatever tag is chosen is not
  synchronized, same limitation as `BoxedTable`.

None of this is designed in detail or implemented -- this section exists
so a future implementation pass starts from "which existing pattern does
this extend" rather than inventing a third one.

## RED4ext-side research (relayed, not written into that repo from here)

Read directly from the already-vendored `RED4ext.SDK` (on disk at
`my-lisp-cyberpunk/adapter/deps/red4ext.sdk`, not re-vendored here) for
context on what a future `my-lisp-cyberpunk`-side implementation would
call into. Recorded here only because this document needs the context to
explain the host-neutral design questions above; the RED4ext-specific
document itself belongs in `my-lisp-cyberpunk/docs/`, written by that
repo's own session.

- `RED4ext::CRTTISystem::Get()` is the entry point; `GetType(CName)`,
  `GetClass(CName)`, `GetFunction(CName)` (global script functions) are
  the lookup primitives (`include/RED4ext/RTTISystem.hpp`).
- Calling a resolved function uses the `ExecuteFunction` family
  (`include/RED4ext/Scripting/Utils.hpp`) -- several overloads taking an
  instance pointer or `CClass*`/`CName` context plus a `CBaseFunction*`
  or `CName` function reference and a `StackArgs_t`.
  `examples/execute_functions/Main.cpp` (in the vendored SDK) shows this
  pattern end-to-end for a global system (`gameTimeSystem`), which is
  closer to `game-version` in shape than `player-position` is.
- **Gap closed (2026-09-10, second research pass)**: the player instance
  is NOT obtained eagerly inside a plugin's `Main`. The vendored SDK's
  `examples/accessing_properties/Main.cpp` shows the real pattern:
  1. `Main(..., EMainReason::Load, ...)` only registers callbacks --
     `RED4ext::CRTTISystem::Get()->AddRegisterCallback(RegisterTypes)`
     and `AddPostRegisterCallback(PostRegisterTypes)`.
  2. `PostRegisterTypes` (called later by RED4ext, not by the plugin)
     registers one or more `CGlobalFunction`s via
     `rtti->RegisterFunction(...)` -- these become the plugin's actual
     callable entry points from the game's own scripting system.
  3. **Only inside one of those registered functions**, when RED4ext or
     game script code actually invokes it, does the code call
     `RED4ext::ExecuteGlobalFunction("GetPlayer;GameInstance", &handle,
     gameInstance)` against a **default-constructed**
     `RED4ext::ScriptGameInstance gameInstance;` -- the instance appears
     to resolve against an implicit "current game" context at call time,
     not something the plugin has to explicitly obtain/store from `Main`.
     The result, `RED4ext::Handle<RED4ext::IScriptable>`, is RED4ext's
     own reference-counted/checked handle type (not a bare pointer) --
     truthy-checked (`if (handle)`) before use.
  4. From that handle, `rtti->GetClass("PlayerPuppet")->GetProperty(...)`
     (e.g. `inCrouch`) or `->GetFunction(...)` (e.g. `GetHudManager`)
     reads a property or calls a further method on the resolved instance.
  This directly answers this doc's earlier open question: a game-facing
  Lisp primitive's C callback (the `HostPrimitiveFn` registered through
  `wsm_register_primitive`) is exactly the right place to call
  `ExecuteGlobalFunction("GetPlayer;GameInstance", ...)` -- not something
  that needs to happen earlier in adapter startup. `RED4ext::Handle<T>`
  is also a plausible concrete type to wrap into whichever opaque word
  representation (`Tag::Boxed` extension or new `CapabilityKind`) the
  open tag question above resolves to.
