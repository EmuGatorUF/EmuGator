.data
.equ TX_ADR, 0xF4
.equ LSR_ADR, 0xF8
.equ TX_READY, 1 << 2
message: .string "Alas!\nPoor\tYorick"

.text
main:
    JAL x20, print              # jump to function, storing return address in x20

loop_forever:
    # Example I-type instruction example
    ADDI x1, x0, 0x76           #   x1 = x0 + 0x76
    ADDI x9, x0, 0x42           #   x9 = x0 + 0x42
    ADDI x2, x0, 1              #   x2 = x0 + 1
    SB  x2, 0x65(x0)
    SB  x0, 0x64(x0)
    ADDI x2, x0, 0              #   x2 = x0 + 0
    EBREAK
    BEQ x0, x0, loop_forever    # branch to infinite loop

print:
    ADDI x1, x0, message        # x0 = address of message
    ADDI x10, x0, TX_ADR        # x10 = address of UART Transmit Register
    ADDI x11, x0, LSR_ADR       # x11 = address of UART Line Status Register
print_loop:
    LB x2, 0(x1)                # load character

    wait:
        LB x3, 0(x11)           # load LSR
        ANDI x4, x3, TX_READY   # check if ready to transmit
        BEQ x4, x0, wait        # if not, try again
    
    SB x2, 0(x10)               # write byte to TX Register
    ADDI x1, x1, 1              # message_ptr += 1
    BNE x2, x0, print_loop      # branch to start if character was not null ('\0')
    EBREAK
    JALR x0, x20, 0x0           # return to whence we came
