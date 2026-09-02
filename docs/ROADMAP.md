# WSM Lisp-on-Lisp Roadmap — for a Thinking Machine

Статус: PLAN, 2026-09-02. Власник: juv4uk. `my-lisp` (Rust) лишається семантичним oracle протягом усього плану — жодна стадія тут не вимагає й не передбачає його видалення.

## Частина I — Парадигма: `()` як фундамент мислячої машини

Це не декоративний вступ. Кожна стадія плану нижче звіряється проти цих правил, і жодна capability не вважається "готовою", якщо вона їх порушує.

### 1. `()` — єдина, нерозкладна точка відсутності

`()` не є "false". Не є контейнером для `unknown`/`possible`/`not-yet-known`. Це первинний, неозначений знак: система нічого не стверджує. Усе, що несе сенс — `unknown`, `not`, `possible`, `supports`/`contradicts`/`inconclusive` — окремі WSM-вирази ПОРУЧ із `()`, ніколи не підвиди чи декомпозиція `()` самого. `(unknown x)` ≠ `()` з доданим тегом; це інший, повноцінний вираз, що з'являється лише коли є хоч крихта підстави щось сказати.

### 2. `() не визначаємо. Ми лише не дозволяємо машині його втратити`

Мова не описує `()` — вона описує лише ВІДНОШЕННЯ до нього: `(atom ())`, `(eq () ())`, `(cond (() ...))`. Жодна форма не каже, ЩО таке `()`. Це поширюється на кожен рівень реалізації:

- **Мовний рівень**: `()` — примітивний, неконструйований токен.
- **Машинне представлення**: тег/біт-патерн може ВПІЗНАВАТИ `()`, ніколи не ОПИСУВАТИ. Тест перед кожним новим machine tag: *чи це представлення вже наявної WSM-відмінності, чи ми щойно винайшли семантику в машині?* Якщо відповідь "винайшли" — тег під питанням, незалежно від того, наскільки він зручний для реалізації (живий приклад: `wsm-os-target::Tag::True` і `fpga-lisp`'s апаратний `TAG_TRUE` — обидва вже знайдені й задокументовані як порушення цього правила, `t` є просто `Symbol("t")`, не примітив).
- **Асемблерний/апаратний рівень**: перед синтезом — той самий тест, до того, як тег застигне в залізі, не після.

### 3. `()` неможливо описати жодною мовою — і це саме та причина, чому воно годиться як фундамент

Санскрит (`śūnya`, `śūnyatā`), Лао-цзи, апофатична традиція, Вітгенштайн — усі впираються в ту саму межу: опис уже є твердженням, а `()` за визначенням є станом ДО будь-якого твердження. Тому: жодна мова (санскрит, англійська, WSM) не отримує права "визначити" `()` — лише окреслювати контур навколо нього через відношення.

### 4. Епістемічна скромність як архітектурний принцип, не риторика

`() має бути більшим, чим що-небудь інше` — не про обсяг інформації (у цьому сенсі `()` якраз мінімальне), а про простір сумісних майбутніх: `()` сумісне з БУДЬ-яким подальшим уточненням, тоді як кожне конкретне значення вже щось відкидає. Найменше знання = найбільша множина сумісних майбутніх. Звідси прямий інженерний наслідок для мислячої машини: **незнання — це default, знання — крихітний острівець поверх нього**, не навпаки. Reasoning-шар, який мовчки трактує "немає доказу" як "доведено ні" (SQL NULL-помилка, `Value::Bool(false)`-помилка, яку ми сьогодні виправили в `crates/my-lisp`) — це не деталь реалізації, це порушення фундаменту.

## Частина II — Технічний план

### Стадія 0 (ЗРОБЛЕНО, 2026-09-02): перший nucleus witness

- `WSM-SELFHOST-META-EVAL-PARITY-C0` (my-lisp) — `my-eval` (WSM-на-WSM evaluator) доведений на C0 (McCarthy-7) рівні, oracle-tier, 21 незалежний fixture, parity 21/21.
- Один C0-fixture (`(atom (quote ()))`) реально скомпільований через CML x86_64-freestanding backend і ВИКОНАНИЙ на реальному x86_64 (не лише зібраний) через `wsm-os-hosted`.
- Ручне x86_64 asm-ядро (`asm/nucleus.s` у цьому репо): 5 примітивів (`wsm_cons`/`wsm_car`/`wsm_cdr`/`wsm_eq`/`wsm_atom`), кожен реально виконаний і звірений з oracle — atom/cons/eq/lambda(bounded)/escaping-closure(bounded curried).
- `external/my-lisp` підключено як git submodule (не copy-paste) — `lib/meta-eval.my` там, точка старту "мій лісп на моєму ліспі".

### Стадія 1 (ВІДКРИТО): named functions на asm-ядрі

Блокер точно діагностований, не форсований: CML `x86_freestanding.rs` відхиляє `Ir::Def` безумовно — жодна named-функція не компілюється цим backend'ом, а весь `meta-eval.my` побудований через `(def name (lambda ...))`.

- `CML-X86-DEF-BOUNDED-SELF-TAIL-RECURSIVE-FUNCTION` (cml, зареєстровано, не done): допустити `Ir::Def` лише для однієї bounded self-tail-recursive функції без вільних змінних — форма `env-lookup`. Довести реальним запуском проти `asm/nucleus.s`, звірити з oracle.
- Наступні, окремі ворота (не робити разом зі Стадією 1): загальна/взаємна рекурсія (`Ir::App`/`Ir::Lambda` за межами bounded single-arg), `Ir::Let`, variadic-форми.

### Стадія 2: закриття `meta-eval.my`-графу викликів

По одному ворота: `my-eval-cond` → `my-apply` → `my-eval-body`/`my-eval-list` → `bind-params` → повний `my-eval-top-form`/`my-eval-program`. Кожен крок — той самий стандарт доказу: реально скомпільовано, реально запущено на `asm/nucleus.s`, звірено з `crates/my-lisp`-oracle. Жодного кроку без виконаного witness.

### Стадія 3: замикання self-hosting

Момент, коли `meta-eval.my` цілком виконується на asm-ядрі без участі Rust-evaluator під час виконання — **"WSM виконує WSM на моєму залізі"** перестає бути метафорою. Rust лишається offline-компілятором/tooling/oracle, не runtime-залежністю.

### Стадія 4: примітиви — Rust чи асемблер, за виміряним/архітектурним обґрунтуванням

`wsm_cons`/`wsm_car`/`wsm_cdr`/`wsm_eq`/`wsm_atom` уже мають ручну asm-реалізацію в цьому репо (паралельний шлях, `wsm-os-runtime`'s Rust лишається робочим, недоторканим). Рішення, який шлях стає основним — за принципом "не performance escape hatch": спочатку WSM, якщо повільно — виміряти, якщо CML lowering може оптимізувати — виправити lowering, лише тоді — hand-written asm із названою причиною.

### Стадія 5: `Tag::True` — fpga-lisp ISA до кінцевого затвердження

`FPGA-TAG-TRUE-MANUFACTURED-PRIMITIVE` (fpga-lisp, зареєстровано, не done): аудит завершено (blast radius чистий, single-layer, мінімальний фікс уже знайдено — `SYM_T=79` вже існує як bootstrap symbol, `TAG_SYMBOL`-механізм уже hardware-verified). Сам RTL-фікс — окрема, явно авторизована дія, бо торкається вже прошитого на реальній платі заліза.

### Стадія 6 (після 0–5, не раніше): reasoning-шар мислячої машини

Тут WSM перестає бути "лише" виконуваною мовою й стає мислячою машиною в буквальному сенсі — але лише на фундаменті, який уже дотримується Частини I. `lib/unify.my`/`reason.my`/`forward.my`/`knowledge.my`/`world.my` і особливо `lib/epistemic.my` (уже написаний, уже протестований 38/38, але НЕ підключений до `cond`/`is_truthy` — заголовок файлу прямо каже "not loaded by lib/core.my") — реальний наступний capability gap: з'єднати цей епістемічний шар (`proposed`/`reviewed`/`rejected`, `supports`/`contradicts`/`inconclusive`) з реальним прийняттям рішень, не ламаючи `() = ()`-нейтральність ядра. Не робити цього передчасно — спершу стадії 0–5, потім reasoning поверх живого self-hosted нуклеуса, не поверх Rust.

### Стадія 7 (довгостроково, названо, не розпочато): асемблер-у-Lisp за межі нуклеуса

`fpga-lisp/assembler.my` уже доводить, що асемблер, написаний у Lisp, може бути реальним, byte-identical шляхом до заліза. Boot/interrupt/stack-frame DSL (`(section boot)`, `(definterrupt ...)`) — логічний наступний крок ПІСЛЯ того, як нуклеус живий, не до. `спочатку народжується маленький Lisp → потім він сам будує собі машину й ОС`, не навпаки.

## Наскрізне правило для кожної стадії

Перед будь-яким новим machine tag, primitive, чи representation choice: **чи це вже наявна WSM-відмінність, чи ми щойно винайшли семантику в машині?** Перед будь-яким reasoning-рішенням: чи воно зберігає `() = ()`-нейтральність, чи мовчки перетворює "немає доказу" на "доведено ні"? Rust лишається oracle, доки конкретна capability не пройде повний шлях: WSM-реалізація існує → незалежні fixtures проходять → Rust↔WSM parity доведено → CML admission проходить → реальне виконання на target доведено.

---

# WSM Lisp-on-Lisp Roadmap — for a Thinking Machine (English mirror)

Status: PLAN, 2026-09-02. `my-lisp` (Rust) remains the semantic oracle for the entire plan — no stage here requires or anticipates its removal.

## Part I — Paradigm: `()` as the foundation of a thinking machine

Not decorative framing. Every stage below is checked against these rules, and no capability counts as "done" if it violates them.

1. **`()` is the single, irreducible point of absence.** Not false, not a container for unknown/possible/not-yet-known. `unknown`, `not`, `possible`, `supports`/`contradicts`/`inconclusive` are separate WSM expressions beside `()`, never subtypes or decompositions of it.
2. **We do not define `()`. We only refuse to let the machine lose it.** The language describes only relations to `()`, never `()` itself. Before any new machine tag: does it represent a WSM distinction that already exists, or did we just invent semantics inside the machine? (Live confirmed violations: `wsm-os-target::Tag::True`, `fpga-lisp`'s hardware `TAG_TRUE` — `t` is plain `Symbol("t")`, not a primitive.)
3. **`()` cannot be described by any language** — Sanskrit, English, or WSM — because a description is itself an assertion, and `()` is defined as the state prior to any assertion. No language earns the right to "define" it, only to outline relations around it.
4. **Epistemic humility as architecture, not rhetoric.** `()` "exceeds everything" not in information content (there it is minimal) but in the space of compatible futures — least knowledge, largest set of compatible extensions. Consequence for a thinking machine: ignorance is the default, knowledge a small island on top of it — never the reverse. A reasoning layer that silently treats "no evidence" as "proven false" violates the foundation, not a detail.

## Part II — Technical plan

Stage 0 (DONE): first nucleus witness — C0/McCarthy-7 oracle parity, one CML→x86_64 fixture actually executed (not just assembled), hand-written asm nucleus for 5 primitives all executed and oracle-checked, `external/my-lisp` submodule in place.

Stage 1 (OPEN): named functions on the asm core — `Ir::Def` is unconditionally rejected by `x86_freestanding.rs`; `CML-X86-DEF-BOUNDED-SELF-TAIL-RECURSIVE-FUNCTION` scopes the smallest real next gate (one bounded self-tail-recursive function, `env-lookup`'s shape).

Stage 2: close the rest of `meta-eval.my`'s call graph gate by gate, each with an executed, oracle-checked witness.

Stage 3: self-hosting closure — `meta-eval.my` runs entirely on the asm core with no Rust evaluator at runtime.

Stage 4: primitives (`wsm_cons` etc.) — Rust or hand-written asm, decided per the performance-substrate discipline (measure first, never as a Rust escape hatch), not by default.

Stage 5: `fpga-lisp`'s `Tag::True` — audit complete, minimal fix identified (`SYM_T=79` already exists), the RTL change itself needs its own explicit authorization since it touches already board-flashed logic.

Stage 6 (after 0–5): the reasoning layer — `lib/epistemic.my` already exists, already tested, not yet wired into `cond`/`is_truthy`; the real next capability gap is connecting it without breaking `() = ()` neutrality, built on the live self-hosted nucleus, not on Rust.

Stage 7 (long-horizon, named not started): assembler-in-Lisp beyond the nucleus — `fpga-lisp/assembler.my` already proves the path is real; boot/interrupt DSL work comes after the nucleus is alive, not before.

## Cross-cutting rule for every stage

Before any new machine tag/primitive/representation: does it represent an existing WSM distinction, or did we just invent semantics in the machine? Before any reasoning decision: does it preserve `() = ()` neutrality, or silently turn "no evidence" into "proven false"? Rust stays oracle until a capability clears its full bar: WSM implementation exists → independent fixtures pass → Rust↔WSM parity proven → CML admission passes → real target execution proven.
