/* nucleus.s -- hand-written x86_64 GNU-asm core primitives for WSM.
 *
 * Not generated, not ported from Rust: this is the small execution nucleus
 * used by the self-hosting witnesses. It implements the same extern "C" ABI
 * consumed by CML-generated wsm_entry code, using the SysV AMD64 calling
 * convention.
 *
 * Reason for hand-writing this is architectural, not performance: Rust is
 * absent from primitive execution semantics. No cycle-count or speed claim is
 * made here. The cons arena is deliberately bounded (4096 bytes / 256 cells),
 * with no GC or growth. The closure arena added for Stage2 is likewise bounded
 * and exists only to realize the already-ratified target ABI closure descriptor.
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
 * tagged cons word. Closure descriptors are also 16-byte aligned; their tagged
 * word is pointer|5, definition_id is at offset 0, environment_ref at offset 8.
 *
 * context (%rdi) is accepted by the ABI and deliberately ignored: this
 * nucleus owns its own static arenas and does not depend on Rust's private
 * RuntimeContext layout.
 */

    .text

    .equ TAG_CONS,    0
    .equ TAG_NIL,     1
    .equ TAG_SYMBOL,  4
    .equ TAG_CLOSURE, 5
    .equ TAG_MASK,    7

    /* Механічна проєкція wsm_os_target::ClosureDescriptor. Значення нижче
     * перевіряються semantic_authority.rs проти pinned target contract. */
    .equ CLOSURE_ALIGNMENT,             16
    .equ CLOSURE_BYTES,                 16
    .equ CLOSURE_DEFINITION_ID_OFFSET,   0
    .equ CLOSURE_ENVIRONMENT_REF_OFFSET, 8

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

/* Stage2 closure ABI.
 *
 * Це не нова Lisp-примітива. Closure=5, layout дескриптора і три імпорти
 * wsm_closure_* уже ратифіковані wsm-target-contract; тут лише мінімальна
 * SysV-механіка, потрібна CML для матеріалізації справжньої identity closure.
 * Окремий bump-арена робить дві однакові closure-конструкції різними Word,
 * тому `eq` природно перевіряє identity, а не структуру -- саме це потрібно
 * provenance-токенам поточного meta-eval.my.
 */

/* wsm_closure_new(context [ignored], definition_id: u32, environment_ref: Word) -> Word */
    .globl wsm_closure_new
    .type wsm_closure_new, @function
wsm_closure_new:
    movq    wsm_closure_arena_next(%rip), %rax
    leaq    CLOSURE_BYTES(%rax), %rcx
    cmpq    wsm_closure_arena_end(%rip), %rcx
    ja      wsm_closure_new_oom
    movl    %esi, CLOSURE_DEFINITION_ID_OFFSET(%rax)
    movl    $0, 4(%rax)             /* deterministic ABI padding */
    movq    %rdx, CLOSURE_ENVIRONMENT_REF_OFFSET(%rax)
    movq    %rcx, wsm_closure_arena_next(%rip)
    orq     $TAG_CLOSURE, %rax
    ret
wsm_closure_new_oom:
    movl    $1, %esi                /* ErrorCode::OutOfMemory = 1 */
    xorl    %edx, %edx
    xorl    %ecx, %ecx
    jmp     wsm_fail
    .size wsm_closure_new, . - wsm_closure_new

/* wsm_closure_definition(context, closure: Word) -> raw u32 definition_id in eax */
    .globl wsm_closure_definition
    .type wsm_closure_definition, @function
wsm_closure_definition:
    movq    %rsi, %rdx              /* keep original word for failure evidence */
    movq    %rsi, %rax
    movq    %rax, %rcx
    andq    $TAG_MASK, %rcx
    cmpq    $TAG_CLOSURE, %rcx
    jne     .Lclosure_definition_type
    andq    $-8, %rax
    testq   $(CLOSURE_ALIGNMENT - 1), %rax
    jne     .Lclosure_definition_abi
    leaq    wsm_closure_arena(%rip), %rcx
    cmpq    %rcx, %rax
    jb      .Lclosure_definition_abi
    movq    wsm_closure_arena_next(%rip), %rcx
    cmpq    %rcx, %rax
    jae     .Lclosure_definition_abi
    movl    CLOSURE_DEFINITION_ID_OFFSET(%rax), %eax
    ret
.Lclosure_definition_type:
    movl    $2, %esi                /* ErrorCode::Type = 2 */
    xorl    %ecx, %ecx
    jmp     wsm_fail
.Lclosure_definition_abi:
    movl    $4, %esi                /* ErrorCode::AbiViolation = 4 */
    xorl    %ecx, %ecx
    jmp     wsm_fail
    .size wsm_closure_definition, . - wsm_closure_definition

/* wsm_closure_environment(context, closure: Word) -> owned WSM environment_ref */
    .globl wsm_closure_environment
    .type wsm_closure_environment, @function
wsm_closure_environment:
    movq    %rsi, %rdx              /* keep original word for failure evidence */
    movq    %rsi, %rax
    movq    %rax, %rcx
    andq    $TAG_MASK, %rcx
    cmpq    $TAG_CLOSURE, %rcx
    jne     .Lclosure_environment_type
    andq    $-8, %rax
    testq   $(CLOSURE_ALIGNMENT - 1), %rax
    jne     .Lclosure_environment_abi
    leaq    wsm_closure_arena(%rip), %rcx
    cmpq    %rcx, %rax
    jb      .Lclosure_environment_abi
    movq    wsm_closure_arena_next(%rip), %rcx
    cmpq    %rcx, %rax
    jae     .Lclosure_environment_abi
    movq    CLOSURE_ENVIRONMENT_REF_OFFSET(%rax), %rax
    ret
.Lclosure_environment_type:
    movl    $2, %esi                /* ErrorCode::Type = 2 */
    xorl    %ecx, %ecx
    jmp     wsm_fail
.Lclosure_environment_abi:
    movl    $4, %esi                /* ErrorCode::AbiViolation = 4 */
    xorl    %ecx, %ecx
    jmp     wsm_fail
    .size wsm_closure_environment, . - wsm_closure_environment

/* wsm_fail(context, code: u32, a: Word, b: Word) -> ! -- unrecoverable
 * condition (OOM/type/ABI violation here). No RuntimeContext::condition record
 * exists in this bounded nucleus, so this reports on stderr via raw Linux
 * write(2) and exits via raw exit(2) -- no libc, matching this file's
 * freestanding style, acceptable because this is a hosted (Linux process)
 * witness harness, not a bare-metal kernel entry. */
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
    .ascii "wsm-my-lisp asm nucleus: unrecoverable condition\n"
    .equ wsm_fail_msg_len, . - wsm_fail_msg

    .section .bss
    .align 16
    /* 4096 bytes = 256 cons cells. Deliberately small and fixed: this pass
     * proves the primitive ABI works, not a general-purpose heap. */
    .equ ARENA_BYTES, 4096
wsm_arena:
    .zero ARENA_BYTES

    .align CLOSURE_ALIGNMENT
    /* 4096 bytes = 256 closure descriptors. Stage2 needs identity-bearing
     * closure values, not a general GC heap; bounded growth stays explicit. */
    .equ CLOSURE_ARENA_BYTES, 4096
wsm_closure_arena:
    .zero CLOSURE_ARENA_BYTES

    /* .data, not .bss: these cells hold initialized addresses (relocations),
     * which a zero-initialized .bss section cannot carry. */
    .section .data
    .align 8
wsm_arena_next:
    .quad wsm_arena
wsm_arena_end:
    .quad wsm_arena + ARENA_BYTES
wsm_closure_arena_next:
    .quad wsm_closure_arena
wsm_closure_arena_end:
    .quad wsm_closure_arena + CLOSURE_ARENA_BYTES

    .section .note.GNU-stack,"",@progbits
