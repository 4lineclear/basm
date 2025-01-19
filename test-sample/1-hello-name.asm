section data
    opening db "What is your name?", 10
    opening_len equ $-opening

    hello db "Hello, "
    hello_len equ $-hello

section bss
    name resb 16

section text
    global _start

_start:
    call display_opening
    call get_name
    call display_hello
    call display_name

    mov rax, 60
    mov rdi, 0
    syscall

display_opening:
    mov rax, 1
    mov rdi, 1
    mov rsi, opening
    mov rdx, opening_len
    syscall
    ret

get_name:
    mov rax, 0
    mov rdi, 0
    mov rsi, name
    mov rdx, 16
    syscall
    ret

display_hello:
    mov rax, 1
    mov rdi, 1
    mov rsi, hello
    mov rdx, hello_len
    syscall
    ret

display_name:
    mov rax, 1
    mov rdi, 1
    mov rsi, name
    mov rdx, 16
    syscall
    ret
