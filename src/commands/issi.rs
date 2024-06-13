use deku::prelude::*;

use super::{BaseOutput, Command};

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
#[deku(endian = "big", magic = b"\x01\x0b")]
pub struct SetISSI {
    #[deku(pad_bytes_before = "8", pad_bytes_after = "4")]
    pub current_issi: u16,
}

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
    use super::*;
    use crate::types::CommandErrorStatus;

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
                    status: CommandErrorStatus::UnknownError(0xab),
                    syndrome: 0x12345678,
                },
                current_issi: 0xaabb,
                supported_issi: std::array::from_fn(|i| i as u8),
            }
        );
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
                    status: CommandErrorStatus::UnknownError(0xab),
                    syndrome: 0x12345678,
                },
            }
        );
    }
}
