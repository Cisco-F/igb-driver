use dma_api::{DVec, Direction};
use log::debug;

use crate::{descriptor::{AdvRxDesc, AdvTxDesc, Descriptor}, err::IgbError, regs::Reg};

pub const DEFAULT_RING_SIZE: usize = 256;

pub struct Ring<D: Descriptor> {
    pub descriptors: DVec<D>,
    reg: Reg,
}

impl<D: Descriptor> Ring<D> {
    pub fn new(reg: Reg, size: usize) -> Result<Self, IgbError> {
        let descriptors =
            DVec::zeros(size, 4096, Direction::Bidirectional).ok_or(IgbError::NoMemory)?;

        Ok(Self { descriptors, reg })
    }

    pub fn init(&mut self) {
        let ptr = self.descriptors.as_ptr() as *mut D;
        let len = self.descriptors.len();
        let descriptors = unsafe { core::slice::from_raw_parts_mut(ptr, len) };

        let mut i = 1;
        for des in descriptors.iter_mut() {
            let buffer: DVec<u8> = DVec::zeros(1, 4096, Direction::Bidirectional)
                .expect("No Memory left");
            let addr = buffer.bus_addr() as u64;

            if des.as_any().is::<AdvRxDesc>() {
                debug!("NO: {i}, type: rx");
            } else if des.as_any().is::<AdvTxDesc>() {
                debug!("NO: {i}, type: tx");
            }

            // 修改描述符
            des.set_addr(addr);
            i += 1;
        }
    }
}
