; check-machine-contracts.lisp — Lisp-first machine-contract gate for wsm-my-lisp.
;
; MACHINE-CONTRACT-1 (wsm-my-lisp#23): WSM does not invent machine facts locally.
; It consumes Lisp-owned machine contracts from the pinned external/sens
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
;   external/sens/target/release/my-lisp scripts/check-machine-contracts.lisp \
;     || { cat .guard-report.txt 2>/dev/null; exit 1; }
;
; Fail mechanism: there is no begin/set!/raise/if; all branches are cond.
; armed-violation first writes details to .guard-report.txt (survives abort),
; then forces a crash through the unknown symbol `violation` -> CLI exit 1 +
; stderr source-line with the message (see docs/parity/lisp-first-guard-parity.md).
;
; Міграційне правило: кожен test у COND має exact D1-значення 0 або 1.
; Історичний T/NIL і truthy/falsy coercion тут не використовуються.

(визначити процес-успішний?
  (функція (команда)
    (тотожне? (перше (запустити-процес "bash" (список "-c" команда))) 0)))

(визначити вивід-процесу
  (функція (команда)
    (друге (запустити-процес "bash" (список "-c" команда)))))

; armed violation: save details to a file (survives abort), then force exit 1
; via the undefined symbol `violation`. `.guard-report.txt` is repo-relative so
; CI can cat it.
(визначити озброєне-порушення
  (функція (звіт)
    (записати-файл ".guard-report.txt" звіт)
    (violation "MACHINE-CONTRACT-FAIL: authority violation -- offenders in .guard-report.txt")))

; --- 0. Single machine-readable import path must exist: pinned submodule ---
(визначити пін-підмодуля
  (вивід-процесу "git -C external/sens rev-parse HEAD 2>/dev/null || true"))

(за-умовою
  ((текст-порожній? пін-підмодуля)
   (озброєне-порушення "external/sens submodule not checked out -- cannot consume machine contracts"))
  ((тотожне? (текст-порожній? пін-підмодуля) 0)
   (показати "MACHINE-CONTRACT: pinned my-lisp = ")
   (показати пін-підмодуля)))

; --- 1. Required Lisp-owned machine contracts must exist at the pin ---
(за-умовою
  ((процес-успішний? "test -f external/sens/machine-lowering-boundary.lisp")
   (показати "MACHINE-CONTRACT OK: machine-lowering-boundary.lisp present\n"))
  ((тотожне? (процес-успішний? "test -f external/sens/machine-lowering-boundary.lisp") 0)
   (озброєне-порушення "missing external/sens/machine-lowering-boundary.lisp (Lisp-owned lowering authority)")))

(за-умовою
  ((процес-успішний? "test -f external/sens/memory-layout-contract.lisp")
   (показати "MACHINE-CONTRACT OK: memory-layout-contract.lisp present\n"))
  ((тотожне? (процес-успішний? "test -f external/sens/memory-layout-contract.lisp") 0)
   (озброєне-порушення "missing external/sens/memory-layout-contract.lisp (Lisp-owned memory layout authority)")))

(за-умовою
  ((процес-успішний? "test -f external/sens/lib/machine/lowering/semantic-x86-64.lisp")
   (показати "MACHINE-CONTRACT OK: lib/machine/lowering/semantic-x86-64.lisp present\n"))
  ((тотожне? (процес-успішний? "test -f external/sens/lib/machine/lowering/semantic-x86-64.lisp") 0)
   (озброєне-порушення "missing external/sens/lib/machine/lowering/semantic-x86-64.lisp (Lisp-owned semantic lowering authority)")))

(за-умовою
  ((процес-успішний? "test -f external/sens/lib/machine/encoding/x86-64.lisp")
   (показати "MACHINE-CONTRACT OK: lib/machine/encoding/x86-64.lisp present\n"))
  ((тотожне? (процес-успішний? "test -f external/sens/lib/machine/encoding/x86-64.lisp") 0)
   (озброєне-порушення "missing external/sens/lib/machine/encoding/x86-64.lisp (Lisp-owned x86-64 encoder authority)")))

; --- 2. Contract schema/version must be known ---
(за-умовою
  ((процес-успішний? "grep -q '(schema machine-lowering-boundary/2)' external/sens/machine-lowering-boundary.lisp")
   (показати "MACHINE-CONTRACT OK: machine-lowering-boundary schema /2\n"))
  ((тотожне? (процес-успішний? "grep -q '(schema machine-lowering-boundary/2)' external/sens/machine-lowering-boundary.lisp") 0)
   (озброєне-порушення "machine-lowering-boundary has unknown/absent schema -- fail-closed")))

(за-умовою
  ((процес-успішний? "grep -q '(version . (1 0))' external/sens/memory-layout-contract.lisp")
   (показати "MACHINE-CONTRACT OK: memory-layout-contract version (1 0)\n"))
  ((тотожне? (процес-успішний? "grep -q '(version . (1 0))' external/sens/memory-layout-contract.lisp") 0)
   (озброєне-порушення "memory-layout-contract has unknown/absent version -- fail-closed")))

; --- 3. Authority direction must be Lisp-owned, one-way, semantic-to-machine ---
(за-умовою
  ((процес-успішний? "grep -q '(semantic-authority my-lisp)' external/sens/machine-lowering-boundary.lisp")
   (показати "MACHINE-CONTRACT OK: semantic authority = my-lisp\n"))
  ((тотожне? (процес-успішний? "grep -q '(semantic-authority my-lisp)' external/sens/machine-lowering-boundary.lisp") 0)
   (озброєне-порушення "boundary does not declare semantic-authority my-lisp")))

(за-умовою
  ((процес-успішний? "grep -q '(lowering-direction semantic-to-machine)' external/sens/machine-lowering-boundary.lisp")
   (показати "MACHINE-CONTRACT OK: lowering direction semantic->machine\n"))
  ((тотожне? (процес-успішний? "grep -q '(lowering-direction semantic-to-machine)' external/sens/machine-lowering-boundary.lisp") 0)
   (озброєне-порушення "boundary lacks one-way semantic-to-machine lowering direction")))

(за-умовою
  ((процес-успішний? "grep -q '(semantic-id-from-isa forbidden)' external/sens/machine-lowering-boundary.lisp")
   (показати "MACHINE-CONTRACT OK: semantic IDs must never originate from ISA/opcode/asm\n"))
  ((тотожне? (процес-успішний? "grep -q '(semantic-id-from-isa forbidden)' external/sens/machine-lowering-boundary.lisp") 0)
   (озброєне-порушення "boundary lacks semantic-id-from-isa forbidden -- semantic IDs could leak into ISA labels")))

; --- 4. Semantic IDs must be consumed, not re-invented at WSM ---
; The witness slice is CONS/CAR/CDR (canonical 8-bit SIDs
; 00000010/00000100/00000101/00000110 in the semantic lowering profile). WSM
; must not mint its own IDs from asm labels; it may only project these Lisp-owned IDs.
(за-умовою
  ((процес-успішний? "for sid in 00000010 00000100 00000101 00000110; do grep -q \"$sid\" external/sens/lib/machine/lowering/semantic-x86-64.lisp || exit 1; done")
   (показати "MACHINE-CONTRACT OK: admitted-slice canonical 8-bit SIDs findable in Lisp-owned lowering profile\n"))
  ((тотожне? (процес-успішний? "for sid in 00000010 00000100 00000101 00000110; do grep -q \"$sid\" external/sens/lib/machine/lowering/semantic-x86-64.lisp || exit 1; done") 0)
   (озброєне-порушення "semantic-x86-64.lisp missing admitted-slice canonical 8-bit SIDs (00000010/00000100/00000101/00000110) -- cannot consume asserted meanings")))

; --- 5. WSM local mirrors must not claim independent machine text authority ---
; The nucleus is a bootstrap/mechanism projection of the pinned target contract.
; It must never declare its own tag/offset/layout truths that already live in
; the Lisp-owned machine contracts ("not a second semantic/memory authority").
(за-умовою
  ((процес-успішний? "grep -qE '^\\.equ +TAG_TRUE' asm/nucleus.s")
   (озброєне-порушення "asm/nucleus.s declares local .equ TAG_TRUE -- historical manufactured truth must stay out of the runtime"))
  ((тотожне? (процес-успішний? "grep -qE '^\\.equ +TAG_TRUE' asm/nucleus.s") 0)
   (показати "MACHINE-CONTRACT OK: no local .equ TAG_TRUE manufactured in nucleus.s\n")))

(за-умовою
  ((процес-успішний? "grep -nE 'hand-written|Not generated' asm/nucleus.s | head -1")
   (показати "MACHINE-CONTRACT OK: nucleus.s self-identifies as hand-written mechanism projection\n"))
  ((тотожне? (процес-успішний? "grep -nE 'hand-written|Not generated' asm/nucleus.s | head -1") 0)
   (озброєне-порушення "nucleus.s does not declare its mechanism/projection status -- must be explicit")))

(показати "Machine-contract gate passed (Lisp-owned machine contracts consumed, fail-closed drift gate armed).\n")
()