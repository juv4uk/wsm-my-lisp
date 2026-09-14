/* witness-bootstrap.s -- hosted-process bootstrap for vertical-slice witnesses.
 *
 * This is NOT a Lisp primitive and NOT Lisp semantics: it is the machine
 * mechanism (crt0-equivalent) that lets a CML-generated wsm_entry run as a
 * standalone Linux ELF. It aligns the stack per SysV AMD64, calls wsm_entry
 * with a null context, writes the returned 64-bit word raw to stdout (little-
 * endian, 8 bytes), and exits. Interpreting the word (t vs ()) is done by the
 * witness harness / oracle comparison, never in this file.
 *
 * Why this is allowed under the Lisp+asm authority: "if logic can live in
 * Lisp -> Lisp. If literally irreducible machine mechanism -> assembler."
 * Before any Lisp runs, some machine code must enter a process, set rsp, and
 * make exit(2); that is irreducible hosted-process mechanism, the same class
 * as the syscalls already used by asm/nucleus.s's wsm_fail.
 */

    .text
    .globl _start
    .type _start, @function
_start:
    xor %rbp, %rbp              /* end of frame chain, per ABI */
    and $-16, %rsp               /* 16-byte stack alignment before call */
    mov %rsp, %rdi               /* context = pointer to argc/argv stack area (ignored by nucleus) */
    call wsm_entry
    lea result_buffer(%rip), %rsi
    mov %rax, (%rsi)             /* store raw 64-bit result word (little-endian) */
    movq $1, %rax                /* SYS_write */
    movq $1, %rdi                /* fd = stdout */
    movq $8, %rdx                /* write 8 bytes */
    syscall
    movq $60, %rax               /* SYS_exit */
    xor %rdi, %rdi
    syscall
    .size _start, . - _start

    .section .bss
    .align 8
result_buffer:
    .zero 8

    .section .note.GNU-stack,"",@progbits
