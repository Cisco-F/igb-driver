use core::{any::TypeId, time::Duration};

use dma_api::{DVec, Direction};
use log::debug;

use crate::{descriptor::{AdvRxDesc, AdvTxDesc, Descriptor}, err::IgbError, regs::{Reg, RXDCTL}};

pub const DEFAULT_RING_SIZE: usize = 256;

pub struct Ring<D: Descriptor + 'static> {
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
            // allocate appropriate size for buffers
            let buffer: DVec<u8> = DVec::zeros(1, 4096, Direction::Bidirectional)
                .expect("No Memory left");
            // store buffer pointer to descriptor
            let addr = buffer.bus_addr() as u64;
            des.set_addr(addr);

            if des.as_any().is::<AdvRxDesc>() {
                debug!("NO: {i}, type: rx");
            } else if des.as_any().is::<AdvTxDesc>() {
                debug!("NO: {i}, type: tx");
            }

            i += 1;
        }

        if TypeId::of::<D>() == TypeId::of::<AdvRxDesc>() {
            // program the descriptor base address with the address of the region
            let base = self.descriptors.bus_addr() as u64;
            let base_addr_low = (base & 0xFFFF_FFF0) as u32;
            let base_addr_high = (base >> 32) as u32;
            self.reg.write_32(0x0C000, base_addr_low);
            self.reg.write_32(0x0C004, base_addr_high);
    
            // Set the length register to the size of the descriptor ring
            let ring_len = (len * core::mem::size_of::<D>()) as u32;
            self.reg.write_32(0x0C008, ring_len & 0xFFFF_FFE0);
    
            // Program SRRCTL
            let mut srrctl = self.reg.read_32(0x0C00C);
            srrctl &= !0x7F;
            srrctl |= 4;
            self.reg.write_32(0x0C00C, srrctl);
    
            // enable rx queue
            self.reg.write_reg::<RXDCTL>(RXDCTL::ENBALE);
            let _ = self.reg.wait_for(
                |reg: RXDCTL| !reg.contains(RXDCTL::ENBALE),
                Duration::from_millis(1),
                Some(1000),
            );
            debug!("rx queue enabled");
        } else if TypeId::of::<D>() == TypeId::of::<AdvTxDesc>() {

        }
    }
}
