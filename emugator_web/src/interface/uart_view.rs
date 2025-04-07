use dioxus::prelude::*;
use dioxus::{prelude::component, signals::Signal};
use emugator_core::emulator::AnyEmulatorState;

#[component]
#[allow(non_snake_case)]
pub fn UartView(
    emulator_state: Signal<Option<AnyEmulatorState>>,
    minimize_console: Signal<bool>,
) -> Element {
    let mut input_text = use_signal(|| String::new());

    let state = emulator_state.read();

    rsx! {
        div { class: "flex-col bg-inherit text-gray-200 font-mono border-t-[0.450px] border-gray-600",
            div {
                div { class: "flex flex-grow p-2 items-center align-center justify-between",
                    "UART Console"
                    button {
                        class: "flex items-center justify-center text-center text-sm bg-inherit px-2 hover:outline outline-gray-400 rounded shadow "
                            .to_owned()
                            + { if *minimize_console.read() { "" } else { "origin-center rotate-180" } },
                        onclick: move |_| {
                            let is_minimized = *minimize_console.read();
                            minimize_console.set(!is_minimized);
                        },
                        svg {
                            width: "16",
                            height: "16",
                            view_box: "0 0 24 24",
                            stroke: "currentColor",
                            fill: "none",
                            "stroke-width": "1",
                            "stroke-linecap": "round",
                            "stroke-linejoin": "round",
                            path { d: "M12 4 L22 14 L20.6 15.4 L12 6.8 L3.4 15.4 L2 14 Z" }
                        }
                    }
                }
                if !*minimize_console.read() {
                    hr {}
                }
            }
            if let Some(memory_io) = emulator_state.read().as_ref().map(|e| e.memory_io()) {
                div { class: if *minimize_console.read() { "h-0" } else { "p-3" },
                    div { class: "relative rounded-sm hover:outline",
                        textarea {
                            class: "relative leading-none w-full p-1 min-h-[3rem] resize-y z-10 focus:outline-none",
                            placeholder: "> Type here",
                            oninput: move |event| {
                                let value = event.value();
                                let old_value = memory_io.get_serial_input();
                                if let Some(memory_io_mut) = emulator_state.write().as_mut().map(|e| e.memory_io_mut()) {
                                    let i = memory_io.get_serial_cursor();
                                    let new_value = String::from_utf8_lossy(&old_value[..i]).into_owned() + &value[i..];
                                    memory_io_mut.set_serial_input(new_value.as_bytes());
                                    input_text.set(new_value);
                                }
                                else {
                                    input_text.set(String::from_utf8_lossy(old_value).to_string());
                                }
                            },
                            "{input_text}"
                        }

                    }
                }
            }
        }
    }
}
