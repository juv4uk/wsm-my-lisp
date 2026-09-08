; tasks.my — durable task plan for wsm-my-lisp, per docs/ROADMAP.md.
; Kept in the same alist shape as sibling repos' tasks.my (my-lisp,
; cml, wsm-os, fpga-lisp) so cross-repo depends-on stays legible.
; my-lisp remains the semantic oracle throughout this whole plan --
; no stage here requires or anticipates its removal.

((tasks . (
  ("BILINGUAL-DOCUMENTATION-AUDIT" .
    ((priority . 9.0)
     (capabilities . (documentation translation audit policy))
     (origin . ecosystem)
     (context . "Found in the 2026-09-08 ecosystem-wide language-debt audit (ecosystem/memory/LANGUAGE-DEBT-REGISTRY-2026-09-08.md): README.md opens with \"## Status, 2026-09-02\" (English, line 26) before \"## Статус, 2026-09-02 (укр.)\" (line 79) -- the worst pattern found across the whole audit, Ukrainian presented as a parenthetical addendum to the English version rather than an equal, first-class section.")
     (description . "Enforce the DOCUMENTATION LANGUAGE / МОВА ДОКУМЕНТАЦІЇ policy. Every human-authored markdown file (README, memory, plans, architecture, research) MUST have a substantive Ukrainian translation. Identify English-only documents and translate them. Do not translate machine identifiers or immutable historical records. Ensure every new or edited markdown document is bilingual.")
     (done . nil)))
  ("WSM-MY-LISP-STAGE0-NUCLEUS-WITNESS" . (
    (priority . 9.5)
    (capabilities . (wsm-my-lisp x86_64 asm nucleus oracle-parity))
    (origin . wsm-my-lisp)
    (description . "Get one WSM expression compiled via CML's x86_64-freestanding backend and actually EXECUTED (not merely assembled) on the owner's own x86_64 hardware, matching the my-lisp oracle. Establish a hand-written asm implementation of the 5 core primitives (wsm_cons/car/cdr/eq/atom) as a parallel path alongside wsm-os-runtime's Rust, each proven by a real executed witness. Bring lib/meta-eval.my in as a git submodule (external/my-lisp), not a copy.")
    (done . (t . "Claude Sonnet 5 (Ecosystem Lead) 2026-09-02: (atom (quote ())) compiled via cml/src/x86_freestanding.rs, executed via wsm-os-hosted, printed t, matched oracle (cml 6d74b8d, wsm-os a2aae35). Extended same day to bounded lambda application ((lambda (x) x) 7) -> 7 and a genuinely escaping curried closure -> (A B), both executed and oracle-checked (cml 56ef40b). asm/nucleus.s: hand-written x86_64 GNU asm for wsm_cons/wsm_car/wsm_cdr/wsm_eq/wsm_atom, same extern C ABI as CML-generated code already expects, bounded 4096-byte/256-cell static arena (no GC/growth), scalar x86_64 only per OWNER-HARDWARE-PROFILE.md. All 5 primitives independently verified executed: harness-atom -> t, harness-cons -> (A . B) (exercises the actual allocator), harness-lambda -> 7, harness-eq -> both cases correct (wsm-my-lisp bcd3d5d). external/my-lisp submodule added per the owner's mid-task correction (same pattern as my-idea's external/my-lisp, not copy-paste). CORRECTION same day, found by the owner directly (\"як так вийшло що потрібно зробити крок 5 замість 0\"): this Stage was originally built using wsm-os-target::Tag::True -- the exact same manufactured-primitive violation later found and fixed in fpga-lisp (Stage 5) -- because it was written before the owner's () discipline (\"do not define (), preserve it\"; the pre-synthesis test) was fully articulated in this same conversation, and never re-audited against it afterward. Fixed same day (wsm-my-lisp a630376, wsm-os d5a65c6): wsm_eq/wsm_atom now emit canonical Symbol(\"t\") (SYMBOL_ID_MAX sentinel, matching wsm-os-runtime's parallel CANONICAL_T fix) instead of Tag::True. All 4 harnesses and all 3 wsm-os-hosted nucleus witnesses re-verified passing after the fix, including two harness assertions that were initially missed and briefly broken (caught by re-running before commit, not assumed)."))
  ))
  ("WSM-MY-LISP-STAGE1-DEF-BOUNDED-RECURSION" . (
    (priority . 9.0)
    (capabilities . (wsm-my-lisp cml x86_64 named-functions))
    (origin . wsm-my-lisp)
    (depends-on . (WSM-MY-LISP-STAGE0-NUCLEUS-WITNESS))
    (context . "done 2026-09-05: Proved end-to-end bounded self-tail-recursion. Source (def countdown (lambda (n) (cond ((eq n 0) (quote done)) (t (countdown (- n 1)))))) (countdown N) verified against my-lisp oracle: done. Compiled via CML commit ae88fd23447c0d90d9d4c51e854fbb03218a95ae into asm/entry-countdown-100k.s. Structural witness: contains .Ltcloop_0 and jmp .Ltcloop_0 with zero recursive calls. Execution witness: linked against asm/nucleus.s (zero Rust runtime primitives linked), executed via harness-countdown, ran N=100000 with constant native stack frame ($112, %rsp), printed done, matched oracle.")
    (description . "DONE 2026-09-05: Prove that a my-lisp recursive definition travels through CML x86_64 lowering and executes against wsm-my-lisp's hand-written asm nucleus with constant native stack. Structural witness (.Ltcloop jump) + execution witness (N=100000 -> done) confirmed against oracle.")
    (done . (2026-09-05 "Oracle + CML + asm/nucleus.s native execution for N=100000 completed with constant stack frame"))
  ))
  ("WSM-MY-LISP-STAGE2-META-EVAL-CALL-GRAPH" . (
    (priority . 8.6)
    (capabilities . (wsm-my-lisp cml x86_64 self-hosting evaluator))
    (origin . wsm-my-lisp)
    (depends-on . (WSM-MY-LISP-STAGE1-DEF-BOUNDED-RECURSION))
    (description . "Close lib/meta-eval.my's real call graph one gate at a time, each with an executed, oracle-checked witness on asm/nucleus.s -- not a single leap: my-eval-cond, then my-apply, then my-eval-body/my-eval-list, then bind-params, then the full my-eval-top-form/my-eval-program. Each gate may require its own small, named CML backend extension (Ir::Let, general Ir::App/Ir::Lambda, etc.) -- scope each such extension narrowly to what the specific gate needs, matching Stage 1's discipline, not a general rewrite of the backend's admitted IR.")
    (done . ())
  ))
  ("WSM-MY-LISP-STAGE3-SELF-HOSTING-CLOSURE" . (
    (priority . 8.5)
    (capabilities . (wsm-my-lisp self-hosting evaluator milestone))
    (origin . wsm-my-lisp)
    (depends-on . (WSM-MY-LISP-STAGE2-META-EVAL-CALL-GRAPH))
    (description . "The graduation moment: lib/meta-eval.my's my-eval runs entirely on asm/nucleus.s with no Rust evaluator present at runtime -- \"WSM executes WSM on the owner's own hardware\" stops being a metaphor. Rust remains the offline compiler/tooling/oracle, not a runtime dependency, for this capability. Requires a real executed witness (a nontrivial WSM program interpreted by my-eval, itself running as compiled asm), not merely that every sub-gate from Stage 2 individually passed.")
    (done . ())
  ))
  ("WSM-MY-LISP-STAGE4-PRIMITIVE-REPRESENTATION-DECISION" . (
    (priority . 7.5)
    (capabilities . (wsm-my-lisp performance asm rust architecture))
    (origin . wsm-my-lisp)
    (depends-on . (WSM-MY-LISP-STAGE3-SELF-HOSTING-CLOSURE))
    (context . "Owner's performance-substrate principle (ecosystem/plans/WSM-FULL-MACHINE-ROADMAP-2026-08-31.md): WSM is the semantic language, target-machine assembly is the performance substrate, Rust is neither -- never reach for Rust as a performance escape hatch. asm/nucleus.s already proves hand-written asm can implement the 5 core primitives; wsm-os-runtime's Rust implementation of the same 5 primitives remains untouched and available as the default/reference path.")
    (description . "Decide, per capability and only once genuinely measured, whether wsm_cons/car/cdr/eq/atom's asm implementation or wsm-os-runtime's Rust implementation is the primary path going forward. Order strictly: start in WSM/Rust, measure if there is an actual bottleneck, check whether CML lowering can absorb it, only then justify hand-written asm with a named, measured reason (hot path, cycle counts, fraction of runtime) -- not by default and not for architectural purity alone.")
    (done . ())
  ))
  ("WSM-MY-LISP-STAGE5-FPGA-TAG-TRUE" . (
    (priority . 8.7)
    (capabilities . (fpga-lisp rtl isa semantics hardware-verified))
    (origin . wsm-my-lisp)
    (description . "Tracked primarily in fpga-lisp's own tasks.my as FPGA-TAG-TRUE-MANUFACTURED-PRIMITIVE -- this entry exists so the roadmap's Stage 5 is visible from wsm-my-lisp too. Blast-radius audit complete (RTL untouched): isa-contract.my's standalone true=4 tag and fpga/rtl/control.sv's OP_ATOM/OP_EQ hardware primitives manufacture a primitive category canonical WSM doesn't have (t is plain Symbol(\"t\")). Minimal fix already grounded: SYM_T=79 already exists as a registered bootstrap symbol and TAG_SYMBOL/LOADSYM is already hardware-verified machinery for arbitrary symbols -- the fix is narrowly tag=TAG_SYMBOL,value=79 instead of tag=TAG_TRUE,value=1. The RTL change itself needs its own explicit authorization: already flashed and board-verified logic, not simulation-only.")
    (done . ())
  ))
  ("WSM-MY-LISP-STAGE6-REASONING-LAYER-WIRING" . (
    (priority . 7.0)
    (capabilities . (wsm-my-lisp reasoning epistemic knowledge cond))
    (origin . wsm-my-lisp)
    (depends-on . (WSM-MY-LISP-STAGE3-SELF-HOSTING-CLOSURE))
    (context . "lib/epistemic.my (my-lisp) already exists, already tested (epistemic.rs, 38/38), and already uses a real finite epistemic-state vocabulary instead of booleans -- claim review: proposed/reviewed/rejected; evidence outcome: supports/contradicts/inconclusive -- no Bool anywhere. But its own header says plainly it is NOT loaded by lib/core.my and NOT wired into reason.my/knowledge.my/world.my: the layer exists, isolated, and nothing that actually decides cond/is_truthy consults it. Do not wire it in before Stage 3 -- this reasoning layer belongs on the live self-hosted nucleus, not bolted onto Rust as a stopgap.")
    (description . "Connect lib/epistemic.my's epistemic-state vocabulary to real decision-making (cond/is_truthy-driven control flow) without collapsing () = () neutrality -- () must stay the neutral \"no basis\" state; the epistemic layer is the richer surface reached only once actual evidence exists, per the paradigm in docs/ROADMAP.md Part I. This is the capability gap that makes WSM a thinking machine rather than merely a self-hosted interpreter: today evidence-supports?/intent-capabilities-satisfied? collapse their positive branch to a bare t instead of returning the actual supporting structure (e.g. (supports evidence-17)) -- close that gap as part of this wiring, not just the negative-branch () correctness that already holds.")
    (done . ())
  ))
  ("WSM-MY-LISP-STAGE7-ASSEMBLER-IN-LISP-DSL" . (
    (priority . 5.5)
    (capabilities . (wsm-my-lisp fpga-lisp assembler dsl bootstrap))
    (origin . wsm-my-lisp)
    (depends-on . (WSM-MY-LISP-STAGE3-SELF-HOSTING-CLOSURE))
    (context . "fpga-lisp/assembler.my already proves a self-hosted Lisp-authored assembler is real, not aspirational -- 423 lines, byte-identical output to the reference assembler.py. What does not yet exist: an ergonomic macro DSL layer above bare instructions ((section boot), (label reset), (definterrupt ...), (with-stack-frame ...)) -- today's assembler operates at the instruction/label level only. Owner's explicit sequencing: this comes after the nucleus is alive, not before -- \"спочатку народжується маленький Lisp, потім він сам будує собі машину й ОС\", not the reverse.")
    (description . "Long-horizon, not started: build the Lisp-macro-based assembler DSL layer for boot/interrupt/stack-frame constructs, on top of the already-proven assembler.my instruction/label machinery, once the self-hosting nucleus (Stage 3) is real. Do not start this before Stage 3 lands.")
    (done . ())
  ))
)))
