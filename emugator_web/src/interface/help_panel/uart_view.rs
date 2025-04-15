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
            "This program transmits a null-terminated string to the UART output by polling the ",
            code { class: "bg-gray-100 rounded px-1", "TX_READY" },
            " bit. You can view the output within the UART Console."
        }

        pre {
            class: "bg-gray-900 text-gray-100 text-xs rounded p-4 overflow-x-auto my-2",
            code { r#".data
.equ DATA_ADR, 0xF0
.equ LSR_ADR, 0xF4
.equ TX_READY, 1 << 2
message: .string "Alas!\nPoor\tYorick\n"

.text
main:
    ADDI x1, x0, message        # x1 = pointer to message
    JAL x20, print              # call print function

loop_forever:
    JAL x0, loop_forever        # infinite loop

print:
    ADDI x10, x0, DATA_ADR      # UART Transmit Register address
    ADDI x11, x0, LSR_ADR       # Line Status Register address
print_loop:
    LB x2, 0(x1)                # load character
    BEQ x2, x0, end             # exit if null terminator

wait_tx:
    LB x3, 0(x11)               # load LSR
    ANDI x4, x3, TX_READY       # check TX_READY bit
    BEQ x4, x0, wait_tx         # wait if not ready

    SB x2, 0(x10)               # transmit character
    ADDI x1, x1, 1              # advance pointer
    JAL x0, print_loop

end:
    JALR x0, x20, 0             # return
"# }
        }

        h4 { class: H4_STYLE, "UART Input Example (Read Line Function)" }
        p { class: P_STYLE,
            "This program reads a line of input from the UART by polling the ",
            code { class: "bg-gray-100 rounded px-1", "RX_READY" },
            " bit. The received characters are stored in memory, and you can view them inside the Data Memory panel."
        }

        pre {
            class: "bg-gray-900 text-gray-100 text-xs rounded p-4 overflow-x-auto my-4",
            code { r#".data
.equ DATA_ADR, 0xF0
.equ LSR_ADR, 0xF4
.equ RX_READY, 1 << 0

input: .zero 50                # input buffer

.text
main:
    ADDI x1, x0, input          # x1 = input buffer
    JAL x20, readline           # call readline function

loop_forever:
    JAL x0, loop_forever        # infinite loop

readline:
    ADDI x10, x0, DATA_ADR      # UART Receive Register address
    ADDI x11, x0, LSR_ADR       # Line Status Register address
    ADDI x12, x0, '\n'          # newline character
read_loop:
wait_rx:
    LB x3, 0(x11)               # load LSR
    ANDI x4, x3, RX_READY       # check RX_READY bit
    BEQ x4, x0, wait_rx         # wait if not ready

    LB x2, 0(x10)               # read character
    SB x2, 0(x1)                # store in buffer
    ADDI x1, x1, 1              # advance buffer

    BNE x2, x12, read_loop      # repeat until newline
    SB x0, 0(x1)                # null-terminate
    JALR x0, x20, 0             # return
"# }
        }
    )
}

