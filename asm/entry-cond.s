.text
.globl wsm_entry
.type wsm_entry, @function
wsm_entry:
    pushq %r12
    subq $64, %rsp
    movq %rdi, %r12
.Lcond_branch_1:
    movabsq $1, %rax
    movabsq $1, %rcx
    cmpq %rcx, %rax
    je .Lcond_branch_2
    movabsq $20, %rax
    jmp .Lcond_end_0
.Lcond_branch_2:
    movabsq $18446744073709551612, %rax
    movabsq $1, %rcx
    cmpq %rcx, %rax
    je .Lcond_branch_3
    movabsq $12, %rax
    jmp .Lcond_end_0
.Lcond_branch_3:
    movabsq $1, %rax
.Lcond_end_0:
    addq $64, %rsp
    popq %r12
    ret
.size wsm_entry, .-wsm_entry
.section .note.GNU-stack,"",@progbits
