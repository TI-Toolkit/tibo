use crate::Tokens;
use deku::prelude::*;
use chrono::prelude::*;

pub const DATA_COMMENT_TOOL_ID: u8 = 0x72;

#[derive(Debug, DekuRead, DekuWrite)]
#[deku(id_type = "u8")]
#[deku(endian = "endian", ctx = "endian: deku::ctx::Endian")]
pub enum Comment {
    #[deku(id = 0xB8)]
    DataComment(DataComment),
    #[deku(id_pat = "_")]
    Bytes(u8, [u8; 41]),
}

#[derive(Debug, DekuRead, DekuWrite)]
#[deku(endian = "endian", ctx = "endian: deku::ctx::Endian")]
pub struct DataComment {
    #[deku(assert = "*magic == 0xB8")]
    magic: u8,
    pub tool_id: u8,
    pub version: [u8; 9],
    export_day: u8,
    export_month: u8,
    export_year_top: u8, // endianness in enum variants sucks
    export_year_bottom: u8,
    #[deku(update = "self.author.len()")]
    author_len: u8,
    #[deku(assert = "author.len() <= 24", count = "author_len")]
    author: Vec<u8>,
    #[deku(assert = "rest.len() + author.len() <= 24", count = "24-author_len")]
    rest: Vec<u8>,
}

#[derive(Debug, DekuRead, DekuWrite)]
#[deku(endian = "little", magic = b"**TI83")]
pub struct TIProgram {
    further_magic: [u8; 5],
    comment: Comment,
    #[deku(update = "self.data.len() + 17")]
    data_length: u16,
    #[deku(assert = "*flash_indicator == 0x0b || *flash_indicator == 0x0d")]
    flash_indicator: u16,
    #[deku(update = "self.data.len() + 2")]
    var_data_length: u16,
    #[deku(assert = "*file_type == 0x05 || *file_type == 0x06")]
    file_type: u8,
    pub name: [u8; 8],
    version: u8,
    flags: u8,
    #[deku(update = "self.data.len() + 2")]
    var_data_length_2: u16,
    #[deku(update = "self.data.len()")]
    token_data_length: u16,
    #[deku(count = "token_data_length")]
    data: Vec<u8>,
    #[deku(update = "self.checksum()")]
    checksum: u16,
}

impl TIProgram {
    pub fn force_data_comment(&mut self, tool_id: Option<u8>) {
        match &self.comment {
            Comment::DataComment(x) => {},

            Comment::Bytes(first, data) => {
                let local: DateTime<Local> = Local::now();
                let day = local.day() as u8;
                let month = local.month() as u8;
                let year = local.year() as u16; // if this crashes... are you trying to put an 8xp on the antikythera mechanism?

                self.comment = Comment::DataComment(DataComment {
                    magic: 0xB8,
                    tool_id: tool_id.unwrap_or(DATA_COMMENT_TOOL_ID),
                    version: [0; 9],
                    export_day: ((day/10) << 4) + (day%10),
                    export_month: ((month/10) << 4) + (month%10),
                    export_year_top: (((year/1000) << 4) + ((year%1000)/100)) as u8,
                    export_year_bottom: ((((year%100)/10) << 4) + (year%10)) as u8,
                    author_len: 0,
                    author: vec![],
                    rest: vec![],
                })
            },
        }
    }

    pub fn read_tokens(&self) -> Tokens {
        Tokens::from_bytes(&self.data, None)
    }

    pub fn update_tokens(&mut self, tokens: Tokens) {
        self.data = tokens.into();
        self.update().unwrap()
    }

    fn checksum(&self) -> u16 {
        self.token_data_length.wrapping_add(
            self.data
                .iter()
                .fold(0u16, |a, &x| a.wrapping_add(x as u16)),
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::ti_connect_file::TIProgram;

    use super::*;

    #[test]
    fn works() {
        let data = include_bytes!("./test/TEST.8xp");
        let mut x = TIProgram::from_bytes((data.as_ref(), 0)).unwrap().1;

        x.update_tokens(Tokens::from_bytes(&[0x31, 0x32], None));

        assert_eq!(x.checksum, 101);
    }
}
