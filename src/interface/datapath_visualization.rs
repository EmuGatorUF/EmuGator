use crate::emulator::EmulatorState;
use dioxus::prelude::*;

fn format_pc(pc: u32) -> String {
    format!("0x{:08X}", pc)
}

#[component]
#[allow(non_snake_case)]
pub fn DatapathVisualization(emulator_state: Signal<EmulatorState>) -> Element {
    let mut hovered_element = use_signal(|| Option::<String>::None);

    rsx! {
        div { class: "w-full h-full bg-white overflow-hidden relative",
            // SVG needs to be inline in RSX to be rendered properly
            svg {
                width: "100%",
                height: "100%",
                view_box: "0 0 1261 660",
                xmlns: "http://www.w3.org/2000/svg",
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
                    y: "218",
                    width: "158",
                    height: "78",
                    stroke: "black",
                    "stroke-width": "2",
                    fill: "none",
                }
                text {
                    id: "alu_label",
                    x: "1040",
                    y: "257",
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
                    width: "158",
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
                    width: "238",
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
                    id: "data_memory_data_label",
                    x: "1090",
                    y: "436",
                    "text-anchor": "middle",
                    "dominant-baseline": "middle",
                    "font-size": "12",
                    fill: "black",
                    "DATA"
                }
                text {
                    id: "data_memory_addr_label",
                    x: "1020",
                    y: "436",
                    "text-anchor": "middle",
                    "dominant-baseline": "middle",
                    "font-size": "12",
                    fill: "black",
                    "ADDR"
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
                    d: "M1178 248.618L1236 277.618V336.382L1178 365.382V248.618Z",
                    stroke: "black",
                    "stroke-width": "2",
                    fill: "none",
                }
                path {
                    id: "opb_mux",
                    d: "M821 356.618L879 385.618V444.382L821 473.382V356.618Z",
                    stroke: "black",
                    "stroke-width": "2",
                    fill: "none",
                }
                text {
                    id: "imm_mux_input1_label",
                    x: "824",
                    y: "386",
                    "text-anchor": "start",
                    "dominant-baseline": "middle",
                    "font-size": "12",
                    fill: "black",
                    "IMM"
                }
                text {
                    id: "rs2_mux_input1_label",
                    x: "824",
                    y: "446",
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
                    d: "M819.707 444.707C820.098 444.317 820.098 443.683 819.707 443.293L813.343 436.929C812.953 436.538 812.319 436.538 811.929 436.929C811.538 437.319 811.538 437.953 811.929 438.343L817.586 444L811.929 449.657C811.538 450.047 811.538 450.681 811.929 451.071C812.319 451.462 812.953 451.462 813.343 451.071L819.707 444.707ZM740 445H819V443H740V445Z",
                    fill: "black",
                }
                path {
                    id: "decoder_to_imm_arrow",
                    d: "M819.707 386.707C820.098 386.317 820.098 385.683 819.707 385.293L813.343 378.929C812.953 378.538 812.319 378.538 811.929 378.929C811.538 379.319 811.538 379.953 811.929 380.343L817.586 386L811.929 391.657C811.538 392.047 811.538 392.681 811.929 393.071C812.319 393.462 812.953 393.462 813.343 393.071L819.707 386.707ZM778 387H819V385H778V387Z",
                    fill: "black",
                }
                path {
                    id: "opamux_to_alu_arrow",
                    d: "M959.707 241.707C960.098 241.317 960.098 240.683 959.707 240.293L953.343 233.929C952.953 233.538 952.319 233.538 951.929 233.929C951.538 234.319 951.538 234.953 951.929 235.343L957.586 241L951.929 246.657C951.538 247.047 951.538 247.681 951.929 248.071C952.319 248.462 952.953 248.462 953.343 248.071L959.707 241.707ZM900 242H959V240H900V242Z",
                    fill: "black",
                }
                path {
                    id: "opamux_to_lsu_arrow",
                    d: "M959.707 341.707C960.098 341.317 960.098 340.683 959.707 340.293L953.343 333.929C952.953 333.538 952.319 333.538 951.929 333.929C951.538 334.319 951.538 334.953 951.929 335.343L957.586 341L951.929 346.657C951.538 347.047 951.538 347.681 951.929 348.071C952.319 348.462 952.953 348.462 953.343 348.071L959.707 341.707ZM898 342H959V340H898V342Z",
                    fill: "black",
                }
                path {
                    id: "opbmux_to_alu_arrow",
                    d: "M959.707 273.707C960.098 273.317 960.098 272.683 959.707 272.293L953.343 265.929C952.953 265.538 952.319 265.538 951.929 265.929C951.538 266.319 951.538 266.953 951.929 267.343L957.586 273L951.929 278.657C951.538 279.047 951.538 279.681 951.929 280.071C952.319 280.462 952.953 280.462 953.343 280.071L959.707 273.707ZM918 274H959V272H918V274Z",
                    fill: "black",
                }
                path {
                    id: "opbmux_to_lsu_arrow",
                    d: "M959.707 373.707C960.098 373.317 960.098 372.683 959.707 372.293L953.343 365.929C952.953 365.538 952.319 365.538 951.929 365.929C951.538 366.319 951.538 366.953 951.929 367.343L957.586 373L951.929 378.657C951.538 379.047 951.538 379.681 951.929 380.071C952.319 380.462 952.953 380.462 953.343 380.071L959.707 373.707ZM919 374H959V372H919V374Z",
                    fill: "black",
                }
                path {
                    id: "alu_to_writemux_arrow",
                    d: "M1176.71 279.707C1177.1 279.317 1177.1 278.683 1176.71 278.293L1170.34 271.929C1169.95 271.538 1169.32 271.538 1168.93 271.929C1168.54 272.319 1168.54 272.953 1168.93 273.343L1174.59 279L1168.93 284.657C1168.54 285.047 1168.54 285.681 1168.93 286.071C1169.32 286.462 1169.95 286.462 1170.34 286.071L1176.71 279.707ZM1120 280H1176V278H1120V280Z",
                    fill: "black",
                }
                text {
                    id: "write_mux_input1_label",
                    x: "1181",
                    y: "279",
                    "text-anchor": "start",
                    "dominant-baseline": "middle",
                    "font-size": "12",
                    fill: "black",
                    "ALU"
                }
                text {
                    id: "write_mux_input2_label",
                    x: "1181",
                    y: "339",
                    "text-anchor": "start",
                    "dominant-baseline": "middle",
                    "font-size": "12",
                    fill: "black",
                    "LSU"
                }
                path {
                    id: "lsu_to_writemux_arrow",
                    d: "M1176.71 337.707C1177.1 337.317 1177.1 336.683 1176.71 336.293L1170.34 329.929C1169.95 329.538 1169.32 329.538 1168.93 329.929C1168.54 330.319 1168.54 330.953 1168.93 331.343L1174.59 337L1168.93 342.657C1168.54 343.047 1168.54 343.681 1168.93 344.071C1169.32 344.462 1169.95 344.462 1170.34 344.071L1176.71 337.707ZM1120 338H1176V336H1120V338Z",
                    fill: "black",
                }
                path {
                    id: "writebackmux_to_im_arrow",
                    d: "M580.707 444.707C581.098 444.317 581.098 443.683 580.707 443.293L574.343 436.929C573.953 436.538 573.319 436.538 572.929 436.929C572.538 437.319 572.538 437.953 572.929 438.343L578.586 444L572.929 449.657C572.538 450.047 572.538 450.681 572.929 451.071C573.319 451.462 573.953 451.462 574.343 451.071L580.707 444.707ZM538 445H580V443H538V445Z",
                    fill: "black",
                }
                path {
                    id: "lsu_to_data_arrow",
                    d: "M1099.29 421.707C1099.68 422.098 1100.32 422.098 1100.71 421.707L1107.07 415.343C1107.46 414.953 1107.46 414.319 1107.07 413.929C1106.68 413.538 1106.05 413.538 1105.66 413.929L1100 419.586L1094.34 413.929C1093.95 413.538 1093.32 413.538 1092.93 413.929C1092.54 414.319 1092.54 414.953 1092.93 415.343L1099.29 421.707ZM1099 397V421H1101V397H1099Z",
                    fill: "black",
                }
                path {
                    id: "data_to_lsu_arrow",
                    d: "M1080.71 397.293C1080.32 396.902 1079.68 396.902 1079.29 397.293L1072.93 403.657C1072.54 404.047 1072.54 404.681 1072.93 405.071C1073.32 405.462 1073.95 405.462 1074.34 405.071L1080 399.414L1085.66 405.071C1086.05 405.462 1086.68 405.462 1087.07 405.071C1087.46 404.681 1087.46 404.047 1087.07 403.657L1080.71 397.293ZM1081 422L1081 398L1079 398L1079 422L1081 422Z",
                    fill: "black",
                }
                path {
                    id: "lsu_to_addr_arrow",
                    d: "M1021.29 421.707C1021.68 422.098 1022.32 422.098 1022.71 421.707L1029.07 415.343C1029.46 414.953 1029.46 414.319 1029.07 413.929C1028.68 413.538 1028.05 413.538 1027.66 413.929L1022 419.586L1016.34 413.929C1015.95 413.538 1015.32 413.538 1014.93 413.929C1014.54 414.319 1014.54 414.953 1014.93 415.343L1021.29 421.707ZM1021 397L1021 421L1023 421L1023 397L1021 397Z",
                    fill: "black",
                }
                path {
                    id: "writemux_to_pcmux_arrow",
                    d: "M79.2928 38.2928C78.9023 38.6833 78.9023 39.3165 79.2928 39.707L85.6569 46.071C86.0474 46.4615 86.6805 46.4615 87.071 46.071C87.4615 45.6804 87.4615 45.0473 87.071 44.6568L81.4142 38.9999L87.071 33.343C87.4615 32.9525 87.4615 32.3194 87.071 31.9288C86.6805 31.5383 86.0474 31.5383 85.6569 31.9288L79.2928 38.2928ZM1259 38L80 37.9999L80 39.9999L1259 40L1259 38Z",
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
                    y2: "386",
                    stroke: "black",
                    "stroke-width": "2",
                }
                line {
                    id: "line8",
                    x1: "880",
                    y1: "167",
                    x2: "900",
                    y2: "167",
                    stroke: "black",
                    "stroke-width": "2",
                }
                line {
                    id: "line9",
                    x1: "899",
                    y1: "168",
                    x2: "899",
                    y2: "342",
                    stroke: "black",
                    "stroke-width": "2",
                }
                line {
                    id: "line10",
                    x1: "880",
                    y1: "419",
                    x2: "920",
                    y2: "419",
                    stroke: "black",
                    "stroke-width": "2",
                }
                line {
                    id: "line11",
                    x1: "919",
                    y1: "420",
                    x2: "919",
                    y2: "273",
                    stroke: "black",
                    "stroke-width": "2",
                }
                line {
                    id: "line12",
                    x1: "1237",
                    y1: "306",
                    x2: "1257",
                    y2: "306",
                    stroke: "black",
                    "stroke-width": "2",
                }
                line {
                    id: "line13",
                    x1: "1258",
                    y1: "39",
                    x2: "1258",
                    y2: "522",
                    stroke: "black",
                    "stroke-width": "2",
                }
                line {
                    id: "line14",
                    x1: "1257",
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
                    id: "opamux_node",
                    cx: "899",
                    cy: "241",
                    r: "3",
                    fill: "black",
                }
                circle {
                    id: "writemux_node",
                    cx: "1258",
                    cy: "306",
                    r: "3",
                    fill: "black",
                }
                circle {
                    id: "opbmux_node",
                    cx: "919",
                    cy: "373",
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
