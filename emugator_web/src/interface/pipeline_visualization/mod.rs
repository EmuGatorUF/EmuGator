use cve2_visualization::CVE2Visualization;
use dioxus::{
    html::geometry::{
        Pixels,
        euclid::{Point2D, Rect, Size2D},
    },
    prelude::*,
};
use dioxus_elements::geometry::WheelDelta;
use dioxus_elements::input_data::MouseButton;
use dioxus_free_icons::Icon;
use dioxus_free_icons::icons::ld_icons::LdRotateCcw;
use emugator_core::emulator::{AnyEmulatorState, EmulatorOption};
use five_stage_visualization::FiveStageVisualization;
use std::rc::Rc;

mod cve2_visualization;
mod five_stage_visualization;

const SCROLL_MULTIPLIER: f64 = 1.1;

/// Calculates the SVG viewport based on the viewBox and dimensions assuming "xMidYMid meet"
fn svg_viewport(
    view_box: (f64, f64, f64, f64),
    dimensions: Rect<f64, Pixels>,
) -> Rect<f64, Pixels> {
    let (view_x, view_y, view_width, view_height) = view_box;
    let (element_width, element_height) = (dimensions.width(), dimensions.height());
    let (width, height) = if view_width / element_width > view_height / element_height {
        (view_width, view_width * element_height / element_width)
    } else {
        (view_height * element_width / element_height, view_height)
    };

    let (x, y) = (
        view_x + (view_width - width) / 2.0,
        view_y + (view_height - height) / 2.0,
    );
    Rect::new(Point2D::new(x, y), Size2D::new(width, height))
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

    // Get the bounding rect of the svg
    let mut bounding_rectangle = use_signal(|| None as Option<Rc<MountedData>>);
    let mut dimensions = use_signal(Rect::zero);
    let read_dims = move |_| async move {
        let read = bounding_rectangle.read();
        let client_rect = read.as_ref().map(|el| el.get_client_rect());

        if let Some(client_rect) = client_rect {
            if let Ok(rect) = client_rect.await {
                dimensions.set(rect);
                return Some(rect);
            }
        }
        None
    };

    rsx! {
        div { class: "w-full h-full rounded bg-white overflow-hidden relative select-none",
            onmounted: move |cx| { bounding_rectangle.set(Some(cx.data())); },
            button {
                class: "absolute top-2 left-2 bg-gray-200 hover:bg-gray-300 p-1 rounded z-10 cursor-pointer",
                title: "Recenter",
                onclick: move |_| {
                    view_box.set(initial_view);
                    scale.set(1.0);
                },
                Icon { width: 16, height: 16, icon: LdRotateCcw }
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
                preserve_aspect_ratio: "xMidYMid meet",
                xmlns: "httk://www.w3.org/2000/svg",
                style: format!(
                    "cursor: {}; user-select: none;",
                    if *is_panning.read() { "grabbing" } else { "default" },
                ),
                onwheel: move |e| {
                    e.stop_propagation();
                    e.prevent_default();
                    let delta = e.delta();
                    let scale_change = match delta {
                        WheelDelta::Pixels(y) => if y.y < 0.0 { SCROLL_MULTIPLIER } else { 1.0f64 / SCROLL_MULTIPLIER },
                        WheelDelta::Lines(y) => if y.y < 0.0 { SCROLL_MULTIPLIER } else { 1.0f64 / SCROLL_MULTIPLIER }
                        _ => 1.0,
                    };

                    let new_scale = *scale.read() * scale_change;
                    if 5.0 >= new_scale && new_scale >= 0.5 {
                        let _view_box = *view_box.read();
                        let dims = *dimensions.read();
                        let viewport = svg_viewport(
                            _view_box,
                            dims
                        );

                        let (u, v) = (e.element_coordinates().x / dims.width(), e.element_coordinates().y / dims.height());
                        let (anchor_x, anchor_y) = (
                            viewport.min_x() + u * viewport.width(),
                            viewport.min_y() + v * viewport.height(),
                        );

                        let initial_viewport = svg_viewport(initial_view, dims);
                        let initial_width = initial_viewport.width();
                        let initial_height = initial_viewport.height();

                        let new_width = initial_width / new_scale;
                        let new_height = initial_height / new_scale;
                        let new_x = anchor_x - u * new_width;
                        let new_y = anchor_y - v * new_height;

                        view_box.set((new_x, new_y, new_width, new_height));
                        scale.set(new_scale);
                    }
                },
                onmousedown: move |e| async move {
                    if e.held_buttons().contains(MouseButton::Primary) {
                        read_dims(e.clone()).await; // Get current dimensions when user clicks down. Should only trigger once.

                        is_panning.set(true);
                        let (view_x, view_y, _, _) = *view_box.read();
                        start_pan.set((e.element_coordinates().x, e.element_coordinates().y, view_x, view_y));
                    }
                },
                onmousemove: move |e|  {

                    if *is_panning.read() {
                        let _view_box = *view_box.read();
                        let dims = *dimensions.read();
                        let (_, _, width, height) = _view_box;

                        let viewport = svg_viewport(
                            _view_box,
                           dims
                        );

                        // Convert mouse coordinates to SVG coordinates
                        let mouse_coords = e.element_coordinates();
                        let (click_x, click_y, initial_x, initial_y) = *start_pan.read();
                        let dx = (mouse_coords.x - click_x) * viewport.width() / dims.width();
                        let dy = (mouse_coords.y - click_y) * viewport.height() / dims.height();
                        view_box.set((initial_x - dx, initial_y - dy, width, height));
                    }
                },
                onmouseup: move |_| is_panning.set(false),
                onmouseenter: move |e| async move {
                    read_dims(e.clone()).await; // Get current dimensions when mouse enters the viewport Should only trigger once.
                },
                onmouseleave: move |_| {
                    is_panning.set(false);
                },
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
                            CVE2Visualization { emulator_state, tooltip_text, show_control_signals }
                        },
                        EmulatorOption::FiveStage => rsx! {
                            FiveStageVisualization { emulator_state, tooltip_text, show_control_signals }
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
