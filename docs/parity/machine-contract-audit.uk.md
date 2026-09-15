# Аудит Machine Contract — MACHINE-CONTRACT-1 (wsm-my-lisp#23)

Дата: 2026-09-14
Pinned my-lisp: `8ffffce9` (origin/main, містить machine-lowering-boundary.lisp + memory-layout-contract.lisp + lib/machine/**)
Verification gate: `scripts/check-machine-contracts.lisp` (Lisp-first, у CI через `my-lisp`)
Тип аудиту: fail-closed; невідомо/відсутньо = RED

## Принцип

**WSM виконує контракт. WSM не переписує контракт на себе.**

## Джерела правди контракту

| Контракт | Шлях у піні | Роль |
|---|---|---|
| `machine-lowering-boundary.lisp` | `external/my-lisp/machine-lowering-boundary.lisp` | Напрям authority: semantic IDs і спостереження належать my-lisp; lowering односпрямований semantic→machine; semantic IDs з ISA/opcode/asm заборонені; machine text/bytes — лише projection |
| `memory-layout-contract.lisp` | `external/my-lisp/memory-layout-contract.lisp` | Спільний layout пам'яті: nan-boxing-64 представлення значень, 4-bit тег у бітах 31-28, heap-представлення string/rational/closure |
| `semantic-x86-64.lisp` | `external/my-lisp/lib/machine/lowering/semantic-x86-64.lisp` | Lowering профіль semantic→x86-64: зіставляє Lisp-owned semantic IDs (0002-1075) з допущеними x86-операціями |
| `x86-64.lisp` | `external/my-lisp/lib/machine/encoding/x86-64.lisp` | x86-64 енкодер: Lisp-авторський, власна authority кодування |

## Аудит локальних машинних фактів

### Допущений зріз — Semantic IDs (projection з Lisp-контракту)

| Місцеве використання | Semantic ID upstream | Джерело upstream | Статус | Нотатки |
|---|---|---|---|---|
| cons (allocate + store pair) | 0004 | semantic-x86-64.lisp | **projection** | `(0004 runtime "allocate+STORE-pair")` — Lisp-owned identity, не народжена з ISA |
| car (load pair head) | 0005 | semantic-x86-64.lisp | **projection** | `(0005 direct "LOAD-pair-head")` |
| cdr (load pair tail) | 0006 | semantic-x86-64.lisp | **projection** | `(0006 direct "LOAD-pair-tail")` |
| tag-test / atom | 0002 | semantic-x86-64.lisp | **projection** | `(0002 sequence "tag-test: TEST/AND/CMP")` |
| cmp / умовні | 0003, 0007 | semantic-x86-64.lisp | **projection** | операції межі |

### asm/nucleus.s — Bootstrap Mechanism (не семантична правда)

| Локальний факт | Значення | Джерело | Статус | Нотатки |
|---|---|---|---|---|
| `.equ TAG_CONS` | 0 | wsm-target-contract (target-contract.wsm) | **bootstrap mirror** | Freestanding ABI; НЕ projection nan-boxing memory-layout-contract (там cons=1). Механізм, не семантична identity. |
| `.equ TAG_NIL` | 1 | wsm-target-contract | **bootstrap mirror** | Те саме: відрізняється від memory-layout-contract nil=3 |
| `.equ TAG_SYMBOL` | 4 | wsm-target-contract | **bootstrap mirror** | Відрізняється від memory-layout-contract symbol=2 |
| `.equ TAG_CLOSURE` | 5 | wsm-target-contract | **bootstrap mirror** | Відрізняється від memory-layout-contract closure=8 |
| `.equ TAG_MASK` | 7 | wsm-target-contract | **bootstrap mirror** | 3-bit маска; memory-layout-contract використовує 4-bit у бітах 31-28 |
| `.equ TAG_TRUE` | — | — | **obsolete / відсутній** | Не повинен існувати; контролюється CI |
| `.equ SYM_T_ID` | SYMBOL_ID_MAX | wsm-target-contract / language-contract | **projection** | Канонічне `t` збігається з language-contract |
| `.equ CLOSURE_*_OFFSET` | 0, 8 | wsm-target-contract | **bootstrap mirror** | Closure ABI — target-рівневі механізми |
| ERR_* коди | 1, 2, 4 | wsm-target-contract | **bootstrap mirror** | Коди помилок runtime: target механізм |

### CI Workflow — Contract Gate

| Перевірка | Fast CI | Deep CI |
|---|---|---|
| Пін `external/my-lisp` = `8ffffce9` | ✅ | ✅ |
| `machine-lowering-boundary.lisp` присутній + schema /2 | ✅ | ✅ |
| `memory-layout-contract.lisp` присутній + version (1 0) | ✅ | ✅ |
| `lib/machine/lowering/semantic-x86-64.lisp` присутній | ✅ | ✅ |
| `lib/machine/encoding/x86-64.lisp` присутній | ✅ | ✅ |
| Напрям: semantic-authority=my-lisp, lowering=semantic→machine, semantic-id-from-isa=forbidden | ✅ | ✅ |
| Semantic IDs (0002/0004/0005/0006) доступні | ✅ | ✅ |
| Немає `.equ TAG_TRUE` у nucleus.s | ✅ | ✅ |
| Повна projection parity (усі witness-и проходять) | — | ✅ |

## Архітектурна напруга: представлення Value Layout

WSM freestanding target (wsm-target-contract, 3-bit теги у молодших бітах) і `memory-layout-contract.lisp` my-lisp (nan-boxing-64, 4-bit теги у бітах 31-28) визначають **різні фізичні представлення значень**. Це не дрейф: це різні архітектурні ролі.

- **memory-layout-contract** описує host-runtime value layout (my-lisp, CML, fpga-lisp).
- **wsm-target-contract** описує freestanding self-hosting target ABI WSM — свідомо незалежний, обмежений механізм фізичного доказу виконання.

Що WSM споживає з Lisp-контрактів (projected):
- Семантична ідентичність (значення 0002-1075) — **авторитетно від Lisp**
- Односпрямований lowering (semantic → machine) — **авторитетно від Lisp**
- Жодних ISA/opcode-label семантичних ID — **авторитетно від Lisp**

Що WSM володіє як власним bootstrap-механізмом (не претензія на мовну правду):
- Кодування тегів (3-bit молодші біти) — **local-only mechanism** для proof виконання
- Pair/closure ABI offset-и — **local-only mechanism** для target-зрізу
- Мапування кодів помилок — **local-only mechanism** для runtime

За issue #24 (STRUCTURAL-LISP-PARITY-1): повна parity layout між Lisp-native та WSM target — наступний крок після цього contract gate.

## Фальсифікація

Цей аудит — **НЕ** твердження, що різниця представлень правильна чи остаточна. Це чесний запис статусу. CI drift gate ловить:
- Видалення файлу контракту/зміна версії → RED
- Відсутність семантичних ID у допущеному зрізі → RED
- Нове `TAG_TRUE` або локальне виробництво правди → RED
- Дрейф upstream-контракту без WSM replay → RED