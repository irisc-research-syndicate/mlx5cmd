use std::ffi::CStr;

use deku::ctx::Endian;
use deku::prelude::*;

pub trait Command: DekuContainerWrite {
    type Output: for<'a> DekuContainerRead<'a> + CommandOutput;

    fn size(&self) -> usize;
    fn outlen(&self) -> usize;
}

pub trait CommandOutput {
    fn status(&self) -> u8;
    fn syndrome(&self) -> u32;
}

macro_rules! impl_command_output {
    ($ty: ty) => {
        impl CommandOutput for $ty {
            fn status(&self) -> u8 {
                self.base.status
            }

            fn syndrome(&self) -> u32 {
                self.base.syndrome
            }
        }
    };
}

#[derive(Debug, PartialEq, DekuRead, DekuWrite)]
#[deku(endian = "ctx_endian", ctx = "ctx_endian: Endian")]
pub struct BaseOutput {
    #[deku(pad_bytes_after = "3")]
    pub status: u8,

    pub syndrome: u32,
}

#[derive(Debug, PartialEq, DekuRead, DekuWrite)]
#[deku(endian = "big", magic = b"\x01\x07")]
pub struct QueryPages {
    #[deku(pad_bytes_before = "4", pad_bytes_after = "8")]
    pub op_mod: QueryPagesOpMod,
}

#[derive(Debug, PartialEq, DekuRead, DekuWrite)]
#[deku(type = "u16", endian = "ctx_endian", ctx = "ctx_endian: Endian")]
pub enum QueryPagesOpMod {
    BootPages = 0x1,
    InitPages = 0x2,
    RegularPages = 0x3,
}

#[derive(Debug, PartialEq, DekuRead, DekuWrite)]
#[deku(endian = "big")]
pub struct QueryPagesOutput {
    #[deku(pad_bytes_after = "4")]
    pub base: BaseOutput,
    pub num_pages: i32,
}

impl_command_output!(QueryPagesOutput);

impl Command for QueryPages {
    type Output = QueryPagesOutput;

    fn size(&self) -> usize {
        0x10
    }

    fn outlen(&self) -> usize {
        0x10
    }
}

#[derive(Debug, PartialEq, DekuRead, DekuWrite)]
#[deku(endian = "big", magic = b"\x01\x08")]
pub struct ManagePages {
    #[deku(pad_bytes_before = "4", pad_bytes_after = "4")]
    pub op_mod: ManagePagesOpMod,

    pub input_num_entries: u32,

    #[deku(count = "input_num_entries")]
    pub items: Vec<u64>,
}

#[derive(Debug, PartialEq, DekuRead, DekuWrite)]
#[deku(endian = "big")]
pub struct ManagePagesOutput {
    pub base: BaseOutput,

    #[deku(pad_bytes_after = "4")]
    pub output_num_entries: u32,

    #[deku(count = "output_num_entries")]
    pub items: Vec<u64>,
}

impl_command_output!(ManagePagesOutput);

impl Command for ManagePages {
    type Output = ManagePagesOutput;

    fn size(&self) -> usize {
        0x10 + self.items.len() * 8
    }

    fn outlen(&self) -> usize {
        0x10 + self.items.len() * 8
    }
}

#[derive(Debug, PartialEq, DekuRead, DekuWrite)]
#[deku(type = "u16", endian = "ctx_endian", ctx = "ctx_endian: Endian")]
pub enum ManagePagesOpMod {
    AllocationFail = 0x0,
    AllocationSuccess = 0x1,
    HCAReturnPages = 0x2,
}

#[derive(Debug, PartialEq, DekuRead, DekuWrite)]
#[deku(endian = "big", magic = b"\x01\x0d")]
pub struct SetDriverVersion {
    #[deku(pad_bytes_before = "14")]
    pub driver_version: [u8; 64],
}

#[derive(Debug, PartialEq, DekuRead, DekuWrite)]
#[deku(endian = "big")]
pub struct SetDriverVersionOutput {
    #[deku(pad_bytes_after = "4")]
    pub base: BaseOutput,
}

impl_command_output!(SetDriverVersionOutput);

impl Command for SetDriverVersion {
    type Output = SetDriverVersionOutput;

    fn size(&self) -> usize {
        0x50
    }

    fn outlen(&self) -> usize {
        0x10
    }
}

#[derive(Debug, PartialEq, DekuRead, DekuWrite)]
#[deku(endian = "big", magic = b"\x01\x02\0\0\0\0\0\0\0\0\0\0\0\0\0\0")]
pub struct InitHCA(pub ());

#[derive(Debug, PartialEq, DekuRead, DekuWrite)]
#[deku(endian = "big")]
pub struct InitHCAOutput {
    #[deku(pad_bytes_after = "4")]
    pub base: BaseOutput,
}
impl_command_output!(InitHCAOutput);

impl Command for InitHCA {
    type Output = InitHCAOutput;

    fn size(&self) -> usize {
        0x10
    }

    fn outlen(&self) -> usize {
        0x10
    }
}

#[derive(Debug, PartialEq, DekuRead, DekuWrite)]
#[deku(endian = "big", magic = b"\x01\x04\0\0\0\0\0\0\0\0\0\0\0\0\0\0")]
pub struct EnableHCA(pub ());

#[derive(Debug, PartialEq, DekuRead, DekuWrite)]
#[deku(endian = "big")]
pub struct EnableHCAOutput {
    #[deku(pad_bytes_after = "4")]
    pub base: BaseOutput,
}
impl_command_output!(EnableHCAOutput);

impl Command for EnableHCA {
    type Output = EnableHCAOutput;

    fn size(&self) -> usize {
        0x10
    }

    fn outlen(&self) -> usize {
        0x10
    }
}

#[derive(Debug, PartialEq, DekuRead, DekuWrite)]
#[deku(endian = "big", magic = b"\x01\x05\0\0\0\0\0\0\0\0\0\0\0\0\0\0")]
pub struct DisableHCA(pub ());

#[derive(Debug, PartialEq, DekuRead, DekuWrite)]
#[deku(endian = "big")]
pub struct DisableHCAOutput {
    #[deku(pad_bytes_after = "4")]
    pub base: BaseOutput,
}

impl_command_output!(DisableHCAOutput);

impl Command for DisableHCA {
    type Output = DisableHCAOutput;

    fn size(&self) -> usize {
        0x10
    }

    fn outlen(&self) -> usize {
        0x10
    }
}

#[derive(Debug, PartialEq, DekuRead, DekuWrite)]
#[deku(endian = "big", magic = b"\x01\x0a\0\0\0\0\0\0\0\0\0\0\0\0\0\0")]
pub struct QueryISSI(pub ());

#[derive(Debug, PartialEq, DekuRead, DekuWrite)]
#[deku(endian = "big")]
pub struct QueryISSIOutput {
    #[deku(pad_bytes_after = "2")]
    pub base: BaseOutput,

    #[deku(pad_bytes_after = "20")]
    pub current_issi: u16,

    pub supported_issi: [u8; 0x50],
}

impl_command_output!(QueryISSIOutput);

impl Command for QueryISSI {
    type Output = QueryISSIOutput;

    fn size(&self) -> usize {
        0x10
    }

    fn outlen(&self) -> usize {
        0x70
    }
}

#[derive(Debug, PartialEq, DekuRead, DekuWrite)]
#[deku(endian = "big", magic = b"\x01\x01\0\0\0\0\0\0\0\0\0\0\0\0\0\0")]
pub struct QueryAdapter(pub ());

#[derive(Debug, PartialEq, DekuRead, DekuWrite)]
#[deku(endian = "big")]
pub struct QueryAdapterOutput {
    // why are we off by 4 here????
    #[deku(pad_bytes_after = "8")]
    pub base: BaseOutput,

    pub query_adapter: QueryAdapterStruct,
}

impl_command_output!(QueryAdapterOutput);

impl Command for QueryAdapter {
    type Output = QueryAdapterOutput;

    fn size(&self) -> usize {
        0x10
    }

    fn outlen(&self) -> usize {
        0x110
    }
}

#[derive(Debug, PartialEq, DekuRead, DekuWrite)]
#[deku(endian = "ctx_endian", ctx = "ctx_endian: Endian")]
pub struct QueryAdapterStruct {
    #[deku(pad_bytes_before = "0x19", pad_bytes_after = "2", bytes = "3")]
    pub ieee_vendor_id: u32,
    pub vsd_vendor_id: u16,
    pub vsd: [u8; 208],
    pub vsd_contd_psid: [u8; 16],
}

#[derive(Debug, PartialEq, DekuRead, DekuWrite)]
#[deku(endian = "big", magic = b"\x01\x0b")]
pub struct SetISSI {
    #[deku(pad_bytes_before = "8", pad_bytes_after = "4")]
    pub current_issi: u16,
}
impl_command_output!(SetISSIOutput);

impl Command for SetISSI {
    type Output = SetISSIOutput;

    fn size(&self) -> usize {
        0x10
    }

    fn outlen(&self) -> usize {
        0x10
    }
}

#[derive(Debug, PartialEq, DekuRead, DekuWrite)]
#[deku(endian = "big")]
pub struct SetISSIOutput {
    #[deku(pad_bytes_after = "8")]
    pub base: BaseOutput,
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use super::*;

    #[test]
    fn test_query_pages() {
        let cmd = QueryPages {
            op_mod: QueryPagesOpMod::BootPages,
        };

        let res = cmd.to_bytes().unwrap();

        assert_eq!(res.len(), cmd.size());
        assert_eq!(
            res,
            &[0x01, 0x07, 0x0, 0x0, 0x0, 0x0, 0x0, 0x1, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0]
        );

        let output: &[u8] = &[
            0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x20, 0x22,
        ];

        assert_eq!(output.len(), cmd.outlen());

        assert_eq!(
            QueryPagesOutput::try_from(output).unwrap(),
            QueryPagesOutput {
                base: BaseOutput {
                    status: 0,
                    syndrome: 0
                },
                num_pages: 8226,
            }
        );
    }

    #[test]
    fn test_manage_pages() {
        let cmd = ManagePages {
            op_mod: ManagePagesOpMod::AllocationSuccess,
            input_num_entries: 3,
            items: vec![0x12345678, 0x0, u64::MAX],
        };

        let res = cmd.to_bytes().unwrap();
        assert_eq!(res.len(), cmd.size());

        #[rustfmt::skip]
        let cmd_bytes = &[
            0x01, 0x08, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x03,
            0x00, 0x00, 0x00, 0x00, 0x12, 0x34, 0x56, 0x78, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
        ];

        assert_eq!(res, cmd_bytes);

        #[rustfmt::skip]
        let output: &[u8] = &[
            0xab, 0x00, 0x00, 0x00, 0x12, 0x34, 0x56, 0x78, 0x00, 0x00, 0x00, 0x03, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x87, 0x65, 0x43, 0x21, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
        ];

        assert_eq!(output.len(), cmd.outlen());
        assert_eq!(
            ManagePagesOutput::try_from(output).unwrap(),
            ManagePagesOutput {
                base: BaseOutput {
                    status: 0xab,
                    syndrome: 0x12345678,
                },
                output_num_entries: 3,
                items: vec![0x87654321, 0x0, u64::MAX],
            }
        );
    }

    #[test]
    fn test_set_driver_version() {
        let mut cmd = SetDriverVersion {
            driver_version: [0; 64],
        };

        cmd.driver_version
            .as_mut_slice()
            .write(b"test-version\0")
            .unwrap();

        let res = cmd.to_bytes().unwrap();
        assert_eq!(res.len(), cmd.size());

        assert_eq!(
            res,
            &[
                1, 13, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 116, 101, 115, 116, 45, 118, 101,
                114, 115, 105, 111, 110, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0
            ]
        );
    }

    #[test]
    fn test_init_hca() {
        let cmd = InitHCA(());

        let res = cmd.to_bytes().unwrap();

        assert_eq!(res.len(), cmd.size());
        assert_eq!(res, &[0x01, 0x02, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
    }

    #[test]
    fn test_enable_hca() {
        let cmd = EnableHCA(());

        let res = cmd.to_bytes().unwrap();

        assert_eq!(res.len(), cmd.size());
        assert_eq!(res, &[0x01, 0x04, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
    }

    #[test]
    fn test_disable_hca() {
        let cmd = DisableHCA(());

        let res = cmd.to_bytes().unwrap();

        assert_eq!(res.len(), cmd.size());
        assert_eq!(res, &[0x01, 0x05, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
    }

    #[test]
    fn test_query_issi() {
        let cmd = QueryISSI(());

        let res = cmd.to_bytes().unwrap();

        assert_eq!(res.len(), cmd.size());
        assert_eq!(res, &[0x01, 0x0a, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);

        #[rustfmt::skip]
        let output: &[u8] = &[
            0xab, 0x00, 0x00, 0x00, 0x12, 0x34, 0x56, 0x78, 0x00, 0x00, 0xaa, 0xbb, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x0f,
            0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1a, 0x1b, 0x1c, 0x1d, 0x1e, 0x1f,
            0x20, 0x21, 0x22, 0x23, 0x24, 0x25, 0x26, 0x27, 0x28, 0x29, 0x2a, 0x2b, 0x2c, 0x2d, 0x2e, 0x2f,
            0x30, 0x31, 0x32, 0x33, 0x34, 0x35, 0x36, 0x37, 0x38, 0x39, 0x3a, 0x3b, 0x3c, 0x3d, 0x3e, 0x3f,
            0x40, 0x41, 0x42, 0x43, 0x44, 0x45, 0x46, 0x47, 0x48, 0x49, 0x4a, 0x4b, 0x4c, 0x4d, 0x4e, 0x4f,
        ];

        assert_eq!(output.len(), cmd.outlen());

        assert_eq!(
            QueryISSIOutput::try_from(output).unwrap(),
            QueryISSIOutput {
                base: BaseOutput {
                    status: 0xab,
                    syndrome: 0x12345678,
                },
                current_issi: 0xaabb,
                supported_issi: std::array::from_fn(|i| i as u8),
            }
        );
    }

    #[test]
    fn test_query_adapter() {
        let cmd = QueryAdapter(());

        let res = cmd.to_bytes().unwrap();

        assert_eq!(res.len(), cmd.size());
        assert_eq!(res, &[0x01, 0x01, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);

        let query_adapter = QueryAdapterStruct {
            ieee_vendor_id: 0x654321,
            vsd_vendor_id: 0xabcd,
            vsd: std::array::from_fn(|i| i as u8),
            vsd_contd_psid: [b'A'; 16],
        };

        let out = QueryAdapterOutput {
            base: BaseOutput {
                status: 0xab,
                syndrome: 0x12345678,
            },
            query_adapter,
        };

        #[rustfmt::skip]
        let output: &[u8] = &[
            0xab, 0x00, 0x00, 0x00, 0x12, 0x34, 0x56, 0x78, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x65, 0x43, 0x21, 0x00, 0x00, 0xab, 0xcd,
            0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x0f,
            0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1a, 0x1b, 0x1c, 0x1d, 0x1e, 0x1f,
            0x20, 0x21, 0x22, 0x23, 0x24, 0x25, 0x26, 0x27, 0x28, 0x29, 0x2a, 0x2b, 0x2c, 0x2d, 0x2e, 0x2f,
            0x30, 0x31, 0x32, 0x33, 0x34, 0x35, 0x36, 0x37, 0x38, 0x39, 0x3a, 0x3b, 0x3c, 0x3d, 0x3e, 0x3f,
            0x40, 0x41, 0x42, 0x43, 0x44, 0x45, 0x46, 0x47, 0x48, 0x49, 0x4a, 0x4b, 0x4c, 0x4d, 0x4e, 0x4f,
            0x50, 0x51, 0x52, 0x53, 0x54, 0x55, 0x56, 0x57, 0x58, 0x59, 0x5a, 0x5b, 0x5c, 0x5d, 0x5e, 0x5f,
            0x60, 0x61, 0x62, 0x63, 0x64, 0x65, 0x66, 0x67, 0x68, 0x69, 0x6a, 0x6b, 0x6c, 0x6d, 0x6e, 0x6f,
            0x70, 0x71, 0x72, 0x73, 0x74, 0x75, 0x76, 0x77, 0x78, 0x79, 0x7a, 0x7b, 0x7c, 0x7d, 0x7e, 0x7f,
            0x80, 0x81, 0x82, 0x83, 0x84, 0x85, 0x86, 0x87, 0x88, 0x89, 0x8a, 0x8b, 0x8c, 0x8d, 0x8e, 0x8f,
            0x90, 0x91, 0x92, 0x93, 0x94, 0x95, 0x96, 0x97, 0x98, 0x99, 0x9a, 0x9b, 0x9c, 0x9d, 0x9e, 0x9f,
            0xa0, 0xa1, 0xa2, 0xa3, 0xa4, 0xa5, 0xa6, 0xa7, 0xa8, 0xa9, 0xaa, 0xab, 0xac, 0xad, 0xae, 0xaf,
            0xb0, 0xb1, 0xb2, 0xb3, 0xb4, 0xb5, 0xb6, 0xb7, 0xb8, 0xb9, 0xba, 0xbb, 0xbc, 0xbd, 0xbe, 0xbf,
            0xc0, 0xc1, 0xc2, 0xc3, 0xc4, 0xc5, 0xc6, 0xc7, 0xc8, 0xc9, 0xca, 0xcb, 0xcc, 0xcd, 0xce, 0xcf,
            0x41, 0x41, 0x41, 0x41, 0x41, 0x41, 0x41, 0x41, 0x41, 0x41, 0x41, 0x41, 0x41, 0x41, 0x41, 0x41
        ];

        let out_bytes = out.to_bytes().unwrap();

        assert_eq!(output.len(), cmd.outlen());
        assert_eq!(out_bytes, output, "{out_bytes:02x?}");
    }

    #[test]
    fn test_set_issi() {
        let cmd = SetISSI {
            current_issi: 0x1337,
        };

        let res = cmd.to_bytes().unwrap();

        assert_eq!(res.len(), cmd.size());
        #[rustfmt::skip]
        assert_eq!(res, &[
            0x01, 0x0b, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x13, 0x37, 0x00, 0x00, 0x00, 0x00
        ]);

        #[rustfmt::skip]
        let output: &[u8] = &[
            0xab, 0x00, 0x00, 0x00, 0x12, 0x34, 0x56, 0x78, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ];

        assert_eq!(output.len(), cmd.outlen());

        assert_eq!(
            SetISSIOutput::try_from(output).unwrap(),
            SetISSIOutput {
                base: BaseOutput {
                    status: 0xab,
                    syndrome: 0x12345678,
                },
            }
        );
    }
}
