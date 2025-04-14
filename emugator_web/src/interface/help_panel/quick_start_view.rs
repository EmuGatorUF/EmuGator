use dioxus::prelude::*;

use crate::interface::help_panel::{H3_STYLE, P_STYLE};

#[component]
#[allow(non_snake_case)]
pub fn QuickStartView() -> Element {
    rsx!(
        h3 { class: H3_STYLE, "⚡ Quick Start" }

        p { class: P_STYLE,
            "EmuGator's interface is simple and intuitive. To begin, paste your code into the assembly editor (or use the default program) and click ",
            strong { "Start" }, "."
        }

        p { class: P_STYLE,
            "To run your program, choose from the available execution controls: ",
            strong { "Tick Clock" }, ", ",
            strong { "Next Instruction" }, ", or ",
            strong { "Until Break" }, "."
        }

        p { class: P_STYLE,
            "If your program reads input via the UART console, simply type or paste text into the input box and press ",
            code { class: "px-1 py-0.5 bg-gray-200 rounded text-sm", "Enter" }, "."
        }

        p { class: P_STYLE,
            "That’s it! You’re now ready to explore each component in more detail."
        }
    )
}

