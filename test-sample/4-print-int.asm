section bss
    digit_buffer        resb 100    ; a buffer for digits.
    digit_buffer_pos    resb 8      ; hold the position
                                    ; held here since rcx changes with syscall

section data

section text
    global _start

_start:
    mov rax, 1337
    call print

    mov rax, 60
    mov rdi, 0
    syscall

print:
    mov rcx, digit_buffer
    mov rbx, 10
    mov [rcx], rbx
    inc rcx
    mov [digit_buffer_pos], rcx
int_buffer:
    mov rdx, 0
    mov rbx, 10
    div rbx
    push rax
    add rdx, 48 ; change remainder to ascii code for number

    mov rcx, [digit_buffer_pos]
    mov [rcx], dl
    inc rcx
    mov [digit_buffer_pos], rcx

    pop rax
    cmp rax, 0
    jne int_buffer
print_loop:
    mov rcx, [digit_buffer_pos]

    mov rax, 1
    mov rdi, 1
    mov rsi, rcx
    mov rdx, 1
    syscall

    mov rcx, [digit_buffer_pos]
    dec rcx
    mov [digit_buffer_pos], rcx

    cmp rcx, digit_buffer
    jge print_loop

    ret
