use dioxus::prelude::*;

use crate::interface::help_panel::{H3_STYLE, H4_STYLE, P_STYLE};

#[component]
#[allow(non_snake_case)]
pub fn UartView() -> Element {
    rsx!(
        h3 { class: H3_STYLE, "UART" }

        p { class: P_STYLE,
            "EmuGator features UART functionality for input and output operations. The UART module supports standard serial communication with a transmit buffer, receive buffer, and line status register."
        }

        h4 { class: H4_STYLE, "Memory Map Locations and Bitmasks" }

        div { class: "ml-6 text-sm",
            ul { class: "list-disc list-inside mb-2",
                li {
                    strong { "DATA Register (TX/RX):" }
                    code { class: "ml-1 bg-gray-100 rounded px-1", "0xF0" }
                    ul {
                        li { class: "ml-4", "Write to this address to transmit a character." }
                        li { class: "ml-4", "Read from this address to receive a character." }
                    }
                }
                li {
                    strong { "Line Status Register (LSR):" }
                    code { class: "ml-1 bg-gray-100 rounded px-1", "0xF4" }
                    ul { class: "mt-1",
                        li {
                            code { class: "bg-gray-100 rounded px-1", "TX_READY = 0x04" }
                            span { class: "ml-4", "Transmit buffer is empty (ready to send)." }
                        }
                        li {
                            code { class: "bg-gray-100 rounded px-1", "RX_READY = 0x01" }
                            span { class: "ml-4", "Receive buffer has data available." }
                        }
                    }
                }
            }
        }

        h4 { class: H4_STYLE, "UART Output Example (Print Function)" }

        p { class: P_STYLE,
            "This program transmits a null-terminated string to the UART output by polling the "
            code { class: "bg-gray-100 rounded px-1", "TX_READY" }
            " bit. You can view the output within the UART Console. Note the proper use of RISC-V register conventions."
        }

        pre { class: "bg-gray-900 text-gray-100 text-xs rounded p-4 overflow-x-auto my-2",
            code {
                r#".data
.equ DATA_ADR, 0xF0
.equ LSR_ADR, 0xF4
.equ TX_READY, 1 << 2
message: .string "Alas!\nPoor\tYorick\n"

.text
main:
    ADDI x10, x0, message       # x10 (a0) = pointer to message (argument register)
    JAL x1, print               # print function, return address in x1 (ra)

loop_forever:
    JAL x0, loop_forever        # infinite loop

print:
    ADDI x28, x0, DATA_ADR      # x28 (t3) = UART Transmit Register address
    ADDI x29, x0, LSR_ADR       # x29 (t4) = Line Status Register address
print_loop:
    LB x5, 0(x10)               # x5 (t0) = load character from string
    BEQ x5, x0, print_end       # exit if null terminator

wait_tx:
    LB x6, 0(x29)               # x6 (t1) = load LSR
    ANDI x7, x6, TX_READY       # x7 (t2) = check TX_READY bit
    BEQ x7, x0, wait_tx         # wait if not ready

    SB x5, 0(x28)               # transmit character
    ADDI x10, x10, 1            # advance string pointer
    JAL x0, print_loop

print_end:
    JALR x0, x1, 0              # return using x1 (ra)
"#
            }
        }

        h4 { class: H4_STYLE, "UART Input Example (Read Line Function)" }
        p { class: P_STYLE,
            "This program reads a line of input from the UART by polling the "
            code { class: "bg-gray-100 rounded px-1", "RX_READY" }
            " bit. The received characters are stored in memory, and you can view them inside the Data Memory panel. Notice how registers follow RISC-V conventions."
        }

        pre { class: "bg-gray-900 text-gray-100 text-xs rounded p-4 overflow-x-auto my-4",
            code {
                r#".data
.equ DATA_ADR, 0xF0
.equ LSR_ADR, 0xF4
.equ RX_READY, 1 << 0

input: .zero 50                # input buffer

.text
main:
    ADDI x10, x0, input         # x10 (a0) = input buffer address (argument register)
    JAL x1, readline            # call readline function, return address in x1 (ra)

loop_forever:
    JAL x0, loop_forever        # infinite loop

readline:
    ADDI x28, x0, DATA_ADR      # x28 (t3) = UART Receive Register address
    ADDI x29, x0, LSR_ADR       # x29 (t4) = Line Status Register address
    ADDI x30, x0, '\n'          # x30 (t5) = newline character
read_loop:
wait_rx:
    LB x6, 0(x29)               # x6 (t1) = load LSR
    ANDI x7, x6, RX_READY       # x7 (t2) = check RX_READY bit
    BEQ x7, x0, wait_rx         # wait if not ready

    LB x5, 0(x28)               # x5 (t0) = read character
    SB x5, 0(x10)               # store in buffer
    ADDI x10, x10, 1            # advance buffer pointer

    BNE x5, x30, read_loop      # repeat until newline
    SB x0, 0(x10)               # null-terminate string
    JALR x0, x1, 0              # return using x1 (ra)
"#
            }
        }

        h4 { class: H4_STYLE, "Register Usage Notes" }
        p { class: P_STYLE, "These examples demonstrate proper RISC-V register conventions:" }
        div { class: "ml-6 text-sm",
            ul { class: "list-disc list-inside mb-2",
                li {
                    strong { "x10 (a0):" }
                    " Used for function arguments (string/buffer addresses)"
                }
                li {
                    strong { "x1 (ra):" }
                    " Return address register for function calls"
                }
                li {
                    strong { "x5-x7 (t0-t2):" }
                    " Temporary registers for intermediate calculations"
                }
                li {
                    strong { "x28-x30 (t3-t5):" }
                    " Additional temporary registers for local variables"
                }
                li {
                    strong { "x2 (sp):" }
                    " Stack pointer - NOT used as general purpose (proper convention!)"
                }
            }
        }
    )
}
