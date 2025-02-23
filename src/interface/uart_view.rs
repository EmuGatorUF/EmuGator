use dioxus::prelude::*;
use dioxus::{prelude::component, signals::Signal};

use crate::uart::Uart;

#[component]
#[allow(non_snake_case)]
pub fn UartView(uart_module: Signal<Uart>, minimize_console: Signal<bool>) -> Element {
    rsx! {
        div { class: "flex-col bg-inherit text-gray-200 font-mono border-t-[0.450px] border-gray-600",
            div {
                div { class: "flex flex-grow p-2 items-center align-center justify-between",
                    "UART Console",
                    button { class: "flex text-sm items-center justify-center text-center text-sm bg-inherit px-2 hover:outline outline-gray-400 rounded shadow ".to_owned() + {if *minimize_console.read() { "" } else { "origin-center rotate-180" }},
                        onclick: move |_| {
                            // For some reason, this can't be a single line :|
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
            div { class: if *minimize_console.read() { "h-0" } else { "p-3 whitespace-pre" },
                "{uart_module.read().to_string()}"
            }
        }
    }
}
