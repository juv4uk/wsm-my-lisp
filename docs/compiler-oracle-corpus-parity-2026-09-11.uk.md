# Паритет компілятора та оракула корпусу — wsm-my-lisp#4 (2026-09-11)

Підмодуль `external/my-lisp` оновлено `ccacc68` → `6d71151` (власний коміт фіксації корпусу my-lisp `#67`), тому цей репозиторій споживає той самий набір фікстур із тегом `(compiler-corpus . t)`, який my-lisp реально зафіксував, а не старіший коміт, що передував цьому тегуванню. Цей документ є триколонковою таблицею спостережень, якої вимагають критерії прийняття #4, складеною шляхом прямого зіставлення `wsm-my-lisp/harness/src/harness_*.rs` із 17 тегованими записами в `external/my-lisp/tests/fixtures/conformance.my` — без припущень та копіювання зі старих аудитів.

## Колонки

- **my-lisp oracle**: власний `expr`/`expected` (або `error`) фікстури з `conformance.my`, без змін, авторитет my-lisp.
- **WSM/meta**: чи очікується, що `external/my-lisp/lib/meta-eval.my` (вихідний код самохостованого обчислювача) взагалі покриває цю форму — усі 17 є звичайним обчисленням, тому в цій колонці всюди `yes`; вона присутня структурно згідно з вимогами #4.
- **compiled/native target**: фактичний свідок у `wsm-my-lisp/harness/` (або `cml`), якщо такий є, перевірений на *точному* виразі та значенні.

## Легенда статусів (непідтримуване має бути вказане явно, без замовчування)

- **confirmed** — свідок виконує саме цей *точний* вираз, і його декодований результат збігається з очікуваним `expected`/`error` фікстури.
- **related** — свідок вправляє той самий цільовий примітив (`wsm_cons`/`wsm_car` тощо), але на інших літеральних значеннях або іншій формі виразу; це доказ роботи механізму, а не проходження конкретної фікстури.
- **pending** — у межах поточної мети Stage2/asm nucleus, свідок ще не написаний.
- **unsupported** — поза поточними можливостями ядра (раціональна арифметика, макроси); це не помилка, а межа поточного обсягу.

## Таблиця

Усі 17 записів із тегом `compiler-corpus`, верифіковані за `external/my-lisp/tests/fixtures/conformance.my`:

| # | expr | expected/error | WSM/meta | compiled/native target | status |
|---|---|---|---|---|---|
| 1 | `(quote radio)` | `radio` | yes | немає | pending |
| 2 | `(atom (quote radio))` | `t` | yes | `harness-atom` перевіряє `(atom (quote ()))`, інший літерал | related |
| 3 | `(eq (quote radio) (quote radio))` | `t` | yes | `harness-eq-symbol` доводить `wsm_eq` на символах | related |
| 4 | `(car (quote (radio antenna)))` | `radio` | yes | `harness-cons` викликає `wsm_car` на інших літералах | related |
| 5 | `(cdr (quote (radio antenna)))` | `(antenna)` | yes | `harness-cons` викликає `wsm_cdr` на інших літералах | related |
| 6 | `(cons (quote radio) (quote (antenna)))` | `radio antenna` | yes | `harness-cons` доводить `(cons (quote A) (quote B))` | related |
| 7 | `(cond (() (quote wrong)) (t (quote right)))` | `right` | yes | `harness-cond` — реальна компіляція cml (`parser` → `lower` → `x86_freestanding`), зібрано з `asm/nucleus.s`, повертає `right` | **confirmed** |
| 8 | `(/ 5 6 8 7)` | `5/336` | yes | немає — раціональна арифметика відсутня в `asm/nucleus.s` | unsupported |
| 9 | `(eq (lambda (x) x) (lambda (x) x))` | `()` | yes | `harness-closure` доводить ідентичність замикань через ABI | related |
| 10 | `(defmacro foo)` | error `Arity` | yes | немає — підтримка макросів відсутня в asm nucleus | unsupported |
| 11 | `(def count-down (lambda (n) (cond ((eq n 0) (quote done)) (t (count-down (- n 1)))))) (count-down 100000)` | `done` | yes | `harness-countdown` — обмежена self-tail рекурсія на 100k, згенерована CML | **confirmed** |
| 12 | `((lambda (a b . rest) rest) 1 2 3 4 5)` | `(3 4 5)` | yes | коміт `cml` `1104657` (`tests/x86_top_level_let_test.rs::row12_corpus_fixture_exact_witness`) доводить варіативне застосування лямбди зі згорткою `wsm_cons` справа наліво, повертає `(3 4 5)` | **confirmed** |
| 13 | `((lambda args args) 1 2 3)` | `(1 2 3)` | yes | коміт `cml` `1104657` (`tests/x86_top_level_let_test.rs::row13_corpus_fixture_exact_witness`) доводить застосування all-rest лямбди зі згорткою `wsm_cons`, повертає `(1 2 3)` | **confirmed** |
| 14 | `((lambda (a b . rest) a) 1)` | error `Arity` | yes | коміт `cml` `1104657` (`tests/x86_freestanding_test.rs::variadic_lambda_under_arity_is_rejected`) доводить відхилення під час компіляції з `InvalidArity { expected: 2, actual: 1 }` | **confirmed** |
| 15 | `(let ((second (lambda (x) (quote shadowed)))) (second (quote (1 2 3))))` | `shadowed` | yes | коміт `cml` `6ce577f` (`tests/x86_top_level_let_test.rs`) доводить застосування замикання у тілі top-level лексичного `let` з лінковкою до `asm/nucleus.s` | **confirmed** |
| 16 | `(let ((car (lambda (x) (quote shadowed)))) (car (quote (1 2))))` | error `InvalidForm` | yes | немає — форми Canon 0+7 не підлягають затіненню (Контракт 6.0) | pending |
| 17 | `(defmacro my-list items (cons (quote quote) (cons items (quote ())))) (my-list 1 2 3)` | `(1 2 3)` | yes | немає — підтримка макросів відсутня в asm nucleus | unsupported |

## Чесний підсумок

**6 з цих фікстур (`count-down`/`countdown` [рядок 11], `cond` [рядок 7], варіативний список rest [рядок 12], all-rest список [рядок 13], помилка arity для варіативної лямбди [рядок 14] та застосування замикання в `let` [рядок 15]) мають точний скомпільований/натівний доказ виконання на сьогодні.**
Рядок `cond` закрився завдяки відкриттю в `wsm-os-lisp`, рядок 15 — завдяки коміту `cml` `6ce577f`, а рядки 12–14 — завдяки коміту `cml` `1104657` (підтримка прямої варіативної та all-rest аплікації лямбд з пакуванням списків через `wsm_cons` та fail-closed перевіркою арності). Все інше залишається `related` або `pending`/`unsupported`.
