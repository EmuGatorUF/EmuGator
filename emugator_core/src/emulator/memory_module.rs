use std::collections::BTreeMap;

use super::uart::Uart;

#[derive(Clone, Debug)]
pub struct MemoryMappedIO {
    ram: BTreeMap<u32, u8>,
    uart_address: u32,
    uart: Uart,
}

impl MemoryMappedIO {
    pub fn new(initial: &BTreeMap<u32, u8>, uart_address: u32) -> Self {
        let ram = initial.clone();
        let uart = Uart::new(20);

        // insert uart addresses
        MemoryMappedIO {
            ram,
            uart_address,
            uart,
        }
    }

    pub fn get(&mut self, address: u32) -> u8 {
        if address == self.uart_address {
            self.uart.rx_read()
        } else if address == self.uart_address + 4 {
            self.uart.lsr()
        } else {
            *self.ram.get(&address).unwrap_or(&0)
        }
    }

    pub fn set(&mut self, address: u32, value: u8) {
        if address == self.uart_address {
            self.uart.tx_write(value);
        }
        self.ram.insert(address, value);
    }

    /// View memory without side effects (used by the UI)
    pub fn preview(&self, address: u32) -> u8 {
        if address == self.uart_address {
            self.uart.rx_peek()
        } else if address == self.uart_address + 4 {
            self.uart.lsr()
        } else {
            *self.ram.get(&address).unwrap_or(&0)
        }
    }

    pub fn len(&self) -> usize {
        self.ram.len()
    }

    pub fn write_word(&mut self, address: u32, value: u32, byte_enable: [bool; 4]) {
        let bytes = value.to_le_bytes();
        for i in 0..4 {
            if byte_enable[i] {
                self.set(address + i as u32, bytes[i]);
            }
        }
    }

    pub fn read_word(&mut self, address: u32, byte_enable: [bool; 4]) -> u32 {
        let mut bytes = [0; 4];
        for i in 0..4 {
            if byte_enable[i] {
                bytes[i] = self.get(address + i as u32);
            }
        }
        u32::from_le_bytes(bytes)
    }

    pub fn set_serial_input(&mut self, data: &[u8]) {
        self.uart.set_input(data);
    }

    pub fn get_serial_input(&self) -> &[u8] {
        self.uart.get_input()
    }

    pub fn get_serial_cursor(&self) -> usize {
        self.uart.get_cursor()
    }

    pub fn get_serial_output(&self) -> &[u8] {
        self.uart.get_output()
    }

    pub fn clock(&mut self) {
        self.uart = self.uart.clock();
    }
}
