use dioxus::prelude::*;

#[component]
#[allow(non_snake_case)]
pub fn IntroPageView() -> Element {
    rsx!(
        h3 {class: "text-3xl font-bold mb-4 border-b-2 border-gray-300 pb-2", "Introduction" }
        p { class: "leading-relaxed mb-2 text-sm",
            "Welcome to EmuGator, a RISC-V emulator that runs entirely in the browser!"
        }
        p { class: "leading-relaxed mb-2 text-sm",
            "EmuGator includes high-quality features, including a text editor, assembler, two and five-stage pipeline visualization, instruction and data memory views, and a memory-mapped UART interface for input and output operations."
        }

        h4 {class: "text-xl font-semibold mb-2 mt-1", "Why EmuGator" }
        p { class: "leading-relaxed mb-2 text-sm",
            "EmuGator was largely inspired by introductory computer organization classes taught at universities. Many students find it hard to map assembly instructions to CPU behavior. Thus, universities use command-line emulators to help bridge this gap."
        }
        p { class: "leading-relaxed mb-2 text-sm",
            "However, setting up these tools can often be complex and temperamental. Professor and Teaching Assistant office hours are spent debugging setup issues rather than discussing theory, wasting everyone's time. EmuGator's goal is to alleviate the headaches caused by these issues while maintaining the high standards and features people have come to expect from similar emulator toolchains."
        }

        h4 {class: "text-xl font-semibold mb-2 mt-1", "How it work" }
        p { class: "leading-relaxed mb-2 text-sm",
            "EmuGator is written entirely in Rust, allowing it to run efficiently in the browser without requiring extra downloads or extensions. For a more detailed overview, please see each component section."
        }
    )
}
