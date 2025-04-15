use dioxus::prelude::*;

use crate::interface::help_panel::{H3_STYLE, P_STYLE};

#[component]
#[allow(non_snake_case)]
pub fn TwoStageView() -> Element {
    rsx!(
        h3 { class: H3_STYLE, "Two Stage" }

        p { class: P_STYLE,
            "EmuGator's two-stage pipelined architecture is inspired by the ",
            a {
                href: "https://github.com/openhwgroup/cve2",
                target: "_blank",
                class: "text-blue-600 underline hover:text-blue-800",
                "CVE2"
            },
            " design created by the OpenHW Group. This simplified model is ideal for beginners exploring how instructions flow through a simple pipeline."
        }

        p { class: P_STYLE,
            "Instructions progress through the following two stages in the text editor:"
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
        }

        p { class: P_STYLE,
            "This view provides a clear and focused look at how instructions are fetched and decoded, perfect for understanding control signals and register reads before exploring more advanced pipeline behavior."
        }
    )
}
