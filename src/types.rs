use deku::ctx::Endian;
use deku::prelude::*;

#[derive(Debug, PartialEq, DekuRead, DekuWrite)]
#[deku(endian = "big", magic = b"\x01\x07")]
pub struct QueryPages {
    #[deku(pad_bytes_before = "4", pad_bytes_after = "8")]
    pub op_mod: OpMod,
}

#[derive(Debug, PartialEq, DekuRead, DekuWrite)]
#[deku(type = "u16", endian = "ctx_endian", ctx = "ctx_endian: Endian")]
pub enum OpMod {
    BootPages = 0x1,
    InitPages = 0x2,
    RegularPages = 0x3,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query_pages() {
        let cmd = QueryPages {
            op_mod: OpMod::BootPages,
        };

        let res = cmd.to_bytes().unwrap();

        assert_eq!(res.len(), 0x10);
        assert_eq!(
            res,
            &[0x01, 0x07, 0x0, 0x0, 0x0, 0x0, 0x0, 0x1, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0]
        );
    }
}
