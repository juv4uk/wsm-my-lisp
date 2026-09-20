# SELFHOST-GATE-2 — поточний доказ

Цей запис фіксує межу роботи над `my-apply`. Семантичним джерелом лишається
Lisp-реалізація `external/my-lisp/lib/meta-eval.lisp`; Rust/C не є evaluator.

## Перевірені свідки

1. Reference oracle: `cargo test -p my-lisp --test mccarthy
   meta_eval_lambda_witness_env_capture_and_application -- --nocapture` —
   `1 passed`. Цей тест перевіряє захоплення lexical environment і виклик
   closure через `my-apply`.
2. Target mechanism: у CML з явним `WSM_NUCLEUS_ASM` проходить
   `cargo test --test x86_runtime_closure_application_test` — `1 passed`.
   Свідок виконує application вже створеного closure value на x86 nucleus.

Це ще не є завершенням Gate 2: потрібен один об'єднаний Lisp-owned witness,
який проведе саме `my-apply` evaluator через WSM target path і порівняє
результат із oracle. Цей файл не додає нової семантики й не замінює цей
відсутній доказ.
