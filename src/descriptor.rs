pub trait Descriptor {
    fn set_addr(&mut self, addr: u64);
}

#[derive(Clone, Copy)]
pub union AdvTxDesc {
    pub read: AdvTxDescRead,
    pub write: AdvTxDescWB,
}

impl Descriptor for AdvTxDesc {
    fn set_addr(&mut self, addr: u64) {
        self.read.buffer_addr = addr;
    }
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct AdvTxDescRead {
    pub buffer_addr: u64,
    pub cmd_type_len: u32,
    pub olinfo_status: u32,
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct AdvTxDescWB {
    pub rsvd: u64,
    pub nxtseq_seed: u32,
    pub status: u32,
}

#[derive(Clone, Copy)]
pub union AdvRxDesc {
    pub read: AdvRxDescRead,
    pub write: AdvRxDescWB,
}

impl Descriptor for AdvRxDesc {
    fn set_addr(&mut self, addr: u64) {
        self.read.pkt_addr = addr;
    }
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct AdvRxDescRead {
    pub pkt_addr: u64,
    pub hdr_addr: u64,
}
#[derive(Clone, Copy)]
#[repr(C)]
pub struct AdvRxDescWB {
    pub lo_dword: LoDword,
    pub hi_dword: HiDword,
    pub status_error: u32,
    pub length: u16,
    pub vlan: u16,
}

#[derive(Clone, Copy)]
pub union LoDword {
    pub data: u32,
    pub hs_rss: HsRss,
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct HsRss {
    pub pkt_info: u16,
    pub hdr_info: u16,
}

#[derive(Clone, Copy)]
#[repr(C)]
pub union HiDword {
    pub rss: u32, // RSS Hash
    pub csum_ip: CsumIp,
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct CsumIp {
    pub ip_id: u16, // IP id
    pub csum: u16,  // Packet Checksum
}
