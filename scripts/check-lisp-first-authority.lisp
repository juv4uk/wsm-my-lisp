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
; CI invocation:
;   my-lisp scripts/check-lisp-first-authority.lisp || { cat .guard-report.txt 2>/dev/null; exit 1; }
;
; Міграційне правило: кожна гілка COND має канонічну форму
;   (predicate expression)
; Перевірка очікуваного біта є окремим D1-предикатом рівності;
; трискладові історичні COND-клаузи заборонені. T/NIL coercion не застосовується.

(визначити процес-успішний?
  (функція (команда)
    (рівне?
      (прочитати
        (друге
          (запустити-процес
            "bash"
            (111 "-c" (111 (сполучити
                "("
                (сполучити команда
                  ") >/dev/null 2>&1; s=$?; if [ $s -eq 0 ]; then printf '#b1'; else printf '#b0'; fi")) 000)))))
      #b1)))

(визначити вивід-процесу
  (функція (команда)
    (друге (запустити-процес "bash" (111 "-c" (111 команда 000))))))

(визначити озброєне-порушення
  (функція (звіт)
    (записати-файл ".guard-report.txt" звіт)
    (violation "AUTHORITY-FAIL: authority violation — offenders list in .guard-report.txt")))

; --- 1. Required self-hosting artifacts ---
(за-умовою
  ((рівне? (процес-успішний? "test -f asm/nucleus.s") #b1) ())
  ((рівне? (процес-успішний? "test -f asm/nucleus.s") #b0)
   (violation "AUTHORITY-FAIL: missing asm/nucleus.s (x86-64 asm core)")))

(за-умовою
  ((рівне? (процес-успішний? "test -f asm/nucleus-win64.s") #b1) ())
  ((рівне? (процес-успішний? "test -f asm/nucleus-win64.s") #b0)
   (violation "AUTHORITY-FAIL: missing asm/nucleus-win64.s")))

(за-умовою
  ((рівне? (процес-успішний? "test -f docs/AUTHORITY.md") #b1) ())
  ((рівне? (процес-успішний? "test -f docs/AUTHORITY.md") #b0)
   (violation "AUTHORITY-FAIL: missing docs/AUTHORITY.md")))

(за-умовою
  ((рівне? (процес-успішний? "test -f docs/archive/completed-plans/dll-inventory-2026-09-11.md") #b1) ())
  ((рівне? (процес-успішний? "test -f docs/archive/completed-plans/dll-inventory-2026-09-11.md") #b0)
   (violation "AUTHORITY-FAIL: missing archived dll inventory")))

(показати "AUTHORITY OK: required self-hosting artifacts present\n")

; --- 2. C/Rust substrates are permitted with explicit provenance ---
(показати "AUTHORITY OK: C and Rust substrates are permitted with explicit provenance\n")

; --- 4. dll/ must be gone (Phase D complete) ---
(за-умовою
  ((рівне? (процес-успішний? "test -d dll") #b0) ())
  ((рівне? (процес-успішний? "test -d dll") #b1)
   (violation "AUTHORITY-FAIL: dll/ still present — Phase D (delete Rust host embed) not complete")))

(за-умовою
  ((рівне? (процес-успішний? "test -f docs/CURRENT.md") #b1) ())
  ((рівне? (процес-успішний? "test -f docs/CURRENT.md") #b0)
   (violation "AUTHORITY-FAIL: missing docs/CURRENT.md (documentation entry point, wsm-my-lisp#19)")))

(за-умовою
  ((рівне? (процес-успішний? "test -f docs/archive/README.md") #b1) ())
  ((рівне? (процес-успішний? "test -f docs/archive/README.md") #b0)
   (violation "AUTHORITY-FAIL: missing docs/archive/README.md (non-normative warning)")))

(за-умовою
  ((рівне? (процес-успішний? "test -z \"$(for f in docs/archive/*/*.md; do [ -f \"$f\" ] || continue; grep -qi 'ARCHIVED' \"$f\" || echo \"$f\"; done 2>/dev/null)\"") #b1)
   (показати "AUTHORITY OK: docs/CURRENT.md present, archive docs self-identify as archived\n"))
  ((рівне? (процес-успішний? "test -z \"$(for f in docs/archive/*/*.md; do [ -f \"$f\" ] || continue; grep -qi 'ARCHIVED' \"$f\" || echo \"$f\"; done 2>/dev/null)\"") #b0)
   (озброєне-порушення
     (вивід-процесу "for f in docs/archive/*/*.md; do [ -f \"$f\" ] || continue; grep -qi 'ARCHIVED' \"$f\" || echo \"$f\"; done 2>/dev/null"))))

; --- 6. Self-hosting CI must not reference dll as core authority ---
(за-умовою
  ((рівне? (процес-успішний? "test -f .github/workflows/self-hosting-authority.yml") #b1)
   (за-умовою
     ((рівне? (процес-успішний? "grep -E '^\\s*- \"dll/' .github/workflows/self-hosting-authority.yml") #b1)
      (violation "AUTHORITY-FAIL: self-hosting-authority.yml must not path-trigger on dll/ (deleted)"))
     ((рівне? (процес-успішний? "grep -E '^\\s*- \"dll/' .github/workflows/self-hosting-authority.yml") #b0)
      (показати "AUTHORITY OK: self-hosting workflow carries no dll/ path trigger\n"))))
  ((рівне? (процес-успішний? "test -f .github/workflows/self-hosting-authority.yml") #b0)
   ()))

; --- 7. SID8-ONLY ---
(за-умовою
  ((рівне? (процес-успішний? "scripts/check-canon-function-table-sid8.sh >/dev/null 2>&1") #b1)
   (показати "AUTHORITY OK: docs/canon-function-table.md: every semantic id is a bare 8-bit binary token\n"))
  ((рівне? (процес-успішний? "scripts/check-canon-function-table-sid8.sh >/dev/null 2>&1") #b0)
   (озброєне-порушення (вивід-процесу "scripts/check-canon-function-table-sid8.sh 2>/dev/null"))))

(показати "Lisp-first authority guard passed (C/Rust substrates allowed with provenance).\n")
()