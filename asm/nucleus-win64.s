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
    .equ TAG_MASK,   7

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
    movl    $1, %ecx                /* ErrorCode::OutOfMemory = 1 (arg1: code) */
    xorl    %edx, %edx               /* arg2: a = 0 */
    xorl    %r8d, %r8d               /* arg3: b = 0 */
    call    wsm_fail_win64
    addq    $40, %rsp
    ret                              /* unreached if wsm_fail_win64 diverges as documented */

/* wsm_car(context [rcx, ignored], pair: Word [rdx]) -> Word */
    .globl wsm_car
wsm_car:
    movq    %rdx, %rax
    andq    $-8, %rax               /* strip any stray tag bits defensively */
    movq    0(%rax), %rax
    ret

/* wsm_cdr(context [rcx, ignored], pair: Word [rdx]) -> Word */
    .globl wsm_cdr
wsm_cdr:
    movq    %rdx, %rax
    andq    $-8, %rax
    movq    8(%rax), %rax
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
    .align 8
wsm_arena_next:
    .quad wsm_arena
wsm_arena_end:
    .quad wsm_arena + ARENA_BYTES

    /* No .note.GNU-stack section here: that is an ELF/Linux-only marker
     * (asm/nucleus.s and asm/entry-*.s all carry it) with no PE/COFF
     * equivalent -- omitted deliberately for the Windows target, not an
     * oversight. */
