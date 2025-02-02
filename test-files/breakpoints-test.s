.text
main:
    ADDI x2, x9, 5
    XORI x3, x2, 0xFF
    SLLI x9, x2, 2
    LUI x4, 0xFFF
    SUB x8, x3, x2
    XOR x3, x3, x3
    SW x1, 4(x9)
    EBREAK
    
    ADDI x2, x9, 5
    XORI x3, x2, 0xFF
    SLLI x9, x2, 2
    LUI x4, 0xFFF
    SUB x8, x3, x2
    XOR x3, x3, x3
    SW x1, 4(x9)

    ADDI x2, x9, 5
    XORI x3, x2, 0xFF
    SLLI x9, x2, 2
    LUI x4, 0xFFF
    SUB x8, x3, x2
    XOR x3, x3, x3
    SW x1, 4(x9)
