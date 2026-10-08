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
    .equ TAG_BOXED,   7
    .equ TAG_MASK,    7

    /* Mechanical projection of target-contract v7's exact SID8 boxed kind.
     * The nucleus canonicalizes one runtime handle per exact 8-bit value so
     * raw target-word equality remains exact bit identity, never a spelling
     * or numeric coercion. */
    .equ BOXED_KIND_SID8, 4
    .equ SID8_BITS,        8
    .equ SID8_MAX,       255
    .equ SID8_ENTRY_BYTES, 2
    .equ SID8_CAPACITY,  256

    /* Draft target-contract #33 / v8 candidate. This nucleus admits one
     * bootstrap runtime context, so its two process-global singleton slots are
     * exactly the two singleton objects for that sole admitted context. The
     * target layer transports bit 0/1 only; SENS owns predicate meaning. */
    .equ BOXED_KIND_PREDICATE_BIT, 5
    .equ PREDICATE_BIT_BITS,       1
    .equ PREDICATE_ENTRY_BYTES,    2
    .equ PREDICATE_BIT0_HANDLE,  257
    .equ PREDICATE_BIT1_HANDLE,  258
    .equ PREDICATE_BIT0_WORD, (PREDICATE_BIT0_HANDLE << 3) | TAG_BOXED
    .equ PREDICATE_BIT1_WORD, (PREDICATE_BIT1_HANDLE << 3) | TAG_BOXED

    /* wsm_os_target::ErrorCode -- mechanical projection, same pattern as
     * SYM_T_WORD above. Only the two variants this nucleus actually raises
     * are named here; harness/tests/semantic_authority.rs checks these
     * against the pinned target contract. */
    .equ ERR_OUT_OF_MEMORY,  1
    .equ ERR_TYPE,           2
    .equ ERR_ABI_VIOLATION,  4

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
    movl    $ERR_OUT_OF_MEMORY, %esi
    xorl    %edx, %edx
    xorl    %ecx, %ecx
    jmp     wsm_fail
    .size wsm_cons, . - wsm_cons

/* wsm_car(context, pair: Word) -> Word
 *
 * Tag::Cons is zero, so a real Cons word has all TAG_MASK bits clear.
 * Any other tag (Nil, Fixnum, Symbol, Closure, Capability) is not a
 * pair -- dereferencing its payload as a pointer would read arbitrary
 * memory (a small shifted integer for Fixnum, address 0 for Nil).
 * my-lisp's own oracle requires (car 5) and (car (quote ())) to raise
 * a Type error, not silently return garbage or segfault; this check
 * is what makes that true here too, using the same bounded-abort
 * mechanism wsm_cons_oom already established rather than inventing a
 * second failure convention. */
    .globl wsm_car
    .type wsm_car, @function
wsm_car:
    testq   $TAG_MASK, %rsi
    jnz     wsm_car_type_error
    movq    %rsi, %rax
    movq    0(%rax), %rax
    ret
wsm_car_type_error:
    movl    $ERR_TYPE, %esi
    xorl    %edx, %edx
    xorl    %ecx, %ecx
    jmp     wsm_fail
    .size wsm_car, . - wsm_car

/* wsm_cdr(context, pair: Word) -> Word -- see wsm_car's comment above,
 * identical reasoning, mirrored for the cdr offset. */
    .globl wsm_cdr
    .type wsm_cdr, @function
wsm_cdr:
    testq   $TAG_MASK, %rsi
    jnz     wsm_cdr_type_error
    movq    %rsi, %rax
    movq    8(%rax), %rax
    ret
wsm_cdr_type_error:
    movl    $ERR_TYPE, %esi
    xorl    %edx, %edx
    xorl    %ecx, %ecx
    jmp     wsm_fail
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

/* Current exact-domain predicate mechanisms.
 *
 * Compatibility entrypoints above keep historical T/NIL behavior. These two
 * entrypoints are selected only after current SENS admission.
 *
 * ATOM_D1 is total: atom -> PredicateBit(1), CONS -> PredicateBit(0).
 * EQ_D1 is atom-domain only: atom/atom -> PredicateBit(1/0); if either
 * operand is CONS, the current SENS law requires the named Type failure.
 */

/* wsm_atom_d1(context [ignored], value: Word) -> PredicateBit Word */
    .globl wsm_atom_d1
    .type wsm_atom_d1, @function
wsm_atom_d1:
    movl    $PREDICATE_BIT1_WORD, %eax
    testq   $TAG_MASK, %rsi
    jnz     1f
    movl    $PREDICATE_BIT0_WORD, %eax
1:  ret
    .size wsm_atom_d1, . - wsm_atom_d1

/* wsm_eq_d1(context [ignored], left: Word, right: Word)
 * -> PredicateBit Word for atom/atom; Type failure for non-atom input.
 */
    .globl wsm_eq_d1
    .type wsm_eq_d1, @function
wsm_eq_d1:
    movq    %rsi, %rax
    andq    $TAG_MASK, %rax
    cmpq    $TAG_CONS, %rax
    je      .Leq_d1_type
    movq    %rdx, %rax
    andq    $TAG_MASK, %rax
    cmpq    $TAG_CONS, %rax
    je      .Leq_d1_type

    movl    $PREDICATE_BIT0_WORD, %eax
    cmpq    %rdx, %rsi
    jne     .Leq_d1_done
    movl    $PREDICATE_BIT1_WORD, %eax
.Leq_d1_done:
    ret

.Leq_d1_type:
    movl    $ERR_TYPE, %esi
    xorl    %edx, %edx
    xorl    %ecx, %ecx
    jmp     wsm_fail
    .size wsm_eq_d1, . - wsm_eq_d1

/* Core1 S5 exact SID8 boxed transport.
 *
 * These two functions implement only the already-ratified target ABI
 * mechanism. They do not resolve names, parse text, or define Lisp meaning.
 * A SID8's target identity is its exact u8 payload. The bounded table has one
 * canonical slot per possible payload, so repeated construction of the same
 * bits returns the same Boxed word and wsm_eq remains exact-bit equality.
 */

/* wsm_sid8_new(context [ignored], bits: u64) -> Boxed Word */
    .globl wsm_sid8_new
    .type wsm_sid8_new, @function
wsm_sid8_new:
    cmpq    $SID8_MAX, %rsi
    ja      .Lsid8_new_abi

    leaq    wsm_sid8_table(%rip), %rcx
    leaq    (%rcx,%rsi,2), %rcx
    cmpb    $0, 0(%rcx)
    je      .Lsid8_new_init
    cmpb    $BOXED_KIND_SID8, 0(%rcx)
    jne     .Lsid8_new_abi
    cmpb    %sil, 1(%rcx)
    jne     .Lsid8_new_abi
    jmp     .Lsid8_new_encode

.Lsid8_new_init:
    movb    $BOXED_KIND_SID8, 0(%rcx)
    movb    %sil, 1(%rcx)

.Lsid8_new_encode:
    leaq    1(%rsi), %rax           /* runtime handle = table index + 1 */
    shlq    $3, %rax
    orq     $TAG_BOXED, %rax
    ret

.Lsid8_new_abi:
    movq    %rsi, %rdx              /* preserve rejected raw bits as evidence */
    movl    $ERR_ABI_VIOLATION, %esi
    xorl    %ecx, %ecx
    jmp     wsm_fail
    .size wsm_sid8_new, . - wsm_sid8_new

/* wsm_sid8_bits(context [ignored], value: Boxed Word) -> raw u8 in eax */
    .globl wsm_sid8_bits
    .type wsm_sid8_bits, @function
wsm_sid8_bits:
    movq    %rsi, %rdx              /* preserve original word for failure evidence */
    movq    %rsi, %rax
    movq    %rax, %rcx
    andq    $TAG_MASK, %rcx
    cmpq    $TAG_BOXED, %rcx
    jne     .Lsid8_bits_type

    shrq    $3, %rax                /* boxed handle */
    testq   %rax, %rax
    jz      .Lsid8_bits_abi
    cmpq    $SID8_CAPACITY, %rax
    ja      .Lsid8_bits_abi

    decq    %rax                    /* exact bits / table index */
    leaq    wsm_sid8_table(%rip), %rcx
    leaq    (%rcx,%rax,2), %rcx
    cmpb    $BOXED_KIND_SID8, 0(%rcx)
    jne     .Lsid8_bits_abi
    cmpb    %al, 1(%rcx)
    jne     .Lsid8_bits_abi
    movzbl  1(%rcx), %eax
    ret

.Lsid8_bits_type:
    movl    $ERR_TYPE, %esi
    xorl    %ecx, %ecx
    jmp     wsm_fail
.Lsid8_bits_abi:
    movl    $ERR_ABI_VIOLATION, %esi
    xorl    %ecx, %ecx
    jmp     wsm_fail
    .size wsm_sid8_bits, . - wsm_sid8_bits

/* Draft v8 exact PredicateBit carrier.
 *
 * These accessors are representation-only. They expose canonical singleton
 * target words for exact bit 0 and bit 1 and recover the exact bit. The
 * function names deliberately use 0/1 rather than NO/YES so this target
 * runtime does not mint predicate semantics.
 */

    .globl wsm_predicate_bit_0
    .type wsm_predicate_bit_0, @function
wsm_predicate_bit_0:
    movl    $PREDICATE_BIT0_WORD, %eax
    ret
    .size wsm_predicate_bit_0, . - wsm_predicate_bit_0

    .globl wsm_predicate_bit_1
    .type wsm_predicate_bit_1, @function
wsm_predicate_bit_1:
    movl    $PREDICATE_BIT1_WORD, %eax
    ret
    .size wsm_predicate_bit_1, . - wsm_predicate_bit_1

    .globl wsm_predicate_bit_bits
    .type wsm_predicate_bit_bits, @function
wsm_predicate_bit_bits:
    movq    %rsi, %rdx
    movq    %rsi, %rax
    movq    %rax, %rcx
    andq    $TAG_MASK, %rcx
    cmpq    $TAG_BOXED, %rcx
    jne     .Lpredicate_bits_type

    shrq    $3, %rax
    cmpq    $PREDICATE_BIT0_HANDLE, %rax
    je      .Lpredicate_bits_0
    cmpq    $PREDICATE_BIT1_HANDLE, %rax
    je      .Lpredicate_bits_1
    jmp     .Lpredicate_bits_abi

.Lpredicate_bits_0:
    leaq    wsm_predicate_bit_table(%rip), %rcx
    cmpb    $BOXED_KIND_PREDICATE_BIT, 0(%rcx)
    jne     .Lpredicate_bits_abi
    cmpb    $0, 1(%rcx)
    jne     .Lpredicate_bits_abi
    xorl    %eax, %eax
    ret

.Lpredicate_bits_1:
    leaq    wsm_predicate_bit_table+PREDICATE_ENTRY_BYTES(%rip), %rcx
    cmpb    $BOXED_KIND_PREDICATE_BIT, 0(%rcx)
    jne     .Lpredicate_bits_abi
    cmpb    $1, 1(%rcx)
    jne     .Lpredicate_bits_abi
    movl    $1, %eax
    ret

.Lpredicate_bits_type:
    movl    $ERR_TYPE, %esi
    xorl    %ecx, %ecx
    jmp     wsm_fail
.Lpredicate_bits_abi:
    movl    $ERR_ABI_VIOLATION, %esi
    xorl    %ecx, %ecx
    jmp     wsm_fail
    .size wsm_predicate_bit_bits, . - wsm_predicate_bit_bits

/* Stage2 closure ABI.
 *
 * Це не нова Lisp-примітива. Closure=5, layout дескриптора і три імпорти
 * wsm_closure_* уже ратифіковані wsm-target-contract; тут лише мінімальна
 * SysV-механіка, потрібна CML для матеріалізації справжньої identity closure.
 * Окремий bump-арена робить дві однакові closure-конструкції різними Word,
 * тому `eq` природно перевіряє identity, а не структуру -- саме це потрібно
 * provenance-токенам поточного meta-eval.lisp.
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
    movl    $ERR_OUT_OF_MEMORY, %esi
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
    movl    $ERR_TYPE, %esi
    xorl    %ecx, %ecx
    jmp     wsm_fail
.Lclosure_definition_abi:
    movl    $ERR_ABI_VIOLATION, %esi
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
    movl    $ERR_TYPE, %esi
    xorl    %ecx, %ecx
    jmp     wsm_fail
.Lclosure_environment_abi:
    movl    $ERR_ABI_VIOLATION, %esi
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
    /* 32768 bytes = 2048 cons cells. Still deliberately bounded: this is
     * a bootstrap/runtime witness arena, not a general-purpose heap. 2048
     * cells are enough to carry the 503-cell Core1 compiler source plus its
     * next-generation emitted IR while preserving explicit fail-closed OOM. */
    .equ ARENA_BYTES, 32768
wsm_arena:
    .zero ARENA_BYTES

    .align CLOSURE_ALIGNMENT
    /* 4096 bytes = 256 closure descriptors. Stage2 needs identity-bearing
     * closure values, not a general GC heap; bounded growth stays explicit. */
    .equ CLOSURE_ARENA_BYTES, 4096
wsm_closure_arena:
    .zero CLOSURE_ARENA_BYTES

    .align 2
    /* One canonical boxed entry for every exact 8-bit identity. Entry byte 0
     * is BoxedKind::Sid8 (=4) once constructed; byte 1 is the exact payload. */
wsm_sid8_table:
    .zero SID8_ENTRY_BYTES * SID8_CAPACITY

    /* PredicateBit singleton descriptors contain initialized bytes and must
     * therefore live in .data, not .bss. Handles 257/258 extend the same
     * runtime-owned Boxed handle space immediately after SID8's 1..256. */
    .section .data
    .align 2
wsm_predicate_bit_table:
    .byte BOXED_KIND_PREDICATE_BIT, 0
    .byte BOXED_KIND_PREDICATE_BIT, 1

    /* These cells also hold initialized addresses (relocations). */
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
