use dioxus::prelude::*;

use dioxus_free_icons::Icon;
use dioxus_free_icons::icons::ld_icons::LdTriangleAlert;

#[component]
#[allow(non_snake_case)]
pub fn PageNotFoundView() -> Element {
    rsx!(
        div { class: "flex flex-col h-full items-center justify-center",
            Icon {
                width: 400,
                icon: LdTriangleAlert
            }
            "404 - Page Not Found"
        }
    )
}
