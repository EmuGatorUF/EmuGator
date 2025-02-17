use dioxus::prelude::*;
use dioxus::{prelude::component, signals::Signal};
use dioxus_logger::tracing::info;

use crate::uart::Uart;

#[component]
#[allow(non_snake_case)]
pub fn UartView(uart_module: Signal<Uart>, minimize_console: Signal<bool>) -> Element {
    rsx! {
        div { class: "flex-col bg-inherit text-gray-200 font-mono border-t-[0.450px] border-gray-600",
            div {
                div { class: "flex flex-grow p-2 items-center align-center justify-between",
                    "UART Console",
                    button { class: "flex items-center justify-center text-sm bg-gray-500 py-1 px-2 border border-gray-400 rounded shadow",
                        onclick: move |_| {
                            info!("Button Pressed! {:?}", *minimize_console.read());
                            // For some reason, this can't be a single line :|
                            let is_minimized = *minimize_console.read();
                            minimize_console.set(!is_minimized);
                        },
                        "^"
                    }
                }
                if !*minimize_console.read() {
                    hr {}
                }
            }
            div { class: if *minimize_console.read() { "h-0" } else { "p-3" },
                "{uart_module.read().to_string()}"
            }
        }
    }
}
