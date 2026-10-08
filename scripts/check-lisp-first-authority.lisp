; check-lisp-first-authority.lisp — Lisp-first authority guard for wsm-my-lisp.
;
; Операційний факт: core self-hosting шлях є Lisp-first; C і Rust дозволені як
; bootstrap/runtime/tooling substrates із provenance. Rust host embed,
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
;
; Міграційне правило: кожен test у COND має exact D1-значення 0 або 1.
; Історичний T/NIL і truthy/falsy coercion тут не використовуються.

(визначити процес-успішний?
  (функція (команда)
    (тотожне? (перше (запустити-процес "bash" (список "-c" команда))) 0)))

(визначити вивід-процесу
  (функція (команда)
    (друге (запустити-процес "bash" (список "-c" команда)))))

; armed violation: зберегти деталі порушення у файл, тоді примусово впасти.
; `.guard-report.txt` — репо-відносний, щоб CI міг його cat-нути.
(визначити озброєне-порушення
  (функція (звіт)
    (записати-файл ".guard-report.txt" звіт)
    (violation "AUTHORITY-FAIL: authority violation — offenders list in .guard-report.txt")))

; --- 1. Required self-hosting artifacts ---
(за-умовою
  ((процес-успішний? "test -f asm/nucleus.s")
   ())
  ((тотожне? (процес-успішний? "test -f asm/nucleus.s") 0)
   (violation "AUTHORITY-FAIL: missing asm/nucleus.s (x86-64 asm core)")))

(за-умовою
  ((процес-успішний? "test -f asm/nucleus-win64.s")
   ())
  ((тотожне? (процес-успішний? "test -f asm/nucleus-win64.s") 0)
   (violation "AUTHORITY-FAIL: missing asm/nucleus-win64.s")))

(за-умовою
  ((процес-успішний? "test -f docs/AUTHORITY.md")
   ())
  ((тотожне? (процес-успішний? "test -f docs/AUTHORITY.md") 0)
   (violation "AUTHORITY-FAIL: missing docs/AUTHORITY.md")))

(за-умовою
  ((процес-успішний? "test -f docs/archive/completed-plans/dll-inventory-2026-09-11.md")
   ())
  ((тотожне? (процес-успішний? "test -f docs/archive/completed-plans/dll-inventory-2026-09-11.md") 0)
   (violation "AUTHORITY-FAIL: missing archived dll inventory")))

(показати "AUTHORITY OK: required self-hosting artifacts present\n")

; --- 2. C/Rust substrates are permitted with explicit provenance ---
(показати "AUTHORITY OK: C and Rust substrates are permitted with explicit provenance\n")

; --- 4. dll/ must be gone (Phase D complete) ---
(за-умовою
  ((процес-успішний? "test -d dll")
   (violation "AUTHORITY-FAIL: dll/ still present — Phase D (delete Rust host embed) not complete"))
  ((тотожне? (процес-успішний? "test -d dll") 0)
   (показати "AUTHORITY OK: dll/ absent — Phase D complete\n")))

; --- 5. Documentation authority: one active entry point, archive stays non-normative ---
(за-умовою
  ((процес-успішний? "test -f docs/CURRENT.md")
   ())
  ((тотожне? (процес-успішний? "test -f docs/CURRENT.md") 0)
   (violation "AUTHORITY-FAIL: missing docs/CURRENT.md (documentation entry point, wsm-my-lisp#19)")))

(за-умовою
  ((процес-успішний? "test -f docs/archive/README.md")
   ())
  ((тотожне? (процес-успішний? "test -f docs/archive/README.md") 0)
   (violation "AUTHORITY-FAIL: missing docs/archive/README.md (non-normative warning)")))

(за-умовою
  ((текст-порожній? (вивід-процесу "for f in docs/archive/*/*.md; do [ -f \"$f\" ] || continue; grep -qi 'ARCHIVED' \"$f\" || echo \"$f\"; done 2>/dev/null"))
   (показати "AUTHORITY OK: docs/CURRENT.md present, archive docs self-identify as archived\n"))
  ((тотожне? (текст-порожній? (вивід-процесу "for f in docs/archive/*/*.md; do [ -f \"$f\" ] || continue; grep -qi 'ARCHIVED' \"$f\" || echo \"$f\"; done 2>/dev/null")) 0)
   (озброєне-порушення
     (вивід-процесу "for f in docs/archive/*/*.md; do [ -f \"$f\" ] || continue; grep -qi 'ARCHIVED' \"$f\" || echo \"$f\"; done 2>/dev/null"))))

; --- 6. Self-hosting CI must not reference dll as core authority ---
(за-умовою
  ((процес-успішний? "test -f .github/workflows/self-hosting-authority.yml")
   (за-умовою
     ((процес-успішний? "grep -E '^\\s*- \"dll/' .github/workflows/self-hosting-authority.yml")
      (violation "AUTHORITY-FAIL: self-hosting-authority.yml must not path-trigger on dll/ (deleted)"))
     ((тотожне? (процес-успішний? "grep -E '^\\s*- \"dll/' .github/workflows/self-hosting-authority.yml") 0)
      (показати "AUTHORITY OK: self-hosting workflow carries no dll/ path trigger\n"))))
  ((тотожне? (процес-успішний? "test -f .github/workflows/self-hosting-authority.yml") 0)
   ()))

; --- 7. SID8-ONLY: function identity is a bare 8-bit binary token, never a
; decimal/hex/short alias (wsm-my-lisp#49, ecosystem#16, 2026-09-23). Shared
; check with the bash guard -- see check-canon-function-table-sid8.sh for
; what this actually checks and why.
(за-умовою
  ((текст-порожній? (вивід-процесу "scripts/check-canon-function-table-sid8.sh 2>/dev/null"))
   (показати "AUTHORITY OK: docs/canon-function-table.md: every semantic id is a bare 8-bit binary token\n"))
  ((тотожне? (текст-порожній? (вивід-процесу "scripts/check-canon-function-table-sid8.sh 2>/dev/null")) 0)
   (озброєне-порушення (вивід-процесу "scripts/check-canon-function-table-sid8.sh 2>/dev/null"))))

(показати "Lisp-first authority guard passed (C/Rust substrates allowed with provenance).\n")
()
