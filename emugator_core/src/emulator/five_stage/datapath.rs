use crate::emulator::controller_common::PCSel;

use super::hazard_detection::Hazard;

#[derive(Clone, Copy, Default, Debug)]
pub struct IfLines {
    pub instr: Option<u32>,
    pub next_pc_sel: PCSel,
    pub next_pc: Option<u32>,
}

#[derive(Clone, Copy, Default, Debug)]
pub struct IfIdBuffer {
    pub id_pc: Option<u32>,
    pub id_inst: Option<u32>,
}

#[derive(Clone, Copy, Default, Debug)]
pub struct IdLines {
    // decode
    pub rs1: u8,
    pub rs2: u8,
    pub imm: Option<u32>,
    pub rd: u8,

    // register reads
    pub rs1_v: u32,
    pub rs2_v: u32,

    // hazard detection
    pub hazard_detected: Hazard,
}

#[derive(Clone, Copy, Default, Debug)]
pub struct IdExBuffer {
    pub ex_pc: Option<u32>,
    pub rs1_v: u32,
    pub rs2_v: u32,
    pub imm: Option<u32>,
    pub rd: Option<u8>,
}

#[derive(Clone, Copy, Default, Debug)]
pub struct ExLines {
    // alu
    pub op_a: Option<u32>,
    pub op_b: Option<u32>,
    pub alu_out: Option<u32>,

    // next pc
    pub jmp_base: Option<u32>,
    pub jmp_dst: Option<u32>,
    pub cmp_result: Option<u32>,
}

#[derive(Clone, Copy, Default, Debug)]
pub struct ExMemBuffer {
    pub mem_pc: Option<u32>,
    pub alu_o: Option<u32>,
    pub rs2_v: u32,
    pub rd: Option<u8>,
}

#[derive(Clone, Copy, Default, Debug)]
pub struct MemLines {
    pub mem_data: Option<u32>,

    // data memory interface
    pub data_req_o: bool,  // Output signal requesting a data memory operation.
    pub data_addr_o: u32,  // Output address for the data memory operation.
    pub data_wdata_o: u32, // Output data to be written to memory.
    pub data_rdata_i: u32, // Input data read from memory.
    pub data_we_o: bool,   // Output write-enable signal for data memory.
    pub data_be_o: [bool; 4], // Output byte-enable (4-bit) for selective byte access in 32-bit words.
    pub data_gnt_i: bool,     // Input signal indicating the data request is granted.
    pub data_rvalid_i: bool,  // Input signal indicating valid data is available.
    pub data_err_i: bool,     // Input signal indicating an error during the data memory operation
}

#[derive(Clone, Copy, Default, Debug)]
pub struct MemWbBuffer {
    pub wb_pc: Option<u32>,
    pub alu: Option<u32>,
    pub lsu: Option<u32>,
    pub rd: Option<u8>,
}

#[derive(Clone, Copy, Default, Debug)]
pub struct WbLines {
    pub wb_data: Option<u32>,
}
