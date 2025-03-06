use std::{collections::BTreeMap, fmt::Display};

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
        Uart::new(vec![], 0xF0, 0xF4, 0xF8, 20, 0)
    }
}

impl Display for Uart {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(std::str::from_utf8(&self.uart_output_buffer).unwrap_or("Invalid UTF-8"))
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

pub fn trigger_uart(uart_module: &Uart, data_memory: &mut BTreeMap<u32, u8>) -> Uart {
    let mut next_uart = uart_module.clone();

    let the_receive_data = *data_memory.get(&next_uart.rx_buffer_address).unwrap();
    let the_transmit_data = *data_memory.get(&next_uart.tx_buffer_address).unwrap();

    if next_uart.uart_cycle_count > 0 {
        next_uart.uart_cycle_count -= 1;
        return next_uart;
    }

    // Clear the busy bits and set the corresponding ready bits
    let old_lsr: u8 = *data_memory.get(&next_uart.lsr_address).unwrap();
    let new_lsr = (old_lsr
        & !(LineStatusRegisterBitMask::TransmitBusy as u8
            | LineStatusRegisterBitMask::ReceiveBusy as u8))
        | (((old_lsr & LineStatusRegisterBitMask::TransmitBusy as u8) != 0) as u8
            * (LineStatusRegisterBitMask::TransmitReady as u8))
        | (((old_lsr & LineStatusRegisterBitMask::ReceiveBusy as u8) != 0) as u8
            * (LineStatusRegisterBitMask::ReceiveReady as u8));
    data_memory.insert(next_uart.lsr_address, new_lsr);

    if the_transmit_data != 0 {
        // Set Tx buffer to empty
        data_memory.insert(next_uart.tx_buffer_address, 0);

        // Set TransmitBusy bit in LSR
        data_memory.insert(
            next_uart.lsr_address,
            LineStatusRegisterBitMask::TransmitBusy as u8,
        );

        // Set byte in buffer
        next_uart.uart_output_buffer.push(the_transmit_data);

        // Set UART busy for delay cycles
        next_uart.uart_cycle_count = next_uart.uart_delay;
    } else if the_receive_data != 0 {
        todo!();
    }

    next_uart
}
