use std::{collections::BTreeSet, ops::Deref};

use dioxus::prelude::*;

use emugator_core::emulator::{
    AnyEmulatorState,
    controller_common::{DataDestSel, LSUDataType, OpASel, OpBSel, PCSel},
    five_stage::FiveStagePipeline,
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

#[derive(Debug, PartialEq, Clone, Copy, Eq, PartialOrd, Ord)]
enum FiveStageElement {
    IFPC,
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
    JMPAddress,
    JMPBaseAddress,
    HazardUnit,
    IFPCWriteEnable,
    IFIDWriteEnable,
    IDEXWriteEnable,
    EXControl,
    MEMControl,
    WBControl,
    PCNextSelControlSignal,
    JumpUncondControlSignal,
    JumpCondControlSignal,
    ALUOpControlSignal,
    ALUOpASelControlSignal,
    ALUOpBSelControlSignal,
    LSUDataTypeControlSignal,
    LSURequestControlSignal,
    LSUWriteEnableControlSignal,
    LSUSignExtControlSignal,
    WBSrcControlSignal,
    RegWriteControlSignal,
    JMPBaseControlSignal,
    ConditionResult,
}

impl FiveStageElement {
    fn tooltip_text(&self, pipeline: &FiveStagePipeline) -> String {
        match self {
            FiveStageElement::ControlUnit => "ID Control (Forwarding EX/MEM/WB)".to_string(),
            FiveStageElement::EXControl => "EX Control (Forwarding MEM/WB)".to_string(),
            FiveStageElement::MEMControl => "MEM Control (Forwarding WB)".to_string(),
            FiveStageElement::WBControl => "WB Control".to_string(),
            FiveStageElement::IFPC => {
                format!("IF PC: 0x{:08X}", pipeline.if_pc)
            }
            FiveStageElement::PCNextSelControlSignal => match pipeline.if_lines.next_pc_sel {
                PCSel::JMP => "Next PC Select: JMP".to_string(),
                PCSel::PC4 => "Next PC Select: PC+4".to_string(),
            },
            FiveStageElement::ConditionResult => {
                let cond_jump_taken =
                    pipeline.ex_lines.alu_out.is_some() && pipeline.ex_control.jump_cond;
                format!(
                    "Condition AND Result: {}",
                    if cond_jump_taken { "True" } else { "False" }
                )
            }
            FiveStageElement::ALUOpControlSignal => {
                format!(
                    "ALU Operation: {}",
                    match pipeline.ex_control.alu_op {
                        Some(op) => format!("{:?}", op),
                        None => "DON'T CARE".to_string(),
                    }
                )
            }
            FiveStageElement::ALUOpASelControlSignal => {
                format!(
                    "ALU OP A Select: {}",
                    match pipeline.ex_control.alu_op_a_sel {
                        Some(OpASel::PC) => "PC",
                        Some(OpASel::RF) => "Register File",
                        None => "DON'T CARE",
                    }
                )
            }
            FiveStageElement::ALUOpBSelControlSignal => {
                format!(
                    "ALU OP B Select: {}",
                    match pipeline.ex_control.alu_op_b_sel {
                        Some(OpBSel::IMM) => "Immediate",
                        Some(OpBSel::RF) => "Register File",
                        Some(OpBSel::Four) => "Four",
                        None => "DON'T CARE",
                    }
                )
            }
            FiveStageElement::JMPBaseControlSignal => {
                format!(
                    "Jump Base: {}",
                    match pipeline.ex_control.jmp_base {
                        Some(OpASel::PC) => "PC",
                        Some(OpASel::RF) => "Register File",
                        None => "DON'T CARE",
                    }
                )
            }
            FiveStageElement::JumpUncondControlSignal => {
                let is_active = pipeline.ex_control.jump_uncond;
                format!(
                    "Unconditional Jump: {}",
                    if is_active { "True" } else { "False" }
                )
            }
            FiveStageElement::JumpCondControlSignal => {
                let is_active = pipeline.ex_control.jump_cond;
                format!(
                    "Conditional Jump: {}",
                    if is_active { "True" } else { "False" }
                )
            }
            FiveStageElement::LSUDataTypeControlSignal => {
                match pipeline.mem_control.lsu_data_type {
                    Some(LSUDataType::Word) => "LSU Data Type: Word (32-bit)".to_string(),
                    Some(LSUDataType::HalfWord) => "LSU Data Type: Half Word (16-bit)".to_string(),
                    Some(LSUDataType::Byte) => "LSU Data Type: Byte (8-bit)".to_string(),
                    None => "LSU Data Type: DON'T CARE".to_string(),
                }
            }
            FiveStageElement::LSURequestControlSignal => match pipeline.mem_control.lsu_request {
                true => "LSU Request: Enabled".to_string(),
                false => "LSU Request: Disabled".to_string(),
            },
            FiveStageElement::LSUWriteEnableControlSignal => {
                let is_active = pipeline.mem_control.lsu_write_enable;
                format!(
                    "LSU Write Enable: {}",
                    if is_active { "Enabled" } else { "Disabled" }
                )
            }
            FiveStageElement::LSUSignExtControlSignal => {
                let sign_ext = pipeline.mem_control.lsu_sign_ext;
                format!(
                    "LSU Sign Extension: {}",
                    if sign_ext { "Enabled" } else { "Disabled" }
                )
            }
            FiveStageElement::WBSrcControlSignal => match pipeline.wb_control.wb_src {
                Some(DataDestSel::ALU) => "Write Back Source: ALU".to_string(),
                Some(DataDestSel::LSU) => "Write Back Source: LSU".to_string(),
                None => "Write Back Source: DON'T CARE".to_string(),
            },
            FiveStageElement::RegWriteControlSignal => {
                let is_active = pipeline.wb_control.reg_write;
                format!(
                    "Register Write: {}",
                    if is_active { "Enabled" } else { "Disabled" }
                )
            }
            FiveStageElement::PCMux => format_opt!("Next PC: 0x{:08X}", pipeline.if_lines.next_pc),
            FiveStageElement::PCPlus4 => {
                format!("PC+4: 0x{:08X}", pipeline.if_pc.wrapping_add(4))
            }
            FiveStageElement::InstructionMemory => {
                format_opt!("Instruction: 0x{:08X}", pipeline.if_lines.instr)
            }
            // IF/ID Buffer
            FiveStageElement::IFIDBuffer => "IF/ID Pipeline Buffer".to_string(),
            FiveStageElement::IFIDPC => format_opt!("ID PC: 0x{:08X}", pipeline.if_id.id_pc),
            FiveStageElement::IFIDInstruction => {
                format_opt!("ID PC: 0x{:08X}", pipeline.if_id.id_inst)
            }
            // ID stage
            FiveStageElement::RegisterFile => "Register File".to_string(),
            FiveStageElement::Decoder => "Instruction Decoder".to_string(),
            FiveStageElement::DecoderRS1 => format!("RS1: {}", pipeline.id_lines.rs1),
            FiveStageElement::DecoderRS2 => format!("RS2: {}", pipeline.id_lines.rs2),
            FiveStageElement::DecoderRD => format!("RD: {}", pipeline.id_lines.rd),
            FiveStageElement::DecoderImm => format_opt!("IMM: 0x{:08X}", pipeline.id_lines.imm),
            FiveStageElement::RegisterFileRS1Value => {
                format!("RS1 Value: 0x{:08X}", pipeline.id_lines.rs1_v)
            }
            FiveStageElement::RegisterFileRS2Value => {
                format!("RS2 Value: 0x{:08X}", pipeline.id_lines.rs2_v)
            }
            FiveStageElement::HazardUnit => "Hazard Unit".to_string(),
            FiveStageElement::IFPCWriteEnable => {
                format_bool!(
                    "IF PC Write Enable: {}",
                    !pipeline.hazard_detector.hazard_detected.stop_if
                )
            }
            FiveStageElement::IFIDWriteEnable => {
                format_bool!(
                    "IFID Write Enable: {}",
                    !pipeline.hazard_detector.hazard_detected.stop_id
                )
            }
            FiveStageElement::IDEXWriteEnable => {
                format_bool!(
                    "IDEX Write Enable: {}",
                    !pipeline.hazard_detector.hazard_detected.stop_ex
                )
            }
            // ID/EX Buffer
            FiveStageElement::IDEXBuffer => "ID/EX Pipeline Buffer".to_string(),
            FiveStageElement::IDEXPC => format_opt!("EX PC: 0x{:08X}", pipeline.id_ex.ex_pc),
            FiveStageElement::IDEXRD => format_opt!("EX RD: 0x{:08X}", pipeline.id_ex.rd),
            FiveStageElement::IDEXRS1 => format!("EX RS1 Value: 0x{:08X}", pipeline.id_ex.rs1_v),
            FiveStageElement::IDEXRS2 => format!("EX RS2 Value: 0x{:08X}", pipeline.id_ex.rs2_v),
            FiveStageElement::IDEXImm => format_opt!("EX IMM: 0x{:08X}", pipeline.id_ex.imm),
            FiveStageElement::ALUMuxA => format_opt!("ALU OP A: 0x{:08X}", pipeline.ex_lines.op_a),
            FiveStageElement::ALUMuxB => format_opt!("ALU OP B: 0x{:08X}", pipeline.ex_lines.op_b),
            FiveStageElement::ALU => format_opt!("ALU Output: 0x{:08X}", pipeline.ex_lines.alu_out),
            // Branch Calculation
            FiveStageElement::JMPBaseAddress => {
                format_opt!("JMP Base Address: 0x{:08X}", pipeline.ex_lines.jmp_base)
            }
            FiveStageElement::JMPAddress => {
                format_opt!("JMP Address: 0x{:08X}", pipeline.ex_lines.jmp_dst)
            }
            // EX/MEM Buffer
            FiveStageElement::EXMEMBuffer => "EX/MEM Pipeline Buffer".to_string(),
            FiveStageElement::EXMEMPC => format_opt!("MEM PC: 0x{:08X}", pipeline.ex_mem.mem_pc),
            FiveStageElement::EXMEMAlu => {
                format_opt!("MEM ALU Output: 0x{:08X}", pipeline.ex_mem.alu_o)
            }
            FiveStageElement::EXMEMRD => format_opt!("MEM RD: 0x{:08X}", pipeline.ex_mem.rd),
            FiveStageElement::EXMEMRS2 => format!("MEM RS2: 0x{:08X}", pipeline.ex_mem.rs2_v),
            // LSU
            FiveStageElement::DataMemory => "Data Memory".to_string(),
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

fn find_active_elements(pipeline: &FiveStagePipeline) -> BTreeSet<FiveStageElement> {
    use FiveStageElement::*;

    let mut active_elements = BTreeSet::new();

    macro_rules! add_element {
        ($signal:expr, $element:ident) => {
            if $signal {
                active_elements.insert($element);
            }
        };

        ($signal:expr, $element:ident, $($rest:ident),+) => {
            {
                add_element!($signal, $element);
                add_element!($signal, $($rest),+);
            }
        };
    }

    // If Control Signals
    if !pipeline.hazard_detector.hazard_detected.stop_if {
        add_element!(true, IFPC, PCMux);
        add_element!(pipeline.if_lines.next_pc_sel == PCSel::PC4, PCPlus4);
    }

    add_element!(
        !pipeline.hazard_detector.hazard_detected.stop_id,
        IFPC,
        InstructionMemory
    );

    // IF/ID Buffer
    add_element!(
        !pipeline.hazard_detector.hazard_detected.stop_id,
        IFIDBuffer
    );

    if !pipeline.hazard_detector.hazard_detected.stop_ex {
        add_element!(pipeline.if_id.id_pc.is_some(), IFIDPC);
        add_element!(pipeline.if_id.id_inst.is_some(), IFIDInstruction);

        // ID Control Signals
        add_element!(
            pipeline.if_id.id_inst.is_some(),
            Decoder,
            DecoderRD,
            DecoderRS1,
            DecoderRS2,
            DecoderImm,
            RegisterFile,
            RegisterFileRS1Value,
            RegisterFileRS2Value,
            ControlUnit
        );
    }

    // ID/EX Buffer
    add_element!(
        !pipeline.hazard_detector.hazard_detected.stop_ex && pipeline.if_id.id_pc.is_some(),
        IDEXBuffer
    );

    add_element!(
        pipeline.ex_control.lsu_request && pipeline.ex_control.lsu_write_enable,
        IDEXRS2
    );
    add_element!(pipeline.ex_control.reg_write, IDEXRD);

    // Ex Control Signals
    match pipeline.ex_control.alu_op_a_sel {
        Some(OpASel::PC) => add_element!(true, ALUMuxA, IDEXPC),
        Some(OpASel::RF) => add_element!(true, ALUMuxA, IDEXRS1),
        _ => {}
    }

    match pipeline.ex_control.alu_op_b_sel {
        Some(OpBSel::IMM) => add_element!(true, ALUMuxB, IDEXImm),
        Some(OpBSel::RF) => add_element!(true, ALUMuxB, IDEXRS2),
        Some(OpBSel::Four) => add_element!(true, ALUMuxB),
        _ => {}
    }
    add_element!(pipeline.ex_control.alu_op.is_some(), ALU);

    match pipeline.ex_control.jmp_base {
        Some(OpASel::PC) => add_element!(true, JMPBaseAddress, JMPAddress, IDEXPC),
        Some(OpASel::RF) => add_element!(true, JMPBaseAddress, IDEXRS1),
        _ => {}
    }

    add_element!(
        pipeline.ex_control.jump_cond,
        JumpCondControlSignal,
        ConditionResult,
        PCNextSelControlSignal
    );
    add_element!(
        pipeline.ex_control.jump_uncond,
        JumpUncondControlSignal,
        PCNextSelControlSignal
    );

    add_element!(pipeline.id_ex.ex_pc.is_some(), EXMEMBuffer);
    add_element!(
        pipeline.mem_control.lsu_request || pipeline.mem_control.reg_write,
        EXMEMAlu
    );
    add_element!(
        pipeline.mem_control.lsu_request && pipeline.ex_control.lsu_write_enable,
        EXMEMRS2
    );
    add_element!(pipeline.mem_control.reg_write, EXMEMRD);

    // Mem Control Signals
    add_element!(
        pipeline.mem_control.lsu_request,
        LSURequestControlSignal,
        LSU,
        LSUADDR,
        LSUDATA,
        LSUREQ,
        LSUWR,
        LSUBYTE_EN,
        LSUVALID,
        DataMemory
    );
    add_element!(
        pipeline.mem_lines.mem_data.is_some() && !pipeline.mem_control.lsu_write_enable,
        LSURDOut
    );

    add_element!(pipeline.ex_mem.mem_pc.is_some(), MEMWBBuffer);
    add_element!(pipeline.wb_control.reg_write, MEMWBRD, RegisterFile);
    // Wb Control Signals
    match pipeline.wb_control.wb_src {
        Some(DataDestSel::ALU) => add_element!(true, WritebackResult, MEMWBAlu),
        Some(DataDestSel::LSU) => add_element!(true, WritebackResult, MEMWBLsu),
        _ => {}
    }

    active_elements
}

#[component]
#[allow(non_snake_case)]
pub fn FiveStageVisualization(
    emulator_state: ReadOnlySignal<Option<AnyEmulatorState>>,
    tooltip_text: Signal<Option<String>>,
    show_control_signals: Signal<bool>,
) -> Element {
    const HOVER_STROKE: &str = "rgba(66, 133, 244, 1)";
    const ACTIVE_STROKE: &str = "rgba(66, 133, 244, 0.7)";
    const HOVER_FILL: &str = "rgba(66, 133, 244, 0.1)";

    let mut hovered_element = use_signal(|| Option::<FiveStageElement>::None);
    let mut active_elements = use_signal(BTreeSet::<FiveStageElement>::new);

    use_effect(move || {
        if let Some(AnyEmulatorState::FiveStage(state)) = &*emulator_state.read() {
            active_elements.set(find_active_elements(&state.pipeline));
        }
    });

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
            } else if active_elements.read().contains(&FiveStageElement::$element) {
                ACTIVE_STROKE
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
        if *show_control_signals.read() {
            g {
                id: "ifpc_write_enable_group",
                style: "pointer-events: all;",
                onmouseenter: move |_| {
                    hovered_element.set(Some(FiveStageElement::IFPCWriteEnable));
                },
                onmouseleave: move |_| {
                    hovered_element.set(None);
                },
                line {
                    id: "ifpc_write_enable_line1",
                    x1: "365",
                    y1: "59",
                    x2: "542",
                    y2: "59",
                    stroke: match &*emulator_state.read() {
                        Some(AnyEmulatorState::FiveStage(state)) => {
                            let is_hovered = *hovered_element.read()
                                == Some(FiveStageElement::IFPCWriteEnable);
                            match state.pipeline.hazard_detector.hazard_detected.stop_if {
                                true => if is_hovered { "red" } else { "rgba(200, 0, 0, 0.4)" }
                                false => if is_hovered { "green" } else { "rgba(0, 200, 0, 0.4)" }
                            }
                        }
                        _ => "gray",
                    },
                    "stroke-width": "2",
                }
                line {
                    id: "ifpc_write_enable_line2",
                    x1: "364",
                    y1: "58",
                    x2: "364",
                    y2: "337",
                    stroke: match &*emulator_state.read() {
                        Some(AnyEmulatorState::FiveStage(state)) => {
                            let is_hovered = *hovered_element.read()
                                == Some(FiveStageElement::IFPCWriteEnable);
                            match state.pipeline.hazard_detector.hazard_detected.stop_if {
                                true => if is_hovered { "red" } else { "rgba(200, 0, 0, 0.4)" }
                                false => if is_hovered { "green" } else { "rgba(0, 200, 0, 0.4)" }
                            }
                        }
                        _ => "gray",
                    },
                    "stroke-width": "2",
                }
                line {
                    id: "ifpc_write_enable_line3",
                    x1: "364",
                    y1: "336",
                    x2: "58",
                    y2: "336",
                    stroke: match &*emulator_state.read() {
                        Some(AnyEmulatorState::FiveStage(state)) => {
                            let is_hovered = *hovered_element.read()
                                == Some(FiveStageElement::IFPCWriteEnable);
                            match state.pipeline.hazard_detector.hazard_detected.stop_if {
                                true => if is_hovered { "red" } else { "rgba(200, 0, 0, 0.4)" }
                                false => if is_hovered { "green" } else { "rgba(0, 200, 0, 0.4)" }
                            }
                        }
                        _ => "gray",
                    },
                    "stroke-width": "2",
                }
                path {
                    id: "ifpc_write_enable_arrow",
                    transform: "translate(-80, 222)",
                    d: "M137.707 94.2929C137.317 93.9024 136.683 93.9024 136.293 94.2929L129.929 100.657C129.538 101.047 129.538 101.681 129.929 102.071C130.319 102.462 130.953 102.462 131.343 102.071L137 96.4142L142.657 102.071C143.047 102.462 143.681 102.462 144.071 102.071C144.462 101.681 144.462 101.047 144.071 100.657L137.707 94.2929ZM138 115V95H136V115H138Z",
                    fill: match &*emulator_state.read() {
                        Some(AnyEmulatorState::FiveStage(state)) => {
                            let is_hovered = *hovered_element.read()
                                == Some(FiveStageElement::IFPCWriteEnable);
                            match state.pipeline.hazard_detector.hazard_detected.stop_if {
                                true => if is_hovered { "red" } else { "rgba(200, 0, 0, 0.4)" }
                                false => if is_hovered { "green" } else { "rgba(0, 200, 0, 0.4)" }
                            }
                        }
                        _ => "gray",
                    },
                }
            }
            g {
                id: "jmp_uncond_control_group",
                style: "pointer-events: all;",
                onmouseenter: move |_| {
                    hovered_element.set(Some(FiveStageElement::JumpUncondControlSignal));
                },
                onmouseleave: move |_| {
                    hovered_element.set(None);
                },
                path {
                    id: "jmp_uncond_arrow",
                    transform: "translate(-134, -35)",
                    d: "M1042.29 38.2929C1041.9 38.6834 1041.9 39.3166 1042.29 39.7071L1048.66 46.0711C1049.05 46.4616 1049.68 46.4616 1050.07 46.0711C1050.46 45.6805 1050.46 45.0474 1050.07 44.6569L1044.41 39L1050.07 33.3431C1050.46 32.9526 1050.46 32.3195 1050.07 31.9289C1049.68 31.5384 1049.05 31.5384 1048.66 31.9289L1042.29 38.2929ZM1088 38L1043 38V40L1088 40V38Z",
                    fill: match &*emulator_state.read() {
                        Some(AnyEmulatorState::FiveStage(state)) => {
                            let is_hovered = *hovered_element.read()
                                == Some(FiveStageElement::JumpUncondControlSignal);
                            match state.pipeline.ex_control.jump_uncond {
                                true => if is_hovered { "green" } else { "rgba(0, 200, 0, 0.4)" }
                                false => if is_hovered { "red" } else { "rgba(200, 0, 0, 0.4)" }
                            }
                        }
                        _ => "gray",
                    },
                }
                line {
                    id: "jmp_uncond_vertical_line",
                    x1: "955",
                    y1: "3",
                    x2: "955",
                    y2: "561",
                    stroke: match &*emulator_state.read() {
                        Some(AnyEmulatorState::FiveStage(state)) => {
                            let is_hovered = *hovered_element.read()
                                == Some(FiveStageElement::JumpUncondControlSignal);
                            match state.pipeline.ex_control.jump_uncond {
                                true => if is_hovered { "green" } else { "rgba(0, 200, 0, 0.4)" }
                                false => if is_hovered { "red" } else { "rgba(200, 0, 0, 0.4)" }
                            }
                        }
                        _ => "gray",
                    },
                    "stroke-width": "2",
                }
            }
            g {
                id: "jmp_cond_control_group",
                style: "pointer-events: all;",
                onmouseenter: move |_| {
                    hovered_element.set(Some(FiveStageElement::JumpCondControlSignal));
                },
                onmouseleave: move |_| {
                    hovered_element.set(None);
                },
                path {
                    id: "jmp_cond_arrow",
                    transform: "translate(0, -65)",
                    d: "M1042.29 38.2929C1041.9 38.6834 1041.9 39.3166 1042.29 39.7071L1048.66 46.0711C1049.05 46.4616 1049.68 46.4616 1050.07 46.0711C1050.46 45.6805 1050.46 45.0474 1050.07 44.6569L1044.41 39L1050.07 33.3431C1050.46 32.9526 1050.46 32.3195 1050.07 31.9289C1049.68 31.5384 1049.05 31.5384 1048.66 31.9289L1042.29 38.2929ZM1098 38L1043 38V40L1098 40V38Z",
                    fill: match &*emulator_state.read() {
                        Some(AnyEmulatorState::FiveStage(state)) => {
                            let is_hovered = *hovered_element.read()
                                == Some(FiveStageElement::JumpCondControlSignal);
                            match state.pipeline.ex_control.jump_cond {
                                true => if is_hovered { "green" } else { "rgba(0, 200, 0, 0.4)" }
                                false => if is_hovered { "red" } else { "rgba(200, 0, 0, 0.4)" }
                            }
                        }
                        _ => "gray",
                    },
                }
                line {
                    id: "jmp_cond_vertical_line",
                    x1: "1099",
                    y1: "-27",
                    x2: "1099",
                    y2: "561",
                    stroke: match &*emulator_state.read() {
                        Some(AnyEmulatorState::FiveStage(state)) => {
                            let is_hovered = *hovered_element.read()
                                == Some(FiveStageElement::JumpCondControlSignal);
                            match state.pipeline.ex_control.jump_cond {
                                true => if is_hovered { "green" } else { "rgba(0, 200, 0, 0.4)" }
                                false => if is_hovered { "red" } else { "rgba(200, 0, 0, 0.4)" }
                            }
                        }
                        _ => "gray",
                    },
                    "stroke-width": "2",
                }
            }
            g {
                id: "ifid_write_enable_group",
                style: "pointer-events: all;",
                onmouseenter: move |_| {
                    hovered_element.set(Some(FiveStageElement::IFIDWriteEnable));
                },
                onmouseleave: move |_| {
                    hovered_element.set(None);
                },
                path {
                    id: "ifid_write_enable_arrow",
                    transform: "translate(1, -100)",
                    d: "M482.293 188.707C481.902 188.317 481.902 187.683 482.293 187.293L488.657 180.929C489.047 180.538 489.681 180.538 490.071 180.929C490.462 181.319 490.462 181.953 490.071 182.343L484.414 188L490.071 193.657C490.462 194.047 490.462 194.681 490.071 195.071C489.681 195.462 489.047 195.462 488.657 195.071L482.293 188.707ZM540 189H483V187H540V189Z",
                    fill: match &*emulator_state.read() {
                        Some(AnyEmulatorState::FiveStage(state)) => {
                            let is_hovered = *hovered_element.read()
                                == Some(FiveStageElement::IFIDWriteEnable);
                            match state.pipeline.hazard_detector.hazard_detected.stop_id {
                                true => if is_hovered { "red" } else { "rgba(200, 0, 0, 0.4)" }
                                false => if is_hovered { "green" } else { "rgba(0, 200, 0, 0.4)" }
                            }
                        }
                        _ => "gray",
                    },
                }
            }
            g {
                id: "idex_write_enable_group",
                style: "pointer-events: all;",
                onmouseenter: move |_| {
                    hovered_element.set(Some(FiveStageElement::IDEXWriteEnable));
                },
                onmouseleave: move |_| {
                    hovered_element.set(None);
                },
                path {
                    id: "idex_write_enable_arrow",
                    transform: "translate(218, -100)",
                    d: "M540.707 188.707C541.098 188.317 541.098 187.683 540.707 187.293L534.343 180.929C533.953 180.538 533.319 180.538 532.929 180.929C532.538 181.319 532.538 181.953 532.929 182.343L538.586 188L532.929 193.657C532.538 194.047 532.538 194.681 532.929 195.071C533.319 195.462 533.953 195.462 534.343 195.071L540.707 188.707ZM483 189H540V187H483V189Z",
                    fill: match &*emulator_state.read() {
                        Some(AnyEmulatorState::FiveStage(state)) => {
                            let is_hovered = *hovered_element.read()
                                == Some(FiveStageElement::IDEXWriteEnable);
                            match state.pipeline.hazard_detector.hazard_detected.stop_ex {
                                true => if is_hovered { "red" } else { "rgba(200, 0, 0, 0.4)" }
                                false => if is_hovered { "green" } else { "rgba(0, 200, 0, 0.4)" }
                            }
                        }
                        _ => "gray",
                    },
                }
            }
            g {
                id: "wb_ctrl_to_wb_src_mux_group",
                style: "pointer-events: all;",
                onmouseenter: move |_| {
                    hovered_element.set(Some(FiveStageElement::WBSrcControlSignal));
                },
                onmouseleave: move |_| {
                    hovered_element.set(None);
                },
                path {
                    id: "wb_ctrl_to_wb_src_mux_arrow",
                    transform: "translate(1600, 426)",
                    d: "M8.70573 0.804236C8.31387 0.415059 7.68071 0.417238 7.29153 0.809106L0.949515 7.19494C0.560338 7.58681 0.562518 8.21997 0.954384 8.60915C1.34625 8.99832 1.97941 8.99614 2.36859 8.60428L8.00593 2.92798L13.6822 8.56532C14.0741 8.9545 14.7073 8.95232 15.0964 8.56046C15.4856 8.16859 15.4834 7.53543 15.0916 7.14625L8.70573 0.804236ZM9.12499 123.5L9.00106 1.51033L7.00107 1.51722L7.12501 123.5L9.12499 123.5Z",
                    fill: match &*emulator_state.read() {
                        Some(AnyEmulatorState::FiveStage(state)) => {
                            let is_hovered = *hovered_element.read()
                                == Some(FiveStageElement::WBSrcControlSignal);
                            match state.pipeline.wb_control.wb_src {
                                Some(DataDestSel::ALU) => {
                                    if is_hovered { "green" } else { "rgba(0, 200, 0, 0.4)" }
                                }
                                Some(DataDestSel::LSU) => {
                                    if is_hovered { "red" } else { "rgba(200, 0, 0, 0.4)" }
                                }
                                None => "gray",
                            }
                        }
                        _ => "gray",
                    },
                }
                line {
                    id: "wb_ctrl_to_wb_src_mux_line",
                    x1: "203",
                    y1: "0",
                    x2: "260",
                    y2: "0",
                    "stroke-width": "2",
                    transform: "translate(1349, 550)",
                    stroke: match &*emulator_state.read() {
                        Some(AnyEmulatorState::FiveStage(state)) => {
                            let is_hovered = *hovered_element.read()
                                == Some(FiveStageElement::WBSrcControlSignal);
                            match state.pipeline.wb_control.wb_src {
                                Some(DataDestSel::ALU) => {
                                    if is_hovered { "green" } else { "rgba(0, 200, 0, 0.4)" }
                                }
                                Some(DataDestSel::LSU) => {
                                    if is_hovered { "red" } else { "rgba(200, 0, 0, 0.4)" }
                                }
                                None => "gray",
                            }
                        }
                        _ => "gray",
                    },
                }
            }
            g {
                id: "ex_ctrl_to_aluopmux_group",
                style: "pointer-events: all;",
                onmouseenter: move |_| {
                    hovered_element.set(Some(FiveStageElement::ALUOpControlSignal));
                },
                onmouseleave: move |_| {
                    hovered_element.set(None);
                },
                path {
                    id: "ex_ctrl_to_aluopmux_arrow",
                    transform: "translate(1015, 458)",
                    d: "M8.70573 0.804236C8.31387 0.415059 7.68071 0.417238 7.29153 0.809106L0.949515 7.19494C0.560338 7.58681 0.562518 8.21997 0.954384 8.60915C1.34625 8.99832 1.97941 8.99614 2.36859 8.60428L8.00593 2.92798L13.6822 8.56532C14.0741 8.9545 14.7073 8.95232 15.0964 8.56046C15.4856 8.16859 15.4834 7.53543 15.0916 7.14625L8.70573 0.804236ZM9.12499 102.5L9.00106 1.51033L7.00107 1.51722L7.12501 102.5L9.12499 102.5Z",
                    fill: match &*emulator_state.read() {
                        Some(AnyEmulatorState::FiveStage(state)) => {
                            match (state.pipeline.ex_control.alu_op, *hovered_element.read()) {
                                (Some(_), Some(FiveStageElement::ALUOpControlSignal)) => "green",
                                (Some(_), _) => "rgba(0, 200, 0, 0.4)",
                                (None, Some(FiveStageElement::ALUOpControlSignal)) => "gray",
                                (None, _) => "gray",
                            }
                        }
                        _ => "gray",
                    },
                }
            }
            g {
                id: "ex_ctrl_to_opamux_group",
                style: "pointer-events: all;",
                onmouseenter: move |_| {
                    hovered_element.set(Some(FiveStageElement::ALUOpASelControlSignal));
                },
                onmouseleave: move |_| {
                    hovered_element.set(None);
                },
                path {
                    id: "ex_ctrl_to_opamux_arrow",
                    transform: "translate(909, 338)",
                    d: "M8.70573 0.804236C8.31387 0.415059 7.68071 0.417238 7.29153 0.809106L0.949515 7.19494C0.560338 7.58681 0.562518 8.21997 0.954384 8.60915C1.34625 8.99832 1.97941 8.99614 2.36859 8.60428L8.00593 2.92798L13.6822 8.56532C14.0741 8.9545 14.7073 8.95232 15.0964 8.56046C15.4856 8.16859 15.4834 7.53543 15.0916 7.14625L8.70573 0.804236ZM9.12499 21.5L9.00106 1.51033L7.00107 1.51722L7.12501 21.5L9.12499 21.5Z",
                    fill: match &*emulator_state.read() {
                        Some(AnyEmulatorState::FiveStage(state)) => {
                            match (state.pipeline.ex_control.alu_op_a_sel, *hovered_element.read()) {
                                (Some(OpASel::PC), Some(FiveStageElement::ALUOpASelControlSignal)) => {
                                    "green"
                                }
                                (Some(OpASel::PC), _) => "rgba(0, 200, 0, 0.4)",
                                (Some(OpASel::RF), Some(FiveStageElement::ALUOpASelControlSignal)) => {
                                    "red"
                                }
                                (Some(OpASel::RF), _) => "rgba(200, 0, 0, 0.4)",
                                (None, _) => "gray",
                            }
                        }
                        _ => "gray",
                    },
                }
                line {
                    id: "ex_ctrl_to_opamux_horizontal_line",
                    x1: "918.13",
                    y1: "360.49",
                    x2: "874",
                    y2: "360.49",
                    stroke: match &*emulator_state.read() {
                        Some(AnyEmulatorState::FiveStage(state)) => {
                            match (state.pipeline.ex_control.alu_op_a_sel, *hovered_element.read()) {
                                (Some(OpASel::PC), Some(FiveStageElement::ALUOpASelControlSignal)) => {
                                    "green"
                                }
                                (Some(OpASel::PC), _) => "rgba(0, 200, 0, 0.4)",
                                (Some(OpASel::RF), Some(FiveStageElement::ALUOpASelControlSignal)) => {
                                    "red"
                                }
                                (Some(OpASel::RF), _) => "rgba(200, 0, 0, 0.4)",
                                (None, _) => "gray",
                            }
                        }
                        _ => "gray",
                    },
                    "stroke-width": "2",
                }
                line {
                    id: "ex_ctrl_to_opamux_horizontal_line",
                    x1: "873",
                    y1: "359.49",
                    x2: "873",
                    y2: "560",
                    stroke: match &*emulator_state.read() {
                        Some(AnyEmulatorState::FiveStage(state)) => {
                            match (state.pipeline.ex_control.alu_op_a_sel, *hovered_element.read()) {
                                (Some(OpASel::PC), Some(FiveStageElement::ALUOpASelControlSignal)) => {
                                    "green"
                                }
                                (Some(OpASel::PC), _) => "rgba(0, 200, 0, 0.4)",
                                (Some(OpASel::RF), Some(FiveStageElement::ALUOpASelControlSignal)) => {
                                    "red"
                                }
                                (Some(OpASel::RF), _) => "rgba(200, 0, 0, 0.4)",
                                (None, _) => "gray",
                            }
                        }
                        _ => "gray",
                    },
                    "stroke-width": "2",
                }
            }
            g {
                id: "ex_ctrl_to_lsu_data_type_mux_group",
                style: "pointer-events: all;",
                onmouseenter: move |_| {
                    hovered_element.set(Some(FiveStageElement::LSUDataTypeControlSignal));
                },
                onmouseleave: move |_| {
                    hovered_element.set(None);
                },
                path {
                    id: "mem_ctrl_to_lsu_data_type_mux_arrow",
                    transform: "translate(1309, 314.5)",
                    d: "M8.70573 0.804236C8.31387 0.415059 7.68071 0.417238 7.29153 0.809106L0.949515 7.19494C0.560338 7.58681 0.562518 8.21997 0.954384 8.60915C1.34625 8.99832 1.97941 8.99614 2.36859 8.60428L8.00593 2.92798L13.6822 8.56532C14.0741 8.9545 14.7073 8.95232 15.0964 8.56046C15.4856 8.16859 15.4834 7.53543 15.0916 7.14625L8.70573 0.804236ZM9.12499 246.5L9.00106 1.51033L7.00107 1.51722L7.12501 246.5L9.12499 246.5Z",
                    fill: match &*emulator_state.read() {
                        Some(AnyEmulatorState::FiveStage(state)) => {
                            let is_hovered = *hovered_element.read()
                                == Some(FiveStageElement::LSUDataTypeControlSignal);
                            match state.pipeline.mem_control.lsu_data_type {
                                Some(LSUDataType::Word) => {
                                    if is_hovered { "green" } else { "rgba(0, 200, 0, 0.4)" }
                                }
                                Some(LSUDataType::HalfWord) => {
                                    if is_hovered { "blue" } else { "rgba(0, 0, 200, 0.4)" }
                                }
                                Some(LSUDataType::Byte) => {
                                    if is_hovered { "red" } else { "rgba(200, 0, 0, 0.4)" }
                                }
                                None => "gray",
                            }
                        }
                        _ => "gray",
                    },
                }
            }
            g {
                id: "mem_ctrl_to_lsu_sign_ext_mux_group",
                style: "pointer-events: all;",
                onmouseenter: move |_| {
                    hovered_element.set(Some(FiveStageElement::LSUSignExtControlSignal));
                },
                onmouseleave: move |_| {
                    hovered_element.set(None);
                },
                path {
                    id: "mem_ctrl_to_lsu_sign_ext_mux_arrow",
                    transform: "translate(1349, 314.5)",
                    d: "M8.70573 0.804236C8.31387 0.415059 7.68071 0.417238 7.29153 0.809106L0.949515 7.19494C0.560338 7.58681 0.562518 8.21997 0.954384 8.60915C1.34625 8.99832 1.97941 8.99614 2.36859 8.60428L8.00593 2.92798L13.6822 8.56532C14.0741 8.9545 14.7073 8.95232 15.0964 8.56046C15.4856 8.16859 15.4834 7.53543 15.0916 7.14625L8.70573 0.804236ZM9.12499 246.5L9.00106 1.51033L7.00107 1.51722L7.12501 246.5L9.12499 246.5Z",
                    fill: match &*emulator_state.read() {
                        Some(AnyEmulatorState::FiveStage(state)) => {
                            let is_hovered = *hovered_element.read()
                                == Some(FiveStageElement::LSUSignExtControlSignal);
                            match state.pipeline.mem_control.lsu_sign_ext {
                                true => if is_hovered { "green" } else { "rgba(0, 200, 0, 0.4)" }
                                false => if is_hovered { "red" } else { "rgba(200, 0, 0, 0.4)" }
                            }
                        }
                        _ => "gray",
                    },
                }
            }
            g {
                id: "mem_ctrl_to_lsu_write_enable_mux_group",
                style: "pointer-events: all;",
                onmouseenter: move |_| {
                    hovered_element.set(Some(FiveStageElement::LSUWriteEnableControlSignal));
                },
                onmouseleave: move |_| {
                    hovered_element.set(None);
                },
                path {
                    id: "mem_ctrl_to_lsu_write_enable_mux_arrow",
                    transform: "translate(1369, 314.5)",
                    d: "M8.70573 0.804236C8.31387 0.415059 7.68071 0.417238 7.29153 0.809106L0.949515 7.19494C0.560338 7.58681 0.562518 8.21997 0.954384 8.60915C1.34625 8.99832 1.97941 8.99614 2.36859 8.60428L8.00593 2.92798L13.6822 8.56532C14.0741 8.9545 14.7073 8.95232 15.0964 8.56046C15.4856 8.16859 15.4834 7.53543 15.0916 7.14625L8.70573 0.804236ZM9.12499 246.5L9.00106 1.51033L7.00107 1.51722L7.12501 246.5L9.12499 246.5Z",
                    fill: match &*emulator_state.read() {
                        Some(AnyEmulatorState::FiveStage(state)) => {
                            let is_hovered = *hovered_element.read()
                                == Some(FiveStageElement::LSUWriteEnableControlSignal);
                            match state.pipeline.mem_control.lsu_write_enable {
                                true => if is_hovered { "green" } else { "rgba(0, 200, 0, 0.4)" }
                                false => if is_hovered { "red" } else { "rgba(200, 0, 0, 0.4)" }
                            }
                        }
                        _ => "gray",
                    },
                }
            }
            g {
                id: "mem_ctrl_to_lsu_request_mux_group",
                style: "pointer-events: all;",
                onmouseenter: move |_| {
                    hovered_element.set(Some(FiveStageElement::LSURequestControlSignal));
                },
                onmouseleave: move |_| {
                    hovered_element.set(None);
                },
                path {
                    id: "mem_ctrl_to_lsu_request_mux_arrow",
                    transform: "translate(1389, 314.5)",
                    d: "M8.70573 0.804236C8.31387 0.415059 7.68071 0.417238 7.29153 0.809106L0.949515 7.19494C0.560338 7.58681 0.562518 8.21997 0.954384 8.60915C1.34625 8.99832 1.97941 8.99614 2.36859 8.60428L8.00593 2.92798L13.6822 8.56532C14.0741 8.9545 14.7073 8.95232 15.0964 8.56046C15.4856 8.16859 15.4834 7.53543 15.0916 7.14625L8.70573 0.804236ZM9.12499 246.5L9.00106 1.51033L7.00107 1.51722L7.12501 246.5L9.12499 246.5Z",
                    fill: match &*emulator_state.read() {
                        Some(AnyEmulatorState::FiveStage(state)) => {
                            let is_hovered = *hovered_element.read()
                                == Some(FiveStageElement::LSURequestControlSignal);
                            match state.pipeline.mem_control.lsu_request {
                                true => if is_hovered { "green" } else { "rgba(0, 200, 0, 0.4)" }
                                false => if is_hovered { "red" } else { "rgba(200, 0, 0, 0.4)" }
                            }
                        }
                        _ => "gray",
                    },
                }
            }
            g {
                id: "mem_ctrl_to_reg_write_mux_group",
                style: "pointer-events: all;",
                onmouseenter: move |_| {
                    hovered_element.set(Some(FiveStageElement::RegWriteControlSignal));
                },
                onmouseleave: move |_| {
                    hovered_element.set(None);
                },
                line {
                    id: "mem_ctrl_to_reg_write_mux_line",
                    x1: "203",
                    y1: "0",
                    x2: "260",
                    y2: "0",
                    "stroke-width": "2",
                    transform: "translate(1349, 570)",
                    stroke: match &*emulator_state.read() {
                        Some(AnyEmulatorState::FiveStage(state)) => {
                            let is_hovered = *hovered_element.read()
                                == Some(FiveStageElement::RegWriteControlSignal);
                            match state.pipeline.mem_control.reg_write {
                                true => if is_hovered { "green" } else { "rgba(0, 200, 0, 0.4)" }
                                false => if is_hovered { "red" } else { "rgba(200, 0, 0, 0.4)" }
                            }
                        }
                        _ => "gray",
                    },
                }
                line {
                    id: "mem_ctrl_to_reg_write_mux_line",
                    x1: "260",
                    y1: "0",
                    x2: "260",
                    y2: "85",
                    "stroke-width": "2",
                    transform: "translate(1349, 570)",
                    stroke: match &*emulator_state.read() {
                        Some(AnyEmulatorState::FiveStage(state)) => {
                            let is_hovered = *hovered_element.read()
                                == Some(FiveStageElement::RegWriteControlSignal);
                            match state.pipeline.mem_control.reg_write {
                                true => if is_hovered { "green" } else { "rgba(0, 200, 0, 0.4)" }
                                false => if is_hovered { "red" } else { "rgba(200, 0, 0, 0.4)" }
                            }
                        }
                        _ => "gray",
                    },
                }
                line {
                    id: "mem_ctrl_to_reg_write_mux_line",
                    x1: "554.7",
                    y1: "656",
                    x2: "1610",
                    y2: "656",
                    "stroke-width": "2",
                    stroke: match &*emulator_state.read() {
                        Some(AnyEmulatorState::FiveStage(state)) => {
                            let is_hovered = *hovered_element.read()
                                == Some(FiveStageElement::RegWriteControlSignal);
                            match state.pipeline.mem_control.reg_write {
                                true => if is_hovered { "green" } else { "rgba(0, 200, 0, 0.4)" }
                                false => if is_hovered { "red" } else { "rgba(200, 0, 0, 0.4)" }
                            }
                        }
                        _ => "gray",
                    },
                }
                text {
                    id: "WE",
                    x: "545",
                    y: "473",
                    "text-anchor": "start",
                    "dominant-baseline": "middle",
                    "font-size": "12",
                    "font-weight": "bold",
                    fill: match &*emulator_state.read() {
                        Some(AnyEmulatorState::FiveStage(state)) => {
                            match (state.pipeline.mem_control.reg_write, *hovered_element.read()) {
                                (true, Some(FiveStageElement::RegWriteControlSignal)) => "green",
                                (true, _) => "rgba(0, 200, 0, 0.4)",
                                (false, Some(FiveStageElement::RegWriteControlSignal)) => "red",
                                (false, _) => "rgba(200, 0, 0, 0.4)",
                            }
                        }
                        _ => "gray",
                    },
                    "WE"
                }
                line {
                    id: "mem_ctrl_to_reg_write_mux_line",
                    x1: "554",
                    y1: "657",
                    x2: "554",
                    y2: "484",
                    "stroke-width": "2",
                    stroke: match &*emulator_state.read() {
                        Some(AnyEmulatorState::FiveStage(state)) => {
                            let is_hovered = *hovered_element.read()
                                == Some(FiveStageElement::RegWriteControlSignal);
                            match state.pipeline.mem_control.reg_write {
                                true => if is_hovered { "green" } else { "rgba(0, 200, 0, 0.4)" }
                                false => if is_hovered { "red" } else { "rgba(200, 0, 0, 0.4)" }
                            }
                        }
                        _ => "gray",
                    },
                }
                path {
                    id: "mem_ctrl_to_reg_write_mux_arrow",
                    transform: "translate(546, 481)",
                    d: "M8.70573 0.804236C8.31387 0.415059 7.68071 0.417238 7.29153 0.809106L0.949515 7.19494C0.560338 7.58681 0.562518 8.21997 0.954384 8.60915C1.34625 8.99832 1.97941 8.99614 2.36859 8.60428L8.00593 2.92798L13.6822 8.56532C14.0741 8.9545 14.7073 8.95232 15.0964 8.56046C15.4856 8.16859 15.4834 7.53543 15.0916 7.14625L8.70573 0.804236Z",
                    fill: match &*emulator_state.read() {
                        Some(AnyEmulatorState::FiveStage(state)) => {
                            let is_hovered = *hovered_element.read()
                                == Some(FiveStageElement::RegWriteControlSignal);
                            match state.pipeline.mem_control.reg_write {
                                true => if is_hovered { "green" } else { "rgba(0, 200, 0, 0.4)" }
                                false => if is_hovered { "red" } else { "rgba(200, 0, 0, 0.4)" }
                            }
                        }
                        _ => "gray",
                    },
                }
            }
            g {
                id: "jmp_base_ctrl_group",
                style: "pointer-events: all;",
                onmouseenter: move |_| {
                    hovered_element.set(Some(FiveStageElement::JMPBaseControlSignal));
                },
                onmouseleave: move |_| {
                    hovered_element.set(None);
                },
                path {
                    id: "jmp_base_ctrl_arrow",
                    transform: "translate(909, 218)",
                    d: "M8.70573 0.804236C8.31387 0.415059 7.68071 0.417238 7.29153 0.809106L0.949515 7.19494C0.560338 7.58681 0.562518 8.21997 0.954384 8.60915C1.34625 8.99832 1.97941 8.99614 2.36859 8.60428L8.00593 2.92798L13.6822 8.56532C14.0741 8.9545 14.7073 8.95232 15.0964 8.56046C15.4856 8.16859 15.4834 7.53543 15.0916 7.14625L8.70573 0.804236ZM9.12499 24.5L9.00106 1.51033L7.00107 1.51722L7.12501 24.5L9.12499 24.5Z",
                    fill: match &*emulator_state.read() {
                        Some(AnyEmulatorState::FiveStage(state)) => {
                            match (state.pipeline.ex_control.jmp_base, *hovered_element.read()) {
                                (Some(OpASel::PC), Some(FiveStageElement::JMPBaseControlSignal)) => {
                                    "green"
                                }
                                (Some(OpASel::PC), _) => "rgba(0, 200, 0, 0.4)",
                                (Some(OpASel::RF), Some(FiveStageElement::JMPBaseControlSignal)) => "red",
                                (Some(OpASel::RF), _) => "rgba(200, 0, 0, 0.4)",
                                (None, _) => "gray",
                            }
                        }
                        _ => "gray",
                    },
                }
                line {
                    id: "jmp_base_ctrl_to_opamux_horizontal_line",
                    x1: "916.13",
                    y1: "243.48",
                    x2: "962",
                    y2: "243.48",
                    stroke: match &*emulator_state.read() {
                        Some(AnyEmulatorState::FiveStage(state)) => {
                            match (state.pipeline.ex_control.jmp_base, *hovered_element.read()) {
                                (Some(OpASel::PC), Some(FiveStageElement::JMPBaseControlSignal)) => {
                                    "green"
                                }
                                (Some(OpASel::PC), _) => "rgba(0, 200, 0, 0.4)",
                                (Some(OpASel::RF), Some(FiveStageElement::JMPBaseControlSignal)) => "red",
                                (Some(OpASel::RF), _) => "rgba(200, 0, 0, 0.4)",
                                (None, _) => "gray",
                            }
                        }
                        _ => "gray",
                    },
                    "stroke-width": "2",
                }
                line {
                    id: "jmp_base_ctrl_to_opamux_vertical_line",
                    x1: "963",
                    y1: "242.48",
                    x2: "963",
                    y2: "560",
                    stroke: match &*emulator_state.read() {
                        Some(AnyEmulatorState::FiveStage(state)) => {
                            match (state.pipeline.ex_control.jmp_base, *hovered_element.read()) {
                                (Some(OpASel::PC), Some(FiveStageElement::JMPBaseControlSignal)) => {
                                    "green"
                                }
                                (Some(OpASel::PC), _) => "rgba(0, 200, 0, 0.4)",
                                (Some(OpASel::RF), Some(FiveStageElement::JMPBaseControlSignal)) => "red",
                                (Some(OpASel::RF), _) => "rgba(200, 0, 0, 0.4)",
                                (None, _) => "gray",
                            }
                        }
                        _ => "gray",
                    },
                    "stroke-width": "2",
                }
            }
            g {
                id: "ex_ctrl_to_opbmux_group",
                style: "pointer-events: all;",
                onmouseenter: move |_| {
                    hovered_element.set(Some(FiveStageElement::ALUOpBSelControlSignal));
                },
                onmouseleave: move |_| {
                    hovered_element.set(None);
                },
                path {
                    id: "ex_ctrl_to_opbmux_arrow",
                    transform: "translate(908.86, 460)",
                    d: "M8.70573 0.804236C8.31387 0.415059 7.68071 0.417238 7.29153 0.809106L0.949515 7.19494C0.560338 7.58681 0.562518 8.21997 0.954384 8.60915C1.34625 8.99832 1.97941 8.99614 2.36859 8.60428L8.00593 2.92798L13.6822 8.56532C14.0741 8.9545 14.7073 8.95232 15.0964 8.56046C15.4856 8.16859 15.4834 7.53543 15.0916 7.14625L8.70573 0.804236ZM9.12499 102L9.00106 1.51033L6.90107 1.51722L7.12501 102L9.12499 102Z",
                    fill: match &*emulator_state.read() {
                        Some(AnyEmulatorState::FiveStage(state)) => {
                            match (state.pipeline.ex_control.alu_op_b_sel, *hovered_element.read()) {
                                (Some(OpBSel::IMM), Some(FiveStageElement::ALUOpBSelControlSignal)) => {
                                    "green"
                                }
                                (Some(OpBSel::IMM), _) => "rgba(0, 200, 0, 0.4)",
                                (Some(OpBSel::RF), Some(FiveStageElement::ALUOpBSelControlSignal)) => {
                                    "red"
                                }
                                (Some(OpBSel::RF), _) => "rgba(200, 0, 0, 0.4)",
                                (Some(OpBSel::Four), Some(FiveStageElement::ALUOpBSelControlSignal)) => {
                                    "blue"
                                }
                                (Some(OpBSel::Four), _) => "rgba(0, 0, 200, 0.4)",
                                (None, _) => "gray",
                            }
                        }
                        _ => "gray",
                    },
                }
            }
        }
        g {
            id: "next_pc_select_group",
            style: "pointer-events: all;",
            onmouseenter: move |_| {
                hovered_element.set(Some(FiveStageElement::PCNextSelControlSignal));
            },
            onmouseleave: move |_| {
                hovered_element.set(None);
            },
            path {
                id: "jmp_unconditional_gate",
                transform: "translate(-40, -570)",
                d: "M914.823 580.07C923.812 583.636 934.877 584.922 951.436 585C947.277 575.843 945.146 568.447 945.144 561C945.141 553.554 947.267 546.159 951.434 537C937.668 537.089 926.638 538.377 917.982 541.938C908.208 545.538 899.775 551.487 891.331 561C899.757 570.564 905.79 576.487 914.823 580.07Z",
                fill: element_fill!(PCNextSelControlSignal),
                stroke: element_stroke!(PCNextSelControlSignal),
                "stroke-width": "2",
            }
            path {
                id: "downward_arrow",
                transform: "translate(-575, -261)",
                d: "M620.293 275.707C620.683 276.098 621.317 276.098 621.707 275.707L628.071 269.343C628.462 268.953 628.462 268.319 628.071 267.929C627.681 267.538 627.047 267.538 626.657 267.929L621 273.586L615.343 267.929C614.953 267.538 614.319 267.538 613.929 267.929C613.538 268.319 613.538 268.953 613.929 269.343L620.293 275.707ZM620 252L620 275L622 275L622 252L620 252Z",
                fill: element_stroke!(PCNextSelControlSignal),
            }
            line {
                id: "jmp_uncond_output_line",
                x1: "852",
                y1: "-9",
                x2: "45",
                y2: "-9",
                stroke: element_stroke!(PCNextSelControlSignal),
                "stroke-width": "2",
            }
        }
        g {
            id: "jmp_conditional_AND_ALU",
            style: "pointer-events: all;",
            onmouseenter: move |_| {
                hovered_element.set(Some(FiveStageElement::ConditionResult));
            },
            onmouseleave: move |_| {
                hovered_element.set(None);
            },
            path {
                id: "jmp_conditional_gate",
                transform: "translate(101, -570)",
                d: "M880 560C880 545.086 891.054 533 905.917 533H940V587H905.917C891.054 587 880 574.914 880 560Z",
                fill: element_fill!(ConditionResult),
                stroke: element_stroke!(ConditionResult),
                "stroke-width": "2",
            }
            path {
                id: "jmp_conditional_arrow",
                transform: "translate(-134, -60)",
                d: "M1042.29 38.2929C1041.9 38.6834 1041.9 39.3166 1042.29 39.7071L1048.66 46.0711C1049.05 46.4616 1049.68 46.4616 1050.07 46.0711C1050.46 45.6805 1050.46 45.0474 1050.07 44.6569L1044.41 39L1050.07 33.3431C1050.46 32.9526 1050.46 32.3195 1050.07 31.9289C1049.68 31.5384 1049.05 31.5384 1048.66 31.9289L1042.29 38.2929ZM1088 38L1043 38V40L1088 40V38Z",
                fill: element_stroke!(ConditionResult),
            }
            line {
                id: "jmp_cond_input_line1",
                x1: "955",
                y1: "-10",
                x2: "980",
                y2: "-10",
                stroke: element_stroke!(ConditionResult),
                "stroke-width": "2",
            }
            line {
                id: "jmp_cond_input_line2",
                x1: "954.9",
                y1: "-9",
                x2: "954.9",
                y2: "-22",
                stroke: element_stroke!(ConditionResult),
                "stroke-width": "2",
            }
        }
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
                hovered_element.set(Some(FiveStageElement::IFPC));
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
                stroke: element_stroke!(IFPC),
                "stroke-width": "2",
                fill: element_fill!(IFPC),
            }
            text {
                x: "58",
                y: "245",
                "font-family": "Arial",
                "font-size": "24",
                "font-weight": "bold",
                "text-anchor": "middle",
                fill: element_stroke!(IFPC),
                "PC"
            }
            path {
                id: "if_pc_arrow1",
                d: "M177.707 236.707C178.098 236.317 178.098 235.683 177.707 235.293L171.343 228.929C170.953 228.538 170.319 228.538 169.929 228.929C169.538 229.319 169.538 229.953 169.929 230.343L175.586 236L169.929 241.657C169.538 242.047 169.538 242.681 169.929 243.071C170.319 243.462 170.953 243.462 171.343 243.071L177.707 236.707ZM98 237H177V235H98V237Z",
                fill: element_stroke!(IFPC),
            }
            path {
                id: "if_pc_arrow2",
                d: "M137.707 94.2929C137.317 93.9024 136.683 93.9024 136.293 94.2929L129.929 100.657C129.538 101.047 129.538 101.681 129.929 102.071C130.319 102.462 130.953 102.462 131.343 102.071L137 96.4142L142.657 102.071C143.047 102.462 143.681 102.462 144.071 102.071C144.462 101.681 144.462 101.047 144.071 100.657L137.707 94.2929ZM138 236V95H136V236H138Z",
                fill: element_stroke!(IFPC),
            }
            path {
                id: "next_pc_arrow3",
                d: "M402.707 109.707C403.098 109.317 403.098 108.683 402.707 108.293L396.343 101.929C395.953 101.538 395.319 101.538 394.929 101.929C394.538 102.319 394.538 102.953 394.929 103.343L400.586 109L394.929 114.657C394.538 115.047 394.538 115.681 394.929 116.071C395.319 116.462 395.953 116.462 396.343 116.071L402.707 109.707ZM137 110H402V108H137V110Z",
                fill: element_stroke!(IFPC),
            }
            circle {
                id: "if_pc_node1",
                cx: "137",
                cy: "110",
                r: "3",
                fill: element_stroke!(IFPC),
            }
            circle {
                id: "if_pc_node2",
                cx: "137",
                cy: "236",
                r: "3",
                fill: element_stroke!(IFPC),
            }
        }
        g {
            id: "jmp_address_group",
            style: "pointer-events: all;",
            onmouseenter: move |_| {
                hovered_element.set(Some(FiveStageElement::JMPAddress));
            },
            onmouseleave: move |_| {
                hovered_element.set(None);
            },
            rect {
                id: "jmp_address_rect",
                x: "978",
                y: "156",
                width: "40",
                height: "40",
                fill: element_fill!(JMPAddress),
                stroke: element_stroke!(JMPAddress),
                "stroke-width": "2",
            }
            line {
                id: "jmp_address_line",
                x1: "998",
                y1: "38",
                x2: "998",
                y2: "155",
                stroke: element_stroke!(JMPAddress),
                "stroke-width": "2",
            }
            path {
                id: "jmp_address_arrow",
                transform: "translate(77.5, 30)",
                d: "M0.804236 8.70573C0.415059 8.31387 0.417238 7.68071 0.809106 7.29153L7.19494 0.949515C7.58681 0.560338 8.21997 0.562518 8.60915 0.954384C8.99832 1.34625 8.99614 1.97941 8.60428 2.36859L2.92798 8.00593L8.56532 13.6822C8.9545 14.0741 8.95232 14.7073 8.56046 15.0964C8.16859 15.4856 7.53543 15.4834 7.14625 15.0916L0.804236 8.70573ZM921.5 9.12499L1.51033 9.00106L1.51722 7.00107L921.5 7.12501L921.5 9.12499Z",
                fill: element_stroke!(JMPAddress),
            }
            text {
                x: "998",
                y: "183",
                "font-family": "Arial",
                "font-size": "18",
                "font-weight": "bold",
                "text-anchor": "middle",
                fill: element_stroke!(JMPAddress),
                "+"
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
                height: "570",
                stroke: element_stroke!(IFIDBuffer),
                "stroke-width": "2",
                fill: element_fill!(IFIDBuffer),
            }
            text {
                x: "443",
                y: "610",
                "font-family": "Arial",
                "font-size": "22",
                "font-weight": "bold",
                "text-anchor": "middle",
                fill: element_stroke!(IFIDBuffer),
                "IF"
            }
            text {
                x: "443",
                y: "630",
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
                height: "570",
                stroke: element_stroke!(IDEXBuffer),
                "stroke-width": "2",
                fill: element_fill!(IDEXBuffer),
            }
            text {
                x: "799",
                y: "610",
                "font-family": "Arial",
                "font-size": "22",
                "font-weight": "bold",
                "text-anchor": "middle",
                fill: element_stroke!(IDEXBuffer),
                "ID"
            }
            text {
                x: "799",
                y: "630",
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
                height: "570",
                stroke: element_stroke!(EXMEMBuffer),
                "stroke-width": "2",
                fill: element_fill!(EXMEMBuffer),
            }
            text {
                x: "1156",
                y: "610",
                "font-family": "Arial",
                "font-size": "22",
                "font-weight": "bold",
                "text-anchor": "middle",
                fill: element_stroke!(EXMEMBuffer),
                "EX"
            }
            text {
                x: "1156",
                y: "630",
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
                height: "570",
                stroke: element_stroke!(MEMWBBuffer),
                "stroke-width": "2",
                fill: element_fill!(MEMWBBuffer),
            }
            text {
                x: "1513",
                y: "610",
                "font-family": "Arial",
                "font-size": "22",
                "font-weight": "bold",
                "text-anchor": "middle",
                fill: element_stroke!(MEMWBBuffer),
                "MEM"
            }
            text {
                x: "1513",
                y: "630",
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
            circle {
                id: "idex_pc_node1",
                cx: "521",
                cy: "188",
                r: "3",
                fill: element_stroke!(IFIDInstruction),
            }
            line {
                x1: "521",
                y1: "72",
                x2: "521",
                y2: "561",
                stroke: element_stroke!(IFIDInstruction),
                "stroke-width": "2",
            }
            path {
                id: "ifid_instruction_arrow1",
                d: "M540.707 188.707C541.098 188.317 541.098 187.683 540.707 187.293L534.343 180.929C533.953 180.538 533.319 180.538 532.929 180.929C532.538 181.319 532.538 181.953 532.929 182.343L538.586 188L532.929 193.657C532.538 194.047 532.538 194.681 532.929 195.071C533.319 195.462 533.953 195.462 534.343 195.071L540.707 188.707ZM483 189H540V187H483V189Z",
                fill: element_stroke!(IFIDInstruction),
            }
            path {
                id: "ifid_instruction_arrow2",
                transform: "translate(0, -115)",
                d: "M540.707 188.707C541.098 188.317 541.098 187.683 540.707 187.293L534.343 180.929C533.953 180.538 533.319 180.538 532.929 180.929C532.538 181.319 532.538 181.953 532.929 182.343L538.586 188L532.929 193.657C532.538 194.047 532.538 194.681 532.929 195.071C533.319 195.462 533.953 195.462 534.343 195.071L540.707 188.707ZM521 189H540V187H521V189Z",
                fill: element_stroke!(IFIDInstruction),
            }
            path {
                id: "ifid_instruction_arrow3",
                transform: "translate(0, 372)",
                d: "M578.707 188.707C579.098 188.317 579.098 187.683 578.707 187.293L572.343 180.929C571.953 180.538 571.319 180.538 570.929 180.929C570.538 181.319 570.538 181.953 570.929 182.343L576.586 188L570.929 193.657C570.538 194.047 570.538 194.681 570.929 195.071C571.319 195.462 571.953 195.462 572.343 195.071L578.707 188.707ZM522 189H578V187H522V189Z",
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
            id: "hazard_unit_group",
            style: "pointer-events: all;",
            onmouseenter: move |_| {
                hovered_element.set(Some(FiveStageElement::HazardUnit));
            },
            onmouseleave: move |_| {
                hovered_element.set(None);
            },
            rect {
                id: "hazard_rect",
                x: "542",
                y: "50",
                width: "158",
                height: "48",
                stroke: element_stroke!(HazardUnit),
                "stroke-width": "2",
                fill: element_fill!(HazardUnit),
            }
            text {
                x: "621",
                y: "80",
                "font-family": "Arial",
                "font-size": "18",
                "font-weight": "bold",
                "text-anchor": "middle",
                fill: element_stroke!(HazardUnit),
                "HAZARD UNIT"
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
            line {
                x1: "661",
                y1: "227",
                x2: "661",
                y2: "250",
                stroke: element_stroke!(DecoderRD),
                "stroke-width": "2",
            }
            line {
                x1: "660",
                y1: "251",
                x2: "710",
                y2: "251",
                stroke: element_stroke!(DecoderRD),
                "stroke-width": "2",
            }
            line {
                x1: "711",
                y1: "250",
                x2: "711",
                y2: "512",
                stroke: element_stroke!(DecoderRD),
                "stroke-width": "2",
            }
            path {
                id: "rs2_value_arrow",
                transform: "translate(0, 64)",
                d: "M758.707 447.707C759.098 447.317 759.098 446.683 758.707 446.293L752.343 439.929C751.953 439.538 751.319 439.538 750.929 439.929C750.538 440.319 750.538 440.953 750.929 441.343L756.586 447L750.929 452.657C750.538 453.047 750.538 453.681 750.929 454.071C751.319 454.462 751.953 454.462 752.343 454.071L758.707 447.707ZM711 448H758V446H711V448Z",
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
                y: "277",
                width: "158",
                height: "203",
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
                transform: "translate(0, -120)",
                d: "M886.707 269.707C887.098 269.317 887.098 268.683 886.707 268.293L880.343 261.929C879.953 261.538 879.319 261.538 878.929 261.929C878.538 262.319 878.538 262.953 878.929 263.343L884.586 269L878.929 274.657C878.538 275.047 878.538 275.681 878.929 276.071C879.319 276.462 879.953 276.462 880.343 276.071L886.707 269.707ZM867 270H886V268H867V270Z",
                fill: element_stroke!(IDEXPC),
            }
            path {
                id: "idex_pc_arrow3",
                d: "M886.707 269.707C887.098 269.317 887.098 268.683 886.707 268.293L880.343 261.929C879.953 261.538 879.319 261.538 878.929 261.929C878.538 262.319 878.538 262.953 878.929 263.343L884.586 269L878.929 274.657C878.538 275.047 878.538 275.681 878.929 276.071C879.319 276.462 879.953 276.462 880.343 276.071L886.707 269.707ZM867 270H886V268H867V270Z",
                fill: element_stroke!(IDEXPC),
            }
            line {
                id: "idex_pc_line",
                x1: "866",
                y1: "270",
                x2: "866",
                y2: "110",
                stroke: element_stroke!(IDEXPC),
                "stroke-width": "2",
            }
            circle {
                id: "idex_pc_node1",
                cx: "866",
                cy: "109",
                r: "3",
                fill: element_stroke!(IDEXPC),
            }
            circle {
                id: "idex_pc_node2",
                cx: "866",
                cy: "149",
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
            line {
                id: "idex_pc_arrow3",
                x1: "850",
                y1: "325",
                x2: "850",
                y2: "205",
                stroke: element_stroke!(IDEXRS1),
                "stroke-width": "2",
            }
            circle {
                id: "idex_pc_node1",
                cx: "850",
                cy: "325",
                r: "3",
                fill: element_stroke!(IDEXRS1),
            }
            path {
                id: "idex_rs1_arrow",
                d: "M886.707 325.707C887.098 325.317 887.098 324.683 886.707 324.293L880.343 317.929C879.953 317.538 879.319 317.538 878.929 317.929C878.538 318.319 878.538 318.953 878.929 319.343L884.586 325L878.929 330.657C878.538 331.047 878.538 331.681 878.929 332.071C879.319 332.462 879.953 332.462 880.343 332.071L886.707 325.707ZM839 326H886V324H839V326Z",
                fill: element_stroke!(IDEXRS1),
            }
            path {
                id: "idex_rs1_arrow_shorter",
                transform: "translate(0, -120)",
                d: "M886.707 325.707C887.098 325.317 887.098 324.683 886.707 324.293L880.343 317.929C879.953 317.538 879.319 317.538 878.929 317.929C878.538 318.319 878.538 318.953 878.929 319.343L884.586 325L878.929 330.657C878.538 331.047 878.538 331.681 878.929 332.071C879.319 332.462 879.953 332.462 880.343 332.071L886.707 325.707ZM849 326H886V324H849V326Z",
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
            id: "idex_rd_register_group",
            style: "pointer-events: all;",
            onmouseenter: move |_| {
                hovered_element.set(Some(FiveStageElement::IDEXRD));
            },
            onmouseleave: move |_| {
                hovered_element.set(None);
            },
            rect {
                id: "idex_rd_rect",
                x: "760",
                y: "491",
                width: "78",
                height: "38",
                stroke: element_stroke!(IDEXRD),
                "stroke-width": "2",
                fill: element_fill!(IDEXRD),
            }
            path {
                id: "idex_rd_arrow",
                d: "M1115.71 325.707C1116.1 325.317 1116.1 324.683 1115.71 324.293L1109.34 317.929C1108.95 317.538 1108.32 317.538 1107.93 317.929C1107.54 318.319 1107.54 318.953 1107.93 319.343L1113.59 325L1107.93 330.657C1107.54 331.047 1107.54 331.681 1107.93 332.071C1108.32 332.462 1108.95 332.462 1109.34 332.071L1115.71 325.707ZM839 326H1115V324H839V326Z",
                fill: element_stroke!(IDEXRD),
                transform: "translate(0, 186)",
            }
            text {
                x: "799",
                y: "517",
                "font-family": "Arial",
                "font-size": "18",
                "font-weight": "bold",
                "text-anchor": "middle",
                fill: element_stroke!(IDEXRD),
                "RD"
            }
        }
        g {
            id: "idex_ctrl_register_group",
            style: "pointer-events: all;",
            onmouseenter: move |_| {
                hovered_element.set(Some(FiveStageElement::EXControl));
            },
            onmouseleave: move |_| {
                hovered_element.set(None);
            },
            rect {
                id: "ex_ctrl_rect",
                x: "760",
                y: "541",
                width: "78",
                height: "38",
                stroke: element_stroke!(EXControl),
                "stroke-width": "2",
                fill: element_fill!(EXControl),
            }
            path {
                id: "ex_ctrl_arrow",
                d: "M1115.71 325.707C1116.1 325.317 1116.1 324.683 1115.71 324.293L1109.34 317.929C1108.95 317.538 1108.32 317.538 1107.93 317.929C1107.54 318.319 1107.54 318.953 1107.93 319.343L1113.59 325L1107.93 330.657C1107.54 331.047 1107.54 331.681 1107.93 332.071C1108.32 332.462 1108.95 332.462 1109.34 332.071L1115.71 325.707ZM839 326H1115V324H839V326Z",
                fill: element_stroke!(EXControl),
                transform: "translate(0, 236)",
            }
            text {
                x: "799",
                y: "567",
                "font-family": "Arial",
                "font-size": "18",
                "font-weight": "bold",
                "text-anchor": "middle",
                fill: element_stroke!(EXControl),
                "CTRL"
            }
        }
        g {
            id: "exmem_rd_register_group",
            style: "pointer-events: all;",
            onmouseenter: move |_| {
                hovered_element.set(Some(FiveStageElement::EXMEMRD));
            },
            onmouseleave: move |_| {
                hovered_element.set(None);
            },
            rect {
                id: "exmem_rd_rect",
                x: "1117",
                y: "491",
                width: "78",
                height: "38",
                stroke: element_stroke!(EXMEMRD),
                "stroke-width": "2",
                fill: element_fill!(EXMEMRD),
            }
            path {
                id: "exmem_rd_arrow",
                d: "M1115.71 325.707C1116.1 325.317 1116.1 324.683 1115.71 324.293L1109.34 317.929C1108.95 317.538 1108.32 317.538 1107.93 317.929C1107.54 318.319 1107.54 318.953 1107.93 319.343L1113.59 325L1107.93 330.657C1107.54 331.047 1107.54 331.681 1107.93 332.071C1108.32 332.462 1108.95 332.462 1109.34 332.071L1115.71 325.707ZM839 326H1115V324H839V326Z",
                fill: element_stroke!(EXMEMRD),
                transform: "translate(357, 186)",
            }
            text {
                x: "1156",
                y: "517",
                "font-family": "Arial",
                "font-size": "18",
                "font-weight": "bold",
                "text-anchor": "middle",
                fill: element_stroke!(EXMEMRD),
                "RD"
            }
        }
        g {

            id: "exmem_ctrl_register_group",
            style: "pointer-events: all;",
            onmouseenter: move |_| {
                hovered_element.set(Some(FiveStageElement::MEMControl));
            },
            onmouseleave: move |_| {
                hovered_element.set(None);
            },
            rect {
                id: "mem_ctrl_rect",
                x: "1117",
                y: "541",
                width: "78",
                height: "38",
                stroke: element_stroke!(MEMControl),
                "stroke-width": "2",
                fill: element_fill!(MEMControl),
            }
            path {
                id: "exmem_ctrl_arrow",
                d: "M1115.71 325.707C1116.1 325.317 1116.1 324.683 1115.71 324.293L1109.34 317.929C1108.95 317.538 1108.32 317.538 1107.93 317.929C1107.54 318.319 1107.54 318.953 1107.93 319.343L1113.59 325L1107.93 330.657C1107.54 331.047 1107.54 331.681 1107.93 332.071C1108.32 332.462 1108.95 332.462 1109.34 332.071L1115.71 325.707ZM839 326H1115V324H839V326Z",
                fill: element_stroke!(MEMControl),
                transform: "translate(357, 236)",
            }
            text {
                x: "1156",
                y: "567",
                "font-family": "Arial",
                "font-size": "18",
                "font-weight": "bold",
                "text-anchor": "middle",
                fill: element_stroke!(MEMControl),
                "CTRL"
            }
        }
        g {
            id: "memwb_rd_register_group",
            style: "pointer-events: all;",
            onmouseenter: move |_| {
                hovered_element.set(Some(FiveStageElement::MEMWBRD));
            },
            onmouseleave: move |_| {
                hovered_element.set(None);
            },
            rect {
                id: "memwb_rd_rect",
                x: "1474",
                y: "491",
                width: "78",
                height: "38",
                stroke: element_stroke!(MEMWBRD),
                "stroke-width": "2",
                fill: element_fill!(MEMWBRD),
            }
            text {
                x: "1513",
                y: "517",
                "font-family": "Arial",
                "font-size": "18",
                "font-weight": "bold",
                "text-anchor": "middle",
                fill: element_stroke!(MEMWBRD),
                "RD"
            }
            line {
                id: "memwb_rd_line1",
                x1: "1552",
                y1: "510",
                x2: "1572",
                y2: "510",
                "stroke-width": "2",
                stroke: element_stroke!(MEMWBRD),
            }
            line {
                id: "memwb_rd_line2",
                x1: "1572",
                y1: "509",
                x2: "1572",
                y2: "650",
                "stroke-width": "2",
                stroke: element_stroke!(MEMWBRD),
            }
            line {
                id: "memwb_rd_line2",
                x1: "1572",
                y1: "650",
                x2: "500",
                y2: "650",
                "stroke-width": "2",
                stroke: element_stroke!(MEMWBRD),
            }
            line {
                id: "memwb_rd_line2",
                x1: "500",
                y1: "651",
                x2: "500",
                y2: "421",
                "stroke-width": "2",
                stroke: element_stroke!(MEMWBRD),
            }
            path {
                id: "writeback_arrow",
                d: "M540.707 447.707C541.098 447.317 541.098 446.683 540.707 446.293L534.343 439.929C533.953 439.538 533.319 439.538 532.929 439.929C532.538 440.319 532.538 440.953 532.929 441.343L538.586 447L532.929 452.657C532.538 453.047 532.538 453.681 532.929 454.071C533.319 454.462 533.953 454.462 534.343 454.071L540.707 447.707ZM500 448H540V446H500V448Z",
                transform: "translate(0, -25)",
                fill: element_stroke!(MEMWBRD),
            }
            text {
                x: "544",
                y: "424",
                "text-anchor": "start",
                "dominant-baseline": "middle",
                "font-size": "12",
                "font-weight": "bold",
                fill: element_stroke!(MEMWBRD),
                "RD"
            }
        }
        g {
            id: "memwb_ctrl_register_group",
            style: "pointer-events: all;",
            onmouseenter: move |_| {
                hovered_element.set(Some(FiveStageElement::WBControl));
            },
            onmouseleave: move |_| {
                hovered_element.set(None);
            },
            rect {
                id: "wb_ctrl_rect",
                x: "1474",
                y: "541",
                width: "78",
                height: "38",
                stroke: element_stroke!(WBControl),
                "stroke-width": "2",
                fill: element_fill!(WBControl),
            }
            text {
                x: "1513",
                y: "567",
                "font-family": "Arial",
                "font-size": "18",
                "font-weight": "bold",
                "text-anchor": "middle",
                fill: element_stroke!(WBControl),
                "CTRL"
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
                id: "idex_imm_arrow1",
                d: "M886.707 395.707C887.098 395.317 887.098 394.683 886.707 394.293L880.343 387.929C879.953 387.538 879.319 387.538 878.929 387.929C878.538 388.319 878.538 388.953 878.929 389.343L884.586 395L878.929 400.657C878.538 401.047 878.538 401.681 878.929 402.071C879.319 402.462 879.953 402.462 880.343 402.071L886.707 395.707ZM839 396H886V394H839V396Z",
                fill: element_stroke!(IDEXImm),
            }
            path {
                id: "idex_imm_arrow2",
                transform: "translate(990, 196.5)",
                d: "M8.70573 0.804236C8.31387 0.415059 7.68071 0.417238 7.29153 0.809106L0.949515 7.19494C0.560338 7.58681 0.562518 8.21997 0.954384 8.60915C1.34625 8.99832 1.97941 8.99614 2.36859 8.60428L8.00593 2.92798L13.6822 8.56532C14.0741 8.9545 14.7073 8.95232 15.0964 8.56046C15.4856 8.16859 15.4834 7.53543 15.0916 7.14625L8.70573 0.804236ZM9.12499 41.5L9.00106 1.51033L7.00107 1.51722L7.12501 41.5L9.12499 41.5Z",
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
            line {
                id: "idex_pc_line1",
                x1: "858",
                y1: "395",
                x2: "858",
                y2: "236",
                stroke: element_stroke!(IDEXImm),
                "stroke-width": "2",
            }
            line {
                id: "idex_pc_line2",
                x1: "859",
                y1: "237",
                x2: "998",
                y2: "237",
                stroke: element_stroke!(IDEXImm),
                "stroke-width": "2",
            }
            circle {
                id: "idex_pc_node1",
                cx: "858",
                cy: "395",
                r: "3",
                fill: element_stroke!(IDEXImm),
            }
        }
        g {
            id: "jmp_addr_mux_group",
            style: "pointer-events: all;",
            onmouseenter: move |_| {
                hovered_element.set(Some(FiveStageElement::JMPBaseAddress));
            },
            onmouseleave: move |_| {
                hovered_element.set(None);
            },
            path {
                id: "jmp_addr_mux",
                d: "M888 243.558L946 270.142V323.858L888 350.442V243.558Z",
                transform: "translate(0, -120)",
                stroke: element_stroke!(JMPBaseAddress),
                "stroke-width": "2",
                fill: element_fill!(JMPBaseAddress),
            }
            path {
                id: "jmp_addr_arrow",
                transform: "translate(0, -120)",
                d: "M976.707 297.707C977.098 297.317 977.098 296.683 976.707 296.293L970.343 289.929C969.953 289.538 969.319 289.538 968.929 289.929C968.538 290.319 968.538 290.953 968.929 291.343L974.586 297L968.929 302.657C968.538 303.047 968.538 303.681 968.929 304.071C969.319 304.462 969.953 304.462 970.343 304.071L976.707 297.707ZM947 298H976V296H947V298Z",
                fill: element_stroke!(JMPBaseAddress),
            }
            text {
                x: "902",
                y: "153",
                "font-family": "Arial",
                "font-size": "14",
                "font-weight": "bold",
                "text-anchor": "middle",
                fill: element_stroke!(JMPBaseAddress),
                "PC"
            }
            text {
                x: "905",
                y: "210",
                "font-family": "Arial",
                "font-size": "14",
                "font-weight": "bold",
                "text-anchor": "middle",
                fill: element_stroke!(JMPBaseAddress),
                "RS1"
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
                transform: "translate(0, -40)",
                d: "M1042.29 38.2929C1041.9 38.6834 1041.9 39.3166 1042.29 39.7071L1048.66 46.0711C1049.05 46.4616 1049.68 46.4616 1050.07 46.0711C1050.46 45.6805 1050.46 45.0474 1050.07 44.6569L1044.41 39L1050.07 33.3431C1050.46 32.9526 1050.46 32.3195 1050.07 31.9289C1049.68 31.5384 1049.05 31.5384 1048.66 31.9289L1042.29 38.2929ZM1088 38L1043 38V40L1088 40V38Z",
                fill: element_stroke!(ALU),
            }
            path {
                id: "alu_arrow3",
                transform: "translate(-341, 20)",
                d: "M1042.29 38.2929C1041.9 38.6834 1041.9 39.3166 1042.29 39.7071L1048.66 46.0711C1049.05 46.4616 1049.68 46.4616 1050.07 46.0711C1050.46 45.6805 1050.46 45.0474 1050.07 44.6569L1044.41 39L1050.07 33.3431C1050.46 32.9526 1050.46 32.3195 1050.07 31.9289C1049.68 31.5384 1049.05 31.5384 1048.66 31.9289L1042.29 38.2929ZM1429 38L1043 38V40L1429 40V38Z",
                fill: element_stroke!(ALU),
            }
            circle {
                id: "alu_node",
                cx: "1087",
                cy: "59",
                r: "3",
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
                y2: "0",
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
                y2: "660",
                stroke: element_stroke!(WritebackResult),
                "stroke-width": "2",
            }
            line {
                id: "writeback_line3",
                x1: "510",
                y1: "660",
                x2: "1665",
                y2: "660",
                stroke: element_stroke!(WritebackResult),
                "stroke-width": "2",
            }
            line {
                id: "writeback_line4",
                x1: "511",
                y1: "660",
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
                x: "544",
                y: "449",
                "text-anchor": "start",
                "dominant-baseline": "middle",
                "font-size": "12",
                "font-weight": "bold",
                fill: element_stroke!(WritebackResult),
                "WB"
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
                x: "580",
                y: "520",
                width: "120",
                height: "80",
                stroke: element_stroke!(ControlUnit),
                "stroke-width": "2",
                fill: element_fill!(ControlUnit),
            }
            text {
                x: "640",
                y: "567",
                "font-family": "Arial",
                "font-size": "20",
                "font-weight": "bold",
                "text-anchor": "middle",
                fill: element_stroke!(ControlUnit),
                "CONTROL"
            }
            path {
                id: "controller_to_idex_arrow",
                d: "M758.707 447.707C759.098 447.317 759.098 446.683 758.707 446.293L752.343 439.929C751.953 439.538 751.319 439.538 750.929 439.929C750.538 440.319 750.538 440.953 750.929 441.343L756.586 447L750.929 452.657C750.538 453.047 750.538 453.681 750.929 454.071C751.319 454.462 751.953 454.462 752.343 454.071L758.707 447.707ZM701 448H758V446H701V448Z",
                fill: element_stroke!(ControlUnit),
                transform: "translate(0, 113)",
            }
        }
    }
}
