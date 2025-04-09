use dioxus::prelude::*;

#[component]
#[allow(non_snake_case)]
pub fn HelpPanelView() -> Element {
    rsx!{
        div { class: "flex min-h-screen text-gray-800 bg-gray-100 w-1/2",
            aside { class: "w-1/4 bg-gray-900 text-white p-4 sticky top-0 h-screen overflow-hidden z-40",
                h2 { class: "text-xl font-semibold mb-4", "Documentation" }
                nav { class: "space-y-1",
                    a {
                        class: "block px-2 py-1 text-gray-300 hover:bg-gray-700 hover:text-white rounded",
                        href: "#introduction",
                        "Introduction"
                    }
                    a {
                        class: "block px-2 py-1 text-gray-300 hover:bg-gray-700 hover:text-white rounded",
                        href: "#installation",
                        "Installation"
                    }
                    a {
                        class: "block px-2 py-1 text-gray-300 hover:bg-gray-700 hover:text-white rounded",
                        href: "#usage",
                        "Usage"
                    }
                    a {
                        class: "block px-2 py-1 text-gray-300 hover:bg-gray-700 hover:text-white rounded",
                        href: "#api",
                        "API"
                    }
                    a {
                        class: "block px-2 py-1 text-gray-300 hover:bg-gray-700 hover:text-white rounded",
                        //onclick: |event| {
                        //    event.prevent_default();
                        //
                        //    document
                        //},
                        //href: "#faq",
                        "FAQ",
                    }
                }
            }
            div { class: "flex-1 p-8 overflow-hidden",
                div { class: "mb-16", id: "introduction",
                    h3 { class: "text-2xl font-bold mb-4 border-b-2 border-gray-300 pb-2",
                        "Introduction"
                    }
                    p { class: "leading-relaxed mb-4",
                        "Welcome to the documentation for your app. This page will guide you through setup, usage, and more."
                    }
                }
                div { class: "mb-16", id: "installation",
                    h3 { class: "text-2xl font-bold mb-4 border-b-2 border-gray-300 pb-2",
                        "Installation"
                    }
                    p { class: "leading-relaxed mb-4",
                        "Instructions on how to install your app go here."
                    }
                }
                div { class: "mb-16", id: "usage",
                    h3 { class: "text-2xl font-bold mb-4 border-b-2 border-gray-300 pb-2",
                        "Usage"
                    }
                    p { class: "leading-relaxed mb-4",
                        "Explain how to use the core features of your app."
                    }
                }
                div { class: "mb-16", id: "api",
                    h3 { class: "text-2xl font-bold mb-4 border-b-2 border-gray-300 pb-2",
                        "API"
                    }
                    p { class: "leading-relaxed mb-4", "Provide API references and examples." }
                }
                div { class: "mb-16", id: "faq",
                    h3 { class: "text-2xl font-bold mb-4 border-b-2 border-gray-300 pb-2",
                        id: "faq-id",
                        "FAQ"
                    }
                    p { class: "leading-relaxed mb-4",
                        "Answer common questions and troubleshooting tips here."
                    }
                }
            }
        }
    }
}
