; check-lisp-first-authority.lisp — Lisp-first authority guard for wsm-my-lisp.
;
; Операційний факт (власник, P0 #15): core self-hosting шлях = Lisp + x86-64
; asm. C і Rust = 0 у production/self-hosting execution path. Rust host embed,
; який колись жив під dll/ (frozen Cyberpunk mirror), було видалено в Phase D
; міграції до my-lisp-cyberpunk/host-runtime; цей repo не несе жодної Rust
; reader/eval/apply реалізації. Цей скрипт fail-closed на дрейф назад.
;
; Це пряма Lisp-версія scripts/check-lisp-first-authority.sh: той самий набір
; перевірок і та сама fail-closed семантика. Рішення про pass/fail приймає
; Lisp; bash лишається механічною підкладкою (test/find/grep), оскільки мова
; ще не має read-dir/glob/string-split. Документований process-run interim —
; див. docs/parity/lisp-first-guard-parity.md.
;
; CI invocation (паритет із bash-версією):
;   my-lisp scripts/check-lisp-first-authority.lisp || { cat .guard-report.txt 2>/dev/null; exit 1; }
;
; Механізм fail: мова не має begin/set!/raise/if; усі розгалуження — cond.
; armed-violation спершу пише деталі порушення у .guard-report.txt (переживає
; abort), потім примусово падає через невідомий символ `violation` →
; CLI exit 1 + stderr-рядок із помилкою (пояснення в parity-документі).
; На success-path CLI друкує усі princ-рядки (result.output).

(def process-ok?
  (lambda (cmd)
    (eq (car (process-run "bash" (list "-c" cmd))) 0)))

(def process-out
  (lambda (cmd)
    (second (process-run "bash" (list "-c" cmd)))))

; armed violation: зберегти деталі порушення у файл, тоді примусово впасти
; (exit 1). `.guard-report.txt` — репо-відносний, щоб CI міг його cat-нути.
(def armed-violation
  (lambda (report)
    (write-file ".guard-report.txt" report)
    (violation "AUTHORITY-FAIL: authority violation — offenders list in .guard-report.txt")))

; --- 1. Required self-hosting artifacts ---
(cond
  ((not (process-ok? "test -f asm/nucleus.s")) (violation "AUTHORITY-FAIL: missing asm/nucleus.s (x86-64 asm core)"))
  (t ()))
(cond
  ((not (process-ok? "test -f asm/nucleus-win64.s")) (violation "AUTHORITY-FAIL: missing asm/nucleus-win64.s"))
  (t ()))
(cond
  ((not (process-ok? "test -f docs/AUTHORITY.md")) (violation "AUTHORITY-FAIL: missing docs/AUTHORITY.md"))
  (t ()))
(cond
  ((not (process-ok? "test -f docs/archive/completed-plans/dll-inventory-2026-09-11.md")) (violation "AUTHORITY-FAIL: missing archived dll inventory"))
  (t ()))
(princ "AUTHORITY OK: required self-hosting artifacts present\n")

; --- 2. No Rust semantic modules anywhere outside harness/ and external/ ---
; harness/ = witness drivers (links asm), не евалуатор.
(let ((rs (process-out "find . -type f \\( -name 'eval.rs' -o -name 'reader.rs' -o -name 'printer.rs' -o -name 'ffi.rs' -o -name 'word.rs' \\) -not -path './external/*' -not -path './harness/*' 2>/dev/null")))
  (cond
    ((not (string-empty? rs)) (armed-violation rs))
    (t (princ "AUTHORITY OK: no eval/reader/printer/ffi/word.rs anywhere in this repo\n"))))

; --- 3. No C runtime modules in production path ---
(let ((cc (process-out "find . -type f \\( -name '*.c' -o -name '*.cc' -o -name '*.cpp' -o -name '*.h' -o -name '*.hpp' \\) -not -path './external/*' -not -path './harness/*' -not -path './asm/*' 2>/dev/null")))
  (cond
    ((not (string-empty? cc)) (armed-violation cc))
    (t (princ "AUTHORITY OK: no C/C++ production modules outside harness/asm/external\n"))))

; --- 4. dll/ must be gone (Phase D complete) ---
(cond
  ((process-ok? "test -d dll") (violation "AUTHORITY-FAIL: dll/ still present — Phase D (delete Rust host embed) not complete"))
  (t (princ "AUTHORITY OK: dll/ absent — Phase D complete\n")))

; --- 5. Documentation authority: one active entry point, archive stays non-normative ---
(cond
  ((not (process-ok? "test -f docs/CURRENT.md")) (violation "AUTHORITY-FAIL: missing docs/CURRENT.md (documentation entry point, wsm-my-lisp#19)"))
  (t ()))
(cond
  ((not (process-ok? "test -f docs/archive/README.md")) (violation "AUTHORITY-FAIL: missing docs/archive/README.md (non-normative warning)"))
  (t ()))
(let ((nm (process-out "for f in docs/archive/*/*.md; do [ -f \"$f\" ] || continue; grep -qi 'ARCHIVED' \"$f\" || echo \"$f\"; done 2>/dev/null")))
  (cond
    ((not (string-empty? nm)) (armed-violation nm))
    (t (princ "AUTHORITY OK: docs/CURRENT.md present, archive docs self-identify as archived\n"))))

; --- 6. Self-hosting CI must not reference dll as core authority ---
(cond
  ((process-ok? "test -f .github/workflows/self-hosting-authority.yml")
   (cond
     ((process-ok? "grep -E '^\\s*- \"dll/' .github/workflows/self-hosting-authority.yml")
      (violation "AUTHORITY-FAIL: self-hosting-authority.yml must not path-trigger on dll/ (deleted)"))
     (t (princ "AUTHORITY OK: self-hosting workflow carries no dll/ path trigger\n"))))
  (t ()))

(princ "Lisp-first authority guard passed (Lisp + x86-64 asm production path).\n")
()