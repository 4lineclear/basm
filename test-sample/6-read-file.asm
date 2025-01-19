FILE_READ       equ 0
SYS_READ        equ 0
SYS_OPEN        equ 2
SYS_Read        equ 0
SYS_CLOSE       equ 3
SYS_EXIT        equ 60
EXIT_SUCCESS    equ 0

section bss
    text resb 18

section data
    filename db "justfile", 0

section text
    global _start

_start:
    mov rax, SYS_OPEN
    mov rdi, filename
    mov rsi, FILE_READ
    mov rdx, 0
    syscall
    
    push rax
    mov rdi, rax
    mov rax, SYS_READ
    mov rsi, text
    mov rdx, 17
    syscall

    mov rax, SYS_CLOSE
    pop rdi
    syscall

    mov rax, SYS_EXIT
    mov rdi, EXIT_SUCCESS
    syscall

