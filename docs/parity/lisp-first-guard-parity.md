# Lisp-first authority guard — parity note (process-run interim)

## Мета

Підтвердити, що `scripts/check-lisp-first-authority.lisp` перевіряє той самий
набір контрактів, що й bash-версія `scripts/check-lisp-first-authority.sh`,
і використовує той самий fail-closed семантичний контур.

## Чому process-run interim?

Мова my-lisp ще не має read-dir / glob / string-split, які потрібні для
перебору файлів. Bash залишається механічною підкладкою для find/test/grep;
рішення про pass/fail приймає Lisp через `cond`.

Детальне обґрунтування — в `AGENTS.md` мови (раздел "process-run interim"):
механіка дозволяє використовувати process-run для тих операцій, де
чистий Lisp ще не має потрібних примітивів, за умови повної парності
перевірок з bash-версією.

## Що саме перевіряємо (парність секцій)

| #  | Контракт                                    | Bash-спосіб                                              | Lisp-спосіб (process-run + cond)                                                                                     | Формат рішення                           |
|----|----------------------------------------------|----------------------------------------------------------|----------------------------------------------------------------------------------------------------------------------|------------------------------------------|
| 1  | Артефакти існують                             | `[[ -f ... ]]`                                          | `(process-ok? "test -f ...")` в cond                                                                                 | Fail-fast; exit 1 на першому пропущеному |
| 2  | Немає semantic .rs поза harness/external/    | find + while + case/basename                             | `(process-out "find ... \| grep ...")` → string-empty? в cond                                                         | armed-violation: offenders → .guard-report.txt; exit 1 |
| 3  | Немає C/C++ поза harness/asm/external/       | find + while + case/skip                                 | `(process-out "find ...")` (exclude harness/external/asm) → string-empty? in cond                                     | armed-violation; exit 1                  |
| 4  | dll/ відсутня                                | `[[ ! -d dll ]]`                                        | `(process-ok? "test -d dll")` in cond                                                                                 | Inline violation; exit 1                 |
| 5  | docs: CURRENT.md + archive README + ARCHIVED | test -f + for + grep -qi                                | `(process-out "for f in ... ARCHIVED ...")` → string-empty? in cond                                                  | armed-violation; exit 1                  |
| 6  | Workflow: dll/ trigger відсутній              | `grep -qE`                                              | `(process-ok? "grep -E '...dll/'...")` в cond                                                                        | Inline violation; exit 1                 |

## Fail-mechanізм

my-lisp не має begin/set!/raise. Armed-violation використовує два канали:

1. **Постійний** (переживає abort): запис деталей у `.guard-report.txt` через
   `write-file`. CI може cat-нути файл для отримання повного списку
   порушень.

2. **Негайний** (CLI exit 1): виклик невідомого символу `violation` викликає
   LanguageError → CLI `eprintln!("Error: ...")` + `process::exit(1)`. Render
   виводить source-рядок з літералом повідомлення.

## Тестові результати (parity)

Поточний стан репо: ** всі секції PASS **. Bash і Lisp return同样的 OK-рядки.

### Fail-тести (Lisp version)

| Тест                                             | Exit | .guard-report.txt містить        | stderr-джерело повідомлення          |
|--------------------------------------------------|------|----------------------------------|--------------------------------------|
| Fake eval.rs у _test_guard/                      | 1    | `./_test_guard/eval.rs`          | `(violation "AUTHORITY-FAIL: ...")`  |
| Fake hello.c у _test_guard/                      | 1    | `./_test_guard/hello.c`          | `(violation "AUTHORITY-FAIL: ...")`  |
| Тимчасова dll/ директорія                        | 1    | —                                | Inline `(violation ...)` на рядку 69 |
| docs/archive/_test/x.md без ARCHIVED             | 1    | `docs/archive/_test/x.md`        | `(violation "AUTHORITY-FAIL: ...")`  |

### Мовні особливості (для агентів)

- `if` / `nil` / `begin` / `set!` / `string-split` — не існують.
- Умовний конструкт: `cond` з `(t ...)` як else-гілка.
- Порожній список = `()`, не `nil`.
- `process-run` — Lisp-замикання з `lib/process.lisp`, повертає
  `(exit-code stdout stderr)` як рядки.
- `\n` у рядкових літералах працює як перенос.
- `\(` у процес-run -c команді проходить як literal.

## CI

CI (`self-hosting-authority.yml`) більше не викликає bash-версію напряму:
крок "Lisp-first authority guard (P0 #15)" збирає `my-lisp-cli` з pinned
submodule (`5500ac2e`) і запускає `scripts/check-lisp-first-authority.lisp`
через бінарник:

```yaml
cargo build --release --locked --manifest-path external/my-lisp/Cargo.toml -p my-lisp-cli
external/my-lisp/target/release/my-lisp scripts/check-lisp-first-authority.lisp || { cat .guard-report.txt 2>/dev/null; exit 1; }
```

Bash-версія `scripts/check-lisp-first-authority.sh` залишається в репо як
reference/fallback і продовжує давати ті самі рішення на тому самому репо
(parity підтверджено; CI крок зараз викликає саме .lisp).
