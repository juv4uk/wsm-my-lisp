# SELFHOST-GATE-4 — Stage2 evaluator witness

Поточний Lisp-owned oracle вже має повний call graph `my-eval-top-form` →
`my-eval-program` і перевірені нетривіальні програми:

- послідовні `def` із видимістю попередніх binding-ів;
- lambda/lexical application через top-level definition;
- `cond`, `cons`, точна арифметика та structural data;
- self-recursive `count-down` і factorial;
- named error parity у `meta_eval_error_kind` tests.

Основний source artifact для provenance:

`external/my-lisp/lib/meta-eval.lisp`

SHA-256 станом на цей commit:

`d7e094a6b001463ce50ee35948f290eb3573100fa694ebe0fce80208b601053f`

Перевірка oracle:

```text
cargo test -p my-lisp --test meta_eval def_extends_the_environment_visible_to_later_top_level_forms
cargo test -p my-lisp --test meta_eval self_recursive_top_level_def_sees_its_own_binding
cargo test -p my-lisp --test meta_eval recursive_factorial_matches_native_language_meaning
```

Цей запис не оголошує WSM target execution завершеним: окремий target witness
ще має пройти тим самим Lisp-owned program artifact без Rust/C evaluator.
