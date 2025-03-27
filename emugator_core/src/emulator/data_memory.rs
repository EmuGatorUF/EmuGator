use std::collections::BTreeMap;

use super::uart::{LineStatusRegisterBitMask, Uart};

#[derive(Clone, Debug)]
pub struct DataMemory {
    mem: BTreeMap<u32, u8>,
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

        DataMemory { mem }
    }

    pub fn get(&self, address: u32) -> u8 {
        *self.mem.get(&address).unwrap_or(&0)
    }

    pub fn set(&mut self, address: u32, value: u8) {
        self.mem.insert(address, value);
    }

    pub fn len(&self) -> usize {
        self.mem.len()
    }
}
