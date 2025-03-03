use crate::emulator::EmulatorState;
use dioxus::prelude::*;
use dioxus_elements::geometry::WheelDelta;
use dioxus_elements::input_data::MouseButton;

fn format_pc(pc: u32) -> String {
    format!("0x{:08X}", pc)
}

#[component]
#[allow(non_snake_case)]
pub fn DatapathVisualization(emulator_state: Signal<EmulatorState>) -> Element {
    let mut hovered_element = use_signal(|| Option::<String>::None);
    let initial_view = (0.0, 0.0, 1261.0, 660.0);
    let mut view_box = use_signal(|| initial_view);
    let mut is_panning = use_signal(|| false);
    let mut start_pan = use_signal(|| (0.0, 0.0, 0.0, 0.0));
    let mut scale = use_signal(|| 1.0);

    rsx! {
        div { class: "w-full h-full bg-white overflow-hidden relative",
            button {
                class: "absolute top-2 left-2 bg-gray-200 hover:bg-gray-300 p-1 rounded z-10",
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
                                e.client_coordinates().x as f64,
                                e.client_coordinates().y as f64,
                                view_x,
                                view_y,
                            ));
                    }
                },
                onmousemove: move |e| {
                    if *is_panning.read() {
                        let (start_x, start_y, initial_x, initial_y) = *start_pan.read();
                        let (_, _, width, height) = *view_box.read();
                        let dx = (e.client_coordinates().x as f64 - start_x) * width / 200.0;
                        let dy = (e.client_coordinates().y as f64 - start_y) * height / 100.0;
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
                // Rectangles & their labels
                rect {
                    id: "pc",
                    x: "21",
                    y: "261",
                    width: "78",
                    height: "158",
                    stroke: "black",
                    "stroke-width": "2",
                    fill: "transparent",
                    onmouseenter: move |_| {
                        hovered_element.set(Some("ifpc".to_string()));
                    },
                    onmouseleave: move |_| {
                        hovered_element.set(None);
                    },
                }
                text {
                    id: "pc_label",
                    x: "60",
                    y: "340",
                    "text-anchor": "middle",
                    "dominant-baseline": "middle",
                    "font-size": "20",
                    "font-weight": "bold",
                    fill: "black",
                    "PC"
                }
                rect {
                    id: "plus4",
                    x: "119",
                    y: "60",
                    width: "38",
                    height: "38",
                    stroke: "black",
                    "stroke-width": "2",
                    fill: "none",
                }
                text {
                    id: "plus4_label",
                    x: "138",
                    y: "79",
                    "text-anchor": "middle",
                    "dominant-baseline": "middle",
                    "font-size": "20",
                    "font-weight": "bold",
                    fill: "black",
                    "+4"
                }
                rect {
                    id: "id_pc",
                    x: "421",
                    y: "101",
                    width: "78",
                    height: "78",
                    stroke: "black",
                    "stroke-width": "2",
                    fill: "none",
                }
                text {
                    id: "instruction_memory_label",
                    x: "460",
                    y: "140",
                    "text-anchor": "middle",
                    "dominant-baseline": "middle",
                    "font-size": "20",
                    "font-weight": "bold",
                    fill: "black",
                    "ID PC"
                }
                rect {
                    id: "id_ir",
                    x: "421",
                    y: "179",
                    width: "78",
                    height: "78",
                    stroke: "black",
                    "stroke-width": "2",
                    fill: "none",
                }
                text {
                    id: "id_ir_label",
                    x: "460",
                    y: "218",
                    "text-anchor": "middle",
                    "dominant-baseline": "middle",
                    "font-size": "20",
                    "font-weight": "bold",
                    fill: "black",
                    "ID IR"
                }
                rect {
                    id: "decoder",
                    x: "581",
                    y: "181",
                    width: "158",
                    height: "78",
                    stroke: "black",
                    "stroke-width": "2",
                    fill: "none",
                }
                text {
                    id: "decoder_label",
                    x: "660",
                    y: "220",
                    "text-anchor": "middle",
                    "dominant-baseline": "middle",
                    "font-size": "20",
                    "font-weight": "bold",
                    fill: "black",
                    "Decoder"
                }
                rect {
                    id: "controller",
                    x: "1",
                    y: "581",
                    width: "1255",
                    height: "78",
                    stroke: "black",
                    "stroke-width": "2",
                    fill: "none",
                }
                text {
                    id: "controller_label",
                    x: "628",
                    y: "620",
                    "text-anchor": "middle",
                    "dominant-baseline": "middle",
                    "font-size": "20",
                    "font-weight": "bold",
                    fill: "black",
                    "Controller"
                }
                rect {
                    id: "instruction_memory",
                    x: "181",
                    y: "261",
                    width: "158",
                    height: "158",
                    stroke: "black",
                    "stroke-width": "2",
                    fill: "none",
                }
                text {
                    id: "instruction_memory_label",
                    x: "260",
                    y: "330",
                    "text-anchor": "middle",
                    "dominant-baseline": "middle",
                    "font-size": "20",
                    "font-weight": "bold",
                    fill: "black",
                    "Instruction"
                }
                text {
                    id: "instruction_memory_label2",
                    x: "260",
                    y: "350",
                    "text-anchor": "middle",
                    "dominant-baseline": "middle",
                    "font-size": "20",
                    "font-weight": "bold",
                    fill: "black",
                    "Memory"
                }
                rect {
                    id: "if_id_buffer",
                    x: "421",
                    y: "101",
                    width: "78",
                    height: "438",
                    stroke: "black",
                    "stroke-width": "2",
                    fill: "none",
                }
                text {
                    id: "if_id_buffer_label",
                    x: "460",
                    y: "520",
                    "text-anchor": "middle",
                    "dominant-baseline": "middle",
                    "font-size": "20",
                    "font-weight": "bold",
                    fill: "black",
                    "IF / ID"
                }
                rect {
                    id: "register_file",
                    x: "581",
                    y: "341",
                    width: "158",
                    height: "158",
                    stroke: "black",
                    "stroke-width": "2",
                    fill: "none",
                }
                text {
                    id: "register_file_label",
                    x: "660",
                    y: "408",
                    "text-anchor": "middle",
                    "dominant-baseline": "middle",
                    "font-size": "20",
                    "font-weight": "bold",
                    fill: "black",
                    "Register"
                }
                text {
                    id: "register_file_label2",
                    x: "660",
                    y: "432",
                    "text-anchor": "middle",
                    "dominant-baseline": "middle",
                    "font-size": "20",
                    "font-weight": "bold",
                    fill: "black",
                    "File"
                }
                text {
                    id: "rs1_input_label",
                    x: "609",
                    y: "355",
                    "text-anchor": "start",
                    "dominant-baseline": "middle",
                    "font-size": "12",
                    fill: "black",
                    "RS1"
                }
                text {
                    id: "rs2_input_label",
                    x: "649",
                    y: "355",
                    "text-anchor": "start",
                    "dominant-baseline": "middle",
                    "font-size": "12",
                    fill: "black",
                    "RS2"
                }
                text {
                    id: "rd_input_label",
                    x: "692",
                    y: "355",
                    "text-anchor": "start",
                    "dominant-baseline": "middle",
                    "font-size": "12",
                    fill: "black",
                    "RD"
                }
                text {
                    id: "rd_v_label",
                    x: "585",
                    y: "445",
                    "text-anchor": "start",
                    "dominant-baseline": "middle",
                    "font-size": "12",
                    fill: "black",
                    "RD_V"
                }
                text {
                    id: "rs1_v_label",
                    x: "700",
                    y: "428",
                    "text-anchor": "start",
                    "dominant-baseline": "middle",
                    "font-size": "12",
                    fill: "black",
                    "RS1_V"
                }
                text {
                    id: "rs2_v_label",
                    x: "700",
                    y: "445",
                    "text-anchor": "start",
                    "dominant-baseline": "middle",
                    "font-size": "12",
                    fill: "black",
                    "RS2_V"
                }
                rect {
                    id: "alu",
                    x: "961",
                    y: "140",
                    width: "158",
                    height: "78",
                    stroke: "black",
                    "stroke-width": "2",
                    fill: "none",
                }
                text {
                    id: "alu_label",
                    x: "1040",
                    y: "179",
                    "text-anchor": "middle",
                    "dominant-baseline": "middle",
                    "font-size": "20",
                    "font-weight": "bold",
                    fill: "black",
                    "ALU"
                }
                rect {
                    id: "lsu",
                    x: "961",
                    y: "318",
                    width: "282",
                    height: "78",
                    stroke: "black",
                    "stroke-width": "2",
                    fill: "none",
                }
                text {
                    id: "lsu_label",
                    x: "1040",
                    y: "357",
                    "text-anchor": "middle",
                    "dominant-baseline": "middle",
                    "font-size": "20",
                    "font-weight": "bold",
                    fill: "black",
                    "LSU"
                }
                rect {
                    id: "data_memory",
                    x: "961",
                    y: "423",
                    width: "282",
                    height: "78",
                    stroke: "black",
                    "stroke-width": "2",
                    fill: "none",
                }
                text {
                    id: "data_memory_label",
                    x: "1080",
                    y: "462",
                    "text-anchor": "middle",
                    "dominant-baseline": "middle",
                    "font-size": "20",
                    "font-weight": "bold",
                    fill: "black",
                    "Data Memory"
                }
                text {
                    id: "addr_label",
                    x: "995",
                    y: "436",
                    "text-anchor": "middle",
                    "dominant-baseline": "middle",
                    "font-size": "12",
                    fill: "black",
                    "ADDR"
                }
                text {
                    id: "data_label",
                    x: "1045",
                    y: "436",
                    "text-anchor": "middle",
                    "dominant-baseline": "middle",
                    "font-size": "12",
                    fill: "black",
                    "DATA"
                }
                text {
                    id: "req_label",
                    x: "1095",
                    y: "436",
                    "text-anchor": "middle",
                    "dominant-baseline": "middle",
                    "font-size": "12",
                    fill: "black",
                    "REQ"
                }
                text {
                    id: "wr_label",
                    x: "1135",
                    y: "436",
                    "text-anchor": "middle",
                    "dominant-baseline": "middle",
                    "font-size": "12",
                    fill: "black",
                    "WR"
                }
                text {
                    id: "byte_en_label",
                    x: "1175",
                    y: "436",
                    "text-anchor": "middle",
                    "dominant-baseline": "middle",
                    "font-size": "12",
                    fill: "black",
                    "BEN"
                }
                text {
                    id: "valid_label",
                    x: "1215",
                    y: "436",
                    "text-anchor": "middle",
                    "dominant-baseline": "middle",
                    "font-size": "12",
                    fill: "black",
                    "VALID"
                }
                // Muxes and their input labels
                path {
                    id: "pc_mux",
                    d: "M79 118.382L21 89.382L21 30.618L79 1.61803L79 118.382Z",
                    stroke: "black",
                    "stroke-width": "2",
                    fill: "none",
                }
                text {
                    id: "pcmux_input1_label",
                    x: "75",
                    y: "38",
                    "text-anchor": "end",
                    "dominant-baseline": "middle",
                    "font-size": "12",
                    fill: "black",
                    "PC+IMM"
                }
                text {
                    id: "pcmux_input2_label",
                    x: "75",
                    y: "79",
                    "text-anchor": "end",
                    "dominant-baseline": "middle",
                    "font-size": "12",
                    fill: "black",
                    "PC+4"
                }
                path {
                    id: "opa_mux",
                    d: "M821 109.618L879 138.618V197.382L821 226.382V109.618Z",
                    stroke: "black",
                    "stroke-width": "2",
                    fill: "none",
                }
                text {
                    id: "opa_mux_input1_label",
                    x: "824",
                    y: "138",
                    "text-anchor": "start",
                    "dominant-baseline": "middle",
                    "font-size": "12",
                    fill: "black",
                    "PC"
                }
                text {
                    id: "rs1_mux_input1_label",
                    x: "824",
                    y: "198",
                    "text-anchor": "start",
                    "dominant-baseline": "middle",
                    "font-size": "12",
                    fill: "black",
                    "RS1"
                }
                path {
                    id: "write_mux",
                    d: "M1302 248.618L1360 277.618V336.382L1302 365.382V248.618Z",
                    stroke: "black",
                    "stroke-width": "2",
                    fill: "none",
                }
                path {
                    id: "opb_mux",
                    d: "M821 246.618L879 275.618V334.382L821 363.382V246.618Z",
                    stroke: "black",
                    "stroke-width": "2",
                    fill: "none",
                }
                text {
                    id: "imm_mux_input1_label",
                    x: "824",
                    y: "288",
                    "text-anchor": "start",
                    "dominant-baseline": "middle",
                    "font-size": "12",
                    fill: "black",
                    "IMM"
                }
                text {
                    id: "rs2_mux_input1_label",
                    x: "824",
                    y: "335",
                    "text-anchor": "start",
                    "dominant-baseline": "middle",
                    "font-size": "12",
                    fill: "black",
                    "RS2"
                }
                // Arrows
                path {
                    id: "ifpc_to_im_arrow",
                    d: "M179.707 340.707C180.098 340.317 180.098 339.683 179.707 339.293L173.343 332.929C172.953 332.538 172.319 332.538 171.929 332.929C171.538 333.319 171.538 333.953 171.929 334.343L177.586 340L171.929 345.657C171.538 346.047 171.538 346.681 171.929 347.071C172.319 347.462 172.953 347.462 173.343 347.071L179.707 340.707ZM100 341H179V339H100V341Z",
                    fill: "black",
                    style: "stroke: transparent; stroke-width: 10px; pointer-events: all;",
                    onmouseenter: move |_| {
                        hovered_element.set(Some("ifpc".to_string()));
                    },
                    onmouseleave: move |_| {
                        hovered_element.set(None);
                    },
                }
                path {
                    id: "im_to_ifid_arrow",
                    d: "M419.707 340.707C420.098 340.317 420.098 339.683 419.707 339.293L413.343 332.929C412.953 332.538 412.319 332.538 411.929 332.929C411.538 333.319 411.538 333.953 411.929 334.343L417.586 340L411.929 345.657C411.538 346.047 411.538 346.681 411.929 347.071C412.319 347.462 412.953 347.462 413.343 347.071L419.707 340.707ZM340 341H419V339H340V341Z",
                    fill: "black",
                }
                path {
                    id: "ifpc_to_ifid_arrow",
                    d: "M419.707 140.707C420.098 140.317 420.098 139.683 419.707 139.293L413.343 132.929C412.953 132.538 412.319 132.538 411.929 132.929C411.538 133.319 411.538 133.953 411.929 134.343L417.586 140L411.929 145.657C411.538 146.047 411.538 146.681 411.929 147.071C412.319 147.462 412.953 147.462 413.343 147.071L419.707 140.707ZM138 141L419 141V139L138 139V141Z",
                    fill: "black",
                    style: "stroke: transparent; stroke-width: 10px; pointer-events: all;",
                    onmouseenter: move |_| {
                        hovered_element.set(Some("ifpc".to_string()));
                    },
                    onmouseleave: move |_| {
                        hovered_element.set(None);
                    },
                }
                path {
                    id: "pcmux_to_pc_arrow",
                    d: "M20.7071 340.707C21.0976 340.317 21.0976 339.683 20.7071 339.293L14.3431 332.929C13.9526 332.538 13.3195 332.538 12.9289 332.929C12.5384 333.319 12.5384 333.953 12.9289 334.343L18.5858 340L12.9289 345.657C12.5384 346.047 12.5384 346.681 12.9289 347.071C13.3195 347.462 13.9526 347.462 14.3431 347.071L20.7071 340.707ZM0 341H20V339H0V341Z",
                    fill: "black",
                }
                path {
                    id: "im_to_ifid_arrow",
                    d: "M579.707 218.707C580.098 218.317 580.098 217.683 579.707 217.293L573.343 210.929C572.953 210.538 572.319 210.538 571.929 210.929C571.538 211.319 571.538 211.953 571.929 212.343L577.586 218L571.929 223.657C571.538 224.047 571.538 224.681 571.929 225.071C572.319 225.462 572.953 225.462 573.343 225.071L579.707 218.707ZM500 219H579V217H500V219Z",
                    fill: "black",
                }
                path {
                    id: "decoder_to_rs1_arrow",
                    d: "M619.293 339.707C619.683 340.098 620.317 340.098 620.707 339.707L627.071 333.343C627.462 332.953 627.462 332.319 627.071 331.929C626.681 331.538 626.047 331.538 625.657 331.929L620 337.586L614.343 331.929C613.953 331.538 613.319 331.538 612.929 331.929C612.538 332.319 612.538 332.953 612.929 333.343L619.293 339.707ZM619 260L619 339L621 339L621 260L619 260Z",
                    fill: "black",
                }
                path {
                    id: "decoder_to_rs2_arrow",
                    d: "M659.293 339.707C659.683 340.098 660.317 340.098 660.707 339.707L667.071 333.343C667.462 332.953 667.462 332.319 667.071 331.929C666.681 331.538 666.047 331.538 665.657 331.929L660 337.586L654.343 331.929C653.953 331.538 653.319 331.538 652.929 331.929C652.538 332.319 652.538 332.953 652.929 333.343L659.293 339.707ZM659 260L659 339L661 339L661 260L659 260Z",
                    fill: "black",
                }
                path {
                    id: "decoder_to_rd_arrow",
                    d: "M699.293 339.707C699.683 340.098 700.317 340.098 700.707 339.707L707.071 333.343C707.462 332.953 707.462 332.319 707.071 331.929C706.681 331.538 706.047 331.538 705.657 331.929L700 337.586L694.343 331.929C693.953 331.538 693.319 331.538 692.929 331.929C692.538 332.319 692.538 332.953 692.929 333.343L699.293 339.707ZM699 260L699 339L701 339L701 260L699 260Z",
                    fill: "black",
                }
                path {
                    id: "ifidpc_to_opa_mux_arrow",
                    d: "M819.707 140.707C820.098 140.317 820.098 139.683 819.707 139.293L813.343 132.929C812.953 132.538 812.319 132.538 811.929 132.929C811.538 133.319 811.538 133.953 811.929 134.343L817.586 140L811.929 145.657C811.538 146.047 811.538 146.681 811.929 147.071C812.319 147.462 812.953 147.462 813.343 147.071L819.707 140.707ZM500 141H819V139H500V141Z",
                    fill: "black",
                }
                path {
                    id: "rf_to_rs1_mux_arrow",
                    d: "M819.707 198.707C820.098 198.317 820.098 197.683 819.707 197.293L813.343 190.929C812.953 190.538 812.319 190.538 811.929 190.929C811.538 191.319 811.538 191.953 811.929 192.343L817.586 198L811.929 203.657C811.538 204.047 811.538 204.681 811.929 205.071C812.319 205.462 812.953 205.462 813.343 205.071L819.707 198.707ZM758 199L819 199V197L758 197V199Z",
                    fill: "black",
                }
                path {
                    id: "rf_to_rs2_mux_arrow",
                    d: "M819.707 334.707C820.098 334.317 820.098 333.683 819.707 333.293L813.343 326.929C812.953 326.538 812.319 326.538 811.929 326.929C811.538 327.319 811.538 327.953 811.929 328.343L817.586 334L811.929 339.657C811.538 340.047 811.538 340.681 811.929 341.071C812.319 341.462 812.953 341.462 813.343 341.071L819.707 334.707ZM790 335H819V333H790V335Z",
                    fill: "black",
                }
                line {
                    id: "rf_to_rs2_mux_line",
                    x1: "790",
                    y1: "333",
                    x2: "790",
                    y2: "446",
                    stroke: "black",
                    "stroke-width": "2",
                }
                line {
                    id: "rf_to_rs2_mux_horizontal_line",
                    x1: "790",
                    y1: "445",
                    x2: "740",
                    y2: "445",
                    stroke: "black",
                    "stroke-width": "2",
                }
                path {
                    id: "decoder_to_imm_arrow",
                    d: "M819.707 286.707C820.098 286.317 820.098 285.683 819.707 285.293L813.343 278.929C812.953 278.538 812.319 278.538 811.929 278.929C811.538 279.319 811.538 279.953 811.929 280.343L817.586 286L811.929 291.657C811.538 292.047 811.538 292.681 811.929 293.071C812.319 293.462 812.953 293.462 813.343 293.071L819.707 286.707ZM778 287H819V285H778V287Z",
                    fill: "black",
                }
                path {
                    id: "opamux_to_alu_arrow",
                    d: "M959.707 167.707C960.098 167.317 960.098 166.683 959.707 166.293L953.343 159.929C952.953 159.538 952.319 159.538 951.929 159.929C951.538 160.319 951.538 160.953 951.929 161.343L957.586 167L951.929 172.657C951.538 173.047 951.538 173.681 951.929 174.071C952.319 174.462 952.953 174.462 953.343 174.071L959.707 167.707ZM880 168H959V166H880V168Z",
                    fill: "black",
                }
                path {
                    id: "opbmux_to_alu_arrow",
                    d: "M959.707 199.707C960.098 199.317 960.098 198.683 959.707 198.293L953.343 191.929C952.953 191.538 952.319 191.538 951.929 191.929C951.538 192.319 951.538 192.953 951.929 193.343L957.586 199L951.929 204.657C951.538 205.047 951.538 205.681 951.929 206.071C952.319 206.462 952.953 206.462 953.343 206.071L959.707 199.707ZM919 200H959V198H919V200Z",
                    fill: "black",
                }
                path {
                    id: "opbmux_to_lsu_arrow",
                    d: "M959.707 373.707C960.098 373.317 960.098 372.683 959.707 372.293L953.343 365.929C952.953 365.538 952.319 365.538 951.929 365.929C951.538 366.319 951.538 366.953 951.929 367.343L957.586 373L951.929 378.657C951.538 379.047 951.538 379.681 951.929 380.071C952.319 380.462 952.953 380.462 953.343 380.071L959.707 373.707ZM790 374H959V372H790V374Z",
                    fill: "black",
                }
                line {
                    id: "alu_to_junction_line",
                    x1: "1119",
                    y1: "179",
                    x2: "1220",
                    y2: "179",
                    stroke: "black",
                    "stroke-width": "2",
                }
                line {
                    id: "junction_to_mux_vertical",
                    x1: "1220",
                    y1: "178",
                    x2: "1220",
                    y2: "279",
                    stroke: "black",
                    "stroke-width": "2",
                }
                path {
                    id: "alu_to_writemux_arrow",
                    d: "M1300.71 279.707C1301.1 279.317 1301.1 278.683 1300.71 278.293L1294.34 271.929C1293.95 271.538 1293.32 271.538 1292.93 271.929C1292.54 272.319 1292.54 272.953 1292.93 273.343L1298.59 279L1292.93 284.657C1292.54 285.047 1292.54 285.681 1292.93 286.071C1293.32 286.462 1293.95 286.462 1294.34 286.071L1300.71 279.707ZM1220 280H1300V278H940V280Z",
                    fill: "black",
                }
                text {
                    id: "write_mux_input1_label",
                    x: "1305",
                    y: "279",
                    "text-anchor": "start",
                    "dominant-baseline": "middle",
                    "font-size": "12",
                    fill: "black",
                    "ALU"
                }
                text {
                    id: "write_mux_input2_label",
                    x: "1305",
                    y: "339",
                    "text-anchor": "start",
                    "dominant-baseline": "middle",
                    "font-size": "12",
                    fill: "black",
                    "LSU"
                }
                path {
                    id: "lsu_to_writemux_arrow",
                    d: "M1300.71 337.707C1301.1 337.317 1301.1 336.683 1300.71 336.293L1294.34 329.929C1293.95 329.538 1293.32 329.538 1292.93 329.929C1292.54 330.319 1292.54 330.953 1292.93 331.343L1298.59 337L1292.93 342.657C1292.54 343.047 1292.54 343.681 1292.93 344.071C1293.32 344.462 1293.95 344.462 1294.34 344.071L1300.71 337.707ZM1244 338H1300V336H1244V338Z",
                    fill: "black",
                }
                path {
                    id: "opbmux_to_lsu_arrow_upper",
                    d: "M959.707 333.707C960.098 333.317 960.098 332.683 959.707 332.293L953.343 325.929C952.953 325.538 952.319 325.538 951.929 325.929C951.538 326.319 951.538 326.953 951.929 327.343L957.586 333L951.929 338.657C951.538 339.047 951.538 339.681 951.929 340.071C952.319 340.462 952.953 340.462 953.343 340.071L959.707 333.707ZM940 334H959V332H940V334Z",
                    fill: "black",
                }
                line {
                    id: "opbmux_upper_to_alu_node_line",
                    x1: "940",
                    y1: "334",
                    x2: "940",
                    y2: "278",
                    stroke: "black",
                    "stroke-width": "2",
                }
                path {
                    id: "lsu_to_addr_arrow",
                    d: "M995.29 421.707C995.68 422.098 996.32 422.098 996.71 421.707L1003.07 415.343C1003.46 414.953 1003.46 414.319 1003.07 413.929C1002.68 413.538 1002.05 413.538 1001.66 413.929L996 419.586L990.34 413.929C989.95 413.538 989.32 413.538 988.93 413.929C988.54 414.319 988.54 414.953 988.93 415.343L995.29 421.707ZM995 397L995 421L997 421L997 397L995 397Z",
                    fill: "black",
                }
                path {
                    id: "lsu_to_data_arrow",
                    d: "M1035.29 421.707C1035.68 422.098 1036.32 422.098 1036.71 421.707L1043.07 415.343C1043.46 414.953 1043.46 414.319 1043.07 413.929C1042.68 413.538 1042.05 413.538 1041.66 413.929L1036 419.586L1030.34 413.929C1029.95 413.538 1029.32 413.538 1028.93 413.929C1028.54 414.319 1028.54 414.953 1028.93 415.343L1035.29 421.707ZM1035 397V421H1037V397H1035Z",
                    fill: "black",
                }
                path {
                    id: "data_to_lsu_arrow",
                    d: "M1055.71 397.293C1055.32 396.902 1054.68 396.902 1054.29 397.293L1047.93 403.657C1047.54 404.047 1047.54 404.681 1047.93 405.071C1048.32 405.462 1048.95 405.462 1049.34 405.071L1055 399.414L1060.66 405.071C1061.05 405.462 1061.68 405.462 1062.07 405.071C1062.46 404.681 1062.46 404.047 1062.07 403.657L1055.71 397.293ZM1056 422L1056 398L1054 398L1054 422L1056 422Z",
                    fill: "black",
                }
                path {
                    id: "lsu_to_req_arrow",
                    d: "M1095.29 421.707C1095.68 422.098 1096.32 422.098 1096.71 421.707L1103.07 415.343C1103.46 414.953 1103.46 414.319 1103.07 413.929C1102.68 413.538 1102.05 413.538 1101.66 413.929L1096 419.586L1090.34 413.929C1089.95 413.538 1089.32 413.538 1088.93 413.929C1088.54 414.319 1088.54 414.953 1088.93 415.343L1095.29 421.707ZM1095 397V421H1097V397H1095Z",
                    fill: "black",
                }
                path {
                    id: "lsu_to_wr_arrow",
                    d: "M1135.29 421.707C1135.68 422.098 1136.32 422.098 1136.71 421.707L1143.07 415.343C1143.46 414.953 1143.46 414.319 1143.07 413.929C1142.68 413.538 1142.05 413.538 1141.66 413.929L1136 419.586L1130.34 413.929C1129.95 413.538 1129.32 413.538 1128.93 413.929C1128.54 414.319 1128.54 414.953 1128.93 415.343L1135.29 421.707ZM1135 397V421H1137V397H1135Z",
                    fill: "black",
                }
                path {
                    id: "lsu_to_byte_en_arrow",
                    d: "M1175.29 421.707C1175.68 422.098 1176.32 422.098 1176.71 421.707L1183.07 415.343C1183.46 414.953 1183.46 414.319 1183.07 413.929C1182.68 413.538 1182.05 413.538 1181.66 413.929L1176 419.586L1170.34 413.929C1169.95 413.538 1169.32 413.538 1168.93 413.929C1168.54 414.319 1168.54 414.953 1168.93 415.343L1175.29 421.707ZM1175 397V421H1177V397H1175Z",
                    fill: "black",
                }
                path {
                    id: "valid_to_lsu_arrow",
                    d: "M1215.71 397.293C1215.32 396.902 1214.68 396.902 1214.29 397.293L1207.93 403.657C1207.54 404.047 1207.54 404.681 1207.93 405.071C1208.32 405.462 1208.95 405.462 1209.34 405.071L1215 399.414L1220.66 405.071C1221.05 405.462 1221.68 405.462 1222.07 405.071C1222.46 404.681 1222.46 404.047 1222.07 403.657L1215.71 397.293ZM1216 422L1216 398L1214 398L1214 422L1216 422Z",
                    fill: "black",
                }
                path {
                    id: "writemux_to_im_arrow",
                    d: "M580.707 444.707C581.098 444.317 581.098 443.683 580.707 443.293L574.343 436.929C573.953 436.538 573.319 436.538 572.929 436.929C572.538 437.319 572.538 437.953 572.929 438.343L578.586 444L572.929 449.657C572.538 450.047 572.538 450.681 572.929 451.071C573.319 451.462 573.953 451.462 574.343 451.071L580.707 444.707ZM538 445H580V443H538V445Z",
                    fill: "black",
                }
                path {
                    id: "writemux_to_pcmux_arrow",
                    d: "M79.2928 38.2928C78.9023 38.6833 78.9023 39.3165 79.2928 39.707L85.6569 46.071C86.0474 46.4615 86.6805 46.4615 87.071 46.071C87.4615 45.6804 87.4615 45.0473 87.071 44.6568L81.4142 38.9999L87.071 33.343C87.4615 32.9525 87.4615 32.3194 87.071 31.9288C86.6805 31.5383 86.0474 31.5383 85.6569 31.9288L79.2928 38.2928ZM1383 38L80 37.9999L80 39.9999L1383 40L1383 38Z",
                    fill: "black",
                }
                path {
                    id: "ifpc_to_plus4_arrow",
                    d: "M139.707 99.2929C139.317 98.9024 138.683 98.9024 138.293 99.2929L131.929 105.657C131.538 106.047 131.538 106.681 131.929 107.071C132.319 107.462 132.953 107.462 133.343 107.071L139 101.414L144.657 107.071C145.047 107.462 145.681 107.462 146.071 107.071C146.462 106.681 146.462 106.047 146.071 105.657L139.707 99.2929ZM140 140V100H138V140H140Z",
                    fill: "black",
                    style: "stroke: transparent; stroke-width: 10px; pointer-events: all;",
                    onmouseenter: move |_| {
                        hovered_element.set(Some("ifpc".to_string()));
                    },
                    onmouseleave: move |_| {
                        hovered_element.set(None);
                    },
                }
                path {
                    id: "plus4_to_pcmux_arrow",
                    d: "M80.2929 78.2929C79.9024 78.6834 79.9024 79.3166 80.2929 79.7071L86.6569 86.0711C87.0474 86.4616 87.6805 86.4616 88.0711 86.0711C88.4616 85.6805 88.4616 85.0474 88.0711 84.6569L82.4142 79L88.0711 73.3431C88.4616 72.9526 88.4616 72.3195 88.0711 71.9289C87.6805 71.5384 87.0474 71.5384 86.6569 71.9289L80.2929 78.2929ZM118 78H81V80H118V78Z",
                    fill: "black",
                }
                // Lines
                rect {
                    id: "ifpc_line",
                    x: "138",
                    y: "140",
                    width: "2",
                    height: "200",
                    fill: "black",
                    style: "stroke: transparent; stroke-width: 10px; pointer-events: all; cursor: default;",
                    onmouseenter: move |_| {
                        hovered_element.set(Some("ifpc".to_string()));
                    },
                    onmouseleave: move |_| {
                        hovered_element.set(None);
                    },
                }
                line {
                    id: "line2",
                    x1: "20",
                    y1: "60",
                    x2: "0",
                    y2: "60",
                    stroke: "black",
                    "stroke-width": "2",
                }
                line {
                    id: "line3",
                    x1: "1",
                    y1: "59",
                    x2: "1",
                    y2: "339",
                    stroke: "black",
                    "stroke-width": "2",
                }
                line {
                    id: "line4",
                    x1: "740",
                    y1: "427",
                    x2: "760",
                    y2: "427",
                    stroke: "black",
                    "stroke-width": "2",
                }
                line {
                    id: "line5",
                    x1: "759",
                    y1: "428",
                    x2: "759",
                    y2: "198",
                    stroke: "black",
                    "stroke-width": "2",
                }
                line {
                    id: "line6",
                    x1: "740",
                    y1: "227",
                    x2: "780",
                    y2: "227",
                    stroke: "black",
                    "stroke-width": "2",
                }
                line {
                    id: "line7",
                    x1: "779",
                    y1: "228",
                    x2: "779",
                    y2: "286",
                    stroke: "black",
                    "stroke-width": "2",
                }
                line {
                    id: "opbmux_out_line1",
                    x1: "880",
                    y1: "310",
                    x2: "920",
                    y2: "310",
                    stroke: "black",
                    "stroke-width": "2",
                }
                line {
                    id: "line11",
                    x1: "920",
                    y1: "199",
                    x2: "919",
                    y2: "311",
                    stroke: "black",
                    "stroke-width": "2",
                }
                line {
                    id: "line12",
                    x1: "1361",
                    y1: "306",
                    x2: "1381",
                    y2: "306",
                    stroke: "black",
                    "stroke-width": "2",
                }
                line {
                    id: "line13",
                    x1: "1382",
                    y1: "39",
                    x2: "1382",
                    y2: "522",
                    stroke: "black",
                    "stroke-width": "2",
                }
                line {
                    id: "line14",
                    x1: "1381",
                    y1: "521",
                    x2: "540",
                    y2: "521",
                    stroke: "black",
                    "stroke-width": "2",
                }
                line {
                    id: "line15",
                    x1: "539",
                    y1: "522",
                    x2: "539",
                    y2: "444",
                    stroke: "black",
                    "stroke-width": "2",
                }
                // Circles/Nodes
                circle {
                    id: "opbmux_lsu_junction",
                    cx: "790",
                    cy: "373",
                    r: "3",
                    fill: "black",
                }
                circle {
                    id: "ifpc_node_1",
                    cx: "139",
                    cy: "140",
                    r: "3",
                    fill: "black",
                    style: "stroke: transparent; stroke-width: 10px; pointer-events: all;",
                    onmouseenter: move |_| {
                        hovered_element.set(Some("ifpc".to_string()));
                    },
                    onmouseleave: move |_| {
                        hovered_element.set(None);
                    },
                }
                circle {
                    id: "writemux_node",
                    cx: "1382",
                    cy: "306",
                    r: "3",
                    fill: "black",
                }
                circle {
                    id: "alu_mux_node",
                    cx: "1220",
                    cy: "279",
                    r: "3",
                    fill: "black",
                }
                circle {
                    id: "ifpc_node_2",
                    cx: "139",
                    cy: "340",
                    r: "3",
                    fill: "black",
                    style: "stroke: transparent; stroke-width: 10px; pointer-events: all;",
                    onmouseenter: move |_| {
                        hovered_element.set(Some("ifpc".to_string()));
                    },
                    onmouseleave: move |_| {
                        hovered_element.set(None);
                    },
                }
            }
            {
                if let Some(element_id) = hovered_element.read().as_ref() {
                    if element_id == "ifpc" {
                        let tooltip_text = format!(
                            "IF PC: {}",
                            format_pc(emulator_state.read().pipeline.IF_pc),
                        );
                        rsx! {
                            div {
                                class: "absolute bg-black text-white px-2 py-1 rounded text-sm",
                                style: "top: 10px; right: 10px; z-index: 100;",
                                "{tooltip_text}"
                            }
                        }
                    } else {
                        rsx! {}
                    }
                } else {
                    rsx! {}
                }
            }
        }
    }
}
