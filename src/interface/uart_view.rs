use dioxus::{prelude::component, signals::Signal};
use dioxus::prelude::*;

use crate::uart::Uart;

#[component]
#[allow(non_snake_case)]
pub fn UartView(uart_module: Signal<Uart>) -> Element {
    rsx! {
        div { class: "flex-col h-full bg-[#1E1E1E] text-gray-200 font-mono border-t-[0.450px] border-gray-600",
            div { class: "font-lg",
                div { class: "p-2",
                    "UART Console",
                }
                hr {}
            }
            div { class: "p-3",
                "{uart_module.read().to_string()}"
            }
        }
    }
}
