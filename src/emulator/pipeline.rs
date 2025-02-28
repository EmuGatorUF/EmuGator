#[allow(non_snake_case)]
#[derive(Copy, Clone, Debug)]
pub struct CVE2Pipeline {
    pub IF: u32,    // Instruction Fetch Buffer
    pub IF_pc: u32, // Program Counter for the IF stage
    pub ID: u32,    // Instruction Decode Buffer
    pub ID_pc: u32, // Program Counter for the ID stage
    pub datapath: CVE2Datapath,
    pub control: CVE2Control,
}

impl Default for CVE2Pipeline {
    fn default() -> Self {
        Self {
            IF: 0,
            IF_pc: 0,
            ID: 0,
            ID_pc: 0,
            datapath: CVE2Datapath::starting(),
            control: CVE2Control::default(),
        }
    }
}

/// Struct representing the datapath for the `cve2_top` module.
/// Taken from https://github.com/openhwgroup/cve2/blob/main/rtl/cve2_top.sv
#[allow(dead_code)]
#[derive(Clone, Copy, Debug, Default)]
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

    // Extra internal lines not in the top module
    // decoded instruction
    pub reg_s1: u8,
    pub reg_s2: u8,
    pub imm: Option<u32>,
    pub reg_d: u8,

    // intermediate data
    pub data_s1: u32,
    pub data_s2: u32,
    pub alu_op_a: Option<u32>, // Operand A input.
    pub alu_op_b: Option<u32>, // Operand B input.
    pub alu_out: Option<u32>,  // ALU output.
    pub lsu_out: Option<u32>,  // Load/Store Unit output.
    pub reg_write_data: Option<u32>,
}

impl CVE2Datapath {
    fn starting() -> Self {
        Self {
            instr_req_o: true,
            instr_addr_o: 0u32,
            fetch_enable_i: true,
            ..Default::default()
        }
    }
}

/// Control signals for the CVE2 datapath.
/// Note: `Option::None` is used to represent a "don't care" value.
#[derive(Clone, Copy, Debug, Default)]
pub struct CVE2Control {
    // ALU Control
    pub alu_op_a_sel: Option<OpASel>, // Mux control for selecting operand A.
    pub alu_op_b_sel: Option<OpBSel>, // Mux control for selecting operand B.
    pub alu_op: Option<ALUOp>,        // ALU operation control.

    // LSU Control
    pub lsu_data_type: Option<LSUDataType>, // Data type for load/store operations.
    pub lsu_request: bool,                  // Request signal for the LSU.
    pub lsu_write_enable: bool,             // Write enable for the LSU.
    pub lsu_sign_ext: bool,                 // Sign extension control for load operations.

    // Register Write Control
    pub data_dest_sel: Option<DataDestSel>, // Mux control for selecting the write-back data.
    pub reg_write: bool,                    // Register write control.
}

impl CVE2Control {
    pub fn register(op: ALUOp) -> Self {
        Self {
            alu_op_a_sel: Some(OpASel::RF),
            alu_op_b_sel: Some(OpBSel::RF),
            alu_op: Some(op),
            data_dest_sel: Some(DataDestSel::ALU),
            reg_write: true,
            ..Default::default()
        }
    }

    pub fn immediate(op: ALUOp) -> Self {
        Self {
            alu_op_a_sel: Some(OpASel::RF),
            alu_op_b_sel: Some(OpBSel::IMM),
            alu_op: Some(op),
            data_dest_sel: Some(DataDestSel::ALU),
            reg_write: true,
            ..Default::default()
        }
    }

    pub fn load_request(data_type: LSUDataType) -> Self {
        Self {
            // address calculation
            alu_op_a_sel: Some(OpASel::RF),
            alu_op_b_sel: Some(OpBSel::IMM),
            alu_op: Some(ALUOp::ADD),

            // lsu inputs
            lsu_data_type: Some(data_type),
            lsu_request: true,
            lsu_write_enable: false,

            ..Default::default()
        }
    }

    pub fn load_write(data_type: LSUDataType, sign_ext: bool) -> Self {
        Self {
            // lsu inputs
            lsu_data_type: Some(data_type),
            lsu_sign_ext: sign_ext,

            // register write
            data_dest_sel: Some(DataDestSel::LSU),
            reg_write: true,
            ..Default::default()
        }
    }

    pub fn store_request(data_type: LSUDataType) -> Self {
        Self {
            // address calculation
            alu_op_a_sel: Some(OpASel::RF),
            alu_op_b_sel: Some(OpBSel::IMM),
            alu_op: Some(ALUOp::ADD),

            // lsu inputs
            lsu_data_type: Some(data_type),
            lsu_request: true,
            lsu_write_enable: true,

            ..Default::default()
        }
    }

    pub fn store_completion() -> Self {
        Self {
            ..Default::default()
        }
    }
}

#[repr(u32)]
#[derive(Copy, Clone, Debug)]
pub enum ALUOp {
    ADD,
    SUB,
    XOR,
    OR,
    AND,
    SLL,
    SRL,
    SRA,
    SLT,
    SLTU,
}

#[allow(dead_code)]
#[repr(u32)]
#[derive(Copy, Clone, Debug)]
pub enum OpASel {
    PC,
    RF,
}

#[allow(dead_code)]
#[repr(u32)]
#[derive(Copy, Clone, Debug)]
pub enum OpBSel {
    RF,
    IMM,
}

#[allow(dead_code)]
#[repr(u32)]
#[derive(Copy, Clone, Debug)]
pub enum DataDestSel {
    ALU,
    LSU,
}

#[allow(dead_code)]
#[repr(u32)]
#[derive(Copy, Clone, Debug)]
pub enum LSUDataType {
    Word,
    HalfWord,
    Byte,
}

impl LSUDataType {
    pub fn byte_enable(&self) -> [bool; 4] {
        match self {
            LSUDataType::Word => [true; 4],
            LSUDataType::HalfWord => [true, true, false, false],
            LSUDataType::Byte => [true, false, false, false],
        }
    }

    pub fn size_in_bits(&self) -> usize {
        match self {
            LSUDataType::Word => 32,
            LSUDataType::HalfWord => 16,
            LSUDataType::Byte => 8,
        }
    }
}
