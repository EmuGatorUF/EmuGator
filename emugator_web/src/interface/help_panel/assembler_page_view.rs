use std::collections::HashMap;

use dioxus::prelude::*;

use crate::{code_editor::language::{DocEntry, DOCS}, interface::help_panel::{H3_STYLE, H4_STYLE, P_STYLE}};

#[component]
pub fn InstructionCard(
    name: String,
    format: String,
    desc: String,
    example: String,
) -> Element {
    rsx! {
        div {
            class: "max-w-md mx-auto bg-white shadow-lg rounded-xl p-6 border border-gray-200 mb-2",
            h2 {
                class: "text-xl font-semibold text-gray-800 mb-1",
                "{name}"
            }
            p {
                class: "text-sm text-gray-500 mb-3 italic",
                "Format: ",
                span {
                    class: "font-mono text-gray-700",
                    "{format}"
                }
            }
            p {
                class: "text-gray-700 mb-2",
                dangerous_inner_html: "{desc}"
            }
            div {
                class: "bg-gray-100 text-sm font-mono text-gray-800 p-3 rounded",
                "Example: ",
                span {
                    class: "text-blue-600",
                    "{example}"
                }
            }
        }
    }
}

#[component]
#[allow(non_snake_case)]
pub fn AssemblerPageView() -> Element {
    let docs: HashMap<&'static str, DocEntry<'static>> =
        serde_json::from_str(DOCS).expect("failed to parse docs.json");

    let mut mnumonics: Vec<_> = docs.keys().copied().collect();
    mnumonics.sort(); // Sorts alphabetically

    rsx!(
        h3 { class: H3_STYLE, "Assembler and Instructions" }

        p { class: P_STYLE,
            "EmuGator features a built-in assembler that converts RISC-V assembly into binary instructions. The assembler is feature-rich and GCC compatible — that means you can take a program compiled with GCC and assemble it using EmuGator! The assembler supports all instructions found within the RV32-I instruction set architecture."
        }

        h4 { class: H4_STYLE, "Instructions" }

        p { class: P_STYLE,
            "Here are all the instructions and assembler directives supported by EmuGator with associated functionality and examples:"
        }

        div { class: "gap-y-1",
            for key in mnumonics {
                InstructionCard {
                    name: key,
                    format: docs.get(key).unwrap().format.clone(),
                    desc: docs.get(key).unwrap().desc.clone(),
                    example: docs.get(key).unwrap().example.clone(),
                }
            }
        }
    )
}

