use serde::{Deserialize, Serialize};

/// Struct representing the datapath for the `cve2_top` module.
/// Taken from https://github.com/openhwgroup/cve2/blob/main/rtl/cve2_top.sv

#[allow(dead_code)]
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct CVE2Datapath {
    // Clock and Reset
    pub clk_i: bool,  // Input clock signal.
    pub rst_ni: bool, // Active-low reset signal.

    // Instruction memory interface
    pub instr_req_o: bool,    // Output signal requesting an instruction fetch.
    pub instr_addr_o: u32,    // Output address for fetching instructions.
    pub instr_rdata_i: u32,   // Input data received as the fetched instruction.
    pub instr_gnt_i: bool,    // Input signal indicating the instruction request is granted.
    pub instr_rvalid_i: bool, // Input signal indicating valid instruction data is available.
    pub instr_err_i: bool,    // Input signal indicating an error during instruction fetch.

    // Data memory interface
    pub data_req_o: bool,  // Output signal requesting a data memory operation.
    pub data_addr_o: u32,  // Output address for the data memory operation.
    pub data_wdata_o: u32, // Output data to be written to memory.
    pub data_rdata_i: u32, // Input data read from memory.
    pub data_we_o: bool,   // Output write-enable signal for data memory.
    pub data_be_o: [bool; 4], // Output byte-enable (4-bit) for selective byte access in 32-bit words.
    pub data_gnt_i: bool,     // Input signal indicating the data request is granted.
    pub data_rvalid_i: bool,  // Input signal indicating valid data is available.
    pub data_err_i: bool,     // Input signal indicating an error during the data memory operation.

    // Core execution control signals
    pub id_multicycle: u32, // Output signal indicating if the instruction is a multi-cycle instruction.
    pub fetch_enable_i: bool, // Input signal enabling instruction fetch.
    pub core_sleep_o: bool, // Output signal indicating if the core is in sleep mode.

    // Interrupt inputs
    pub irq_software_i: bool, // Input software interrupt request signal.
    pub irq_timer_i: bool,    // Input timer interrupt request signal.
    pub irq_external_i: bool, // Input external interrupt request signal.
    pub irq_fast_i: u16,      // Input fast interrupt vector, 16 bits for fast IRQs.
    pub irq_nm_i: bool,       // Input non-maskable interrupt request signal.

    // Debug Interface
    pub debug_req_i: bool, // Input signal indicating a debug request.
}

impl Default for CVE2Datapath {
    fn default() -> Self {
        Self {
            clk_i: Default::default(),
            rst_ni: Default::default(),
            instr_req_o: true,
            instr_addr_o: 0u32,
            instr_rdata_i: Default::default(),
            instr_gnt_i: Default::default(),
            instr_rvalid_i: Default::default(),
            instr_err_i: Default::default(),
            data_req_o: Default::default(),
            data_addr_o: Default::default(),
            data_wdata_o: Default::default(),
            data_rdata_i: Default::default(),
            data_we_o: Default::default(),
            data_be_o: Default::default(),
            data_gnt_i: Default::default(),
            data_rvalid_i: Default::default(),
            data_err_i: Default::default(),
            fetch_enable_i: true,
            core_sleep_o: Default::default(),
            irq_software_i: Default::default(),
            irq_timer_i: Default::default(),
            irq_external_i: Default::default(),
            irq_fast_i: Default::default(),
            irq_nm_i: Default::default(),
            debug_req_i: Default::default(),
            id_multicycle: Default::default(),
        }
    }
}

#[allow(non_snake_case)]
#[derive(Copy, Clone, Default, Debug, Serialize, Deserialize)]
pub struct CVE2Pipeline {
    pub IF: u32,    // Instruction Fetch Buffer
    pub IF_pc: u32, // Program Counter for the IF stage
    pub ID: u32,    // Instruction Decode Buffer
    pub ID_pc: u32, // Program Counter for the ID stage
    pub datapath: CVE2Datapath,
}

impl CVE2Pipeline {}
