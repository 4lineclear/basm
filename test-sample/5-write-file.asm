FILE_CREATE     equ 64
FILE_WRITE      equ 1
SYS_OPEN        equ 2
SYS_WRITE       equ 1
SYS_CLOSE       equ 3
SYS_EXIT        equ 60
EXIT_SUCCESS    equ 0

    filename    db "asm-file-write.txt", 0
    text        db "This was written from assembly!"
    text_len    equ $-text

    global _start

_start:
    mov rax, SYS_OPEN
    mov rdi, filename
    mov rsi, FILE_CREATE+FILE_WRITE
    mov rdx, 0644o
    syscall
    
    push rax
    mov rdi, rax
    mov rax, SYS_WRITE
    mov rsi, text
    mov rdx, text_len
    syscall

    mov rax, SYS_CLOSE
    pop rdi
    syscall

    mov rax, SYS_EXIT
    mov rdi, EXIT_SUCCESS
    syscall

