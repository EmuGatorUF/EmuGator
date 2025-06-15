.data
.equ value, (0x21 * 2) << 4
.equ DATA_ADR, 0xF0
.equ LSR_ADR, 0xF4
.equ RX_READY, 1 << 0
.equ TX_READY, 1 << 2
message: .string "Alas!\nPoor\tYorick\n"
numbers: .word 1, 2, 3, 4
bytes: .byte 0xFF, 0x42, 0x33
array: .ascii "test"

input: .zero 50

.text
main:
    # Example I-type instruction example
    ADDI x5, x8, value          # x5 (t0) = x8 (s0) + calculated value
    XORI x6, x5, 0xFF           # x6 (t1) = x5 (t0) ^ 0xFF
    SLLI x8, x5, 2              # x8 (s0) = x5 (t0) << 2

    # U-type instruction example
    LUI x7, 0xFFF               # (load upper immediate) x7 (t2) = 0xFFF << 12

    # R-type instruction example
    SUB x9, x6, x5              # x9 (s1) = x6 (t1) - x5 (t0)

    # J-type instruction - prepare function call
    ADDI x10, x0, message       # x10 (a0) = address of message (first argument register)
    JAL x1, print               # jump to function, storing return address in x1 (ra)

    ADDI x10, x0, input         # x10 (a0) = address of input buffer
    JAL x1, readline            # jump to function, storing return address in x1 (ra)

    ADDI x10, x0, input         # x10 (a0) = address of input buffer
    JAL x1, print               # jump to function, storing return address in x1 (ra)
    
    XOR x6, x6, x6              # will not be executed until after return
    SW x10, 4(x8)               # store value in x10 (a0) to address x8 (s0) + 4 
loop_forever:
    BNE x6, x5, loop_forever

branch_target:
    ADDI x5, x18, 10            # x5 (t0) = x18 (s2) + 10
    BEQ x0, x0, loop_forever    # branch to infinite loop

# UART Print Function
# Input: x10 (a0) = string address
print:
    ADDI x28, x0, DATA_ADR      # x28 (t3) = address of UART Transmit Register
    ADDI x29, x0, LSR_ADR       # x29 (t4) = address of UART Line Status Register
print_loop:
    LB x5, 0(x10)               # load character from string into x5 (t0)
    BEQ x5, x0, print_end       # branch to end if character was null ('\0')

    wait_tx:
        LB x6, 0(x29)           # load LSR into x6 (t1)
        ANDI x7, x6, TX_READY   # check if ready to transmit using x7 (t2)
        BEQ x7, x0, wait_tx     # if not, try again
    
    SB x5, 0(x28)               # write byte to TX Register
    ADDI x10, x10, 1            # string_ptr += 1
    JAL x0, print_loop
print_end:
    EBREAK                      # environment breakpoint
    JALR x0, x1, 0x0            # return to caller using x1 (ra)

# UART Read Function
# Input: x10 (a0) = input buffer address
readline:
    ADDI x28, x0, DATA_ADR      # x28 (t3) = address of UART Receive Register
    ADDI x29, x0, LSR_ADR       # x29 (t4) = address of UART Line Status Register
    ADDI x30, x0, '\n'          # x30 (t5) = newline character
read_loop:
    wait_rx:
        LB x6, 0(x29)           # load LSR into x6 (t1)
        ANDI x7, x6, RX_READY   # check if ready to receive using x7 (t2)
        BEQ x7, x0, wait_rx     # if not, try again
    
    LB x5, 0(x28)               # read byte from RX Register into x5 (t0)
    SB x5, 0(x10)               # store byte in input buffer
    ADDI x10, x10, 1            # input_ptr += 1

    BNE x5, x30, read_loop      # branch to start if character was not newline ('\n')
    EBREAK                      # environment breakpoint
    JALR x0, x1, 0x0            # return to caller using x1 (ra)
