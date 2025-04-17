.data
.equ DATA_ADR, 0xF0
.equ LSR_ADR, 0xF4
.equ RX_READY, 1 << 0
.equ TX_READY, 1 << 2

input: .zero 8
is_palindrome_msg: .string "Palindrome: True\n"
not_palindrome_msg: .string "Palindrome: False\n"
prompt_msg: .string "Enter a string and press enter!\n"
length_msg: .string "String length: "
newline: .string "\n"

.text
############################################################
# MAIN FUNCTION - IMPLEMENTED BY STUDENTS!
# Main basically prints the prompt, gets the input string,
# prints the length, calls palindrome, and prints results.
############################################################
main:
    ADDI x10, x0, prompt_msg
    JAL x1, print
    
    ADDI x10, x0, input
    JAL x1, read_string
    
    ADDI x8, x10, 0
    
    ADDI x10, x0, length_msg
    JAL x1, print
    
    ADDI x10, x8, 0
    JAL x1, print_decimal
    
    ADDI x10, x0, newline
    JAL x1, print
    
    ADDI x10, x0, input
    JAL x1, check_palindrome
    
    BNE x10, x0, print_is_palindrome
    
    ADDI x10, x0, not_palindrome_msg
    JAL x1, print
    EBREAK
    JAL x0, end_program
    
print_is_palindrome:
    ADDI x10, x0, is_palindrome_msg
    JAL x1, print
    
end_program:
    JAL x0, end_program

############################################################
# CHECK PALINDROME FUNCTION - IMPLEMENTED BY STUDENTS!
# Input: X10 (Input String Address)
# Output: X10 (Palindrome Result)
############################################################
check_palindrome:
    
    ADDI x11, x10, 0
    ADDI x12, x0, 0
    
length_loop:
    LB x13, 0(x11)
    BEQ x13, x0, length_done
    ADDI x14, x0, '\n'
    BEQ x13, x14, length_done
    ADDI x12, x12, 1
    ADDI x11, x11, 1
    JAL x0, length_loop
    
length_done:
    ADDI x11, x10, 0
    ADD x14, x10, x12
    ADDI x14, x14, -1
    
    ADDI x10, x0, 1
    BGE x11, x14, return_from_palindrome
    
palindrome_check_loop:
    BGE x11, x14, return_from_palindrome
    
    LB x15, 0(x11)
    LB x16, 0(x14)
    
    BEQ x15, x16, continue_check
    ADDI x10, x0, 0
    JAL x0, return_from_palindrome
    
continue_check:
    ADDI x11, x11, 1
    ADDI x14, x14, -1
    JAL x0, palindrome_check_loop
    
return_from_palindrome:
    JALR x0, x1, 0

############################################################
# READ STRING FUNCTION - GIVEN TO STUDENTS
# Input: X10 (Address to Input String Storage Buffer)
# Output: X10 (Length of Input String)
############################################################
read_string:
    ADDI x2, x2, -12
    SW x1, 0(x2)
    SW x5, 4(x2)
    SW x6, 8(x2)
    
    ADDI x5, x10, 0
    ADDI x6, x0, 0
    
read_char_loop:
    ADDI x11, x0, DATA_ADR
    ADDI x12, x0, LSR_ADR
    
wait_for_data:
    LB x3, 0(x12)
    ANDI x4, x3, RX_READY
    BEQ x4, x0, wait_for_data
    
    LB x7, 0(x11)
    SB x7, 0(x5)
    ADDI x5, x5, 1
    ADDI x6, x6, 1
    
    ADDI x8, x0, '\n'
    BEQ x7, x8, end_read_string
    JAL x0, read_char_loop
    
end_read_string:
    SB x0, 0(x5)
    ADDI x6, x6, -1
    ADDI x10, x6, 0
    
    LW x1, 0(x2)
    LW x5, 4(x2)
    LW x6, 8(x2)
    ADDI x2, x2, 12
    JALR x0, x1, 0

############################################################
# PRINT STRING FUNCTION - GIVEN TO STUDENTS
# Input: X10 (Address to Null Terminated String)
# Output: X10 (Address of Byte After Null Terminator)
############################################################
print:
    ADDI x2, x2, -8
    SW x1, 0(x2)
    SW x5, 4(x2)
    
    ADDI x5, x10, 0
    
print_loop:
    LB x6, 0(x5)
    BEQ x6, x0, end_print
    ADDI x11, x0, DATA_ADR
    ADDI x12, x0, LSR_ADR
    
wait_for_tx:
    LB x3, 0(x12)
    ANDI x4, x3, TX_READY
    BEQ x4, x0, wait_for_tx
    
    SB x6, 0(x11)
    ADDI x5, x5, 1
    
    JAL x0, print_loop
    
end_print:
    ADDI x10, x5, 0
    
    LW x1, 0(x2)
    LW x5, 4(x2)
    ADDI x2, x2, 8
    JALR x0, x1, 0

############################################################
# PRINT DECIMAL FUNCTION - GIVEN TO STUDENTS
# Input: X10 (Integer to Print)
# Output: None
############################################################
print_decimal:
    ADDI x2, x2, -16
    SW x1, 0(x2)
    SW x5, 4(x2)
    SW x6, 8(x2)
    SW x7, 12(x2)
    
    BNE x10, x0, not_zero
    ADDI x6, x0, '0'
    ADDI x11, x0, DATA_ADR
    ADDI x12, x0, LSR_ADR
    
wait_for_tx_zero:
    LB x3, 0(x12)
    ANDI x4, x3, TX_READY
    BEQ x4, x0, wait_for_tx_zero
    
    SB x6, 0(x11)
    JAL x0, print_decimal_done
    
not_zero:
    ADDI x5, x2, -16
    ADDI x6, x5, 0
    
extract_digits:
    ADDI x4, x0, 0
    ADDI x7, x0, 0
    ADDI x9, x0, 10
    
div_loop:
    SUB x3, x10, x9
    BLT x3, x0, div_done
    ADDI x4, x4, 1
    ADDI x10, x3, 0
    JAL x0, div_loop
    
div_done:
    ADDI x7, x10, '0'
    
    ADDI x6, x6, -1
    SB x7, 0(x6)
    
    ADDI x10, x4, 0
    BNE x10, x0, extract_digits
    
print_digits:
    ADDI x11, x0, DATA_ADR
    ADDI x12, x0, LSR_ADR
    
print_digit_loop:
    LB x7, 0(x6)
    
wait_for_tx_digit:
    LB x3, 0(x12)
    ANDI x4, x3, TX_READY
    BEQ x4, x0, wait_for_tx_digit
    
    SB x7, 0(x11)
    ADDI x6, x6, 1
    
    BNE x6, x5, print_digit_loop
    
print_decimal_done:
    LW x1, 0(x2)
    LW x5, 4(x2)
    LW x6, 8(x2)
    LW x7, 12(x2)
    ADDI x2, x2, 16
    JALR x0, x1, 0
