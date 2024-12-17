use core::{ptr::NonNull, time::Duration};

use crate::{
    descriptor::{AdvRxDesc, AdvTxDesc},
    err::IgbError,
    phy::Phy,
    regs::{Reg, CTRL, CTRL_EXT, EIMS, RCTL, STATUS, TCTL},
    ring::{Ring, DEFAULT_RING_SIZE},
};

pub struct Igb {
    reg: Reg,
    tx_ring: Ring<AdvTxDesc>,
    rx_ring: Ring<AdvRxDesc>,
    phy: Phy,
}

impl Igb {
    pub fn new(bar0: NonNull<u8>) -> Result<Self, IgbError> {
        let reg = Reg::new(bar0);
        let tx_ring = Ring::new(reg, DEFAULT_RING_SIZE)?;
        let rx_ring = Ring::new(reg, DEFAULT_RING_SIZE)?;

        Ok(Self {
            reg,
            tx_ring,
            rx_ring,
            phy: Phy::new(reg),
        })
    }

    pub fn open(&mut self) -> Result<(), IgbError> {
        self.reg.disable_interrupts();

        self.reg.write_reg(CTRL::RST);

        self.reg.wait_for(
            |reg: CTRL| !reg.contains(CTRL::RST),
            Duration::from_millis(1),
            Some(1000),
        )?;
        self.reg.disable_interrupts();

        self.reg
            .modify_reg(|reg: CTRL_EXT| CTRL_EXT::DRV_LOAD | reg);

        self.setup_phy_and_the_link()?;

        self.init_stat();

        self.init_rx();
        self.init_tx();

        self.enable_interrupts();

        self.reg
            .write_reg(CTRL::SLU | CTRL::FD | CTRL::SPD_1000 | CTRL::FRCDPX | CTRL::FRCSPD);

        Ok(())
    }

    ///  Initialize all statistical counters
    fn init_stat(&mut self) {
        //TODO
    }
    /// 4.5.9 Receive Initialization
    fn init_rx(&mut self) {
        // disable rx when configing.
        self.reg.write_reg(RCTL::empty());

        self.rx_ring.init();

        self.reg.write_reg(RCTL::RXEN | RCTL::SZ_4096);
    }

    fn init_tx(&mut self) {
        self.reg.write_reg(TCTL::empty());

        self.tx_ring.init();

        self.reg.write_reg(TCTL::EN);
    }

    fn setup_phy_and_the_link(&mut self) -> Result<(), IgbError> {
        self.phy.power_up()?;
        Ok(())
    }

    pub fn mac(&self) -> [u8; 6] {
        self.reg.read_mac()
    }

    fn enable_interrupts(&self) {
        self.reg.write_32(EIMS, u32::MAX);
    }

    pub fn status(&self) -> IgbStatus {
        let raw = self.reg.read_reg::<STATUS>();
        let speed_raw = (raw.bits() >> 6) & 0b11;

        IgbStatus {
            link_up: raw.contains(STATUS::LU),
            speed: match speed_raw {
                0 => Speed::Mb10,
                1 => Speed::Mb100,
                0b10 => Speed::Mb1000,
                _ => Speed::Mb1000,
            },
            full_duplex: raw.contains(STATUS::FD),
            phy_reset_asserted: raw.contains(STATUS::PHYRA),
        }
    }
}

#[derive(Debug, Clone)]
pub struct IgbStatus {
    pub full_duplex: bool,
    pub link_up: bool,
    pub speed: Speed,
    pub phy_reset_asserted: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Speed {
    Mb10,
    Mb100,
    Mb1000,
}
