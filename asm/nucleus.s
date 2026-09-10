/* nucleus.s -- hand-written x86_64 GNU-asm core primitives for WSM.
 *
 * Not generated, not ported from Rust: this is the small execution nucleus
 * used by the self-hosting witnesses. It implements the same extern "C" ABI
 * consumed by CML-generated wsm_entry code, using the SysV AMD64 calling
 * convention.
 *
 * Reason for hand-writing this is architectural, not performance: Rust is
 * absent from primitive execution semantics. No cycle-count or speed claim is
 * made here. The arena is deliberately bounded (4096 bytes / 256 cons cells),
 * with no GC or growth.
 *
 * Target-word representation is MECHANISM, not language authority. The
 * authoritative machine representation comes from the pinned
 * wsm-target-contract. Its current ABI still contains a historical
 * Tag::True=2 slot, but canonical WSM truth is the ordinary Symbol("t") word
 * CANONICAL_T = encode_symbol(SYMBOL_ID_MAX). This nucleus therefore does NOT
 * declare a local TAG_TRUE and must never emit that historical tag. `wsm_eq`
 * and `wsm_atom` return SYM_T_WORD on their positive branch and TAG_NIL on the
 * negative branch. harness/tests/semantic_authority.rs checks these assembly
 * projections against the pinned target contract and includes a deliberately
 * bad manufactured-truth fixture that must fail closed.
 *
 * Word encoding used by the admitted slice: Cons=0, Nil=1, Fixnum=3,
 * Symbol=4, Closure=5, Capability=6; TAG_BITS=3. Cons cells are 16 bytes,
 * 16-byte aligned, car at offset 0 and cdr at offset 8. Because Tag::Cons is
 * zero and every allocation here is 16-byte aligned, the raw pointer is the
 * tagged cons word.
 *
 * context (%rdi) is accepted by the ABI and deliberately ignored: this
 * nucleus owns its own static arena and does not depend on Rust's private
 * RuntimeContext layout.
 */

    .text

    .equ TAG_CONS,   0
    .equ TAG_NIL,    1
    .equ TAG_SYMBOL, 4
    .equ TAG_MASK,   7

    /* Mechanical projection of wsm_os_target::CANONICAL_T. The Rust harness
     * verifies these values against the pinned target contract, so changing
     * the contract without updating this projection fails CI rather than
     * silently inventing target semantics here. */
    .equ SYM_T_ID,   0x1FFFFFFFFFFFFFFF   /* wsm_os_target::SYMBOL_ID_MAX */
    .equ SYM_T_WORD, (SYM_T_ID << 3) | TAG_SYMBOL

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
