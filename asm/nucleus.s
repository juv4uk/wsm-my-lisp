/* nucleus.s -- hand-written x86_64 GNU-asm core primitives for WSM.
 *
 * Not generated, not ported from Rust: this is the "ядро на асамблері під
 * моє залізо" the owner asked for. It implements the same extern "C" ABI
 * that wsm-os-runtime's Rust wsm_cons/wsm_car/wsm_cdr/wsm_eq/wsm_atom
 * already provide (see wsm-os/crates/wsm-os-runtime/src/lib.rs:409-449),
 * calling convention SysV AMD64 (RUNTIME_IMPORTS / CALLING_CONVENTION in
 * wsm-os/crates/wsm-os-target/src/lib.rs), so CML-generated wsm_entry code
 * can call these instead without any change to the generated assembly.
 *
 * Reason for hand-writing this is architectural, not performance: remove
 * Rust from primitive *semantics* at execution time, per the owner's
 * 2026-09-02 bootstrap-boundary strategy (WSM defines WSM, not Rust). No
 * cycle counts are claimed here -- this has not been profiled and makes no
 * speed claim.
 *
 * Deliberately bounded, not a general allocator: a small fixed-size bump
 * arena (see ARENA_BYTES below), no GC, no growth, matching
 * wsm-os/docs/OWNER-HARDWARE-PROFILE.md's own M1/M2 guidance ("should use
 * a deliberately small fixed heap and explicitly test OOM") and this
 * task's own boundary (5 primitives only -- cons/car/cdr/eq/atom -- no
 * closures/GC/heap-growth in this pass). Scalar x86_64 only: no AVX2/BMI2
 * instructions, matching the owner's own documented hardware baseline
 * (Intel Core i5-6400 profile explicitly defers those extensions).
 *
 * Word encoding (wsm-os-target::Tag, WORD_BITS=64, TAG_BITS=3,
 * PAYLOAD_BITS=61): Cons=0, Nil=1, True=2, Fixnum=3, Symbol=4, Closure=5,
 * Capability=6. Cons cells are 16 bytes, 16-byte aligned, car at offset 0,
 * cdr at offset 8 (CONS_ALIGNMENT/CONS_BYTES/CONS_CAR_OFFSET/
 * CONS_CDR_OFFSET). Because Tag::Cons is 0 and every allocation here is
 * 16-byte aligned, a cons pointer's low 3 bits are always already zero --
 * the raw pointer IS the tagged word, no OR/mask needed to tag it, only a
 * mask to strip a caller's tag bits back off before dereferencing (defensive
 * only; every cons word this code itself produces already has zero low
 * bits).
 *
 * context (%rdi) is accepted, per the ABI signature every wsm_* function
 * must have, and deliberately ignored: this nucleus does not reuse Rust's
 * RuntimeContext layout (that struct is a private Rust implementation
 * detail, not a stable cross-language ABI -- only the wsm-os-target word
 * encoding and the wsm_* function ABI are the actual contract). This
 * nucleus owns its own static arena instead. This is why it is a parallel,
 * additive path in a new repository, not a drop-in replacement for
 * wsm-os-runtime: swapping it into wsm-os-hosted/-kernel as-is would lose
 * whatever those callers still expect from a real RuntimeContext (closure
 * heap, condition record) that this nucleus does not implement.
 */

    .text

    .equ TAG_CONS,   0
    .equ TAG_NIL,    1
    .equ TAG_TRUE,   2   /* superseded below: wsm_eq/wsm_atom no longer emit this.
                          * Left declared, matching wsm-os-target::Tag::True itself
                          * (not removed there either -- a separate, bigger question).
                          */
    .equ TAG_SYMBOL, 4
    .equ TAG_MASK,   7

    /* Canonical `t` as an ordinary Symbol, not a manufactured Tag::True
     * primitive -- 2026-09-02 owner directive: "() не визначаємо... t має
     * пройти тим самим шляхом, що й будь-який інший Symbol." Applied here
     * the same way as fpga-lisp's SYM_T=79 fix, with one honest difference
     * this ABI has and fpga-lisp's frozen bootstrap table does not:
     * wsm-os-target's Symbol ids are documented as "image-local-interned"
     * (cml/src/x86_freestanding.rs assigns each compiled program's own
     * quoted symbols sequential ids from a sorted BTreeSet, starting at 1)
     * -- there is no single canonical id for `t` shared across programs to
     * reuse, unlike fpga-lisp's fixed global symbol table. Reserving
     * SYMBOL_ID_MAX (wsm-os-target's own documented maximum valid symbol
     * id, 2^61-1) as a sentinel for `t` makes a real collision with a
     * per-program-interned id practically unreachable (a program would
     * need to intern ~2^61 distinct symbols), but it is NOT a proof of
     * uniqueness the way a frozen global table would be. A caller that
     * compares this nucleus's eq/atom "true" result against a *literal*
     * quoted `t` appearing in that same compiled program's own source via
     * a second `eq` call would get that program's own (different, small)
     * interned id for `t` on the other side -- the two encodings would not
     * compare equal. That composition is not exercised by any of this
     * repo's harness/ examples today; closing it for real needs either a
     * shared reserved-id convention baked into cml's symbol assignment
     * itself, or wsm_eq/wsm_atom consulting the caller's own symbol table
     * (which these free functions have no access to). Flagging this
     * honestly rather than presenting the sentinel as a complete fix. */
    .equ SYM_T_ID,   0x1FFFFFFFFFFFFFFF   /* wsm_os_target::SYMBOL_ID_MAX */
    .equ SYM_T_WORD, (SYM_T_ID << 3) | TAG_SYMBOL   /* wsm_os_target::encode_symbol(SYM_T_ID) */

/* wsm_cons(context: *mut RuntimeContext [ignored], car: Word, cdr: Word) -> Word */
    .globl wsm_cons
    .type wsm_cons, @function
wsm_cons:
    movq    wsm_arena_next(%rip), %rax
    leaq    16(%rax), %rcx
    cmpq    wsm_arena_end(%rip), %rcx
    ja      wsm_cons_oom
    movq    %rsi, 0(%rax)          /* car */
    movq    %rdx, 8(%rax)          /* cdr */
    movq    %rcx, wsm_arena_next(%rip)
    ret                             /* %rax already holds the tagged (tag=0) pointer */
wsm_cons_oom:
    movl    $1, %esi                /* ErrorCode::OutOfMemory = 1 */
    xorl    %edx, %edx
    xorl    %ecx, %ecx
    jmp     wsm_fail
    .size wsm_cons, . - wsm_cons

/* wsm_car(context, pair: Word) -> Word */
    .globl wsm_car
    .type wsm_car, @function
wsm_car:
    movq    %rsi, %rax
    andq    $-8, %rax               /* strip any stray tag bits defensively */
    movq    0(%rax), %rax
    ret
    .size wsm_car, . - wsm_car

/* wsm_cdr(context, pair: Word) -> Word */
    .globl wsm_cdr
    .type wsm_cdr, @function
wsm_cdr:
    movq    %rsi, %rax
    andq    $-8, %rax
    movq    8(%rax), %rax
    ret
    .size wsm_cdr, . - wsm_cdr

/* wsm_eq(context, left: Word, right: Word) -> Word */
    .globl wsm_eq
    .type wsm_eq, @function
wsm_eq:
    movl    $TAG_NIL, %eax
    cmpq    %rdx, %rsi
    jne     1f
    movabsq $SYM_T_WORD, %rax
1:  ret
    .size wsm_eq, . - wsm_eq

/* wsm_atom(context, value: Word) -> Word */
    .globl wsm_atom
    .type wsm_atom, @function
wsm_atom:
    movabsq $SYM_T_WORD, %rax
    testq   $TAG_MASK, %rsi
    jnz     1f
    movl    $TAG_NIL, %eax          /* low 3 bits all zero => Tag::Cons => not an atom */
1:  ret
    .size wsm_atom, . - wsm_atom

/* wsm_fail(context, code: u32, a: Word, b: Word) -> ! -- unrecoverable
 * condition (OOM here). No RuntimeContext::condition record exists in this
 * nucleus (out of this pass's bounded scope), so this reports on stderr via
 * a raw Linux write(2) syscall and exits via raw exit(2) -- no libc, matching
 * this file's freestanding style, acceptable because this is a hosted
 * (Linux process) witness harness, not a bare-metal kernel entry. */
    .globl wsm_fail
    .type wsm_fail, @function
wsm_fail:
    movq    $1, %rax                /* SYS_write */
    movq    $2, %rdi                /* fd = stderr */
    leaq    wsm_fail_msg(%rip), %rsi
    movq    $wsm_fail_msg_len, %rdx
    syscall
    movq    $60, %rax               /* SYS_exit */
    movq    $97, %rdi               /* exit code 97: distinguishable, arbitrary */
    syscall
    .size wsm_fail, . - wsm_fail

    .section .rodata
wsm_fail_msg:
    .ascii "wsm-my-lisp asm nucleus: unrecoverable condition (arena exhausted)\n"
    .equ wsm_fail_msg_len, . - wsm_fail_msg

    .section .bss
    .align 16
    /* 4096 bytes = 256 cons cells. Deliberately small and fixed: this pass
     * proves the primitive ABI works, not a general-purpose heap. */
    .equ ARENA_BYTES, 4096
wsm_arena:
    .zero ARENA_BYTES

    /* .data, not .bss: these two cells hold an initialized address (a
     * relocation/fixup against wsm_arena), which a zero-initialized .bss
     * section cannot carry. */
    .section .data
    .align 8
wsm_arena_next:
    .quad wsm_arena
wsm_arena_end:
    .quad wsm_arena + ARENA_BYTES

    .section .note.GNU-stack,"",@progbits
