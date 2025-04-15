mod assembler_page_view;
mod interface_page_view;
mod intro_page_view;
mod page_not_found_view;
mod quick_start_view;
mod two_stage_view;
mod five_stage_view;
mod uart_view;

use dioxus::prelude::*;

use assembler_page_view::AssemblerPageView;
use interface_page_view::InterfacePageView;
use intro_page_view::IntroPageView;
use page_not_found_view::PageNotFoundView;
use quick_start_view::QuickStartView;
use two_stage_view::TwoStageView;
use five_stage_view::FiveStageView;
use uart_view::UartView;

// Style class constants
pub const H3_STYLE: &str = "text-3xl font-bold mb-4 border-b-2 border-gray-300 pb-2";
pub const H4_STYLE: &str = "text-xl font-semibold mb-2 mt-1";
pub const H5_STYLE: &str = "text-md font-semibold mb-2 mt-1";
pub const P_STYLE: &str = "leading-relaxed mb-2 text-sm";

#[component]
#[allow(non_snake_case)]
pub fn HelpPanelView() -> Element {
    // RS - Realistically speaking, this should just parse a markdown file and auto-generate
    // all of this

    let subsections = [
        "Introduction",
        "Quick Start",
        "Interface",
        "Assembler and Instructions",
        "Two stage",
        "Five stage",
        "UART",
    ];
    let mut displayed_menu = use_signal(|| subsections[0]);

    rsx! {
        div { class: "flex text-gray-800 bg-gray-100 w-1/2",
            aside { class: "w-1/4 bg-gray-900 text-white p-4",
                h2 { class: "text-sm xl:text-xl font-semibold mb-4", "Documentation" }
                nav { class: "space-y-1 text-gray-300",
                    for subsection in subsections {
                        div {
                            class: "text-sm block px-2 py-1 hover:bg-gray-700 hover:text-white rounded cursor-pointer",
                            onclick: move |_| { displayed_menu.set(subsection) },
                            "{subsection}"
                        }
                    }
                }
            }
            div { class: "flex-1 p-8 overflow-y-auto",
                match *displayed_menu.read() {
                    "Introduction" => rsx!(IntroPageView {}),
                    "Quick Start" => rsx!(QuickStartView {}),
                    "Interface" => rsx!(InterfacePageView {}),
                    "Assembler and Instructions" => rsx!(AssemblerPageView {}),
                    "Two stage" => rsx!(TwoStageView {}),
                    "Five stage" => rsx!(FiveStageView {}),
                    "UART" => rsx!(UartView {}),
                    _ => rsx!(PageNotFoundView {}),
                }
            }
        }
    }
}
