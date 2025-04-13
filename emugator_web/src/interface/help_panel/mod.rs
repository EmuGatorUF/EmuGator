mod intro_page_view;
mod page_not_found_view;

use dioxus::prelude::*;

use intro_page_view::IntroPageView;
use page_not_found_view::PageNotFoundView;

#[component]
#[allow(non_snake_case)]
pub fn HelpPanelView() -> Element {
    // RS - Realistically speaking, this should just parse a markdown file and auto-generate
    // all of this

    let subsections = ["Introduction", "Two stage", "Five stage", "UART"];
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
                    _ => rsx!(PageNotFoundView {}),
                }
            }
        }
    }
}
