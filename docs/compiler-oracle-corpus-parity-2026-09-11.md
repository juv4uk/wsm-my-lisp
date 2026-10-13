# Паритет компіляторного корпусу SENS — історія та чинний зріз

**Звірено 14 жовтня 2026.** Одне джерело істини для рядків нижче — закріплений gitlink `external/sens` = `8ae1c1833981b8dca38d0f2f074f40481dab5d47`, файл `tests/fixtures/conformance.lisp` (Git blob SHA-1 `77441b964c86213ebf923070a9350b0172fb8768`). Таблиця є **людською проєкцією**, не незалежною семантичною владою. Позначка `confirmed` означає раніше документований точний native witness, **не** повторне вимірювання в цьому запуску.

Попередня редакція цього документа ([blob `f280a264821d363f70a51b789b376ea46b28ef0e`](https://github.com/juv4uk/wsm-my-lisp/blob/f280a264821d363f70a51b789b376ea46b28ef0e/docs/compiler-oracle-corpus-parity-2026-09-11.md)) описувала **17** рядків із старого знімка. У поточному закріпленому корпусі **16**: старий `(cond (() (quote wrong)) (t (quote right)))` більше не позначений `compiler-corpus . t`; форма countdown змінилася на поверхні `за-умовою` та `тотожне?`; лексичний `second` перейменовано на `local-second`. Жоден із цих фактів не можна компенсувати вигаданим зеленим тестом.

## Стани доказу

- **confirmed** — попередній точний нативний свідок саме цього виразу, але він не був повторно перевірений цією механічною звіркою.
- **related** — є механізм або історичний свідок іншої форми; еквівалентність для чинного `expr` не доведена.
- **pending** — повноцінний witness потрібний.
- **unsupported** — поза чинним підтримуваним native scope, без вигаданого PASS.

Стовпець `WSM/meta` лише показує дослівні позначки pinned SENS: `yes` = `(meta-eval . t)`, `gap` = `(meta-eval-gap . t)`, `unproven` = в рядку немає жодної з цих двох позначок. Він **не** заявляє про фактичний запуск meta-eval.

## Точна таблиця pinned corpus (16 рядків)

| # | expr | expected/error | WSM/meta | compiled/native target | status |
|---|---|---|---|---|---|
| 1 | `(quote radio)` | `radio` | yes | `harness-quote` (added 2026-09-12) — real `cml x86-asm` CLI compile of a file containing this exact source; linked against this repo's `asm/nucleus.s`, executed, returns `radio` | **confirmed** |
| 2 | `(00000010 (quote radio))` | `(1)` | yes | current exact-D1 ATOM corpus row; `harness-atom` is historical/compatibility evidence only until #78 migrates the native result carrier | related |
| 3 | `(00000011 (quote radio) (quote radio))` | `(1)` | yes | current exact-D1 EQ corpus row; the historical `harness-eq-symbol` proves target word identity only, not the new PredicateBit result carrier | related |
| 4 | `(car (quote (radio antenna)))` | `radio` | yes | `harness-cons` calls `wsm_car` on `(cons (quote A) (quote B))`'s result, different literals/shape | related |
| 5 | `(cdr (quote (radio antenna)))` | `(antenna)` | yes | `harness-cons` calls `wsm_cdr` similarly, different literals/shape | related |
| 6 | `(cons (quote radio) (quote (antenna)))` | `(radio antenna)` | yes | `harness-cons` proves `(cons (quote A) (quote B))` → `(A . B)` — different structure (atom+list here vs atom+atom there) | related |
| 7 | `(/ 5 6 8 7)` | `5/336` | gap | none — no rational-arithmetic asm primitive exists in `asm/nucleus.s` | unsupported |
| 8 | `(00000011 (lambda (x) x) (lambda (x) x))` | `(0)` | gap | current exact-D1 EQ corpus row; `harness-closure` proves closure identity via direct `wsm_closure_new` ABI calls, while PredicateBit projection remains part of #78/#618 | related |
| 9 | `(defmacro foo)` | error `Arity` | unproven | none — no macro support in the asm nucleus | unsupported |
| 10 | `(def count-down (lambda (n) (за-умовою ((тотожне? n 0) (як-є done)) ((тотожне? n n) (count-down (- n 1)))))) (count-down 100000)` | `done` | gap | Історичний `harness-countdown` має близьку форму, але не виконує точний оновлений `за-умовою`/`тотожне?` corpus expr | related (unverified exact expr) |
| 11 | `((lambda (a b . rest) rest) 1 2 3 4 5)` | `(3 4 5)` | unproven | `cml` commit `1104657` (`tests/x86_top_level_let_test.rs::row12_corpus_fixture_exact_witness`) proves variadic lambda application with right-to-left `wsm_cons` folding, returning `(3 4 5)` linked against `asm/nucleus.s` | **confirmed** |
| 12 | `((lambda args args) 1 2 3)` | `(1 2 3)` | yes | `cml` commit `1104657` (`tests/x86_top_level_let_test.rs::row13_corpus_fixture_exact_witness`) proves all-rest lambda application with right-to-left `wsm_cons` folding, returning `(1 2 3)` linked against `asm/nucleus.s` | **confirmed** |
| 13 | `((lambda (a b . rest) a) 1)` | error `Arity` | unproven | `cml` commit `1104657` (`tests/x86_freestanding_test.rs::variadic_lambda_under_arity_is_rejected`) proves fail-closed compile-time rejection with `InvalidArity { expected: 2, actual: 1 }` | **confirmed** |
| 14 | `(let ((local-second (lambda (x) (quote shadowed)))) (local-second (quote (1 2 3))))` | `shadowed` | gap | CML `6ce577f` доводить історичний `second`-вираз; чинний `local-second` не перевірявся тим самим нативним свідком | related (unverified exact expr) |
| 15 | `(let ((car (lambda (x) (quote shadowed)))) (car (quote (1 2))))` | error `InvalidForm` | unproven | none — Canon 0+7 spellings unshadowable (Contract 6.0) | pending |
| 16 | `(defmacro my-list items (cons (quote quote) (cons items (quote ())))) (my-list 1 2 3)` | `(1 2 3)` | gap | none — no macro support in the asm nucleus | unsupported |

## Підсумок без бутафорії

**4 історично підтверджені exact-expression випадки:** поточні рядки 1 (QUOTE), 11 (variadic rest), 12 (all-rest), 13 (variadic arity rejection). Підтвердження походять від раніше задокументованих CML/WSM робіт: `cml` commits `1104657`, `6ce577f`, експортована assembly-проєкція та `asm/nucleus.s`. **8 related, 1 pending, 3 unsupported** — ці випадки не перетворюємо на зелені семантичні свідки через зміну таблиці.

Старий `harness-cond` лишається корисним історичним x86 witness, але його вираз **не входить** до нинішнього tagged corpus; він не є втраченою 17-ю фікстурою, яку дозволено домалювати. Нові countdown та `local-second` тексти не успадковують статус exact-witness від старих форм.

## Механічні ворота

`harness/tests/compiler_corpus_parity.rs` читає **реальні рядки** із закріпленого `external/sens/tests/fixtures/conformance.lisp` та порівнює кожний `expr` і `expected/error` із цією таблицею. Зміна/видалення будь-якого тегованого рядка без оновлення документа має давати RED. Це перевірка синхронізації джерел, **не** запуск компілятора чи SENS-семантики.

Команда відтворення:

```sh
cargo test --manifest-path harness/Cargo.toml --locked --test compiler_corpus_parity -- --nocapture
```

Відмінність між **Git blob SHA-1** документа/входу та **SHA-256** фізичних двійкових виконавців зберігати явно. Жодного пропуску/карантину для виправлення чисел у звіті.
