use dioxus::prelude::*;
use dioxus::{prelude::component, signals::Signal};
use emugator_core::emulator::AnyEmulatorState;

use dioxus_free_icons::icons::ld_icons::{LdChevronUp, LdChevronDown};
use dioxus_free_icons::Icon;

#[component]
#[allow(non_snake_case)]
pub fn UartView(
    emulator_state: Signal<Option<AnyEmulatorState>>,
    minimize_console: Signal<bool>,
) -> Element {
    let mut input_text = use_signal(|| String::new());

    let state = emulator_state.read();

    let icon_width = 25;
    rsx! {
        div { class: "flex-col bg-inherit text-gray-200 font-mono border-t-[0.450px] border-gray-600",
            div {
                div { class: "flex flex-grow p-2 items-center align-center justify-between",
                    "UART Console"
                    button {
                        class: "flex items-center justify-center text-center text-sm bg-inherit px-2 hover:outline outline-gray-400 rounded cursor-pointer",
                        onclick: move |_| {
                            let is_minimized = *minimize_console.read();
                            minimize_console.set(!is_minimized);
                        },
                        if *minimize_console.read() {
                            Icon {
                                width: icon_width,
                                icon: LdChevronUp
                            }
                        } else {
                            Icon {
                                width: icon_width,
                                icon: LdChevronDown
                            }
                        }
                    }
                }
                if !*minimize_console.read() {
                    hr {}
                }
            }
            if let Some(memory_io) = state.as_ref().map(|e| e.memory_io()) {

                div { class: if *minimize_console.read() { "h-0" } else { "p-3" },
                    div { class: "relative rounded-sm hover:outline",
                        textarea {
                            class: "relative leading-none w-full p-1 min-h-[3rem] resize-y z-10 focus:outline-none",
                            placeholder: "> Type here",
                            oninput: move |event| {
                                let value = event.value();
                                if let Some(memory_io_mut) = emulator_state
                                    .write()
                                    .as_mut()
                                    .map(|e| e.memory_io_mut())
                                {
                                    let i = memory_io_mut.get_serial_cursor();
                                    let new_value = String::from_utf8_lossy(
                                            &memory_io_mut.get_serial_input()[..i],
                                        )
                                        .to_string() + if value.len() > i { &value[i..] } else { "" };
                                    dioxus_logger::tracing::info!(
                                        "Serial input: {}\nNew text: {}\nCursor: {}",
                                        String::from_utf8_lossy(memory_io_mut.get_serial_input()), new_value, i
                                    );
                                    memory_io_mut.set_serial_input(new_value.as_bytes());
                                    input_text
                                        .set(
                                            String::from_utf8_lossy(&memory_io_mut.get_serial_input())
                                                .to_string(),
                                        );
                                } else {
                                    input_text.set(value);
                                }
                            },
                            value: "{input_text}",
                        }
                    }
                    div { class: "whitespace-pre",
                        "{String::from_utf8_lossy(memory_io.get_serial_output()).to_string()}"
                    }
                }
            }
        }
    }
}
