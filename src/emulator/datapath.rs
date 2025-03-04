/// Lines in the datapath
///
/// Initially based on the `cve2_top` module.
/// Taken from https://github.com/openhwgroup/cve2/blob/main/rtl/cve2_top.sv
#[derive(Clone, Copy, Debug, Default)]
pub struct CVE2Datapath {
    // Clock and Reset
    // pub clk_i: bool,  // Input clock signal.
    // pub rst_ni: bool, // Active-low reset signal.

    // Instruction memory interface
    // pub instr_req_o: bool, // Output signal requesting an instruction fetch.
    // pub instr_addr_o: u32,    // Output address for fetching instructions.
    // pub instr_rdata_i: u32, // Input data received as the fetched instruction.
    pub instr_gnt_i: bool, // Input signal indicating the instruction request is granted.
    pub instr_rvalid_i: bool, // Input signal indicating valid instruction data is available.
    pub instr_err_i: bool, // Input signal indicating an error during instruction fetch.

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
    // pub id_multicycle: u32, // Output signal indicating if the instruction is a multi-cycle instruction.
    // pub fetch_enable_i: bool, // Input signal enabling instruction fetch.
    // pub core_sleep_o: bool, // Output signal indicating if the core is in sleep mode.

    // Interrupt inputs
    // pub irq_software_i: bool, // Input software interrupt request signal.
    // pub irq_timer_i: bool,    // Input timer interrupt request signal.
    // pub irq_external_i: bool, // Input external interrupt request signal.
    // pub irq_fast_i: u16,      // Input fast interrupt vector, 16 bits for fast IRQs.
    // pub irq_nm_i: bool,       // Input non-maskable interrupt request signal.

    // Debug Interface
    // pub debug_req_i: bool, // Input signal indicating a debug request.

    // Extra internal lines not in the top module
    // decoded instruction
    pub reg_s1: u8,
    pub reg_s2: u8,
    pub imm: Option<u32>,
    pub reg_d: u8,

    // register file outputs
    pub data_s1: u32,
    pub data_s2: u32,

    /// alu and lsu
    pub alu_op_a: Option<u32>, // Operand A input.
    pub alu_op_b: Option<u32>, // Operand B input.
    pub alu_out: Option<u32>,  // ALU output.
    pub lsu_out: Option<u32>,  // Load/Store Unit output.
    pub reg_write_data: Option<u32>,

    // program counter
    pub cmp_result: bool,       // Result of the branch comparison operation.
    pub should_cond_jump: bool, // If the branch should be taken
    pub next_pc_sel: PCSel,     // Mux control for selecting the next program counter.
    pub next_pc: Option<u32>,   // Next program counter value.
}

#[repr(u32)]
#[derive(Copy, Clone, Debug, Default)]
pub enum PCSel {
    #[default]
    PC4,
    ALU,
}
