use log::{debug, error};

use crate::{
    err::IgbError,
    regs::{Reg, SwFwSync, MDIC, SWSM},
};

const PHY_CONTROL: u32 = 0;
const MII_CR_POWER_DOWN: u16 = 0x0800;

pub struct Phy {
    reg: Reg,
    addr: u32,
}

impl Phy {
    pub fn new(reg: Reg) -> Self {
        let mdic = reg.read_reg::<MDIC>();
        let addr = (mdic.bits() & MDIC::PHYADD.bits()) >> 21;
        debug!("phy addr: {}", addr);
        Self { reg, addr: 1 }
    }

    pub fn read_mdic(&self, offset: u32) -> Result<u16, IgbError> {
        let mut mdic = MDIC::from_bits_retain((offset << 16) | (self.addr << 21)) | MDIC::OP_READ;
        self.reg.write_reg(mdic);

        loop {
            mdic = self.reg.read_reg::<MDIC>();
            if mdic.contains(MDIC::READY) {
                break;
            }
            if mdic.contains(MDIC::E) {
                error!("MDIC read error");
                return Err(IgbError::Unknown);
            }
        }

        Ok(mdic.bits() as u16)
    }

    pub fn write_mdic(&self, offset: u32, data: u16) -> Result<(), IgbError> {
        let mut mdic = MDIC::from_bits_retain((offset << 16) | (self.addr << 21) | (data as u32))
            | MDIC::OP_WRITE;
        self.reg.write_reg(mdic);

        loop {
            mdic = self.reg.read_reg::<MDIC>();
            if mdic.contains(MDIC::READY) {
                break;
            }
            if mdic.contains(MDIC::E) {
                error!("MDIC read error");
                return Err(IgbError::Unknown);
            }
        }

        Ok(())
    }

    pub fn aquire_sync(&self, mask: u16) -> Result<Synced, IgbError> {
        Synced::new(self.reg, mask)
    }

    pub fn power_up(&self) -> Result<(), IgbError> {
        let mut mii_reg = self.read_mdic(PHY_CONTROL)?;
        mii_reg &= !MII_CR_POWER_DOWN;
        self.write_mdic(PHY_CONTROL, mii_reg)
    }

    pub fn configure(&mut self) {
        let status = self.read_mdic(1).unwrap();
        debug!("status: {status:b}");

        let mut pctrl = self.read_mdic(0).unwrap();
        debug!("{pctrl:b}");
        pctrl |= 1 << 9;
        let _ = self.write_mdic(0, pctrl);

        let status = self.read_mdic(0).unwrap();
        debug!("status: {status:b}");
    }
}

pub struct Synced {
    reg: Reg,
    mask: u16,
}

impl Synced {
    pub fn new(reg: Reg, mask: u16) -> Result<Self, IgbError> {
        let semaphore = Semaphore::new(reg)?;
        let swmask = mask as u32;
        let fwmask = (mask as u32) << 16;
        let mut swfw_sync = SwFwSync::empty();
        loop {
            swfw_sync = reg.read_reg::<SwFwSync>();
            if (swfw_sync.bits() & (swmask | fwmask)) == 0 {
                break;
            }
        }

        swfw_sync |= SwFwSync::from_bits_retain(swmask);
        reg.write_reg(swfw_sync);

        drop(semaphore);
        Ok(Self { reg, mask })
    }
}

impl Drop for Synced {
    fn drop(&mut self) {
        let semaphore = Semaphore::new(self.reg).unwrap();
        let mask = self.mask as u32;
        self.reg.modify_reg(|mut swfw_sync: SwFwSync| {
            swfw_sync.remove(SwFwSync::from_bits_retain(mask));
            swfw_sync
        });

        drop(semaphore);
    }
}

pub struct Semaphore {
    reg: Reg,
}

impl Semaphore {
    pub fn new(reg: Reg) -> Result<Self, IgbError> {
        loop {
            let swsm = reg.read_reg::<SWSM>();

            reg.write_reg(swsm | SWSM::SWESMBI);

            if reg.read_reg::<SWSM>().contains(SWSM::SWESMBI) {
                break;
            }
        }

        Ok(Self { reg })
    }
}

impl Drop for Semaphore {
    fn drop(&mut self) {
        self.reg.modify_reg(|mut reg: SWSM| {
            reg.remove(SWSM::SMBI);
            reg.remove(SWSM::SWESMBI);
            reg
        });
    }
}
