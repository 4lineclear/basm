    digit db 0
    comma db ","
    nl db 10

    global _start

_start:
    xor rbx, rbx
    call print10

    mov rax, 10
    sub rax, 8
    mul rax
    call printRaxDigit

    call next_line

    mov rax, 60
    mov rdi, 0
    syscall

print10:
    mov rax, rbx
    call printRaxDigit
    call print_comma
    inc rbx
    cmp rbx, 9
    jle print10
    ret

print_comma:
    mov rax, 1
    mov rdi, 1
    mov rsi, comma
    mov rdx, 1
    syscall
    ret

next_line:
    mov rax, 1
    mov rdi, 1
    mov rsi, nl
    mov rdx, 1
    syscall
    ret

printRaxDigit:
    add rax, 48
    mov [digit], al
    mov rax, 1
    mov rdi, 1
    mov rsi, digit
    mov rdx, 1
    syscall
    ret
