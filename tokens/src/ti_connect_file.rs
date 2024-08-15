use crate::Tokens;
use deku::prelude::*;

#[derive(Debug, DekuRead, DekuWrite)]
#[deku(endian = "little", magic = b"**TI83")]
pub struct TIProgram {
    further_magic: [u8; 5],
    comment: [u8; 42],
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
    pub fn comment(&self) -> [u8; 42] {
        self.comment
    }

    /// Sets the `comment` to the first 42 chars of the provided string.
    pub fn set_comment(&mut self, comment: &str) {
        let mut vec_data = comment.bytes().take(42).collect::<Vec<u8>>();
        vec_data.resize(42, 0x00);
        self.comment = vec_data.try_into().unwrap();
    }

    pub fn read_tokens(&self) -> Tokens {
        Tokens::from_bytes(&self.data, None)
    }
    pub fn update_tokens(&mut self, tokens: Tokens) {
        self.data = tokens.into();
        self.update().unwrap()
    }

    fn checksum(&self) -> u16 {
        (self.token_data_length as u16).wrapping_add(
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
