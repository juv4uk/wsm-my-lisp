/* nucleus-win64.s -- Win64 ABI port of nucleus.s's 5 core primitives.
 *
 * Same primitive logic and word encoding as asm/nucleus.s (see that file's
 * header for the full rationale) -- this file changes ONLY the calling
 * convention, for the my-lisp-cyberpunk DLL-embedding effort (owner go-ahead
 * 2026-09-10, coordinated with my-lisp/cml/cyberpunk sessions). nucleus.s
 * itself is untouched: it remains the SysV/Linux parity-tested source of
 * truth; this is a parallel, additive file, not a replacement.
 *
 * Calling convention differences from SysV AMD64 (nucleus.s) to Microsoft
 * x64 (this file), per the Windows x64 ABI:
 *   - Integer args 1-4: SysV rdi,rsi,rdx,rcx  -->  Win64 rcx,rdx,r8,r9
 *   - Caller must reserve 32 bytes of "shadow space" on the stack before
 *     any `call`, even though these leaf functions take no more than 4
 *     args and callees here don't spill into it themselves. Not needed
 *     internally by cons/car/cdr/eq/atom (they call nothing except the OOM
 *     path), so omitted from their own bodies -- the DLL wrapper that calls
 *     *into* these functions is responsible for shadow space at its call
 *     sites, per the ABI.
 *   - Callee-saved registers: SysV saves rbx/rbp/r12-r15; Win64 additionally
 *     saves rsi/rdi/xmm6-15. Not relevant here: none of these functions
 *     touch rsi/rdi/xmm as scratch, so nothing needs saving/restoring.
 *
 * wsm_fail (OOM path) cannot reuse nucleus.s's raw Linux `syscall`
 * (SYS_write/SYS_exit): Windows has no stable raw syscall ABI equivalent
 * for user code to call directly. Instead this delegates to an externally
 * linked `wsm_fail_win64` symbol (Win64 ABI, args code/a/b in rcx/rdx/r8),
 * which dll/src/lib.rs provides -- it reports to stderr and exits via
 * normal Rust std calls.
 *
 * No `.type`/`.size` directives here (unlike nucleus.s): those are ELF
 * assembler directives with no COFF/PE equivalent, and LLVM's integrated
 * assembler (used for the windows-msvc target) rejects them outright.
 *
 * TAG_TRUE (0x1FFFFFFFFFFFFFFF sentinel `t` encoding) and the arena are
 * copied unchanged from nucleus.s -- see that file for why.
 */

    .text

    .equ TAG_CONS,   0
    .equ TAG_NIL,    1
    .equ TAG_SYMBOL, 4
    .equ TAG_BOXED,  7
    .equ TAG_MASK,   7

    .equ BOXED_KIND_PREDICATE_BIT, 5
    .equ PREDICATE_BIT_BITS,       1
    .equ PREDICATE_ENTRY_BYTES,    2
    .equ PREDICATE_BIT0_HANDLE,  257
    .equ PREDICATE_BIT1_HANDLE,  258
    .equ PREDICATE_BIT0_WORD, (PREDICATE_BIT0_HANDLE << 3) | TAG_BOXED
    .equ PREDICATE_BIT1_WORD, (PREDICATE_BIT1_HANDLE << 3) | TAG_BOXED

    /* wsm_os_target::ErrorCode -- mechanical projection, same values as
     * nucleus.s's own ERR_* constants (kept in sync by hand; both files
     * are mirrors of the same primitive set for different calling
     * conventions, not independent specifications). */
    .equ ERR_OUT_OF_MEMORY, 1
    .equ ERR_TYPE,          2
    .equ ERR_ABI_VIOLATION, 4

    .equ SYM_T_ID,   0x1FFFFFFFFFFFFFFF   /* wsm_os_target::SYMBOL_ID_MAX */
    .equ SYM_T_WORD, (SYM_T_ID << 3) | TAG_SYMBOL   /* wsm_os_target::encode_symbol(SYM_T_ID) */

/* wsm_cons(context: *mut RuntimeContext [ignored, rcx], car: Word [rdx], cdr: Word [r8]) -> Word */
    .globl wsm_cons
wsm_cons:
    movq    wsm_arena_next(%rip), %rax
    leaq    16(%rax), %r9
    cmpq    wsm_arena_end(%rip), %r9
    ja      wsm_cons_oom
    movq    %rdx, 0(%rax)          /* car */
    movq    %r8,  8(%rax)          /* cdr */
    movq    %r9, wsm_arena_next(%rip)
    ret                             /* %rax already holds the tagged (tag=0) pointer */
wsm_cons_oom:
    /* 40, not 32: Win64 ABI requires RSP % 16 == 0 immediately before a
     * `call`. On entry to wsm_cons (i.e. right after ITS OWN caller's
     * `call`), RSP % 16 == 8 (a `call` pushes an 8-byte return address
     * onto a 16-aligned stack) -- so an even subtraction like 32 leaves
     * RSP % 16 == 8 at our own `call` below, not 0. subq $40 (32 bytes of
     * mandatory shadow space + 8 bytes of padding) restores 16-alignment.
     * CONFIRMED BY ACTUALLY CRASHING: an earlier `subq $32` version of
     * this function made dll/tests/oom_path.rs's subprocess exit via
     * STATUS_ACCESS_VIOLATION (0xC0000005) instead of wsm_fail_win64's
     * intended exit(97) -- eprintln!'s formatting path apparently uses an
     * SSE instruction that faults on a misaligned stack. Not a hypothetical
     * concern flagged in a comment; a real, reproduced, then-fixed bug. */
    subq    $40, %rsp
    movl    $ERR_OUT_OF_MEMORY, %ecx /* arg1: code */
    xorl    %edx, %edx               /* arg2: a = 0 */
    xorl    %r8d, %r8d               /* arg3: b = 0 */
    call    wsm_fail_win64
    addq    $40, %rsp
    ret                              /* unreached if wsm_fail_win64 diverges as documented */

/* wsm_car(context [rcx, ignored], pair: Word [rdx]) -> Word
 *
 * Same fail-closed reasoning as nucleus.s's wsm_car: Tag::Cons is
 * zero, so any word with a nonzero TAG_MASK bit is not a pair and
 * must not be dereferenced. my-lisp's oracle requires (car 5) and
 * (car (quote ())) to raise a Type error. */
    .globl wsm_car
wsm_car:
    testq   $TAG_MASK, %rdx
    jnz     wsm_car_type_error
    movq    %rdx, %rax
    movq    0(%rax), %rax
    ret
wsm_car_type_error:
    subq    $40, %rsp               /* Win64 16-alignment, see wsm_cons_oom above */
    movl    $ERR_TYPE, %ecx           /* arg1: code */
    xorl    %edx, %edx
    xorl    %r8d, %r8d
    call    wsm_fail_win64
    addq    $40, %rsp
    ret

/* wsm_cdr(context [rcx, ignored], pair: Word [rdx]) -> Word -- mirrors
 * wsm_car's reasoning above for the cdr offset. */
    .globl wsm_cdr
wsm_cdr:
    testq   $TAG_MASK, %rdx
    jnz     wsm_cdr_type_error
    movq    %rdx, %rax
    movq    8(%rax), %rax
    ret
wsm_cdr_type_error:
    subq    $40, %rsp
    movl    $ERR_TYPE, %ecx           /* arg1: code */
    xorl    %edx, %edx
    xorl    %r8d, %r8d
    call    wsm_fail_win64
    addq    $40, %rsp
    ret

/* wsm_eq(context [rcx, ignored], left: Word [rdx], right: Word [r8]) -> Word */
    .globl wsm_eq
wsm_eq:
    movl    $TAG_NIL, %eax
    cmpq    %r8, %rdx
    jne     1f
    movabsq $SYM_T_WORD, %rax
1:  ret

/* wsm_atom(context [rcx, ignored], value: Word [rdx]) -> Word */
    .globl wsm_atom
wsm_atom:
    movabsq $SYM_T_WORD, %rax
    testq   $TAG_MASK, %rdx
    jnz     1f
    movl    $TAG_NIL, %eax          /* low 3 bits all zero => Tag::Cons => not an atom */
1:  ret

/* Current exact-domain predicate mechanisms.
 *
 * Compatibility wsm_atom/wsm_eq above keep historical T/NIL behavior.
 * Current SENS admission selects these D1 entrypoints instead.
 */

/* wsm_atom_predicate_bit(context [rcx, ignored], value: Word [rdx]) -> PredicateBit Word */
    .globl wsm_atom_predicate_bit
wsm_atom_predicate_bit:
    movl    $PREDICATE_BIT1_WORD, %eax
    movq    %rdx, %r9
    andq    $TAG_MASK, %r9
    cmpq    $TAG_CONS, %r9
    jne     .Latom_d1_done_win64
    movl    $PREDICATE_BIT0_WORD, %eax
.Latom_d1_done_win64:
    ret

/* wsm_eq_predicate_bit(context [rcx, ignored], left [rdx], right [r8])
 * -> PredicateBit Word for atom/atom; Type failure for non-atom input.
 */
    .globl wsm_eq_predicate_bit
wsm_eq_predicate_bit:
    movq    %rdx, %rax
    andq    $TAG_MASK, %rax
    cmpq    $TAG_CONS, %rax
    je      .Leq_d1_type_win64
    movq    %r8, %rax
    andq    $TAG_MASK, %rax
    cmpq    $TAG_CONS, %rax
    je      .Leq_d1_type_win64

    movl    $PREDICATE_BIT0_WORD, %eax
    cmpq    %r8, %rdx
    jne     .Leq_d1_done_win64
    movl    $PREDICATE_BIT1_WORD, %eax
.Leq_d1_done_win64:
    ret

.Leq_d1_type_win64:
    subq    $40, %rsp
    movl    $ERR_TYPE, %ecx
    call    wsm_fail_win64
    addq    $40, %rsp
    ret

/* Draft target-contract #33 PredicateBit carrier. The Win64 target mirrors
 * SysV's representation-only bit0/bit1 singleton ABI; no predicate semantics
 * are assigned in this file. */
    .globl wsm_predicate_bit_0
wsm_predicate_bit_0:
    movl    $PREDICATE_BIT0_WORD, %eax
    ret

    .globl wsm_predicate_bit_1
wsm_predicate_bit_1:
    movl    $PREDICATE_BIT1_WORD, %eax
    ret

    .globl wsm_predicate_bit_bits
wsm_predicate_bit_bits:
    movq    %rdx, %rax
    movq    %rax, %r9
    andq    $TAG_MASK, %r9
    cmpq    $TAG_BOXED, %r9
    jne     .Lpredicate_bits_type_win64

    shrq    $3, %rax
    cmpq    $PREDICATE_BIT0_HANDLE, %rax
    je      .Lpredicate_bits_0_win64
    cmpq    $PREDICATE_BIT1_HANDLE, %rax
    je      .Lpredicate_bits_1_win64
    jmp     .Lpredicate_bits_abi_win64

.Lpredicate_bits_0_win64:
    leaq    wsm_predicate_bit_table(%rip), %r9
    cmpb    $BOXED_KIND_PREDICATE_BIT, 0(%r9)
    jne     .Lpredicate_bits_abi_win64
    cmpb    $0, 1(%r9)
    jne     .Lpredicate_bits_abi_win64
    xorl    %eax, %eax
    ret

.Lpredicate_bits_1_win64:
    leaq    wsm_predicate_bit_table+PREDICATE_ENTRY_BYTES(%rip), %r9
    cmpb    $BOXED_KIND_PREDICATE_BIT, 0(%r9)
    jne     .Lpredicate_bits_abi_win64
    cmpb    $1, 1(%r9)
    jne     .Lpredicate_bits_abi_win64
    movl    $1, %eax
    ret

.Lpredicate_bits_type_win64:
    subq    $40, %rsp
    movl    $ERR_TYPE, %ecx
    xorl    %r8d, %r8d
    call    wsm_fail_win64
    addq    $40, %rsp
    ret

.Lpredicate_bits_abi_win64:
    subq    $40, %rsp
    movl    $ERR_ABI_VIOLATION, %ecx
    xorl    %r8d, %r8d
    call    wsm_fail_win64
    addq    $40, %rsp
    ret

/* wsm_arena_reset(context [rcx, ignored]) -> void -- rewinds the bump
 * pointer back to the arena's start, discarding every cons cell
 * allocated since the last reset (or since load, if never reset). NOT
 * a general-purpose free: this invalidates every Word still reachable
 * only through those discarded cells. Added 2026-09-10 (owner go-ahead)
 * after dll/'s own bench.rs found the fixed 256-cell arena, never
 * reset, hard-crashes the whole process after exactly 128 calls to
 * wsm_eval_string("(quote a)") -- see dll/README.md's "Performance"
 * section and dll/src/ffi.rs's wsm_eval_string for the actual call
 * site and its documented safety preconditions (this is deliberately
 * NOT safe to call while any previously-returned cons-containing Word
 * is still expected to be valid). */
    .globl wsm_arena_reset
wsm_arena_reset:
    leaq    wsm_arena(%rip), %rax
    movq    %rax, wsm_arena_next(%rip)
    ret

    /* wsm_fail_win64(code: u32 [ecx], a: Word [rdx], b: Word [r8]) -> ! --
     * NOT defined in this file. Provided by the Rust DLL wrapper crate
     * (extern "C" fn wsm_fail_win64, Win64 ABI, must not return). See
     * this file's header comment for why nucleus.s's raw-syscall wsm_fail
     * cannot be reused as-is on Windows. */

    .section .bss
    .align 16
    /* 4096 bytes = 256 cons cells, same bound as nucleus.s -- see that
     * file's header for why this stays small and fixed. */
    .equ ARENA_BYTES, 4096
wsm_arena:
    .zero ARENA_BYTES

    .section .data
    .align 2
wsm_predicate_bit_table:
    .byte BOXED_KIND_PREDICATE_BIT, 0
    .byte BOXED_KIND_PREDICATE_BIT, 1

    .align 8
wsm_arena_next:
    .quad wsm_arena
wsm_arena_end:
    .quad wsm_arena + ARENA_BYTES

    /* No .note.GNU-stack section here: that is an ELF/Linux-only marker
     * (asm/nucleus.s and asm/entry-*.s all carry it) with no PE/COFF
     * equivalent -- omitted deliberately for the Windows target, not an
     * oversight. */
