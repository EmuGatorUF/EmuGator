use cve2_visualization::CVE2Visualization;
use dioxus::prelude::*;
use dioxus_elements::geometry::WheelDelta;
use dioxus_elements::input_data::MouseButton;
use emugator_core::emulator::{AnyEmulatorState, EmulatorOption};
use five_stage_visualization::FiveStageVisualization;

mod cve2_visualization;
mod five_stage_visualization;

fn format_pc(pc: u32) -> String {
    format!("0x{:08X}", pc)
}

#[component]
#[allow(non_snake_case)]
pub fn PipelineVisualization(
    emulator_state: ReadOnlySignal<Option<AnyEmulatorState>>,
    selected_emulator: ReadOnlySignal<EmulatorOption>,
) -> Element {
    let initial_view = (0.0, 0.0, 1261.0, 660.0);
    let mut view_box = use_signal(|| initial_view);
    let mut is_panning = use_signal(|| false);
    let mut start_pan = use_signal(|| (0.0, 0.0, 0.0, 0.0));
    let mut scale = use_signal(|| 1.0);
    let tooltip_text: Signal<Option<String>> = use_signal(|| None);
    let mut show_control_signals = use_signal(|| true);

    rsx! {
        div { class: "w-full h-full rounded bg-white overflow-hidden relative",
            button {
                class: "absolute top-2 left-2 bg-gray-200 hover:bg-gray-300 p-1 rounded z-10 cursor-pointer",
                onclick: move |_| {
                    view_box.set(initial_view);
                    scale.set(1.0);
                },
                svg {
                    width: "16",
                    height: "16",
                    view_box: "0 0 24 24",
                    stroke: "currentColor",
                    fill: "none",
                    "stroke-width": "2",
                    "stroke-linecap": "round",
                    "stroke-linejoin": "round",
                    path { d: "M3 12a9 9 0 1 0 9-9 9.75 9.75 0 0 0-6.74 2.74L3 8" }
                    path { d: "M3 3v5h5" }
                }
            }
            button {
                class: format!(
                    "absolute top-2 left-12 p-1 rounded z-10 cursor-pointer {}",
                    if *show_control_signals.read() {
                        "bg-blue-500 text-white"
                    } else {
                        "bg-gray-200 text-white"
                    },
                ),
                onclick: move |_| {
                    let current = *show_control_signals.read();
                    show_control_signals.set(!current);
                },
                title: if *show_control_signals.read() { "Hide Control Signals" } else { "Show Control Signals" },
                svg {
                    width: "16",
                    height: "16",
                    view_box: "0 0 24 24",
                    stroke: "currentColor",
                    fill: "none",
                    "stroke-width": "2",
                    "stroke-linecap": "round",
                    "stroke-linejoin": "round",
                    path { d: "M22 12H2" }
                    path { d: "M5 12V4" }
                    path { d: "M19 12v7" }
                    path { d: "M5 19v1" }
                    path { d: "M19 5V4" }
                }
            }
            svg {
                width: "100%",
                height: "100%",
                view_box: format!(
                    "{} {} {} {}",
                    view_box.read().0,
                    view_box.read().1,
                    view_box.read().2,
                    view_box.read().3,
                ),
                xmlns: "http://www.w3.org/2000/svg",
                style: format!(
                    "cursor: {}; user-select: none;",
                    if *is_panning.read() { "grabbing" } else { "default" },
                ),
                onwheel: move |e| {
                    e.stop_propagation();
                    e.prevent_default();
                    let delta = e.delta();
                    let scale_change = match delta {
                        WheelDelta::Pixels(y) => if y.y < 0.0 { 1.25 } else { 0.8 }
                        WheelDelta::Lines(y) => if y.y < 0.0 { 1.25 } else { 0.8 }
                        _ => 1.0,
                    };
                    let new_scale = *scale.read() * scale_change;
                    if new_scale >= 0.5 {
                        let (old_x, old_y, old_width, old_height) = *view_box.read();
                        let center_x = old_x + old_width / 2.0;
                        let center_y = old_y + old_height / 2.0;
                        let new_width = 1261.0 / new_scale;
                        let new_height = 660.0 / new_scale;
                        let new_x = center_x - new_width / 2.0;
                        let new_y = center_y - new_height / 2.0;
                        view_box.set((new_x, new_y, new_width, new_height));
                        scale.set(new_scale);
                    }
                },
                onmousedown: move |e| {
                    if e.held_buttons().contains(MouseButton::Primary) {
                        is_panning.set(true);
                        let (view_x, view_y, _, _) = *view_box.read();
                        start_pan
                            .set((
                                e.client_coordinates().x,
                                e.client_coordinates().y,
                                view_x,
                                view_y,
                            ));
                    }
                },
                onmousemove: move |e| {
                    if *is_panning.read() {
                        let (start_x, start_y, initial_x, initial_y) = *start_pan.read();
                        let (_, _, width, height) = *view_box.read();
                        let dx = (e.client_coordinates().x - start_x) * width / 200.0;
                        let dy = (e.client_coordinates().y - start_y) * height / 100.0;
                        view_box.set((initial_x - dx, initial_y - dy, width, height));
                    }
                },
                onmouseup: move |_| is_panning.set(false),
                onmouseleave: move |_| is_panning.set(false),
                // Background
                rect {
                    id: "background",
                    width: "1261",
                    height: "660",
                    fill: "white",
                }
                {
                    match *selected_emulator.read() {
                        EmulatorOption::CVE2 => rsx! {
                            CVE2Visualization { emulator_state, tooltip_text }
                        },
                        EmulatorOption::FiveStage => rsx! {
                            FiveStageVisualization { emulator_state, tooltip_text }
                        },
                    }
                }
            }
            if let Some(tooltip_text) = &*tooltip_text.read() {
                div {
                    class: "absolute bg-black text-white px-2 py-1 rounded text-sm",
                    style: "top: 10px; right: 10px; z-index: 100;",
                    "{tooltip_text}"
                }
            }
        }
    }
}
