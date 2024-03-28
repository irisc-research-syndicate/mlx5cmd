use std::ffi::CStr;

use deku::ctx::Endian;
use deku::prelude::*;

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
    #[deku(pad_bytes_after = "3")]
    pub status: u8,

    #[deku(pad_bytes_after = "4")]
    pub syndrome: u32,

    pub num_pages: u32,
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
#[deku(endian = "big", magic = b"\x01\x02\0\0\0\0\0\0\0\0\0\0\0\0\0\0")]
pub struct InitHCA(());

#[derive(Debug, PartialEq, DekuRead, DekuWrite)]
#[deku(endian = "big")]
pub struct InitHCAOutput {
    #[deku(pad_bytes_after = "3")]
    pub status: u8,

    #[deku(pad_bytes_after = "4")]
    pub syndrome: u32,
}

#[derive(Debug, PartialEq, DekuRead, DekuWrite)]
#[deku(endian = "big", magic = b"\x01\x04\0\0\0\0\0\0\0\0\0\0\0\0\0\0")]
pub struct EnableHCA(());

#[derive(Debug, PartialEq, DekuRead, DekuWrite)]
#[deku(endian = "big")]
pub struct EnableHCAOutput {
    #[deku(pad_bytes_after = "3")]
    pub status: u8,

    #[deku(pad_bytes_after = "4")]
    pub syndrome: u32,
}

#[derive(Debug, PartialEq, DekuRead, DekuWrite)]
#[deku(endian = "big", magic = b"\x01\x05\0\0\0\0\0\0\0\0\0\0\0\0\0\0")]
pub struct DisableHCA(());

#[derive(Debug, PartialEq, DekuRead, DekuWrite)]
#[deku(endian = "big")]
pub struct DisableHCAOutput {
    #[deku(pad_bytes_after = "3")]
    pub status: u8,

    #[deku(pad_bytes_after = "4")]
    pub syndrome: u32,
}

#[derive(Debug, PartialEq, DekuRead, DekuWrite)]
#[deku(endian = "big", magic = b"\x01\x0a\0\0\0\0\0\0\0\0\0\0\0\0\0\0")]
pub struct QueryISSI(());

#[derive(Debug, PartialEq, DekuRead, DekuWrite)]
#[deku(endian = "big")]
pub struct QueryISSIOutput {
    #[deku(pad_bytes_after = "3")]
    pub status: u8,

    #[deku(pad_bytes_after = "2")]
    pub syndrome: u32,

    #[deku(pad_bytes_after = "20")]
    pub current_issi: u16,

    pub supported_issi: [u8; 0x50],
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

        assert_eq!(res.len(), 0x10);
        assert_eq!(
            res,
            &[0x01, 0x07, 0x0, 0x0, 0x0, 0x0, 0x0, 0x1, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0]
        );

        let output: &[u8] = &[
            0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x20, 0x22,
        ];

        assert_eq!(
            QueryPagesOutput::try_from(output).unwrap(),
            QueryPagesOutput {
                status: 0,
                syndrome: 0,
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

        assert_eq!(res.len(), 0x10 + cmd.input_num_entries as usize * 8);
        assert_eq!(
            res,
            &[
                1, 8, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 0, 18, 52, 86, 120, 0, 0,
                0, 0, 0, 0, 0, 0, 255, 255, 255, 255, 255, 255, 255, 255
            ]
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

        assert_eq!(res.len(), 0x50);
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

        assert_eq!(res.len(), 0x10);
        assert_eq!(res, &[0x01, 0x02, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
    }

    #[test]
    fn test_start_hca() {
        let cmd = EnableHCA(());

        let res = cmd.to_bytes().unwrap();

        assert_eq!(res.len(), 0x10);
        assert_eq!(res, &[0x01, 0x04, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
    }

    #[test]
    fn test_disable_hca() {
        let cmd = DisableHCA(());

        let res = cmd.to_bytes().unwrap();

        assert_eq!(res.len(), 0x10);
        assert_eq!(res, &[0x01, 0x05, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
    }

    #[test]
    fn test_query_issi() {
        let cmd = QueryISSI(());

        let res = cmd.to_bytes().unwrap();

        assert_eq!(res.len(), 0x10);
        assert_eq!(res, &[0x01, 0x0a, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);

        let out = QueryISSIOutput {
            status: 0,
            syndrome: 0,
            current_issi: 0,
            supported_issi: [0; 0x50],
        };
        println!("{}", out.to_bytes().unwrap().len());

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

        assert_eq!(
            QueryISSIOutput::try_from(output).unwrap(),
            QueryISSIOutput {
                status: 0xab,
                syndrome: 0x12345678,
                current_issi: 0xaabb,
                supported_issi: std::array::from_fn(|i| i as u8),
            }
        );
    }
}
