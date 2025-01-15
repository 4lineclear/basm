section .data
    hello_world db "Hello, World!", 10, 0
    whats_up db "What's up", 10, 0
    long_text db "this is a longer line of text.", 10, 0

section .text
    global _start

_start:
    mov rax, hello_world
    call print

    mov rax, whats_up
    call print

    mov rax, long_text
    call print

    mov rax, 60
    mov rdi, 0
    syscall
print:
    push rax
    mov rbx, 0
print_loop:
    inc rax
    inc rbx
    mov cl, [rax]
    cmp cl, 0
    jne print_loop

    mov rax, 1
    mov rdi, 1
    pop rsi
    mov rdx, rbx
    syscall

    ret
