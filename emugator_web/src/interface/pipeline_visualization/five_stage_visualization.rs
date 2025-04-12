use std::ops::Deref;

use dioxus::prelude::*;

use emugator_core::emulator::{
    AnyEmulatorState, controller_common::OpASel, five_stage::FiveStagePipeline,
};

macro_rules! format_opt {
    ($fmt:literal, $val:expr) => {
        match $val {
            Some(value) => format!($fmt, value),
            None => "None".to_string(),
        }
    };
}

macro_rules! format_bool {
    ($fmt:literal, $val:expr) => {
        format!($fmt, if $val { "1" } else { "0" })
    };
}

#[derive(Debug, PartialEq, Clone, Copy)]
enum FiveStageElement {
    PC,
    PCMux,
    PCPlus4,
    InstructionMemory,
    IFIDBuffer,
    IFIDPC,
    IFIDInstruction,
    RegisterFile,
    ALU,
    ALUMuxA,
    ALUMuxB,
    DataMemory,
    IDEXBuffer,
    IDEXPC,
    IDEXRS1,
    IDEXRS2,
    IDEXImm,
    EXMEMBuffer,
    EXMEMPC,
    EXMEMAlu,
    EXMEMRS2,
    MEMWBBuffer,
    MEMWBPC,
    MEMWBLsu,
    MEMWBAlu,
    Decoder,
    ControlUnit,
    DecoderRS1,
    DecoderRS2,
    DecoderRD,
    DecoderImm,
    RegisterFileRS1Value,
    RegisterFileRS2Value,
    BranchUnit,
    LSU,
    LSUADDR,
    LSUDATA,
    LSUREQ,
    LSUWR,
    LSUBYTE_EN,
    LSUVALID,
    LSURDOut,
    WritebackResult,
    IDEXRD,
    EXMEMRD,
    MEMWBRD,
}

impl FiveStageElement {
    fn tooltip_text(&self, pipeline: &FiveStagePipeline) -> String {
        match self {
            FiveStageElement::PC => {
                format!("IF PC: 0x{:08X}", pipeline.if_pc)
            }
            FiveStageElement::PCMux => format_opt!("Next PC: 0x{:08X}", pipeline.if_lines.next_pc),
            FiveStageElement::PCPlus4 => {
                format!("PC+4: 0x{:08X}", pipeline.if_pc.wrapping_add(4))
            }
            FiveStageElement::InstructionMemory => {
                format_opt!("Instruction: 0x{:08X}", pipeline.if_lines.instr)
            }
            FiveStageElement::IFIDBuffer => "IF/ID Pipeline Buffer".to_string(),
            FiveStageElement::IFIDPC => format_opt!("ID PC: 0x{:08X}", pipeline.if_id.id_pc),
            FiveStageElement::IFIDInstruction => {
                format_opt!("ID PC: 0x{:08X}", pipeline.if_id.id_inst)
            }
            FiveStageElement::RegisterFile => "Register File".to_string(),
            FiveStageElement::DataMemory => "Data Memory".to_string(),
            FiveStageElement::Decoder => "Instruction Decoder".to_string(),
            FiveStageElement::ControlUnit => "Control Unit".to_string(),
            FiveStageElement::DecoderRS1 => format!("RS1: {}", pipeline.id_lines.rs1),
            FiveStageElement::DecoderRS2 => format!("RS2: {}", pipeline.id_lines.rs2),
            FiveStageElement::DecoderRD => format!("RD: {}", pipeline.id_lines.rd),
            FiveStageElement::DecoderImm => format_opt!("IMM: 0x{:08X}", pipeline.id_lines.imm),
            FiveStageElement::RegisterFileRS1Value => {
                format!("RS1_V: 0x{:08X}", pipeline.id_lines.rs1_v)
            }
            FiveStageElement::RegisterFileRS2Value => {
                format!("RS2_V: 0x{:08X}", pipeline.id_lines.rs2_v)
            }
            // ID/EX Buffer
            FiveStageElement::IDEXBuffer => "ID/EX Pipeline Buffer".to_string(),
            FiveStageElement::IDEXPC => format_opt!("EX PC: 0x{:08X}", pipeline.id_ex.ex_pc),
            FiveStageElement::IDEXRD => format_opt!("EX RD: 0x{:08X}", pipeline.id_ex.rd),
            FiveStageElement::IDEXRS1 => format!("EX RS1_V: 0x{:08X}", pipeline.id_ex.rs1_v),
            FiveStageElement::IDEXRS2 => format!("EX RS2_V: 0x{:08X}", pipeline.id_ex.rs2_v),
            FiveStageElement::IDEXImm => format_opt!("EX IMM: 0x{:08X}", pipeline.id_ex.imm),
            // Branch Unit
            FiveStageElement::BranchUnit => {
                format_opt!("Branch PC: 0x{:08X}", pipeline.ex_lines.jmp_dst)
            }
            // ALU
            FiveStageElement::ALUMuxA => format_opt!("ALU OP A: 0x{:08X}", pipeline.ex_lines.op_a),
            FiveStageElement::ALUMuxB => format_opt!("ALU OP B: 0x{:08X}", pipeline.ex_lines.op_b),
            FiveStageElement::ALU => format_opt!("ALU Output: 0x{:08X}", pipeline.ex_lines.alu_out),
            // EX/MEM Buffer
            FiveStageElement::EXMEMBuffer => "EX/MEM Pipeline Buffer".to_string(),
            FiveStageElement::EXMEMPC => format_opt!("MEM PC: 0x{:08X}", pipeline.ex_mem.mem_pc),
            FiveStageElement::EXMEMAlu => {
                format_opt!("MEM ALU Output: 0x{:08X}", pipeline.ex_mem.alu_o)
            }
            FiveStageElement::EXMEMRD => format_opt!("MEM RD: 0x{:08X}", pipeline.ex_mem.rd),
            FiveStageElement::EXMEMRS2 => format!("MEM RS2: 0x{:08X}", pipeline.ex_mem.rs2_v),
            // LSU
            FiveStageElement::LSU => "LSU".to_string(),
            FiveStageElement::LSUADDR => {
                format!("Memory Address: 0x{:08X}", pipeline.mem_lines.data_addr_o)
            }
            FiveStageElement::LSUDATA => {
                let write_data = format!("0x{:08X}", pipeline.mem_lines.data_wdata_o);
                let read_data = format!("0x{:08X}", pipeline.mem_lines.data_rdata_i);
                let write_enable = pipeline.mem_lines.data_we_o;
                if write_enable {
                    format!("Memory Write Data: {}", write_data)
                } else {
                    format!("Memory Read Data: {}", read_data)
                }
            }
            FiveStageElement::LSUREQ => {
                format_bool!("Memory Request: {}", pipeline.mem_lines.data_req_o)
            }
            FiveStageElement::LSUWR => {
                format_bool!("Memory Write Enable: {}", pipeline.mem_lines.data_we_o)
            }
            FiveStageElement::LSUBYTE_EN => {
                let byte_en = pipeline.mem_lines.data_be_o;
                let byte_en_str = format!(
                    "[{}, {}, {}, {}]",
                    if byte_en[0] { "1" } else { "0" },
                    if byte_en[1] { "1" } else { "0" },
                    if byte_en[2] { "1" } else { "0" },
                    if byte_en[3] { "1" } else { "0" },
                );
                format!("Byte Enable: {}", byte_en_str)
            }
            FiveStageElement::LSUVALID => {
                format_bool!("Memory Valid: {}", pipeline.mem_lines.data_rvalid_i)
            }
            FiveStageElement::LSURDOut => {
                format_opt!("LSU Output: {}", pipeline.mem_lines.mem_data)
            }
            // MEM/WB Buffer
            FiveStageElement::MEMWBBuffer => "MEM/WB Pipeline Buffer".to_string(),
            FiveStageElement::MEMWBPC => {
                format_opt!("WB PC: 0x{:08X}", pipeline.mem_wb.wb_pc)
            }
            FiveStageElement::MEMWBRD => {
                format_opt!("WB RD: 0x{:08X}", pipeline.mem_wb.rd)
            }
            FiveStageElement::MEMWBAlu => {
                format_opt!("MEM/WB ALU Output: 0x{:08X}", pipeline.mem_wb.alu)
            }
            FiveStageElement::MEMWBLsu => {
                format_opt!("MEM/WB LSU Output: 0x{:08X}", pipeline.mem_wb.lsu)
            }
            // Writeback
            FiveStageElement::WritebackResult => {
                format_opt!("Register Write Data: 0x{:08X}", pipeline.wb_lines.wb_data)
            }
        }
    }
}

#[component]
#[allow(non_snake_case)]
pub fn FiveStageVisualization(
    emulator_state: ReadOnlySignal<Option<AnyEmulatorState>>,
    tooltip_text: Signal<Option<String>>,
) -> Element {
    const HOVER_STROKE: &'static str = "rgba(66, 133, 244, 1)";
    const ACTIVE_STROKE: &'static str = "rgba(66, 133, 244, 0.7)";
    const HOVER_FILL: &'static str = "rgba(66, 133, 244, 0.1)";

    let mut hovered_element = use_signal(|| Option::<FiveStageElement>::None);

    use_effect(move || {
        let Some(AnyEmulatorState::FiveStage(state)) = &*emulator_state.read() else {
            dioxus_logger::tracing::error!("Expected FiveStage emulator state");
            return;
        };
        tooltip_text.set(
            hovered_element
                .read()
                .deref()
                .map(|e| e.tooltip_text(&state.pipeline)),
        );
    });

    // NOTE: THIS IS WHERE WE WILL ADD THE ACTIVE ELEMENTS TO MATCH LIAM's CVE2 IMPLEMENTATION
    macro_rules! element_stroke {
        ($element:ident) => {
            if *hovered_element.read() == Some(FiveStageElement::$element) {
                HOVER_STROKE
            } else {
                "black"
            }
        };
    }

    macro_rules! element_fill {
        ($element:ident) => {
            if *hovered_element.read() == Some(FiveStageElement::$element) {
                HOVER_FILL
            } else {
                "none"
            }
        };
    }
    rsx! {

        g {
            id: "next_pc_group",
            style: "pointer-events: all;",
            onmouseenter: move |_| {
                hovered_element.set(Some(FiveStageElement::PCMux));
            },
            onmouseleave: move |_| {
                hovered_element.set(None);
            },
            path {
                id: "next_pc_mux",
                d: "M77 108.442L19 81.8583L19 28.1417L77 1.55836L77 108.442Z",
                stroke: element_stroke!(PCMux),
                "stroke-width": "2",
                fill: element_fill!(PCMux),
            }
            text {
                x: "60",
                y: "43",
                "font-family": "Arial",
                "font-size": "12",
                "font-weight": "bold",
                "text-anchor": "middle",
                fill: element_stroke!(PCMux),
                "JMP"
            }
            text {
                x: "57",
                y: "77",
                "font-family": "Arial",
                "font-size": "12",
                "font-weight": "bold",
                "text-anchor": "middle",
                fill: element_stroke!(PCMux),
                "PC+4"
            }
            line {
                id: "next_pc_line1",
                x1: "20",
                y1: "56",
                x2: "2",
                y2: "56",
                stroke: element_stroke!(PCMux),
                "stroke-width": "2",
            }
            line {
                id: "next_pc_line2",
                x1: "1",
                y1: "55",
                x2: "0.999993",
                y2: "235",
                stroke: element_stroke!(PCMux),
                "stroke-width": "2",
            }
            path {
                id: "next_pc_arrow1",
                d: "M18.7071 236.707C19.0976 236.317 19.0976 235.683 18.7071 235.293L12.3431 228.929C11.9526 228.538 11.3195 228.538 10.9289 228.929C10.5384 229.319 10.5384 229.953 10.9289 230.343L16.5858 236L10.9289 241.657C10.5384 242.047 10.5384 242.681 10.9289 243.071C11.3195 243.462 11.9526 243.462 12.3431 243.071L18.7071 236.707ZM0 237H18V235H0V237Z",
                fill: element_stroke!(PCMux),
            }
        }
        g {
            id: "if_pc_group",
            style: "pointer-events: all;",
            onmouseenter: move |_| {
                hovered_element.set(Some(FiveStageElement::PC));
            },
            onmouseleave: move |_| {
                hovered_element.set(None);
            },
            rect {
                id: "if_pc_rect",
                x: "19",
                y: "157",
                width: "78",
                height: "158",
                stroke: element_stroke!(PC),
                "stroke-width": "2",
                fill: element_fill!(PC),
            }
            text {
                x: "58",
                y: "245",
                "font-family": "Arial",
                "font-size": "24",
                "font-weight": "bold",
                "text-anchor": "middle",
                fill: element_stroke!(PC),
                "PC"
            }
            path {
                id: "if_pc_arrow1",
                d: "M177.707 236.707C178.098 236.317 178.098 235.683 177.707 235.293L171.343 228.929C170.953 228.538 170.319 228.538 169.929 228.929C169.538 229.319 169.538 229.953 169.929 230.343L175.586 236L169.929 241.657C169.538 242.047 169.538 242.681 169.929 243.071C170.319 243.462 170.953 243.462 171.343 243.071L177.707 236.707ZM98 237H177V235H98V237Z",
                fill: element_stroke!(PC),
            }
            path {
                id: "if_pc_arrow2",
                d: "M137.707 94.2929C137.317 93.9024 136.683 93.9024 136.293 94.2929L129.929 100.657C129.538 101.047 129.538 101.681 129.929 102.071C130.319 102.462 130.953 102.462 131.343 102.071L137 96.4142L142.657 102.071C143.047 102.462 143.681 102.462 144.071 102.071C144.462 101.681 144.462 101.047 144.071 100.657L137.707 94.2929ZM138 236V95H136V236H138Z",
                fill: element_stroke!(PC),
            }
            path {
                id: "next_pc_arrow3",
                d: "M402.707 109.707C403.098 109.317 403.098 108.683 402.707 108.293L396.343 101.929C395.953 101.538 395.319 101.538 394.929 101.929C394.538 102.319 394.538 102.953 394.929 103.343L400.586 109L394.929 114.657C394.538 115.047 394.538 115.681 394.929 116.071C395.319 116.462 395.953 116.462 396.343 116.071L402.707 109.707ZM137 110H402V108H137V110Z",
                fill: element_stroke!(PC),
            }
            circle {
                id: "if_pc_node1",
                cx: "137",
                cy: "110",
                r: "3",
                fill: element_stroke!(PC),
            }
            circle {
                id: "if_pc_node2",
                cx: "137",
                cy: "236",
                r: "3",
                fill: element_stroke!(PC),
            }
        }
        g {
            id: "instruction_memory_group",
            style: "pointer-events: all;",
            onmouseenter: move |_| {
                hovered_element.set(Some(FiveStageElement::InstructionMemory));
            },
            onmouseleave: move |_| {
                hovered_element.set(None);
            },
            rect {
                id: "instruction_memory_rect",
                x: "179",
                y: "157",
                width: "158",
                height: "158",
                stroke: element_stroke!(InstructionMemory),
                "stroke-width": "2",
                fill: element_fill!(InstructionMemory),
            }
            path {
                id: "instruction_memory_arrow",
                d: "M402.707 188.707C403.098 188.317 403.098 187.683 402.707 187.293L396.343 180.929C395.953 180.538 395.319 180.538 394.929 180.929C394.538 181.319 394.538 181.953 394.929 182.343L400.586 188L394.929 193.657C394.538 194.047 394.538 194.681 394.929 195.071C395.319 195.462 395.953 195.462 396.343 195.071L402.707 188.707ZM338 189H402V187H338V189Z",
                fill: element_stroke!(InstructionMemory),
            }
            text {
                x: "258",
                y: "230",
                "font-family": "Arial",
                "font-size": "20",
                "font-weight": "bold",
                "text-anchor": "middle",
                fill: element_stroke!(InstructionMemory),
                "INSTRUCTION"
            }
            text {
                x: "258",
                y: "260",
                "font-family": "Arial",
                "font-size": "20",
                "font-weight": "bold",
                "text-anchor": "middle",
                fill: element_stroke!(InstructionMemory),
                "MEMORY"
            }
        }
        g {
            id: "plus4_group",
            style: "pointer-events: all;",
            onmouseenter: move |_| {
                hovered_element.set(Some(FiveStageElement::PCPlus4));
            },
            onmouseleave: move |_| {
                hovered_element.set(None);
            },
            rect {
                id: "plus4_rect",
                x: "118",
                y: "55",
                width: "38",
                height: "38",
                stroke: element_stroke!(PCPlus4),
                "stroke-width": "2",
                fill: element_fill!(PCPlus4),
            }
            path {
                id: "plus4_arrow",
                d: "M78.2929 73.2929C77.9024 73.6834 77.9024 74.3166 78.2929 74.7071L84.6569 81.0711C85.0474 81.4616 85.6805 81.4616 86.0711 81.0711C86.4616 80.6805 86.4616 80.0474 86.0711 79.6569L80.4142 74L86.0711 68.3431C86.4616 67.9526 86.4616 67.3195 86.0711 66.9289C85.6805 66.5384 85.0474 66.5384 84.6569 66.9289L78.2929 73.2929ZM117 73H79V75H117V73Z",
                fill: element_stroke!(PCPlus4),
            }
            text {
                x: "137",
                y: "80",
                "font-family": "Arial",
                "font-size": "16",
                "font-weight": "bold",
                "text-anchor": "middle",
                fill: element_stroke!(PCPlus4),
                "+4"
            }
        }
        g {
            id: "ifid_buffer_group",
            style: "pointer-events: all;",
            onmouseenter: move |_| {
                hovered_element.set(Some(FiveStageElement::IFIDBuffer));
            },
            onmouseleave: move |_| {
                hovered_element.set(None);
            },
            rect {
                id: "ifid_buffer",
                x: "404",
                y: "70",
                width: "78",
                height: "490",
                stroke: element_stroke!(IFIDBuffer),
                "stroke-width": "2",
                fill: element_fill!(IFIDBuffer),
            }
            text {
                x: "443",
                y: "530",
                "font-family": "Arial",
                "font-size": "22",
                "font-weight": "bold",
                "text-anchor": "middle",
                fill: element_stroke!(IFIDBuffer),
                "IF"
            }
            text {
                x: "443",
                y: "550",
                "font-family": "Arial",
                "font-size": "22",
                "font-weight": "bold",
                "text-anchor": "middle",
                fill: element_stroke!(IFIDBuffer),
                "ID"
            }
        }
        g {
            id: "idex_buffer_group",
            style: "pointer-events: all;",
            onmouseenter: move |_| {
                hovered_element.set(Some(FiveStageElement::IDEXBuffer));
            },
            onmouseleave: move |_| {
                hovered_element.set(None);
            },
            rect {
                id: "idex_buffer",
                x: "760",
                y: "71",
                width: "78",
                height: "490",
                stroke: element_stroke!(IDEXBuffer),
                "stroke-width": "2",
                fill: element_fill!(IDEXBuffer),
            }
            text {
                x: "799",
                y: "530",
                "font-family": "Arial",
                "font-size": "22",
                "font-weight": "bold",
                "text-anchor": "middle",
                fill: element_stroke!(IDEXBuffer),
                "ID"
            }
            text {
                x: "799",
                y: "550",
                "font-family": "Arial",
                "font-size": "22",
                "font-weight": "bold",
                "text-anchor": "middle",
                fill: element_stroke!(IDEXBuffer),
                "EX"
            }
        }
        g {
            id: "exmem_buffer_group",
            style: "pointer-events: all;",
            onmouseenter: move |_| {
                hovered_element.set(Some(FiveStageElement::EXMEMBuffer));
            },
            onmouseleave: move |_| {
                hovered_element.set(None);
            },
            rect {
                id: "exmem_buffer",
                x: "1117",
                y: "70",
                width: "78",
                height: "490",
                stroke: element_stroke!(EXMEMBuffer),
                "stroke-width": "2",
                fill: element_fill!(EXMEMBuffer),
            }
            text {
                x: "1156",
                y: "530",
                "font-family": "Arial",
                "font-size": "22",
                "font-weight": "bold",
                "text-anchor": "middle",
                fill: element_stroke!(EXMEMBuffer),
                "EX"
            }
            text {
                x: "1156",
                y: "550",
                "font-family": "Arial",
                "font-size": "22",
                "font-weight": "bold",
                "text-anchor": "middle",
                fill: element_stroke!(EXMEMBuffer),
                "MEM"
            }
        }
        g {
            id: "memwb_buffer_group",
            style: "pointer-events: all;",
            onmouseenter: move |_| {
                hovered_element.set(Some(FiveStageElement::MEMWBBuffer));
            },
            onmouseleave: move |_| {
                hovered_element.set(None);
            },
            rect {
                id: "memwb_buffer",
                x: "1474",
                y: "70",
                width: "78",
                height: "490",
                stroke: element_stroke!(MEMWBBuffer),
                "stroke-width": "2",
                fill: element_fill!(MEMWBBuffer),
            }
            text {
                x: "1513",
                y: "530",
                "font-family": "Arial",
                "font-size": "22",
                "font-weight": "bold",
                "text-anchor": "middle",
                fill: element_stroke!(MEMWBBuffer),
                "MEM"
            }
            text {
                x: "1513",
                y: "550",
                "font-family": "Arial",
                "font-size": "22",
                "font-weight": "bold",
                "text-anchor": "middle",
                fill: element_stroke!(MEMWBBuffer),
                "WB"
            }
        }
        g {
            id: "ifid_instruction_group",
            style: "pointer-events: all;",
            onmouseenter: move |_| {
                hovered_element.set(Some(FiveStageElement::IFIDInstruction));
            },
            onmouseleave: move |_| {
                hovered_element.set(None);
            },
            rect {
                id: "ifid_instruction_rect",
                x: "404",
                y: "169",
                width: "78",
                height: "38",
                stroke: element_stroke!(IFIDInstruction),
                "stroke-width": "2",
                fill: element_fill!(IFIDInstruction),
            }
            path {
                id: "ifid_instruction_arrow",
                d: "M540.707 188.707C541.098 188.317 541.098 187.683 540.707 187.293L534.343 180.929C533.953 180.538 533.319 180.538 532.929 180.929C532.538 181.319 532.538 181.953 532.929 182.343L538.586 188L532.929 193.657C532.538 194.047 532.538 194.681 532.929 195.071C533.319 195.462 533.953 195.462 534.343 195.071L540.707 188.707ZM483 189H540V187H483V189Z",
                fill: element_stroke!(IFIDInstruction),
            }
            text {
                x: "443",
                y: "194",
                "font-family": "Arial",
                "font-size": "18",
                "font-weight": "bold",
                "text-anchor": "middle",
                fill: element_stroke!(IFIDInstruction),
                "INST"
            }
        }
        g {
            id: "ifid_pc_group",
            style: "pointer-events: all;",
            onmouseenter: move |_| {
                hovered_element.set(Some(FiveStageElement::IFIDPC));
            },
            onmouseleave: move |_| {
                hovered_element.set(None);
            },
            rect {
                id: "ifid_pc_rect",
                x: "404",
                y: "70",
                width: "78",
                height: "78",
                stroke: element_stroke!(IFIDPC),
                "stroke-width": "2",
                fill: element_fill!(IFIDPC),
            }
            path {
                id: "ifid_pc_arrow",
                d: "M758.707 109.707C759.098 109.317 759.098 108.683 758.707 108.293L752.343 101.929C751.953 101.538 751.319 101.538 750.929 101.929C750.538 102.319 750.538 102.953 750.929 103.343L756.586 109L750.929 114.657C750.538 115.047 750.538 115.681 750.929 116.071C751.319 116.462 751.953 116.462 752.343 116.071L758.707 109.707ZM483 110H758V108H483V110Z",
                fill: element_stroke!(IFIDPC),
            }
            text {
                x: "443",
                y: "114",
                "font-family": "Arial",
                "font-size": "18",
                "font-weight": "bold",
                "text-anchor": "middle",
                fill: element_stroke!(IFIDPC),
                "IDPC"
            }
        }
        g {
            id: "decoder_group",
            style: "pointer-events: all;",
            onmouseenter: move |_| {
                hovered_element.set(Some(FiveStageElement::Decoder));
            },
            onmouseleave: move |_| {
                hovered_element.set(None);
            },
            rect {
                id: "decoder_rect",
                x: "542",
                y: "149",
                width: "158",
                height: "78",
                stroke: element_stroke!(Decoder),
                "stroke-width": "2",
                fill: element_fill!(Decoder),
            }
            text {
                x: "621",
                y: "193",
                "font-family": "Arial",
                "font-size": "18",
                "font-weight": "bold",
                "text-anchor": "middle",
                fill: element_stroke!(Decoder),
                "DECODER"
            }
        }
        g {
            id: "decoder_rs1_group",
            style: "pointer-events: all;",
            onmouseenter: move |_| {
                hovered_element.set(Some(FiveStageElement::DecoderRS1));
            },
            onmouseleave: move |_| {
                hovered_element.set(None);
            },
            path {
                id: "decoder_rs1_arrow",
                d: "M580.293 275.707C580.683 276.098 581.317 276.098 581.707 275.707L588.071 269.343C588.462 268.953 588.462 268.319 588.071 267.929C587.681 267.538 587.047 267.538 586.657 267.929L581 273.586L575.343 267.929C574.953 267.538 574.319 267.538 573.929 267.929C573.538 268.319 573.538 268.953 573.929 269.343L580.293 275.707ZM580 227L580 275L582 275L582 227L580 227Z",
                fill: element_stroke!(DecoderRS1),
            }
        }
        g {
            id: "decoder_rs2_group",
            style: "pointer-events: all;",
            onmouseenter: move |_| {
                hovered_element.set(Some(FiveStageElement::DecoderRS2));
            },
            onmouseleave: move |_| {
                hovered_element.set(None);
            },
            path {
                id: "decoder_rs2_arrow",
                d: "M620.293 275.707C620.683 276.098 621.317 276.098 621.707 275.707L628.071 269.343C628.462 268.953 628.462 268.319 628.071 267.929C627.681 267.538 627.047 267.538 626.657 267.929L621 273.586L615.343 267.929C614.953 267.538 614.319 267.538 613.929 267.929C613.538 268.319 613.538 268.953 613.929 269.343L620.293 275.707ZM620 227L620 275L622 275L622 227L620 227Z",
                fill: element_stroke!(DecoderRS2),
            }
        }
        g {
            id: "decoder_rd_group",
            style: "pointer-events: all;",
            onmouseenter: move |_| {
                hovered_element.set(Some(FiveStageElement::DecoderRD));
            },
            onmouseleave: move |_| {
                hovered_element.set(None);
            },
            path {
                id: "decoder_rd_arrow",
                d: "M660.293 275.707C660.683 276.098 661.317 276.098 661.707 275.707L668.071 269.343C668.462 268.953 668.462 268.319 668.071 267.929C667.681 267.538 667.047 267.538 666.657 267.929L661 273.586L655.343 267.929C654.953 267.538 654.319 267.538 653.929 267.929C653.538 268.319 653.538 268.953 653.929 269.343L660.293 275.707ZM660 227L660 275L662 275L662 227L660 227Z",
                fill: element_stroke!(DecoderRD),
            }
        }
        g {
            id: "decoder_imm_group",
            style: "pointer-events: all;",
            onmouseenter: move |_| {
                hovered_element.set(Some(FiveStageElement::DecoderImm));
            },
            onmouseleave: move |_| {
                hovered_element.set(None);
            },
            path {
                id: "decoder_imm_arrow",
                d: "M758.707 395.707C759.098 395.317 759.098 394.683 758.707 394.293L752.343 387.929C751.953 387.538 751.319 387.538 750.929 387.929C750.538 388.319 750.538 388.953 750.929 389.343L756.586 395L750.929 400.657C750.538 401.047 750.538 401.681 750.929 402.071C751.319 402.462 751.953 402.462 752.343 402.071L758.707 395.707ZM730 396H758V394H730V396Z",
                fill: element_stroke!(DecoderImm),
            }
            line {
                id: "decoder_imm_line1",
                x1: "701",
                y1: "207",
                x2: "730",
                y2: "207",
                stroke: element_stroke!(DecoderImm),
                "stroke-width": "2",
            }
            line {
                id: "decoder_imm_line2",
                x1: "729",
                y1: "396",
                x2: "729",
                y2: "208",
                stroke: element_stroke!(DecoderImm),
                "stroke-width": "2",
            }
        }
        g {
            id: "register_file_group",
            style: "pointer-events: all;",
            onmouseenter: move |_| {
                hovered_element.set(Some(FiveStageElement::RegisterFile));
            },
            onmouseleave: move |_| {
                hovered_element.set(None);
            },
            rect {
                id: "register_file_rect",
                x: "542",
                y: "276.669",
                width: "158",
                height: "202.497",
                stroke: element_stroke!(RegisterFile),
                "stroke-width": "2",
                fill: element_fill!(RegisterFile),
            }
            text {
                x: "621",
                y: "370",
                "font-family": "Arial",
                "font-size": "20",
                "font-weight": "bold",
                "text-anchor": "middle",
                fill: element_stroke!(RegisterFile),
                "REGISTER"
            }
            text {
                x: "621",
                y: "400",
                "font-family": "Arial",
                "font-size": "20",
                "font-weight": "bold",
                "text-anchor": "middle",
                fill: element_stroke!(RegisterFile),
                "FILE"
            }
        }
        g {
            id: "lsu_group",
            style: "pointer-events: all;",
            onmouseenter: move |_| {
                hovered_element.set(Some(FiveStageElement::LSU));
            },
            onmouseleave: move |_| {
                hovered_element.set(None);
            },
            rect {
                id: "lsu_rect",
                x: "1215",
                y: "236",
                width: "238",
                height: "78",
                stroke: element_stroke!(LSU),
                "stroke-width": "2",
                fill: element_fill!(LSU),
            }
            text {
                x: "1334",
                y: "285",
                "font-family": "Arial",
                "font-size": "20",
                "font-weight": "bold",
                "text-anchor": "middle",
                fill: element_stroke!(LSU),
                "LSU"
            }
        }
        g {
            id: "rs1_value_group",
            style: "pointer-events: all;",
            onmouseenter: move |_| {
                hovered_element.set(Some(FiveStageElement::RegisterFileRS1Value));
            },
            onmouseleave: move |_| {
                hovered_element.set(None);
            },
            path {
                id: "rs1_value_arrow",
                d: "M758.707 325.707C759.098 325.317 759.098 324.683 758.707 324.293L752.343 317.929C751.953 317.538 751.319 317.538 750.929 317.929C750.538 318.319 750.538 318.953 750.929 319.343L756.586 325L750.929 330.657C750.538 331.047 750.538 331.681 750.929 332.071C751.319 332.462 751.953 332.462 752.343 332.071L758.707 325.707ZM701 326H758V324H701V326Z",
                fill: element_stroke!(RegisterFileRS1Value),
            }
            text {
                x: "682",
                y: "330",
                "font-family": "Arial",
                "font-size": "14",
                "font-weight": "bold",
                "text-anchor": "middle",
                fill: element_stroke!(RegisterFileRS1Value),
                "RS1"
            }
        }
        g {
            id: "rs2_value_group",
            style: "pointer-events: all;",
            onmouseenter: move |_| {
                hovered_element.set(Some(FiveStageElement::RegisterFileRS2Value));
            },
            onmouseleave: move |_| {
                hovered_element.set(None);
            },
            path {
                id: "rs2_value_arrow",
                d: "M758.707 447.707C759.098 447.317 759.098 446.683 758.707 446.293L752.343 439.929C751.953 439.538 751.319 439.538 750.929 439.929C750.538 440.319 750.538 440.953 750.929 441.343L756.586 447L750.929 452.657C750.538 453.047 750.538 453.681 750.929 454.071C751.319 454.462 751.953 454.462 752.343 454.071L758.707 447.707ZM701 448H758V446H701V448Z",
                fill: element_stroke!(RegisterFileRS2Value),
            }
            text {
                x: "682",
                y: "451",
                "font-family": "Arial",
                "font-size": "14",
                "font-weight": "bold",
                "text-anchor": "middle",
                fill: element_stroke!(RegisterFileRS2Value),
                "RS2"
            }
        }
        g {
            id: "idex_pc_group",
            style: "pointer-events: all;",
            onmouseenter: move |_| {
                hovered_element.set(Some(FiveStageElement::IDEXPC));
            },
            onmouseleave: move |_| {
                hovered_element.set(None);
            },
            rect {
                id: "idex_pc_rect",
                x: "760",
                y: "70",
                width: "78",
                height: "78",
                stroke: element_stroke!(IDEXPC),
                "stroke-width": "2",
                fill: element_fill!(IDEXPC),
            }
            path {
                id: "idex_pc_arrow1",
                d: "M1115.71 109.707C1116.1 109.317 1116.1 108.683 1115.71 108.293L1109.34 101.929C1108.95 101.538 1108.32 101.538 1107.93 101.929C1107.54 102.319 1107.54 102.953 1107.93 103.343L1113.59 109L1107.93 114.657C1107.54 115.047 1107.54 115.681 1107.93 116.071C1108.32 116.462 1108.95 116.462 1109.34 116.071L1115.71 109.707ZM839 110H1115V108H839V110Z",
                fill: element_stroke!(IDEXPC),
            }
            path {
                id: "idex_pc_arrow2",
                d: "M886.707 269.707C887.098 269.317 887.098 268.683 886.707 268.293L880.343 261.929C879.953 261.538 879.319 261.538 878.929 261.929C878.538 262.319 878.538 262.953 878.929 263.343L884.586 269L878.929 274.657C878.538 275.047 878.538 275.681 878.929 276.071C879.319 276.462 879.953 276.462 880.343 276.071L886.707 269.707ZM867 270H886V268H867V270Z",
                fill: element_stroke!(IDEXPC),
            }
            line {
                id: "idex_pc_arrow3",
                x1: "866",
                y1: "270",
                x2: "866",
                y2: "110",
                stroke: element_stroke!(IDEXPC),
                "stroke-width": "2",
            }
            circle {
                id: "idex_pc_node",
                cx: "866",
                cy: "109",
                r: "3",
                fill: element_stroke!(IDEXPC),
            }
            text {
                x: "799",
                y: "115",
                "font-family": "Arial",
                "font-size": "18",
                "font-weight": "bold",
                "text-anchor": "middle",
                fill: element_stroke!(IDEXPC),
                "EXPC"
            }
        }
        g {
            id: "memwb_lsu_group",
            style: "pointer-events: all;",
            onmouseenter: move |_| {
                hovered_element.set(Some(FiveStageElement::MEMWBLsu));
            },
            onmouseleave: move |_| {
                hovered_element.set(None);
            },
            rect {
                id: "memwb_lsu_rect",
                x: "1474",
                y: "393",
                width: "78",
                height: "38",
                stroke: element_stroke!(MEMWBLsu),
                "stroke-width": "2",
                fill: element_fill!(MEMWBLsu),
            }
            path {
                id: "memwb_lsu_arrow",
                d: "M1578.71 412.707C1579.1 412.317 1579.1 411.683 1578.71 411.293L1572.34 404.929C1571.95 404.538 1571.32 404.538 1570.93 404.929C1570.54 405.319 1570.54 405.953 1570.93 406.343L1576.59 412L1570.93 417.657C1570.54 418.047 1570.54 418.681 1570.93 419.071C1571.32 419.462 1571.95 419.462 1572.34 419.071L1578.71 412.707ZM1553 413H1578V411H1553V413Z",
                fill: element_stroke!(MEMWBLsu),
            }
            text {
                x: "1513",
                y: "417",
                "font-family": "Arial",
                "font-size": "18",
                "font-weight": "bold",
                "text-anchor": "middle",
                fill: element_stroke!(MEMWBLsu),
                "LSU"
            }
        }
        g {
            id: "idex_rs1_group",
            style: "pointer-events: all;",
            onmouseenter: move |_| {
                hovered_element.set(Some(FiveStageElement::IDEXRS1));
            },
            onmouseleave: move |_| {
                hovered_element.set(None);
            },
            rect {
                id: "idex_rs1_rect",
                x: "760",
                y: "305",
                width: "78",
                height: "38",
                stroke: element_stroke!(IDEXRS1),
                "stroke-width": "2",
                fill: element_fill!(IDEXRS1),
            }
            path {
                id: "idex_rs1_arrow",
                d: "M886.707 325.707C887.098 325.317 887.098 324.683 886.707 324.293L880.343 317.929C879.953 317.538 879.319 317.538 878.929 317.929C878.538 318.319 878.538 318.953 878.929 319.343L884.586 325L878.929 330.657C878.538 331.047 878.538 331.681 878.929 332.071C879.319 332.462 879.953 332.462 880.343 332.071L886.707 325.707ZM839 326H886V324H839V326Z",
                fill: element_stroke!(IDEXRS1),
            }
            text {
                x: "799",
                y: "329",
                "font-family": "Arial",
                "font-size": "18",
                "font-weight": "bold",
                "text-anchor": "middle",
                fill: element_stroke!(IDEXRS1),
                "RS1"
            }
        }
        g {
            id: "idex_rs2_group",
            style: "pointer-events: all;",
            onmouseenter: move |_| {
                hovered_element.set(Some(FiveStageElement::IDEXRS2));
            },
            onmouseleave: move |_| {
                hovered_element.set(None);
            },
            rect {
                id: "idex_rs2_rect",
                x: "760",
                y: "428",
                width: "78",
                height: "38",
                stroke: element_stroke!(IDEXRS2),
                "stroke-width": "2",
                fill: element_fill!(IDEXRS2),
            }
            path {
                id: "idex_rs2_arrow1",
                d: "M886.707 447.707C887.098 447.317 887.098 446.683 886.707 446.293L880.343 439.929C879.953 439.538 879.319 439.538 878.929 439.929C878.538 440.319 878.538 440.953 878.929 441.343L884.586 447L878.929 452.657C878.538 453.047 878.538 453.681 878.929 454.071C879.319 454.462 879.953 454.462 880.343 454.071L886.707 447.707ZM839 448H886V446H839V448Z",
                fill: element_stroke!(IDEXRS2),
            }
            path {
                id: "idex_rs2_arrow2",
                d: "M1115.71 447.707C1116.1 447.317 1116.1 446.683 1115.71 446.293L1109.34 439.929C1108.95 439.538 1108.32 439.538 1107.93 439.929C1107.54 440.319 1107.54 440.953 1107.93 441.343L1113.59 447L1107.93 452.657C1107.54 453.047 1107.54 453.681 1107.93 454.071C1108.32 454.462 1108.95 454.462 1109.34 454.071L1115.71 447.707ZM1091 448H1115V446H1091V448Z",
                fill: element_stroke!(IDEXRS2),
            }
            line {
                id: "idex_rs2_line1",
                x1: "863",
                y1: "446",
                x2: "863",
                y2: "488",
                stroke: element_stroke!(IDEXRS2),
                "stroke-width": "2",
            }
            line {
                id: "idex_rs2_line2",
                x1: "863",
                y1: "487",
                x2: "1091",
                y2: "487",
                stroke: element_stroke!(IDEXRS2),
                "stroke-width": "2",
            }
            line {
                id: "idex_rs2_line3",
                x1: "1092",
                y1: "446",
                x2: "1092",
                y2: "488",
                stroke: element_stroke!(IDEXRS2),
                "stroke-width": "2",
            }
            circle {
                id: "idex_rs2_node",
                cx: "863",
                cy: "448",
                r: "3",
                fill: element_stroke!(IDEXRS2),
            }
            text {
                x: "799",
                y: "454",
                "font-family": "Arial",
                "font-size": "18",
                "font-weight": "bold",
                "text-anchor": "middle",
                fill: element_stroke!(IDEXRS2),
                "RS2"
            }
        }
        g {
            id: "idex_imm_group",
            style: "pointer-events: all;",
            onmouseenter: move |_| {
                hovered_element.set(Some(FiveStageElement::IDEXImm));
            },
            onmouseleave: move |_| {
                hovered_element.set(None);
            },
            rect {
                id: "idex_imm_rect",
                x: "760",
                y: "376",
                width: "78",
                height: "38",
                stroke: element_stroke!(IDEXImm),
                "stroke-width": "2",
                fill: element_fill!(IDEXImm),
            }
            path {
                id: "idex_imm_arrow",
                d: "M886.707 395.707C887.098 395.317 887.098 394.683 886.707 394.293L880.343 387.929C879.953 387.538 879.319 387.538 878.929 387.929C878.538 388.319 878.538 388.953 878.929 389.343L884.586 395L878.929 400.657C878.538 401.047 878.538 401.681 878.929 402.071C879.319 402.462 879.953 402.462 880.343 402.071L886.707 395.707ZM839 396H886V394H839V396Z",
                fill: element_stroke!(IDEXImm),
            }
            text {
                x: "799",
                y: "402",
                "font-family": "Arial",
                "font-size": "18",
                "font-weight": "bold",
                "text-anchor": "middle",
                fill: element_stroke!(IDEXImm),
                "IMM"
            }
        }
        g {
            id: "branch_unit_group",
            style: "pointer-events: all;",
            onmouseenter: move |_| {
                hovered_element.set(Some(FiveStageElement::BranchUnit));
            },
            onmouseleave: move |_| {
                hovered_element.set(None);
            },
            rect {
                id: "branch_unit_rect",
                x: "878",
                y: "11",
                width: "163",
                height: "57",
                stroke: element_stroke!(BranchUnit),
                "stroke-width": "2",
                fill: element_fill!(BranchUnit),
            }
            path {
                id: "branch_unit_arrow",
                d: "M78.2929 38.293C77.9024 38.6835 77.9024 39.3167 78.2929 39.7072L84.6569 46.0711C85.0474 46.4617 85.6805 46.4617 86.0711 46.0711C86.4616 45.6806 86.4616 45.0474 86.0711 44.6569L80.4142 39.0001L86.0711 33.3432C86.4616 32.9527 86.4616 32.3195 86.0711 31.929C85.6805 31.5385 85.0474 31.5385 84.6569 31.929L78.2929 38.293ZM877 38L79 38.0001L79 40.0001L877 40L877 38Z",
                fill: element_stroke!(BranchUnit),
            }
            text {
                x: "960",
                y: "45",
                "font-family": "Arial",
                "font-size": "18",
                "font-weight": "bold",
                "text-anchor": "middle",
                fill: element_stroke!(BranchUnit),
                "BRANCH UNIT"
            }
        }
        g {
            id: "opa_mux_group",
            style: "pointer-events: all;",
            onmouseenter: move |_| {
                hovered_element.set(Some(FiveStageElement::ALUMuxA));
            },
            onmouseleave: move |_| {
                hovered_element.set(None);
            },
            path {
                id: "opa_mux",
                d: "M888 243.558L946 270.142V323.858L888 350.442V243.558Z",
                stroke: element_stroke!(ALUMuxA),
                "stroke-width": "2",
                fill: element_fill!(ALUMuxA),
            }
            path {
                id: "opa_arrow",
                d: "M976.707 297.707C977.098 297.317 977.098 296.683 976.707 296.293L970.343 289.929C969.953 289.538 969.319 289.538 968.929 289.929C968.538 290.319 968.538 290.953 968.929 291.343L974.586 297L968.929 302.657C968.538 303.047 968.538 303.681 968.929 304.071C969.319 304.462 969.953 304.462 970.343 304.071L976.707 297.707ZM947 298H976V296H947V298Z",
                fill: element_stroke!(ALUMuxA),
            }
            text {
                x: "902",
                y: "273",
                "font-family": "Arial",
                "font-size": "14",
                "font-weight": "bold",
                "text-anchor": "middle",
                fill: element_stroke!(ALUMuxA),
                "PC"
            }
            text {
                x: "905",
                y: "330",
                "font-family": "Arial",
                "font-size": "14",
                "font-weight": "bold",
                "text-anchor": "middle",
                fill: element_stroke!(ALUMuxA),
                "RS1"
            }
        }
        g {
            id: "opb_mux_group",
            style: "pointer-events: all;",
            onmouseenter: move |_| {
                hovered_element.set(Some(FiveStageElement::ALUMuxB));
            },
            onmouseleave: move |_| {
                hovered_element.set(None);
            },
            path {
                id: "opb_mux",
                d: "M888 365.558L946 392.142V445.858L888 472.442V365.558Z",
                stroke: element_stroke!(ALUMuxB),
                "stroke-width": "2",
                fill: element_fill!(ALUMuxB),
            }
            path {
                id: "opb_arrow",
                d: "M976.707 419.707C977.098 419.317 977.098 418.683 976.707 418.293L970.343 411.929C969.953 411.538 969.319 411.538 968.929 411.929C968.538 412.319 968.538 412.953 968.929 413.343L974.586 419L968.929 424.657C968.538 425.047 968.538 425.681 968.929 426.071C969.319 426.462 969.953 426.462 970.343 426.071L976.707 419.707ZM947 420H976V418H947V420Z",
                fill: element_stroke!(ALUMuxB),
            }
            text {
                x: "905",
                y: "400",
                "font-family": "Arial",
                "font-size": "14",
                "font-weight": "bold",
                "text-anchor": "middle",
                fill: element_stroke!(ALUMuxB),
                "IMM"
            }
            text {
                x: "900",
                y: "426",
                "font-family": "Arial",
                "font-size": "14",
                "font-weight": "bold",
                "text-anchor": "middle",
                fill: element_stroke!(ALUMuxB),
                "#4"
            }
            text {
                x: "905",
                y: "452",
                "font-family": "Arial",
                "font-size": "14",
                "font-weight": "bold",
                "text-anchor": "middle",
                fill: element_stroke!(ALUMuxB),
                "RS2"
            }
        }
        g {
            id: "alu_group",
            style: "pointer-events: all;",
            onmouseenter: move |_| {
                hovered_element.set(Some(FiveStageElement::ALU));
            },
            onmouseleave: move |_| {
                hovered_element.set(None);
            },
            rect {
                id: "alu_rect",
                x: "978",
                y: "259",
                width: "86.7684",
                height: "198.341",
                stroke: element_stroke!(ALU),
                "stroke-width": "2",
                fill: element_fill!(ALU),
            }
            path {
                id: "alu_arrow1",
                d: "M1115.71 358.707C1116.1 358.317 1116.1 357.683 1115.71 357.293L1109.34 350.929C1108.95 350.538 1108.32 350.538 1107.93 350.929C1107.54 351.319 1107.54 351.953 1107.93 352.343L1113.59 358L1107.93 363.657C1107.54 364.047 1107.54 364.681 1107.93 365.071C1108.32 365.462 1108.95 365.462 1109.34 365.071L1115.71 358.707ZM1066 359H1115V357H1066V359Z",
                fill: element_stroke!(ALU),
            }
            path {
                id: "alu_arrow2",
                d: "M1042.29 38.2929C1041.9 38.6834 1041.9 39.3166 1042.29 39.7071L1048.66 46.0711C1049.05 46.4616 1049.68 46.4616 1050.07 46.0711C1050.46 45.6805 1050.46 45.0474 1050.07 44.6569L1044.41 39L1050.07 33.3431C1050.46 32.9526 1050.46 32.3195 1050.07 31.9289C1049.68 31.5384 1049.05 31.5384 1048.66 31.9289L1042.29 38.2929ZM1088 38L1043 38V40L1088 40V38Z",
                fill: element_stroke!(ALU),
            }
            circle {
                id: "alu_node",
                cx: "1087",
                cy: "358",
                r: "3",
                fill: element_stroke!(ALU),
            }
            line {
                id: "alu_line",
                x1: "1087",
                y1: "358",
                x2: "1087",
                y2: "39",
                stroke: element_stroke!(ALU),
                "stroke-width": "2",
            }
            text {
                x: "1022",
                y: "366",
                "font-family": "Arial",
                "font-size": "24",
                "font-weight": "bold",
                "text-anchor": "middle",
                fill: element_stroke!(ALU),
                "ALU"
            }
            text {
                x: "983",
                y: "302",
                "font-family": "Arial",
                "font-size": "14",
                "font-weight": "bold",
                "text-anchor": "start",
                fill: element_stroke!(ALU),
                "OPA"
            }
            text {
                x: "983",
                y: "424",
                "font-family": "Arial",
                "font-size": "14",
                "font-weight": "bold",
                "text-anchor": "start",
                fill: element_stroke!(ALU),
                "OPB"
            }
        }
        g {
            id: "exmem_pc_group",
            style: "pointer-events: all;",
            onmouseenter: move |_| {
                hovered_element.set(Some(FiveStageElement::EXMEMPC));
            },
            onmouseleave: move |_| {
                hovered_element.set(None);
            },
            rect {
                id: "exmem_pc_rect",
                x: "1117",
                y: "70",
                width: "78",
                height: "78",
                stroke: element_stroke!(EXMEMPC),
                "stroke-width": "2",
                fill: element_fill!(EXMEMPC),
            }
            path {
                id: "exmem_pc_arrow",
                d: "M1472.71 109.707C1473.1 109.317 1473.1 108.683 1472.71 108.293L1466.34 101.929C1465.95 101.538 1465.32 101.538 1464.93 101.929C1464.54 102.319 1464.54 102.953 1464.93 103.343L1470.59 109L1464.93 114.657C1464.54 115.047 1464.54 115.681 1464.93 116.071C1465.32 116.462 1465.95 116.462 1466.34 116.071L1472.71 109.707ZM1196 110H1472V108H1196V110Z",
                fill: element_stroke!(EXMEMPC),
            }
            text {
                x: "1156",
                y: "118",
                "font-family": "Arial",
                "font-size": "18",
                "font-weight": "bold",
                "text-anchor": "middle",
                fill: element_stroke!(EXMEMPC),
                "MEMPC"
            }
        }
        g {
            id: "exmem_rs2_group",
            style: "pointer-events: all;",
            onmouseenter: move |_| {
                hovered_element.set(Some(FiveStageElement::EXMEMRS2));
            },
            onmouseleave: move |_| {
                hovered_element.set(None);
            },
            rect {
                id: "ex_mem_rs2_rect",
                x: "1117",
                y: "428",
                width: "78",
                height: "38",
                stroke: element_stroke!(EXMEMRS2),
                "stroke-width": "2",
                fill: element_fill!(EXMEMRS2),
            }
            line {
                id: "exmem_rs2_value_line",
                x1: "1196",
                y1: "446",
                x2: "1337",
                y2: "446",
                stroke: element_stroke!(EXMEMRS2),
                "stroke-width": "2",
            }
            path {
                id: "exmem_rs2_value_arrow",
                d: "M1336.71 315.293C1336.32 314.902 1335.68 314.902 1335.29 315.293L1328.93 321.657C1328.54 322.047 1328.54 322.681 1328.93 323.071C1329.32 323.462 1329.95 323.462 1330.34 323.071L1336 317.414L1341.66 323.071C1342.05 323.462 1342.68 323.462 1343.07 323.071C1343.46 322.681 1343.46 322.047 1343.07 321.657L1336.71 315.293ZM1337 447L1337 316L1335 316L1335 447L1337 447Z",
                fill: element_stroke!(EXMEMRS2),
            }
            text {
                x: "1156",
                y: "453",
                "font-family": "Arial",
                "font-size": "18",
                "font-weight": "bold",
                "text-anchor": "middle",
                fill: element_stroke!(EXMEMRS2),
                "RS2"
            }
            text {
                x: "1337",
                y: "309",
                "font-family": "Arial",
                "font-size": "11",
                "font-weight": "bold",
                "text-anchor": "middle",
                fill: element_stroke!(EXMEMRS2),
                "WR_DATA"
            }
        }
        g {
            id: "exmem_alu_group",
            style: "pointer-events: all;",
            onmouseenter: move |_| {
                hovered_element.set(Some(FiveStageElement::EXMEMAlu));
            },
            onmouseleave: move |_| {
                hovered_element.set(None);
            },
            rect {
                id: "exmem_alu_rect",
                x: "1117",
                y: "339",
                width: "78",
                height: "38",
                stroke: element_stroke!(EXMEMAlu),
                "stroke-width": "2",
                fill: element_fill!(EXMEMAlu),
            }
            path {
                id: "exmem_alu_arrow1",
                d: "M1472.71 358.707C1473.1 358.317 1473.1 357.683 1472.71 357.293L1466.34 350.929C1465.95 350.538 1465.32 350.538 1464.93 350.929C1464.54 351.319 1464.54 351.953 1464.93 352.343L1470.59 358L1464.93 363.657C1464.54 364.047 1464.54 364.681 1464.93 365.071C1465.32 365.462 1465.95 365.462 1466.34 365.071L1472.71 358.707ZM1196 359H1472V357H1196V359Z",
                fill: element_stroke!(EXMEMAlu),
            }
            path {
                id: "exmem_alu_arrow2",
                d: "M1253.71 315.293C1253.32 314.902 1252.68 314.902 1252.29 315.293L1245.93 321.657C1245.54 322.047 1245.54 322.681 1245.93 323.071C1246.32 323.462 1246.95 323.462 1247.34 323.071L1253 317.414L1258.66 323.071C1259.05 323.462 1259.68 323.462 1260.07 323.071C1260.46 322.681 1260.46 322.047 1260.07 321.657L1253.71 315.293ZM1254 358L1254 316L1252 316L1252 358L1254 358Z",
                fill: element_stroke!(EXMEMAlu),
            }
            circle {
                id: "exmem_alu_node",
                cx: "1253",
                cy: "358",
                r: "3",
                fill: element_stroke!(EXMEMAlu),
            }
            text {
                x: "1156",
                y: "364",
                "font-family": "Arial",
                "font-size": "18",
                "font-weight": "bold",
                "text-anchor": "middle",
                fill: element_stroke!(EXMEMAlu),
                "ALU"
            }
            text {
                x: "1253",
                y: "309",
                "font-family": "Arial",
                "font-size": "11",
                "font-weight": "bold",
                "text-anchor": "middle",
                fill: element_stroke!(EXMEMAlu),
                "ADDR"
            }
        }
        g {
            id: "data_memory_group",
            style: "pointer-events: all;",
            onmouseenter: move |_| {
                hovered_element.set(Some(FiveStageElement::DataMemory));
            },
            onmouseleave: move |_| {
                hovered_element.set(None);
            },
            rect {
                id: "data_memory_rect",
                x: "1209",
                y: "131",
                width: "254",
                height: "78",
                stroke: element_stroke!(DataMemory),
                "stroke-width": "2",
                fill: element_fill!(DataMemory),
            }
            text {
                x: "1334",
                y: "180",
                "font-family": "Arial",
                "font-size": "20",
                "font-weight": "bold",
                "text-anchor": "middle",
                fill: element_stroke!(DataMemory),
                "DATA MEMORY"
            }
        }
        g {
            id: "addr_group",
            style: "pointer-events: all;",
            onmouseenter: move |_| {
                hovered_element.set(Some(FiveStageElement::LSUADDR));
            },
            onmouseleave: move |_| {
                hovered_element.set(None);
            },
            path {
                id: "addr_arrow",
                d: "M1229.71 210.293C1229.32 209.902 1228.68 209.902 1228.29 210.293L1221.93 216.657C1221.54 217.047 1221.54 217.681 1221.93 218.071C1222.32 218.462 1222.95 218.462 1223.34 218.071L1229 212.414L1234.66 218.071C1235.05 218.462 1235.68 218.462 1236.07 218.071C1236.46 217.681 1236.46 217.047 1236.07 216.657L1229.71 210.293ZM1230 235V211H1228V235H1230Z",
                fill: element_stroke!(LSUADDR),
            }
            text {
                x: "1229",
                y: "204",
                "font-family": "Arial",
                "font-size": "11",
                "font-weight": "bold",
                "text-anchor": "middle",
                fill: element_stroke!(LSUADDR),
                "ADDR"
            }
        }
        g {
            id: "data_arrow_group",
            style: "pointer-events: all;",
            onmouseenter: move |_| {
                hovered_element.set(Some(FiveStageElement::LSUDATA));
            },
            onmouseleave: move |_| {
                hovered_element.set(None);
            },
            path {
                id: "data_arrow1",
                d: "M1267.71 210.293C1267.32 209.902 1266.68 209.902 1266.29 210.293L1259.93 216.657C1259.54 217.047 1259.54 217.681 1259.93 218.071C1260.32 218.462 1260.95 218.462 1261.34 218.071L1267 212.414L1272.66 218.071C1273.05 218.462 1273.68 218.462 1274.07 218.071C1274.46 217.681 1274.46 217.047 1274.07 216.657L1267.71 210.293ZM1268 235L1268 211L1266 211L1266 235L1268 235Z",
                fill: element_stroke!(LSUDATA),
            }
            path {
                id: "data_arrow2",
                d: "M1286.29 234.707C1286.68 235.098 1287.32 235.098 1287.71 234.707L1294.07 228.343C1294.46 227.953 1294.46 227.319 1294.07 226.929C1293.68 226.538 1293.05 226.538 1292.66 226.929L1287 232.586L1281.34 226.929C1280.95 226.538 1280.32 226.538 1279.93 226.929C1279.54 227.319 1279.54 227.953 1279.93 228.343L1286.29 234.707ZM1286 210V234H1288V210H1286Z",
                fill: element_stroke!(LSUDATA),
            }
            text {
                x: "1276",
                y: "204",
                "font-family": "Arial",
                "font-size": "11",
                "font-weight": "bold",
                "text-anchor": "middle",
                fill: element_stroke!(LSUDATA),
                "DATA"
            }
        }
        g {
            id: "wr_group",
            style: "pointer-events: all;",
            onmouseenter: move |_| {
                hovered_element.set(Some(FiveStageElement::LSUWR));
            },
            onmouseleave: move |_| {
                hovered_element.set(None);
            },
            path {
                id: "wr_arrow",
                d: "M1354.71 210.293C1354.32 209.902 1353.68 209.902 1353.29 210.293L1346.93 216.657C1346.54 217.047 1346.54 217.681 1346.93 218.071C1347.32 218.462 1347.95 218.462 1348.34 218.071L1354 212.414L1359.66 218.071C1360.05 218.462 1360.68 218.462 1361.07 218.071C1361.46 217.681 1361.46 217.047 1361.07 216.657L1354.71 210.293ZM1355 235V211H1353V235H1355Z",
                fill: element_stroke!(LSUWR),
            }
            text {
                x: "1354",
                y: "204",
                "font-family": "Arial",
                "font-size": "11",
                "font-weight": "bold",
                "text-anchor": "middle",
                fill: element_stroke!(LSUWR),
                "WR"
            }
        }
        g {
            id: "valid_group",
            style: "pointer-events: all;",
            onmouseenter: move |_| {
                hovered_element.set(Some(FiveStageElement::LSUVALID));
            },
            onmouseleave: move |_| {
                hovered_element.set(None);
            },
            path {
                id: "valid_arrow",
                d: "M1438.29 234.707C1438.68 235.098 1439.32 235.098 1439.71 234.707L1446.07 228.343C1446.46 227.953 1446.46 227.319 1446.07 226.929C1445.68 226.538 1445.05 226.538 1444.66 226.929L1439 232.586L1433.34 226.929C1432.95 226.538 1432.32 226.538 1431.93 226.929C1431.54 227.319 1431.54 227.953 1431.93 228.343L1438.29 234.707ZM1438 210V234H1440V210H1438Z",
                fill: element_stroke!(LSUVALID),
            }
            text {
                x: "1439",
                y: "204",
                "font-family": "Arial",
                "font-size": "11",
                "font-weight": "bold",
                "text-anchor": "middle",
                fill: element_stroke!(LSUVALID),
                "VALID"
            }
        }
        g {
            id: "byte_en_group",
            style: "pointer-events: all;",
            onmouseenter: move |_| {
                hovered_element.set(Some(FiveStageElement::LSUBYTE_EN));
            },
            onmouseleave: move |_| {
                hovered_element.set(None);
            },
            path {
                id: "byte_en_arrow",
                d: "M1397.71 210.293C1397.32 209.902 1396.68 209.902 1396.29 210.293L1389.93 216.657C1389.54 217.047 1389.54 217.681 1389.93 218.071C1390.32 218.462 1390.95 218.462 1391.34 218.071L1397 212.414L1402.66 218.071C1403.05 218.462 1403.68 218.462 1404.07 218.071C1404.46 217.681 1404.46 217.047 1404.07 216.657L1397.71 210.293ZM1398 235V211H1396V235H1398Z",
                fill: element_stroke!(LSUBYTE_EN),
            }
            text {
                x: "1395",
                y: "204",
                "font-family": "Arial",
                "font-size": "11",
                "font-weight": "bold",
                "text-anchor": "middle",
                fill: element_stroke!(LSUBYTE_EN),
                "BYTE_EN"
            }
        }
        g {
            id: "req_group",
            style: "pointer-events: all;",
            onmouseenter: move |_| {
                hovered_element.set(Some(FiveStageElement::LSUREQ));
            },
            onmouseleave: move |_| {
                hovered_element.set(None);
            },
            path {
                id: "req_arrow",
                d: "M1321.71 210.293C1321.32 209.902 1320.68 209.902 1320.29 210.293L1313.93 216.657C1313.54 217.047 1313.54 217.681 1313.93 218.071C1314.32 218.462 1314.95 218.462 1315.34 218.071L1321 212.414L1326.66 218.071C1327.05 218.462 1327.68 218.462 1328.07 218.071C1328.46 217.681 1328.46 217.047 1328.07 216.657L1321.71 210.293ZM1322 235V211H1320V235H1322Z",
                fill: element_stroke!(LSUREQ),
            }
            text {
                x: "1321",
                y: "204",
                "font-family": "Arial",
                "font-size": "11",
                "font-weight": "bold",
                "text-anchor": "middle",
                fill: element_stroke!(LSUREQ),
                "REQ"
            }
        }
        g {
            id: "rd_data_group",
            style: "pointer-events: all;",
            onmouseenter: move |_| {
                hovered_element.set(Some(FiveStageElement::LSURDOut));
            },
            onmouseleave: move |_| {
                hovered_element.set(None);
            },
            line {
                id: "rd_data_line",
                x1: "1417",
                y1: "315",
                x2: "1417",
                y2: "412",
                stroke: element_stroke!(LSURDOut),
                "stroke-width": "2",
            }
            path {
                id: "rd_data_arrow",
                d: "M1472.71 412.707C1473.1 412.317 1473.1 411.683 1472.71 411.293L1466.34 404.929C1465.95 404.538 1465.32 404.538 1464.93 404.929C1464.54 405.319 1464.54 405.953 1464.93 406.343L1470.59 412L1464.93 417.657C1464.54 418.047 1464.54 418.681 1464.93 419.071C1465.32 419.462 1465.95 419.462 1466.34 419.071L1472.71 412.707ZM1416 413H1472V411H1416V413Z",
                fill: element_stroke!(LSURDOut),
            }
            text {
                x: "1417",
                y: "309",
                "font-family": "Arial",
                "font-size": "11",
                "font-weight": "bold",
                "text-anchor": "middle",
                fill: element_stroke!(LSURDOut),
                "DATA_OUT"
            }
        }
        g {
            id: "memwb_pc_group",
            style: "pointer-events: all;",
            onmouseenter: move |_| {
                hovered_element.set(Some(FiveStageElement::MEMWBPC));
            },
            onmouseleave: move |_| {
                hovered_element.set(None);
            },
            rect {
                id: "memwb_pc_rect",
                x: "1474",
                y: "70",
                width: "78",
                height: "78",
                stroke: element_stroke!(MEMWBPC),
                "stroke-width": "2",
                fill: element_fill!(MEMWBPC),
            }
            text {
                x: "1513",
                y: "118",
                "font-family": "Arial",
                "font-size": "18",
                "font-weight": "bold",
                "text-anchor": "middle",
                fill: element_stroke!(MEMWBPC),
                "WBPC"
            }
        }
        g {
            id: "memwb_alu_group",
            style: "pointer-events: all;",
            onmouseenter: move |_| {
                hovered_element.set(Some(FiveStageElement::MEMWBAlu));
            },
            onmouseleave: move |_| {
                hovered_element.set(None);
            },
            rect {
                id: "memwb_alu_rect",
                x: "1474",
                y: "341",
                width: "78",
                height: "38",
                stroke: element_stroke!(MEMWBAlu),
                "stroke-width": "2",
                fill: element_fill!(MEMWBAlu),
            }
            path {
                id: "memwb_alu_arrow",
                d: "M1578.71 360.707C1579.1 360.317 1579.1 359.683 1578.71 359.293L1572.34 352.929C1571.95 352.538 1571.32 352.538 1570.93 352.929C1570.54 353.319 1570.54 353.953 1570.93 354.343L1576.59 360L1570.93 365.657C1570.54 366.047 1570.54 366.681 1570.93 367.071C1571.32 367.462 1571.95 367.462 1572.34 367.071L1578.71 360.707ZM1553 361H1578V359H1553V361Z",
                fill: element_stroke!(MEMWBAlu),
            }
            text {
                x: "1513",
                y: "366",
                "font-family": "Arial",
                "font-size": "18",
                "font-weight": "bold",
                "text-anchor": "middle",
                fill: element_stroke!(MEMWBAlu),
                "ALU"
            }
        }
        g {
            id: "writeback_group",
            style: "pointer-events: all;",
            onmouseenter: move |_| {
                hovered_element.set(Some(FiveStageElement::WritebackResult));
            },
            onmouseleave: move |_| {
                hovered_element.set(None);
            },
            path {
                id: "writeback_mux",
                d: "M1580 331.558L1638 358.142V411.858L1580 438.442V331.558Z",
                stroke: element_stroke!(WritebackResult),
                "stroke-width": "2",
                fill: element_fill!(WritebackResult),
            }
            line {
                id: "writeback_line1",
                x1: "1639",
                y1: "384",
                x2: "1664",
                y2: "384",
                stroke: element_stroke!(WritebackResult),
                "stroke-width": "2",
            }
            line {
                id: "writeback_line2",
                x1: "1664",
                y1: "383",
                x2: "1664",
                y2: "581",
                stroke: element_stroke!(WritebackResult),
                "stroke-width": "2",
            }
            line {
                id: "writeback_line3",
                x1: "512",
                y1: "580",
                x2: "1663",
                y2: "580",
                stroke: element_stroke!(WritebackResult),
                "stroke-width": "2",
            }
            line {
                id: "writeback_line4",
                x1: "511",
                y1: "581",
                x2: "511",
                y2: "447",
                stroke: element_stroke!(WritebackResult),
                "stroke-width": "2",
            }
            path {
                id: "writeback_arrow",
                d: "M540.707 447.707C541.098 447.317 541.098 446.683 540.707 446.293L534.343 439.929C533.953 439.538 533.319 439.538 532.929 439.929C532.538 440.319 532.538 440.953 532.929 441.343L538.586 447L532.929 452.657C532.538 453.047 532.538 453.681 532.929 454.071C533.319 454.462 533.953 454.462 534.343 454.071L540.707 447.707ZM510 448H540V446H510V448Z",
                fill: element_stroke!(WritebackResult),
            }
            text {
                x: "1595",
                y: "365",
                "font-family": "Arial",
                "font-size": "12",
                "font-weight": "bold",
                "text-anchor": "middle",
                fill: element_stroke!(WritebackResult),
                "ALU"
            }
            text {
                x: "1595",
                y: "416",
                "font-family": "Arial",
                "font-size": "12",
                "font-weight": "bold",
                "text-anchor": "middle",
                fill: element_stroke!(WritebackResult),
                "LSU"
            }
        }
        g {
            id: "controller_group",
            style: "pointer-events: all;",
            onmouseenter: move |_| {
                hovered_element.set(Some(FiveStageElement::ControlUnit));
            },
            onmouseleave: move |_| {
                hovered_element.set(None);
            },
            rect {
                id: "controller_rect",
                x: "1",
                y: "599",
                width: "1663",
                height: "84",
                stroke: element_stroke!(ControlUnit),
                "stroke-width": "2",
                fill: element_fill!(ControlUnit),
            }
            text {
                x: "832",
                y: "651",
                "font-family": "Arial",
                "font-size": "20",
                "font-weight": "bold",
                "text-anchor": "middle",
                fill: element_stroke!(ControlUnit),
                "CONTROLLER"
            }
        }
    }
}
