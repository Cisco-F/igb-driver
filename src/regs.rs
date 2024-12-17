#![allow(non_camel_case_types)]

use core::{ptr::NonNull, time::Duration};

use bitflags::{bitflags, Flags};

use crate::{err::IgbError, sleep};

pub const EIMS: u32 = 0x01524;
pub const EIMC: u32 = 0x01528;
pub const EICR: u32 = 0x01580;

#[derive(Clone, Copy)]
pub struct Reg {
    pub addr: NonNull<u8>,
}

impl Reg {
    pub fn new(addr: NonNull<u8>) -> Self {
        Self { addr }
    }

    pub fn read_32(&self, reg: u32) -> u32 {
        unsafe {
            let ptr = self.addr.add(reg as _);
            ptr.cast().read_volatile()
        }
    }

    pub fn write_32(&self, reg: u32, val: u32) {
        unsafe {
            let ptr = self.addr.add(reg as _);
            ptr.cast().write_volatile(val);
        }
    }

    pub fn read_reg<F: FlagReg>(&self) -> F {
        F::from_bits_retain(self.read_32(F::REG))
    }

    pub fn write_reg<F: FlagReg>(&self, val: F) {
        self.write_32(F::REG, val.bits())
    }

    pub fn modify_reg<F: FlagReg>(&self, f: impl Fn(F) -> F) {
        let old = self.read_reg::<F>();
        self.write_reg(f(old));
    }

    pub fn wait_for<R: FlagReg, F: Fn(R) -> bool>(
        &self,
        f: F,
        interval: Duration,
        try_count: Option<usize>,
    ) -> Result<(), IgbError> {
        for _ in 0..try_count.unwrap_or(usize::MAX) {
            if f(self.read_reg::<R>()) {
                return Ok(());
            }

            sleep(interval);
        }
        Err(IgbError::Timeout)
    }

    /// Disable all interrupts for all queues.
    pub fn disable_interrupts(&self) {
        // Clear interrupt mask to stop from interrupts being generated
        self.write_32(EIMS, 0);
        self.clear_interrupts();
    }

    /// Clear all interrupt masks for all queues.
    pub fn clear_interrupts(&self) {
        // Clear interrupt mask
        self.write_32(EIMC, u32::MAX);
        self.read_32(EICR);
    }

    pub fn read_mac(&self) -> [u8; 6] {
        let low = self.read_32(ral(0));
        let high = self.read_32(rah(0));

        [
            (low & 0xff) as u8,
            (low >> 8 & 0xff) as u8,
            (low >> 16 & 0xff) as u8,
            (low >> 24) as u8,
            (high & 0xff) as u8,
            (high >> 8 & 0xff) as u8,
        ]
    }
}

pub trait FlagReg: Flags<Bits = u32> {
    const REG: u32;
}

/* Multicast Table Array - 128 entries */
#[allow(unused)]
fn mta(i: u32) -> u32 {
    0x05200 + i * 4
}

fn ral(i: u32) -> u32 {
    if i <= 15 {
        0x05400 + i * 8
    } else {
        0x0A200 + i * 8
    }
}

fn rah(i: u32) -> u32 {
    if i <= 15 {
        0x05404 + i * 8
    } else {
        0x0A204 + i * 8
    }
}

bitflags! {
    pub struct CTRL: u32 {
        const FD = 0x00000001;  // Full duplex. 0=half; 1=full
        const PRIOR = 0x00000004;  // Priority on PCI. 0=rx,1=fair
        const GIO_PRIMARY_DISABLE = 0x00000004;  // Blocks new Primary reqs
        const LRST = 0x00000008;  // Link reset. 0=normal,1=reset
        const ASDE = 0x00000020;  // Auto-speed detect enable
        const SLU = 0x00000040;  // Set link up (Force Link)
        const ILOS = 0x00000080;  // Invert Loss-Of Signal
        const SPD_SEL = 0x00000300;  // Speed Select Mask
        const SPD_10 = 0x00000000;  // Force 10Mb
        const SPD_100 = 0x00000100;  // Force 100Mb
        const SPD_1000 = 0x00000200;  // Force 1Gb
        const FRCSPD = 0x00000800;  // Force Speed
        const FRCDPX = 0x00001000;  // Force Duplex
        const SWDPIN0 = 0x00040000;  // SWDPIN 0 value
        const SWDPIN1 = 0x00080000;  // SWDPIN 1 value
        const SWDPIN2 = 0x00100000;  // SWDPIN 2 value
        const ADVD3WUC = 0x00100000;  // D3 WUC
        const SWDPIN3 = 0x00200000;  // SWDPIN 3 value
        const SWDPIN0_IO = 0x00400000;  // SWDPIN 0 Input or output
        const DEV_RST = 0x20000000;  // Device reset
        const RST = 0x04000000;  // Global reset
        const RFCE = 0x08000000;  // Receive Flow Control enable
        const TFCE = 0x10000000;  // Transmit flow control enable
        const VME = 0x40000000;  // IEEE VLAN mode enable
        const PHY_RST = 0x80000000;  // PHY Reset
        const I2C_ENA = 0x02000000;  // I2C enable
    }
}

impl FlagReg for CTRL {
    const REG: u32 = 0x00000;
}

bitflags! {
    pub struct STATUS: u32 {
        const FD = 0x00000001;  // Full duplex. 0=half; 1=full
        const LU = 1 << 1;
        const LAN_ID = 1 << 2;
        const TXOFF = 1 << 4;
        const SPEED = 1 << 6;
        const PHYRA = 1 << 10;
    }
}

impl FlagReg for STATUS {
    const REG: u32 = 0x00008;
}

bitflags! {
    pub struct CTRL_EXT: u32 {
        const EE_RST = 1 << 13;
        const DRV_LOAD = 1 << 28;
    }
}

impl FlagReg for CTRL_EXT {
    const REG: u32 = 0x18;
}

bitflags! {
    pub struct RCTL: u32 {
        const RST  = 0x00000001;        // Software reset
        const RXEN = 0x00000002;        // Receiver Enable. 0=disabled; 1=enabled
        const SBP  = 0x00000004;        // Store Bad Packets. 0=do not store; 1=store bad packets
        const UPE  = 0x00000008;        // Unicast Promiscuous Enabled. 0=disabled; 1=enabled
        const MPE  = 0x00000010;        // Multicast Promiscuous Enabled. 0=disabled; 1=enabled
        const LPE  = 0x00000020;        // Long Packet Reception Enable. 0=disabled; 1=enabled
        const LBM_NO = 0;               // Loopback Mode. 00=normal; 01=MAC loopback; 10=undefined; 11=reserved
        const LBM_MAC  = 0x00000040;
        const LBM_TCVR = 0x000000C0;    // Loopback Mode. 00=normal; 01=MAC loopback; 10=undefined; 11=reserved
        const MO_3   = 0x00003000;  // Multicast Offset. 00=bits[47:36]; 01=bits[46:35]; 10=bits[45:34]; 11=bits[43:32]
        const BAM  = 0x00008000;  // Broadcast Accept Mode. 0=ignore; 1=accept broadcast packets

        const SZ_2048   = 0x00000000; /* Rx buffer size 2048 */
        const SZ_1024   = 0x00010000; /* Rx buffer size 1024 */
        const SZ_512    = 0x00020000; /* Rx buffer size 512 */
        const SZ_256    = 0x00030000; /* Rx buffer size 256 */
        /* these buffer sizes are valid if E1000_RCTL_BSEX is 1 */
        const SZ_16384  = 0x00010000; /* Rx buffer size 16384 */
        const SZ_8192	= 0x00020000; /* Rx buffer size 8192 */
        const SZ_4096	= 0x00030000; /* Rx buffer size 4096 */

        const VFE  = 0x00040000;  // VLAN Filter Enable. 0=disabled; 1=enabled
        const CFIEN= 0x00080000;  // Canonical Form Indicator Enable. 0=disabled; 1=enabled
        const CFI  = 0x00100000;  // Canonical Form Indicator bit value. 0=accept; 1=discard
        const PSP  = 0x00200000;  // Pad Small Receive Packets. 0=disabled; 1=enabled
        const DPF  = 0x00400000;  // Discard Pause Frames with Station MAC Address. 0=forward; 1=discard
        const PMCF = 0x00800000;  // Pass MAC Control Frames. 0=pass; 1=filter
        const BSEX = 0x02000000;  // Buffer size extension
        const SECRC= 0x04000000;  // Strip Ethernet CRC from Incoming Packet. 0=do not strip; 1=strip CRC
    }
}

impl FlagReg for RCTL {
    const REG: u32 = 0x00100;
}

bitflags! {
    pub struct TCTL: u32 {
        const EN = 0x00000002;        // Receiver Enable. 0=disabled; 1=enabled
    }
}

impl FlagReg for TCTL {
    const REG: u32 = 0x00400;
}

bitflags! {
    pub struct MDIC: u32 {
        const DATA = 0xFFFF;
        const REGADD = 0b11111 << 16;
        const PHYADD = 0b11111 << 21;
        const OP_WRITE = 0x04000000;
        const OP_READ = 0x08000000;
        const READY = 1 << 28;
        const I = 1 << 29;
        const E = 1 << 30;
        const Destination = 1 << 31;
    }
}

impl FlagReg for MDIC {
    const REG: u32 = 0x00020;
}

bitflags! {
    pub struct SWSM: u32 {
        const SMBI = 1;
        const SWESMBI = 1<<1;
        const WMNG =1<<2;
        const EEUR = 1<<3;
    }
}

impl FlagReg for SWSM {
    const REG: u32 = 0x05B50;
}

bitflags! {
    pub struct SwFwSync: u32 {
        const SW_EEP_SM = 1;
        const SW_PHY_SM0 = 1 << 1;
        const SW_PHY_SM1 = 1 << 2;
    }
}

impl FlagReg for SwFwSync {
    const REG: u32 = 0x05B5C;
}

bitflags! {
    pub struct RXDCTL: u32 {
        const ENBALE = 1 << 25;
    }
}

impl FlagReg for RXDCTL {
    const REG: u32 = 0x0C028;
}

bitflags! {
    pub struct TXDCTL: u32 {
        const ENABLE = 1 << 25;
    }
}

impl FlagReg for TXDCTL {
    const REG: u32 = 0x0E028;
}
