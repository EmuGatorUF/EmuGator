use std::fmt::Display;

// UART Module for EmuGator

// Fun Fact: John Uart created the UART protocol in ⧫︎♒︎♋︎■︎🙵⬧︎ ♐︎□︎❒︎ ●︎□︎□︎🙵♓︎■︎♑︎✏︎
// RS - Looks like the baud rate wasn't set correctly

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct Uart {
    pub uart_cycle_count: u32,

    tx_buffer: u8,
    rx_buffer: u8,
    lsr: u8,
    tx_delay: u32,
    rx_delay: u32,
    rx_cursor: usize,

    output_buffer: Vec<u8>,
    input_buffer: Vec<u8>,
}

impl Uart {
    pub fn new(uart_cycle_count: u32) -> Self {
        Self {
            uart_cycle_count,

            tx_buffer: 0u8,
            rx_buffer: 0u8,
            lsr: LSRBitmask::ReceiveComplete as u8
                | LSRBitmask::TransmitComplete as u8
                | LSRBitmask::DataRegisterEmpty as u8,

            tx_delay: 0,
            rx_delay: 0,
            rx_cursor: 0,

            output_buffer: vec![],
            input_buffer: vec![].into(),
        }
    }

    pub fn lsr(&self) -> u8 {
        self.lsr
    }

    pub fn tx_write(&mut self, data: u8) {
        self.lsr &= !(LSRBitmask::DataRegisterEmpty as u8);
        self.tx_buffer = data;
    }

    pub fn rx_read(&mut self) -> u8 {
        self.lsr &= !(LSRBitmask::ReceiveComplete as u8);
        self.rx_buffer
    }

    pub fn rx_peek(&self) -> u8 {
        self.rx_buffer
    }

    pub fn get_output(&self) -> &[u8] {
        &self.output_buffer
    }

    pub fn get_input(&self) -> &[u8] {
        &self.input_buffer
    }

    pub fn set_input(&mut self, data: &[u8]) {
        self.input_buffer.clear();
        self.input_buffer.extend(data);
    }

    pub fn get_cursor(&self) -> usize {
        self.rx_cursor
    }

    pub fn clock(&self) -> Self {
        let mut next_uart = self.clone();

        next_uart.tx_delay = self.tx_delay.saturating_sub(1);
        next_uart.rx_delay = self.rx_delay.saturating_sub(1);
        if self.tx_delay == 0 {
            if self.lsr & LSRBitmask::DataRegisterEmpty as u8 == 0 {
                // If data has been written to the UART,
                // Move the byte into the "transmit shift register"
                next_uart.output_buffer.push(next_uart.tx_buffer);
                next_uart.tx_buffer = 0;
                next_uart.lsr |= LSRBitmask::DataRegisterEmpty as u8; // Data Register is empty now
                next_uart.lsr &= !(LSRBitmask::TransmitComplete as u8); // Transmit shift register is non-empty
                next_uart.tx_delay = next_uart.uart_cycle_count;
            } else {
                // If there is no data to transmit, set the TransmitComplete bit
                next_uart.lsr |= LSRBitmask::TransmitComplete as u8;
            }
        }

        if self.rx_delay == 0 {
            if self.lsr & LSRBitmask::ReceiveComplete as u8 == 0 {
                // If data has been read, move the byte into the rx_buffer"
                if let Some(&data) = next_uart.input_buffer.get(self.rx_cursor) {
                    next_uart.rx_buffer = data;
                    next_uart.rx_cursor += 1;
                    next_uart.lsr |= LSRBitmask::ReceiveComplete as u8; // Receive buffer has new data
                    next_uart.rx_delay = next_uart.uart_cycle_count;
                }
            }
        }

        next_uart
    }
}

impl Default for Uart {
    fn default() -> Self {
        Uart::new(20)
    }
}

impl Display for Uart {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(std::str::from_utf8(&self.output_buffer).unwrap_or("Invalid UTF-8"))
    }
}

#[allow(dead_code)]
/// Bitmask enum for the Line Status Register
pub enum LSRBitmask {
    /// '1' if the rx_buffer has unread contents
    ReceiveComplete = 1 << 0,
    /// ''' if the tx_buffer is ready to accept data
    DataRegisterEmpty = 1 << 2,
    /// '1' if the transmission is complete
    TransmitComplete = 1 << 3,
    Error = 1 << 7, // Probably not used
}
