use std::fmt::Display;

use super::data_memory::DataMemory;

// UART Module for EmuGator

// Fun Fact: John Uart created the UART protocol in ⧫︎♒︎♋︎■︎🙵⬧︎ ♐︎□︎❒︎ ●︎□︎□︎🙵♓︎■︎♑︎✏︎
// RS - Looks like the baud rate wasn't set correctly

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct Uart {
    pub rx_buffer_address: u32,
    pub tx_buffer_address: u32,
    pub lsr_address: u32,
    pub uart_delay: u32,
    pub uart_cycle_count: u32,
    pub uart_output_buffer: Vec<u8>,
    pub uart_input_buffer: String,
    pub characters_read_in: Vec<u8>
}

impl Uart {
    pub fn new(
        uart_output_buffer: Vec<u8>,
        uart_input_buffer: String,
        rx_buffer_address: u32,
        tx_buffer_address: u32,
        lsr_address: u32,
        uart_delay: u32,
        uart_cycle_count: u32,
        characters_read_in: Vec<u8>
    ) -> Self {
        Self {
            uart_output_buffer,
            uart_input_buffer,
            rx_buffer_address,
            tx_buffer_address,
            uart_delay,
            uart_cycle_count,
            lsr_address,
            characters_read_in
        }
    }

    pub fn set_input_string(&mut self, input: &str) {
       self.uart_input_buffer = input.to_string(); 
    }

    pub fn get_characters_read_in(&self) -> String {
        std::str::from_utf8(&self.characters_read_in).unwrap_or("Invalid UTF-8").to_string()
    }
}

impl Default for Uart {
    fn default() -> Self {
        Uart::new(vec![], String::new(), 0xF0, 0xF4, 0xF8, 20, 0, vec![])
    }
}

impl Display for Uart {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(std::str::from_utf8(&self.uart_output_buffer).unwrap_or("Invalid UTF-8"))
    }
}

#[allow(dead_code)]
pub enum LineStatusRegisterBitMask {
    ReceiveReady = 1 << 0, // Register is empty TODO: Changes this
    ReceiveBusy = 1 << 1, // Register is full
    TransmitReady = 1 << 2,
    TransmitBusy = 1 << 3,
    Error = 1 << 7, // Probably not used
}

pub fn trigger_uart(uart_module: &Uart, data_memory: &mut DataMemory) -> Uart {
    let mut next_uart = uart_module.clone();

    // RS - I don't like the clone here
    let the_receive_data = uart_module.uart_input_buffer.clone();
    let the_transmit_data = data_memory.get(next_uart.tx_buffer_address);

    if next_uart.uart_cycle_count > 0 {
        next_uart.uart_cycle_count -= 1;
        return next_uart;
    }

    // Clear the busy bits and set the corresponding ready bits
    let old_lsr: u8 = data_memory.get(next_uart.lsr_address);
    let new_lsr = (old_lsr
        & !(LineStatusRegisterBitMask::TransmitBusy as u8
            | LineStatusRegisterBitMask::ReceiveBusy as u8))
        | (((old_lsr & LineStatusRegisterBitMask::TransmitBusy as u8) != 0) as u8
            * (LineStatusRegisterBitMask::TransmitReady as u8))
        | (((old_lsr & LineStatusRegisterBitMask::ReceiveBusy as u8) != 0) as u8
            * (LineStatusRegisterBitMask::ReceiveReady as u8));
    data_memory.set(next_uart.lsr_address, new_lsr);

    if the_transmit_data != 0 {
        // Set Tx buffer to empty
        data_memory.set(next_uart.tx_buffer_address, 0);

        // Set TransmitBusy bit in LSR
        data_memory.set(
            next_uart.lsr_address,
            LineStatusRegisterBitMask::TransmitBusy as u8,
        );

        // Set byte in buffer
        next_uart.uart_output_buffer.push(the_transmit_data);

        // Set UART busy for delay cycles
        next_uart.uart_cycle_count = next_uart.uart_delay;
    } else if !the_receive_data.is_empty() && data_memory.get(next_uart.rx_buffer_address) == 0 {
        // Input buffer is not empty, so we can take a char and place it in data memory
        let character = if next_uart.characters_read_in.len() < the_receive_data.len() {
            the_receive_data.as_bytes()[next_uart.characters_read_in.len()] as u8
        } else {
            0
        };

        data_memory.set(next_uart.rx_buffer_address, character);

        // Set ReceiveBusy bit in LSR
        data_memory.set(
            next_uart.lsr_address,
            LineStatusRegisterBitMask::ReceiveBusy as u8,
        );

        // Increment characters read
        next_uart.characters_read_in.push(character);

        // Set UART busy for delay cycles
        next_uart.uart_cycle_count = next_uart.uart_delay;
    }

    next_uart
}
