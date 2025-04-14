use dioxus::prelude::*;

use crate::interface::help_panel::{H3_STYLE, H4_STYLE, H5_STYLE, P_STYLE};

#[component]
#[allow(non_snake_case)]
pub fn InterfacePageView() -> Element {
    rsx!(
        h3 { class: H3_STYLE, "Interface" }

        p { class: P_STYLE,
            "EmuGator features a split-pane design that helps students clearly correlate written assembly instructions with live changes in the processor pipeline."
        }

        h4 { class: H4_STYLE, "Control Bar" }
        p { class: P_STYLE,
            "Across the top of the page, you'll find EmuGator's main control bar. This set of buttons allows you to control the execution of your RISC-V program:"
        }
        ul { class: "list-disc list-inside text-sm mb-2 ml-",
            li { strong { "Start/Reload" }, ": Assembles, initializes, and reloads the program." }
            li { strong { "Tick Clock" }, ": Advances the emulator by one clock cycle." }
            li { strong { "Next Instruction" }, ": Executes a single instruction through the pipeline." }
            li { strong { "Until Break" }, ": Continues execution until a breakpoint is reached." }
            li { strong { "Pipeline Toggle" }, ": Toggles between the Two and Five-stage pipelines." }
        }
        p { class: P_STYLE,
            "You'll also find a ", strong { "status indicator" }, " that shows whether the program is running, ready, or has errors."
        }

        h4 { class: H4_STYLE, "Editor" }
        p { class: P_STYLE,
            "The left-hand side of the page features a ",
            a {
                href: "https://microsoft.github.io/monaco-editor/",
                target: "_blank",
                class: "text-blue-600 underline hover:text-blue-800",
                "Monaco text editor"
            },
            " — yes, the same one as VSCode! This means all your favorite key binds and shortcuts will work out of the box inside EmuGator. If you're unfamiliar with these button combinations, don't worry! Simply right-click or press ",
            code { class: "px-1 py-0.5 bg-gray-200 rounded text-sm", "F1" },
            " while focused on the editor to bring up a menu that lists helpful shortcuts."
        }

        h5 { class: H5_STYLE, "Editor Breakpoints" }
        p { class: P_STYLE,
            "You can set breakpoints within the editor to use while running your program. To add a breakpoint, click to the left of the line number in the editor's gutter."
        }

        h4 { class: H4_STYLE, "UART Console" }
        p { class: P_STYLE,
            "Underneath the editor lies the UART console used for input and output."
        }
        p { class: P_STYLE,
            "After starting a program, the UART console will populate with a text box. This text box allows you to type or paste text that can be read during program execution. Any characters your program outputs using the UART module will appear below the text box."
        }

        h4 { class: H4_STYLE, "Pipeline Visualization" }
        p { class: P_STYLE,
            "On the top right, the ", strong { "Pipeline Visualization" }, " dynamically renders the processor's internal architecture. Instructions will move through the pipeline stages as your program runs, helping you trace data and control flow in real-time."
        }
        p { class: P_STYLE,
            "Switching between ", strong { "Two-Stage" }, " and ", strong { "Five-Stage" }, " views allows for flexible learning depending on your course or curiosity."
        }

        h4 { class: H4_STYLE, "Register View" }
        p { class: P_STYLE,
            "Located below the pipeline diagram, the ", strong { "Register View" }, " displays the values of all 32 RISC-V registers. As instructions execute, affected registers are updated live. Special-purpose registers like ",
            code { class: "bg-gray-200 rounded px-1", "zero" }, ", ",
            code { class: "bg-gray-200 rounded px-1", "ra" }, ", ",
            code { class: "bg-gray-200 rounded px-1", "sp" }, ", and ",
            code { class: "bg-gray-200 rounded px-1", "a0–a7" },
            " are labeled for quick identification."
        }

        h4 { class: H4_STYLE, "Memory View" }
        p { class: P_STYLE,
            "At the bottom right, the ", strong { "Memory View" }, " shows instruction and data memory contents. You can observe how instructions are stored and watch memory values change as your program reads and writes data. Binary encodings and memory addresses are displayed side-by-side, helping reinforce how high-level assembly maps to actual machine code. Additionally, data memory is displayed in both raw bytes expressed as hex and as ASCII if the value at that location is a valid ASCII character."
        }
    )
}
