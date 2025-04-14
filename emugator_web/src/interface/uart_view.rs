use dioxus::prelude::*;
use dioxus::{prelude::component, signals::Signal};
use emugator_core::emulator::AnyEmulatorState;

use dioxus_free_icons::Icon;
use dioxus_free_icons::icons::ld_icons::{LdChevronDown, LdChevronUp};

#[component]
#[allow(non_snake_case)]
pub fn UartView(
    emulator_state: Signal<Option<AnyEmulatorState>>,
    serial_input: Signal<String>,
    minimize_console: Signal<bool>,
) -> Element {
    let state = emulator_state.read();

    let icon_width = 25;
    rsx! {
        div { class: "flex-col bg-inherit text-gray-200 font-mono border-t-[0.450px] border-gray-600 h-full",
            div {
                div { class: "flex flex-grow font-bold p-2 items-center align-center justify-between",
                    "UART Console"
                    button {
                        class: "flex items-center justify-center text-center text-sm bg-inherit px-2 hover:outline outline-gray-400 rounded cursor-pointer",
                        onclick: move |_| {
                            let is_minimized = *minimize_console.read();
                            minimize_console.set(!is_minimized);
                        },
                        if *minimize_console.read() {
                            Icon { width: icon_width, icon: LdChevronUp }
                        } else {
                            Icon { width: icon_width, icon: LdChevronDown }
                        }
                    }
                }
                if !*minimize_console.read() {
                    hr {}
                }
            }
            div { class: if *minimize_console.read() { "h-0" } else { "flex h-full" },
                div { class: "flex flex-col w-full h-full",
                    div { class: "p-2 font-semibold border-b border-r", "Serial Input" }
                    textarea {
                        class: "flex-1 leading-none p-3 border-r h-full resize-none overflow-auto focus:outline-none",
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
                                memory_io_mut.set_serial_input(new_value.as_bytes());
                                serial_input
                                    .set(
                                        String::from_utf8_lossy(memory_io_mut.get_serial_input()).to_string(),
                                    );
                            } else {
                                serial_input.set(value);
                            }
                        },
                        value: "{serial_input}",
                    }
                }
                div { class: "flex flex-col h-full",
                    div { class: "w-full p-2 font-semibold border-b border-l", "Serial Output" }
                    textarea {
                        class: "flex-1 leading-none p-3 border-l h-full resize-none overflow-auto focus:outline-none",
                        placeholder: "> UART Output",
                        readonly: "true",
                        value: if let Some(memory_io) = state.as_ref().map(|e| e.memory_io()) { String::from_utf8_lossy(memory_io.get_serial_output()).to_string() } else { "".to_string() },
                    }
                }
            }
        }
    }
}
