.data
.equ value, (0x21 * 2) << 4
message: .string "test\n"
numbers: .word 1, 2, 3, 4
bytes: .byte 0xFF, 0x42, 0x33
array: .ascii "test"

.text
main:
    # Example I-type instruction example
    ADDI x2, x9, value  #   x2 = x9 + 5
    XORI x3, x2, 0xFF   #   x3 = x2 ^ 0xFF
    SLLI x9, x2, 2      #   x9 = x2 << 2

    # U-type instruction example
    LUI x4, 0xFFF       # (load upper immediate) x4 = 0xFFF << 12

    # R-type instruction example
    SUB x8, x3, x2      # x8 = x3 - x2

    # J-type instruction
    JAL x1, function    # jump to function, storing return address in x1
    XOR x3, x3, x3      # will not be executed
    SW x1, 4(x9)        # store value in x1 to address x9 + 4 
loop_forever:
    BNE x3, x2, loop_forever

branch_target:
    ADDI x2, x6, 10     # x2 = x6 + 10
    JALR x1, x1, 0x04   # jump to address at x1 + 4

function:
    # B-type instruction
    BNE x3, x2, branch_target   # jumps to branch target