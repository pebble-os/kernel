; Copyright (C) 2017, Isaac Woods.
; See LICENCE.md

[BITS 64]
mov rax, 0xDEADBEEF
int 0x80
loop:
    jmp loop