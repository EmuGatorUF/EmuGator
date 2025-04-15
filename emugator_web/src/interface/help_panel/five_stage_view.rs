use dioxus::prelude::*;

use crate::interface::help_panel::{H3_STYLE, P_STYLE};

#[component]
#[allow(non_snake_case)]
pub fn FiveStageView() -> Element {
    rsx!(
        h3 { class: H3_STYLE, "Five Stage" }

        p { class: P_STYLE,
            "EmuGator's five-stage pipelined architecture follows the conventional RISC-V pipeline design found in many computer architecture textbooks. This model includes five distinct stages that mirror real-world hardware implementations used in academic settings."
        }

        p { class: P_STYLE,
            "Instructions move through the following stages in the text editor:"
        }

        ul { class: "list-disc list-inside text-sm mb-2 ml-6",
            li {
                code { class: "bg-gray-200 rounded px-1", "IF" }, ": ",
                span { class: "inline-block w-4 h-4 align-middle rounded ml-1 mr-2", style: "background-color: #3b82f6;" },
                "Instruction Fetch"
            }
            li {
                code { class: "bg-gray-200 rounded px-1", "ID" }, ": ",
                span { class: "inline-block w-4 h-4 align-middle rounded ml-1 mr-2", style: "background-color: #d97706;" },
                "Instruction Decode"
            }
            li {
                code { class: "bg-gray-200 rounded px-1", "EX" }, ": ",
                span { class: "inline-block w-4 h-4 align-middle rounded ml-1 mr-2", style: "background-color: #dc2626;" },
                "Execute"
            }
            li {
                code { class: "bg-gray-200 rounded px-1", "MEM" }, ": ",
                span { class: "inline-block w-4 h-4 align-middle rounded ml-1 mr-2", style: "background-color: #4ade80;" },
                "Memory Access"
            }
            li {
                code { class: "bg-gray-200 rounded px-1", "WB" }, ": ",
                span { class: "inline-block w-4 h-4 align-middle rounded ml-1 mr-2", style: "background-color: #9333ea;" },
                "Write Back"
            }
        }

        p { class: P_STYLE,
            "Each instruction is visually tracked as it progresses through these stages, helping students develop a deeper understanding of pipeline behavior and instruction-level parallelism."
        }
    )
}

