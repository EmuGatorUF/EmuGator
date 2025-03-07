.data
.equ value, (0x21 * 2) << 4
.equ TX_ADR, 0xF4
.equ LSR_ADR, 0xF8
.equ TX_READY, 1 << 2
message: .string "Alas!\nPoor\tYorick"
numbers: .word 1, 2, 3, 4
bytes: .byte 0xFF, 0x42, 0x33
array: .ascii "test"

.text
main:
    # Example I-type instruction example
    ADDI x2, x9, value          #   x2 = x9 + 5
    XORI x3, x2, 0xFF           #   x3 = x2 ^ 0xFF
    SLLI x9, x2, 2              #   x9 = x2 << 2

    # U-type instruction example
    LUI x4, 0xFFF               # (load upper immediate) x4 = 0xFFF << 12

    # R-type instruction example
    SUB x8, x3, x2              # x8 = x3 - x2

    # J-type instruction
    JAL x20, print              # jump to function, storing return address in x20
    XOR x3, x3, x3              # will not be executed until after return
    SW x1, 4(x9)                # store value in x1 to address x9 + 4 
loop_forever:
    BNE x3, x2, loop_forever

branch_target:
    ADDI x2, x6, 10             # x2 = x6 + 10
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
    EBREAK                      # environment breakpoint
    JALR x0, x20, 0x0           # return to whence we came
