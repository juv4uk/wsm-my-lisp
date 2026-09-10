; Cyberpunk embed task recommendations for wsm-my-lisp (2026-09-10).
; Parallel to tasks.my stages 0–7. Claim only from this repo.
; Cross-repo map: cml evidence/cyberpunk/CROSS-REPO-TASK-RECOMMENDATIONS-2026-09-10.my

((kind . cyberpunk-recommendations)
 (version . 1)
 (tasks .
  (("CP-IN-GAME-LOAD-WITNESS" .
    ((priority . 9.8)
     (done . (2026-09-10 "Owner ran Cyberpunk 2077 v2.31 + RED4ext v1.30.0 with the built plugin. Full transcript in my-lisp-cyberpunk#1: plugin + wsm DLL loaded, wsm_session_init succeeded, запиши-лог registered and evaluated, () logged. Read-only, no save/inventory/player-state touched."))
     (capabilities . (red4ext windows dll cyberpunk proof))
     (description . "Load plugin + wsm DLL inside a real Cyberpunk 2077 process; confirm wsm_session_init via RED4ext Logger. MSVC build alone is not this task.")
     (acceptance . "Log or capture from live game; unload path calls wsm_session_free; failures named.")))
   ("CP-HOST-PRIM-ONE" .
    ((priority . 9.5)
     (done . (2026-09-10 "запиши-лог: one real host primitive, end-to-end Lisp -> FFI -> RED4ext log, proven in-game (see CP-IN-GAME-LOAD-WITNESS). NOT done: the requested source|oracle|dll|in-game|status|reason matrix table specifically -- only prose evidence across docs/vertical-slice.md (my-lisp-cyberpunk) and this repo's commits exists so far."))
     (depends-on . ("CP-IN-GAME-LOAD-WITNESS"))
     (description . "One real or explicitly stubbed game-facing host primitive end-to-end (Lisp -> FFI -> RED4ext or explicit Unsupported).")
     (acceptance . "Matrix row: source | my-lisp oracle | dll | in-game | status | reason.")))
   ("CP-FIXTURE-MATRIX" .
    ((priority . 9.2)
     (done . (2026-09-10 "dll/tests/my_lisp_fixture_parity.rs added (wsm-my-lisp 8c2e809): one test per row in my-lisp's docs/cyberpunk-host-dispatch-fixtures.md, all 4 sections, runs in CI. HONEST LIMIT (documented in the file itself): hard-coded harness, not a live oracle -- does not re-run my-lisp.exe in CI, so a fixture-doc change still needs a human/agent to notice and update this file to match, it does not auto-detect drift."))
     (description . "Automate my-lisp cyberpunk-host-dispatch-fixtures against dll; UnknownSymbol text and nested args must not drift.")
     (acceptance . "CI or documented harness fails closed on mismatch.")))
   ("CP-TAG-BOXED-CONSUME" .
    ((priority . 8.8)
     (done . (2026-09-10 "wsm-target-contract#1 ratified Tag::Boxed=7 (commit bb6e119, contract v3). wsm-my-lisp 8be0ed6 migrated dll/word.rs to a pinned git dependency on wsm_os_target, consuming Tag::Boxed/encode_boxed/decode_boxed instead of a local constant. No dual TAG_STRING/TAG_BOXED story remains."))
     (description . "After wsm-target-contract ratifies TAG_BOXED, drop TENTATIVE and pin one tag value.")
     (acceptance . "No dual TAG_STRING/TAG_BOXED story in docs.")))
   ("CP-ARENA-THREAD-POLICY" .
    ((priority . 8.0)
     (done . (2026-09-10 "dll/README.md added (wsm-my-lisp 00dcd8c): documents the single-threaded-per-session embed contract, the actual reproduced test-suite race that motivated it, and Session ownership/panic rules. docs/game-injection-plan.md cross-references it. RUST_TEST_THREADS=1 enforcement already existed (2c9dec6)."))
     (description . "Document single-threaded arena as embed contract or implement per-session arena; do not claim thread-safe.")
     (acceptance . "README/game-injection-plan states the rule; tests enforce or document RUST_TEST_THREADS=1.")))
   ("CP-NO-EVAL-PUSH-TO-CML" .
    ((priority . 7.5)
     (done . (2026-09-10 "Standing constraint held throughout: wsm_eval_string/reader/evaluator only ever implemented in dll/ (Rust). cml explicitly declined a runtime-reader role when asked (cml#3's own \"Не робити\": \"Не реалізовувати runtime reader / wsm_eval_string в CML\")."))
     (description . "Standing: wsm_eval_string stays in this repo; cml remains offline compiler only."))))))
