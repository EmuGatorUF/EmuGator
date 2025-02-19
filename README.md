# EmuGator
University of Florida Computer Engineering Design Project

## Completed Work
The assembler, front-end, and user-interface are now connected. Users can step through programs. Most instructions in the RV-32I instruction set are implemented except EBREAK and ECALL. The assembler assembles the code and detects syntax errors. The UI includes syntax highlighting and allows users to step through clock cycles through calls to the emulator functions. Users can see the current instruction executing, the last instruction fetched, registers, instruction memory, and data memory.

# Project Architecture

## Monaco, Tailwind, and Dioxus Frontend
The architecture of the editor was designed and established during the pre-alpha stage. It consists of two key components:
- **Low-Level Component**: Manages adding and activating the Monaco editor to the DOM.
- **Top-Level Component**: Provides default configurations to improve appearance and expose the model of the editor to the Dioxus state life cycle.

Remaining architectural challenges include:
- Getting diagnostics from the assembler
- Implementing hover documentation

## EmuGator Core
The emulator is modeled as a finite state machine. Instructions are decoded according to the instruction formats defined in the [RV32I Base Integer Instruction Set Version 2.0](https://riscv.org/wp-content/uploads/2017/05/riscv-spec-v2.2.pdf). Each instruction resolves to an instruction handler that consumes the current state and produces the next state. This design allows the ability to replay states forwards or backwards, facilitating debugging of programs executed on the platform.

The emulator's internal state is modeled after the OpenHW CVE2 2-stage pipelined RISC-V processor. By reviewing the top-level SystemVerilog file, we extracted the CVE2's datapath and pipeline components to enable cycle-level debugging.

## RISC-V Assembler
The assembler is long yet simple and integrates with the frontend to take in assembly code as input, producing machine code output for the emulator.

### Key Features:
- The assembler operates on a two-pass system:
  1. **First Pass**: Looks at labels and memory sections (e.g., `.data` and `.text`), associating labels with memory addresses.
  2. **Second Pass**: Generates the actual machine code.

- The main function, `assemble`, handles the assembly string and coordinates the parsing passes.

## Known Bugs
- Users cannot continuously run programs, they must step through the program by clock cycles.

## Development

1. Install npm: https://docs.npmjs.com/downloading-and-installing-node-js-and-npm
2. Install the tailwind css cli: https://tailwindcss.com/docs/installation
3. Install Rust and Cargo: https://doc.rust-lang.org/cargo/getting-started/installation.html 
4. Install the Dioxus CLI: `cargo install dioxus-cli`
5. Then run these two commands in parallel on the root director of the project
  - Start Tailwind compiler: `npm run tailwind-dev`
  - Start Dioxus developement server: `dx serve`
6. Open the browser to http://localhost:8080
