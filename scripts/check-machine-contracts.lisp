; check-machine-contracts.lisp — Lisp-first machine-contract gate for wsm-my-lisp.
;
; MACHINE-CONTRACT-1 (wsm-my-lisp#23): WSM does not invent machine facts locally.
; It consumes Lisp-owned machine contracts from the pinned external/my-lisp
; submodule and fails closed on:
;   - missing/unknown contract version or schema;
;   - drift between a consumed contract fact and the pinned upstream authority;
;   - any attempt to make GNU as / Rust enum / C header / hand-written .s a
;     second semantic/memory authority.
;
; This is a Lisp-first gate in the same spirit as check-lisp-first-authority.lisp:
; the pass/fail decision is made in Lisp; bash (process-run) is only the
; mechanical substrate for file/text checks, because the language does not yet
; expose read-dir/glob/string-split. CI invocation:
;
;   external/my-lisp/target/release/my-lisp scripts/check-machine-contracts.lisp \
;     || { cat .guard-report.txt 2>/dev/null; exit 1; }
;
; Fail mechanism: there is no begin/set!/raise/if; all branches are cond.
; armed-violation first writes details to .guard-report.txt (survives abort),
; then forces a crash through the unknown symbol `violation` -> CLI exit 1 +
; stderr source-line with the message (see docs/parity/lisp-first-guard-parity.md).

(def process-ok?
  (lambda (cmd)
    (eq (car (process-run "bash" (list "-c" cmd))) 0)))

(def process-out
  (lambda (cmd)
    (second (process-run "bash" (list "-c" cmd)))))

; armed violation: save details to a file (survives abort), then force exit 1
; via the undefined symbol `violation`. `.guard-report.txt` is repo-relative so
; CI can cat it.
(def armed-violation
  (lambda (report)
    (write-file ".guard-report.txt" report)
    (violation "MACHINE-CONTRACT-FAIL: authority violation -- offenders in .guard-report.txt")))

; --- 0. Single machine-readable import path must exist: pinned submodule ---
(let ((pin (process-out "git -C external/my-lisp rev-parse HEAD 2>/dev/null || true")))
  (cond
    ((string-empty? pin)
     (armed-violation "external/my-lisp submodule not checked out -- cannot consume machine contracts"))
    (t
      (princ "MACHINE-CONTRACT: pinned my-lisp = "))
    ))

; --- 1. Required Lisp-owned machine contracts must exist at the pin ---
(cond
  ((process-ok? "test -f external/my-lisp/machine-lowering-boundary.lisp")
   (princ "MACHINE-CONTRACT OK: machine-lowering-boundary.lisp present\n"))
  (t (armed-violation "missing external/my-lisp/machine-lowering-boundary.lisp (Lisp-owned lowering authority)")))

(cond
  ((process-ok? "test -f external/my-lisp/memory-layout-contract.lisp")
   (princ "MACHINE-CONTRACT OK: memory-layout-contract.lisp present\n"))
  (t (armed-violation "missing external/my-lisp/memory-layout-contract.lisp (Lisp-owned memory layout authority)")))

(cond
  ((process-ok? "test -f external/my-lisp/lib/machine/lowering/semantic-x86-64.lisp")
   (princ "MACHINE-CONTRACT OK: lib/machine/lowering/semantic-x86-64.lisp present\n"))
  (t (armed-violation "missing external/my-lisp/lib/machine/lowering/semantic-x86-64.lisp (Lisp-owned semantic lowering authority)")))

(cond
  ((process-ok? "test -f external/my-lisp/lib/machine/encoding/x86-64.lisp")
   (princ "MACHINE-CONTRACT OK: lib/machine/encoding/x86-64.lisp present\n"))
  (t (armed-violation "missing external/my-lisp/lib/machine/encoding/x86-64.lisp (Lisp-owned x86-64 encoder authority)")))

; --- 2. Contract schema/version must be known ---
(cond
  ((process-ok? "grep -q '(schema machine-lowering-boundary/2)' external/my-lisp/machine-lowering-boundary.lisp")
   (princ "MACHINE-CONTRACT OK: machine-lowering-boundary schema /2\n"))
  (t (armed-violation "machine-lowering-boundary has unknown/absent schema -- fail-closed")))

(cond
  ((process-ok? "grep -q '(version . (1 0))' external/my-lisp/memory-layout-contract.lisp")
   (princ "MACHINE-CONTRACT OK: memory-layout-contract version (1 0)\n"))
  (t (armed-violation "memory-layout-contract has unknown/absent version -- fail-closed")))

; --- 3. Authority direction must be Lisp-owned, one-way, semantic-to-machine ---
(cond
  ((process-ok? "grep -q '(semantic-authority my-lisp)' external/my-lisp/machine-lowering-boundary.lisp")
   (princ "MACHINE-CONTRACT OK: semantic authority = my-lisp\n"))
  (t (armed-violation "boundary does not declare semantic-authority my-lisp")))

(cond
  ((process-ok? "grep -q '(lowering-direction semantic-to-machine)' external/my-lisp/machine-lowering-boundary.lisp")
   (princ "MACHINE-CONTRACT OK: lowering direction semantic->machine\n"))
  (t (armed-violation "boundary lacks one-way semantic-to-machine lowering direction")))

(cond
  ((process-ok? "grep -q '(semantic-id-from-isa forbidden)' external/my-lisp/machine-lowering-boundary.lisp")
   (princ "MACHINE-CONTRACT OK: semantic IDs must never originate from ISA/opcode/asm\n"))
  (t (armed-violation "boundary lacks semantic-id-from-isa forbidden -- semantic IDs could leak into ISA labels")))

; --- 4. Semantic IDs must be consumed, not re-invented at WSM ---
; The witness slice is CONS/CAR/CDR (memory IDs 0002/0004/0005/0006 in the
; semantic lowering profile). WSM must not mint its own IDs from asm labels;
; it may only project these Lisp-owned IDs.
(let ((ids (process-out "grep -oE '[ ]*\\(\\(000[2-6] ' external/my-lisp/lib/machine/lowering/semantic-x86-64.lisp 2>/dev/null | head -1")))
  (cond
    ((string-empty? ids)
     (armed-violation "semantic-x86-64.lisp missing admitted-slice semantic IDs (0002/0004/0005/0006) -- cannot consume asserted meanings"))
    (t (princ "MACHINE-CONTRACT OK: admitted-slice semantic IDs findable in Lisp-owned lowering profile\n"))))

; --- 5. WSM local mirrors must not claim independent machine text authority ---
; The nucleus is a bootstrap/mechanism projection of the pinned target contract.
; It must never declare its own tag/offset/layout truths that already live in
; the Lisp-owned machine contracts ("not a second semantic/memory authority").
(cond
  ((process-ok? "grep -qE '^\\.equ +TAG_TRUE' asm/nucleus.s")
   (armed-violation "asm/nucleus.s declares local .equ TAG_TRUE -- historical manufactured truth must stay out of the runtime"))
  (t (princ "MACHINE-CONTRACT OK: no local .equ TAG_TRUE manufactured in nucleus.s\n")))

(cond
  ((process-ok? "grep -nE 'hand-written|Not generated' asm/nucleus.s | head -1")
   (princ "MACHINE-CONTRACT OK: nucleus.s self-identifies as hand-written mechanism projection\n"))
  (t (armed-violation "nucleus.s does not declare its mechanism/projection status -- must be explicit")))

(princ "Machine-contract gate passed (Lisp-owned machine contracts consumed, fail-closed drift gate armed).\n")
()