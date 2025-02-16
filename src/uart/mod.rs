use std::collections::BTreeMap;

use dioxus_logger::tracing::info;

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
}

impl Uart {
    pub fn new(
        uart_output_buffer: Vec<u8>,
        rx_buffer_address: u32,
        tx_buffer_address: u32,
        lsr_address: u32,
        uart_delay: u32,
        uart_cycle_count: u32,
    ) -> Self {
        Self {
            uart_output_buffer,
            rx_buffer_address,
            tx_buffer_address,
            uart_delay,
            uart_cycle_count,
            lsr_address,
        }
    }

    pub fn default() -> Self {
        // TODO: Switch to these values once larger immediates are supported
        // Uart::new(vec![], 0x3FF0, 0x3FF4, 0, 0, 0x3FF8)
        Uart::new(vec![], 0xF0, 0xF4, 0, 0, 0xF8)
    }
}

#[allow(dead_code)]
pub enum LineStatusRegisterBitMask {
    ReceiveReady = 1 << 0,
    ReceiveBusy = 1 << 1,
    TransmitReady = 1 << 2,
    TransmitBusy = 1 << 3,
    Error = 1 << 7, // Probably not used
}

pub fn trigger_uart(uart_module: Uart, data_memory: &mut BTreeMap<u32, u8>) -> Uart {
    let mut next_uart = uart_module.clone();
    //
    // Debug print the output buffer
    info!("UART Output Buffer: {:?}", next_uart.uart_output_buffer);
    info!("Data Memory: {:?}", data_memory);

    // Check if Tx buffer is empty
    if data_memory.get(&next_uart.tx_buffer_address).is_none() {
        // Set TransmitReady bit in LSR and return
        data_memory.insert(next_uart.lsr_address, LineStatusRegisterBitMask::TransmitReady as u8);
        return next_uart;
    }

    // Tx buffer is not empty, so we can send a byte if count is zero
    if next_uart.uart_cycle_count == 0 {
        // Get byte from Tx buffer
        let byte = *data_memory.get(&next_uart.tx_buffer_address).unwrap();

        // Set Tx buffer to empty
        data_memory.insert(next_uart.tx_buffer_address, 0);

        // Set TransmitReady bit in LSR
        data_memory.insert(next_uart.lsr_address, LineStatusRegisterBitMask::TransmitReady as u8);

        // Set byte in buffer
        next_uart.uart_output_buffer.push(byte);

        // Set Tx cycle count to delay
        next_uart.uart_cycle_count = next_uart.uart_delay;
    } else {
        // Decrement cycle count
        next_uart.uart_cycle_count -= 1;

        // Set LSR to TransmitBusy
        data_memory.insert(next_uart.lsr_address, LineStatusRegisterBitMask::TransmitBusy as u8);
    }


    next_uart
}
