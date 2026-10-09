; check-machine-contracts.lisp — Lisp-first machine-contract gate for wsm-my-lisp.
;
; MACHINE-CONTRACT-1 (wsm-my-lisp#23): WSM does not invent machine facts locally.
; It consumes Lisp-owned machine contracts from the pinned external/sens
; submodule and fails closed on missing/unknown schema, authority drift, or
; locally manufactured machine truth.
;
; CI:
;   external/sens/target/release/my-lisp scripts/check-machine-contracts.lisp \
;     || { cat .guard-report.txt 2>/dev/null; exit 1; }
;
; Міграційне правило: кожна гілка COND має канонічну форму
;   (typed-D1-query expression)
; Порівняння з D1:0/D1:1 робиться явно через (рівне? query #b0/#b1).
; Трискладові клаузи й T/NIL/host-truthiness заборонені.

; External subprocess result is (exit-code stdout stderr). Exact D3 codes:
; 101 EQ, 100 CAR, 011 CDR, 111 CONS, 001 QUOTE.
; No D4 LIST binding and no shell-generated synthetic D1 text.
(визначити процес-успішний?
  (функція (команда)
    (101
      (100
        (запустити-процес "bash"
          (111 "-c" (111 команда (001 ())))))
      #d0)))

(визначити вивід-процесу
  (функція (команда)
    (100
      (011
        (запустити-процес "bash"
          (111 "-c" (111 команда (001 ()))))))))

(визначити озброєне-порушення
  (функція (звіт)
    (записати-файл ".guard-report.txt" звіт)
    (violation "MACHINE-CONTRACT-FAIL: authority violation -- offenders in .guard-report.txt")))

; --- 0. Single machine-readable import path must exist: pinned submodule ---
(визначити пін-підмодуля
  (вивід-процесу "git -C external/sens rev-parse HEAD 2>/dev/null || true"))

(за-умовою
  ((рівне? (процес-успішний? "git -C external/sens rev-parse --verify HEAD >/dev/null 2>&1") #b1) 
   (показати (сполучити "MACHINE-CONTRACT: pinned SENS = " пін-підмодуля)))
  ((рівне? (процес-успішний? "git -C external/sens rev-parse --verify HEAD >/dev/null 2>&1") #b0) 
   (озброєне-порушення "external/sens submodule not checked out -- cannot consume machine contracts")))

; --- 1. Required Lisp-owned machine contracts must exist at the pin ---
(за-умовою
  ((рівне? (процес-успішний? "test -f external/sens/machine-lowering-boundary.lisp") #b1) 
   (показати "MACHINE-CONTRACT OK: machine-lowering-boundary.lisp present\n"))
  ((рівне? (процес-успішний? "test -f external/sens/machine-lowering-boundary.lisp") #b0) 
   (озброєне-порушення "missing external/sens/machine-lowering-boundary.lisp (Lisp-owned lowering authority)")))

(за-умовою
  ((рівне? (процес-успішний? "test -f external/sens/memory-layout-contract.lisp") #b1) 
   (показати "MACHINE-CONTRACT OK: memory-layout-contract.lisp present\n"))
  ((рівне? (процес-успішний? "test -f external/sens/memory-layout-contract.lisp") #b0) 
   (озброєне-порушення "missing external/sens/memory-layout-contract.lisp (Lisp-owned memory layout authority)")))

(за-умовою
  ((рівне? (процес-успішний? "test -f external/sens/lib/machine/lowering/semantic-x86-64.lisp") #b1) 
   (показати "MACHINE-CONTRACT OK: lib/machine/lowering/semantic-x86-64.lisp present\n"))
  ((рівне? (процес-успішний? "test -f external/sens/lib/machine/lowering/semantic-x86-64.lisp") #b0) 
   (озброєне-порушення "missing external/sens/lib/machine/lowering/semantic-x86-64.lisp (Lisp-owned semantic lowering authority)")))

(за-умовою
  ((рівне? (процес-успішний? "test -f external/sens/lib/machine/encoding/x86-64.lisp") #b1) 
   (показати "MACHINE-CONTRACT OK: lib/machine/encoding/x86-64.lisp present\n"))
  ((рівне? (процес-успішний? "test -f external/sens/lib/machine/encoding/x86-64.lisp") #b0) 
   (озброєне-порушення "missing external/sens/lib/machine/encoding/x86-64.lisp (Lisp-owned x86-64 encoder authority)")))

; --- 2. Contract schema/version must be known ---
(за-умовою
  ((рівне? (процес-успішний? "grep -q '(schema machine-lowering-boundary/2)' external/sens/machine-lowering-boundary.lisp") #b1) 
   (показати "MACHINE-CONTRACT OK: machine-lowering-boundary schema /2\n"))
  ((рівне? (процес-успішний? "grep -q '(schema machine-lowering-boundary/2)' external/sens/machine-lowering-boundary.lisp") #b0) 
   (озброєне-порушення "machine-lowering-boundary has unknown/absent schema -- fail-closed")))

(за-умовою
  ((рівне? (процес-успішний? "grep -q '(version . (1 0))' external/sens/memory-layout-contract.lisp") #b1) 
   (показати "MACHINE-CONTRACT OK: memory-layout-contract version (1 0)\n"))
  ((рівне? (процес-успішний? "grep -q '(version . (1 0))' external/sens/memory-layout-contract.lisp") #b0) 
   (озброєне-порушення "memory-layout-contract has unknown/absent version -- fail-closed")))

; --- 3. Authority direction must be Lisp-owned, one-way, semantic-to-machine ---
(за-умовою
  ((рівне? (процес-успішний? "grep -q '(semantic-authority my-lisp)' external/sens/machine-lowering-boundary.lisp") #b1) 
   (показати "MACHINE-CONTRACT OK: semantic authority = my-lisp\n"))
  ((рівне? (процес-успішний? "grep -q '(semantic-authority my-lisp)' external/sens/machine-lowering-boundary.lisp") #b0) 
   (озброєне-порушення "boundary does not declare semantic-authority my-lisp")))

(за-умовою
  ((рівне? (процес-успішний? "grep -q '(lowering-direction semantic-to-machine)' external/sens/machine-lowering-boundary.lisp") #b1) 
   (показати "MACHINE-CONTRACT OK: lowering direction semantic->machine\n"))
  ((рівне? (процес-успішний? "grep -q '(lowering-direction semantic-to-machine)' external/sens/machine-lowering-boundary.lisp") #b0) 
   (озброєне-порушення "boundary lacks one-way semantic-to-machine lowering direction")))

(за-умовою
  ((рівне? (процес-успішний? "grep -q '(semantic-id-from-isa forbidden)' external/sens/machine-lowering-boundary.lisp") #b1) 
   (показати "MACHINE-CONTRACT OK: semantic IDs must never originate from ISA/opcode/asm\n"))
  ((рівне? (процес-успішний? "grep -q '(semantic-id-from-isa forbidden)' external/sens/machine-lowering-boundary.lisp") #b0) 
   (озброєне-порушення "boundary lacks semantic-id-from-isa forbidden -- semantic IDs could leak into ISA labels")))

; --- 4. Semantic IDs must be consumed, not re-invented at WSM ---
(за-умовою
  ((рівне? (процес-успішний? "for sid in 00000010 00000100 00000101 00000110; do grep -q \"$sid\" external/sens/lib/machine/lowering/semantic-x86-64.lisp || exit 1; done") #b1) 
   (показати "MACHINE-CONTRACT OK: admitted-slice canonical 8-bit SIDs findable in Lisp-owned lowering profile\n"))
  ((рівне? (процес-успішний? "for sid in 00000010 00000100 00000101 00000110; do grep -q \"$sid\" external/sens/lib/machine/lowering/semantic-x86-64.lisp || exit 1; done") #b0) 
   (озброєне-порушення "semantic-x86-64.lisp missing admitted-slice canonical 8-bit SIDs (00000010/00000100/00000101/00000110) -- cannot consume asserted meanings")))

; --- 5. WSM local mirrors must not claim independent machine text authority ---
(за-умовою
  ((рівне? (процес-успішний? "grep -qE '^\\.equ +TAG_TRUE' asm/nucleus.s") #b1) 
   (озброєне-порушення "asm/nucleus.s declares local .equ TAG_TRUE -- historical manufactured truth must stay out of the runtime"))
  ((рівне? (процес-успішний? "grep -qE '^\\.equ +TAG_TRUE' asm/nucleus.s") #b0) 
   (показати "MACHINE-CONTRACT OK: no local .equ TAG_TRUE manufactured in nucleus.s\n")))

(за-умовою
  ((рівне? (процес-успішний? "grep -nE 'hand-written|Not generated' asm/nucleus.s | head -1") #b1) 
   (показати "MACHINE-CONTRACT OK: nucleus.s self-identifies as hand-written mechanism projection\n"))
  ((рівне? (процес-успішний? "grep -nE 'hand-written|Not generated' asm/nucleus.s | head -1") #b0) 
   (озброєне-порушення "nucleus.s does not declare its mechanism/projection status -- must be explicit")))

(показати "Machine-contract gate passed (Lisp-owned machine contracts consumed, fail-closed drift gate armed).\n")
()