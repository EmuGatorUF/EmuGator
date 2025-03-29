use std::collections::BTreeMap;

use super::uart::{LineStatusRegisterBitMask, Uart};

#[derive(Clone, Debug)]
pub struct DataMemory {
    mem: BTreeMap<u32, u8>,
    rx_buffer_address: u32,
    tx_buffer_address: u32,
    lsr_address: u32
}

impl DataMemory {
    pub fn new(initial: &BTreeMap<u32, u8>, uart: &Uart) -> Self {
        let mut mem = initial.clone();

        // insert uart addresses
        mem.insert(uart.rx_buffer_address, 0);
        mem.insert(uart.tx_buffer_address, 0);
        mem.insert(
            uart.lsr_address,
            LineStatusRegisterBitMask::TransmitReady as u8
                | LineStatusRegisterBitMask::ReceiveReady as u8,
        );

        DataMemory { mem, rx_buffer_address: uart.rx_buffer_address, tx_buffer_address: uart.tx_buffer_address, lsr_address: uart.lsr_address }
    }

    pub fn get(&mut self, address: u32) -> u8 {
        if address == self.rx_buffer_address {
            // return the data, clear the memory address, and update lsr
            let data = *self.mem.get(&address).unwrap_or(&0);
            self.mem.insert(address, 0);

            let mut lsr = *self.mem.get(&self.lsr_address).unwrap_or(&0);
            lsr ^= LineStatusRegisterBitMask::ReceiveReady as u8;
            lsr |= LineStatusRegisterBitMask::ReceiveBusy as u8;
            self.mem.insert(self.lsr_address, lsr);

            return data;
        }

        *self.mem.get(&address).unwrap_or(&0)
    }

    pub fn preview(&self, address: u32) -> u8 {
        *self.mem.get(&address).unwrap_or(&0)
    }

    pub fn set(&mut self, address: u32, value: u8) {
        self.mem.insert(address, value);
    }

    pub fn len(&self) -> usize {
        self.mem.len()
    }

    pub fn write_word(&mut self, address: u32, value: u32, byte_enable: [bool; 4]) {
        let bytes = value.to_le_bytes();
        for i in 0..4 {
            if byte_enable[i] {
                self.set(address + i as u32, bytes[i]);
            }
        }
    }

    pub fn read_word(&self, address: u32, byte_enable: [bool; 4]) -> u32 {
        let mut bytes = [0; 4];
        for i in 0..4 {
            if byte_enable[i] {
                bytes[i] = self.get(address + i as u32);
            }
        }
        u32::from_le_bytes(bytes)
    }
}
