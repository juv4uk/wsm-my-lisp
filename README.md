# wsm-my-lisp

Self-hosted growth target for WSM: Lisp + the target machine's own
assembler, growing toward the owner's own x86_64 hardware (Intel Core
i5-6400, see `wsm-os/docs/OWNER-HARDWARE-PROFILE.md`), capability by
capability.

`my-lisp` (the Rust implementation) remains the canonical semantic oracle
and reference implementation for every capability until that capability
specifically proves parity here: WSM implementation exists, independent
semantic fixtures pass, Rust↔WSM parity passes, CML admission passes,
native target execution passes (QEMU/FPGA where in scope). This repo does
not become authoritative over language semantics by existing — it earns
authority one proven capability at a time.

See `repo.my` for the full scope declaration.

## Самостійна ціль WSM

WSM повністю в Lisp + асемблер цільової машини, під власне x86_64-залізо
власника (Intel Core i5-6400). `my-lisp` (Rust) лишається канонічним
семантичним oracle, доки кожна capability окремо не доведе паритет тут:
є WSM-реалізація, незалежні semantic fixtures проходять, Rust↔WSM parity
проходить, CML admission проходить, реальне виконання на target проходить
(QEMU/FPGA де застосовно). Цей репозиторій не стає авторитетним над
мовною семантикою самим фактом існування — авторитет здобувається по
одній доведеній capability за раз.

Повна декларація scope — у `repo.my`.
