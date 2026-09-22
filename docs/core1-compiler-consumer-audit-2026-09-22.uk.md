# Core1 consumer audit: compiler seed

**Статус:** підтверджено локальним executable witness.  
**Задача:** `wsm-my-lisp#43`.  
**Scope:** споживач `lib/compiler.lisp`; Core1 не редагувався.

## Точні артефакти

| Шар | Артефакт |
|---|---|
| S0 historical seed | `mccarthy-eval@1c6acdb9dac890b2a0535ec0453bd83dfdda7248` |
| S1 Core1 | `my-lisp@01b67f398bb5bff1aa0ab0e25f7e09a73e51b86b:lib/core1.lisp` |
| S1 SHA-256 | `e56cbbeb4f3aaededf14a8af6ce164ccc219cd3b535fcaf575329f0f5c8c3b34` |
| S2 compiler seed | `wsm-my-lisp@a6bd9747196f2d676ab6883cbb4b84c7fcfc6947:lib/compiler.lisp` |
| S2 SHA-256 | `b401cd09a0040c64668aa98369cad4932c06767343de5214a0eae2dae3ce6cce` |

## Виконуваний доказ

```bash
scripts/check-core1-compiler-consumer.sh
```

Checker бере Core1 через `git show` за exact revision, розгортає S0 через
`git archive`, компілює незмінений `mccarthy-kernel.s`, цитує S2 compiler
forms як дані для `C1-EVAL-PROGRAM-THEN` і не пише в жоден upstream checkout.

Найменший whole-program witness:

```lisp
(compiler-program (quote ((quote A))))
```

Двічі дає ті самі байти друкованого IR:

```lisp
((quote A))
```

Тому це саме шлях `Core1 runtime → compiler-program → deterministic IR`, а не
лише завантаження файла чи статичний аналіз.

## Класифікація залежностей

| Елемент | Клас | Доказ |
|---|---|---|
| `quote`, `atom`, `eq`, `cons`, `car`, `cdr`, `cond`, `lambda`, recursion | runtime-required | compiler definitions завантажуються і `compiler-program` виконується через C1 evaluator. |
| `list` | Lisp-derived, runtime-required | `compiler-clause` повертає `((quote A) (quote yes) (quote no))`; C1 реалізує LIST як Lisp-owned operation, не S0 primitive. |
| `not` | Lisp-derived, runtime-required | `compiler-clause-shape?` для three-field clause повертає `T`; C1 реалізує NOT у Lisp. |
| `def` | S1 compatibility projection | увесь lowercase compiler source передається як data у `C1-EVAL-PROGRAM-THEN`; Core1 explicit accepts `def` alongside `DEFINE`. Нормалізація source не потрібна. |
| `t` | historical control convention | compiler source використовує двочастинний historical `cond`; Core1 contract фіксує цей режим окремо від Core4. |
| `+`, `-` | emitted-data-only | `(compiler-primitive? (quote +))` і те саме для `-` повертають `T`; жодна Core1 arithmetic operation не викликається. |
| `prim`, `var`, `app`, `cond-match`, `compile-error` | compiler-only IR data | вони емітуються або порівнюються як quoted data, не є Core1 runtime operations. |

`+` і `-` не доводять потреби додавати арифметику в Core1. S0 reader/print
поверхня для цих glyphs має власні історичні межі, тому checker перевіряє
їхній semantic role через `compiler-primitive?`, а не видає S0 printer за
канонічний IR renderer.

## Межа висновку

Цей доказ не є CML lowering і не є fixed point. Він доводить лише, що
поточний compiler seed входить у мінімальний Core1 без арифметики, I/O,
macros або нової assembly closure machinery. Наступна межа — S3 artifact,
якою володіють `wsm-my-lisp#38/#39` та `cml#190`.
