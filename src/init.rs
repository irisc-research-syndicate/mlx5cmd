use pci_driver::{
    pci_struct,
    regions::structured::{PciRegisterRo, PciRegisterRw},
};

pci_struct! {
    pub struct InitSegment<'a> : 0x1000 {
        fw_rev_major        @ 0x0000 : PciRegisterRo<'a, u16>,
        fw_rev_minor        @ 0x0002 : PciRegisterRo<'a, u16>,
        fw_rev_subminor     @ 0x0004 : PciRegisterRo<'a, u16>,
        cmd_interface_rev   @ 0x0006 : PciRegisterRo<'a, u16>,
        cmdq_phy_addr_hi    @ 0x0010 : PciRegisterRw<'a, u32>,
        cmdq_phy_addr_lo    @ 0x0014 : PciRegisterRw<'a, u32>,
        cmdq_doorbell       @ 0x0018 : PciRegisterRw<'a, u32>,
        initializing        @ 0x01fc : PciRegisterRo<'a, u32>,
        internal_timer_hi   @ 0x1000 : PciRegisterRo<'a, u32>,
        internal_timer_lo   @ 0x1004 : PciRegisterRo<'a, u32>,
        clear_interrupt     @ 0x100c : PciRegisterRo<'a, u32>,
        health_syndrom      @ 0x1010 : PciRegisterRo<'a, u32>,
    }
}
