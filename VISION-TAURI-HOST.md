# Vision: Tauri as a replaceable host shell for the Lisp Machine

Status: long-range architectural vision, not an immediate implementation mandate.

## Thesis

The Lisp Machine is the system. Tauri is one possible body/host for it.

```text
                 Lisp Machine
                      |
              Capability Boundary
                 /          \
                /            \
          Tauri host       future host
              |                 |
       Linux / Windows      WSM OS Lisp
```

Tauri must not become the semantic authority of the machine. It supplies mechanisms that already exist and are expensive or pointless for us to reinvent early: windows, WebView, keyboard/mouse input, filesystem/process/network access and OS integration.

## Architectural law

If replacing Tauri with another host changes the meaning of a Lisp program, the boundary is wrong.

- Lisp owns semantics, state, commands, plugins and policy.
- CML/compiler owns lowering/compilation toward the machine.
- Host adapters own mechanism only.
- Host-specific APIs are exposed through explicit capabilities.
- No Lisp semantic truth is duplicated in Rust, JavaScript or UI glue.

## Web capability

Do not build a browser engine merely to make the Lisp Machine useful on the Web. Tauri already provides access to a system WebView. Treat Web rendering/navigation as a host capability first.

Conceptual boundary:

```text
(web-open uri)
      |
      v
Web capability
      |
      +-- Tauri/WebView adapter
      |
      +-- future WSM OS adapter
```

A WebView engine is not itself a complete browser product. Browser policy, tabs, history, permissions and similar behavior should be added only when an experiment demonstrates that the Lisp Machine needs them.

## Evolution

Phase A: Lisp Machine uses Tauri on existing operating systems.

Phase B: the same Lisp capability contracts have at least two implementations: Tauri host and WSM OS Lisp host.

Phase C: evidence determines whether Tauri remains useful. Do not predetermine that it must disappear.

## Required future witnesses

Before claiming host independence, create executable RED -> GREEN witnesses proving:

1. the same canonical `.lisp` program executes through two host adapters without semantic changes;
2. host capabilities are explicit and fail closed when unavailable;
3. no expected Lisp answers are encoded in a host adapter;
4. Web capability can use an existing WebView without making WebView/Tauri semantic authority;
5. compiler/artifact/host provenance is observable;
6. replacing a host does not require rewriting Lisp-side commands/plugins/policy.

## Non-goals

- no home-grown browser engine merely for independence;
- no Tauri-specific Lisp dialect;
- no giant universal capability API before concrete use cases;
- no claim that Tauri is WSM OS Lisp;
- no forced removal of Tauri once a native WSM host exists.

The objective is simple: **give the Lisp Machine a useful body today without giving that body authority over the mind.**
