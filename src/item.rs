use std::{fmt::Display, hash::{Hash, Hasher}};

#[derive(Clone, Debug)]
pub struct Item {
    keylen: u32,
    flags: u32,
    exptime: u32,
    valuelen: u32,
    data: Vec<u8>,
}

impl Item {
    pub fn new(key: &str, flags: u32, exptime: u32, valuelen: u32) -> Self {
        let mut data = vec![0u8; key.len() + valuelen as usize];
        data.copy_from_slice(key.as_bytes());
        Item { keylen: key.len() as u32, flags, exptime, valuelen, data }
    }

    pub fn key(&self) -> &[u8] {
        &self.data[0..self.keylen as usize]
    }

    pub fn value(&self) -> &[u8] {
        &self.data[self.keylen as usize..]
    }

    pub fn keylen(&self) -> u32 {
        self.keylen
    }

    pub fn flags(&self) -> u32 {
        self.flags
    }

    pub fn exptime(&self) -> u32 {
        self.exptime
    }

    pub fn valuelen(&self) -> u32 {
        self.valuelen
    }
}

impl Display for Item {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Item {{ keylen: {}, flags: {}, exptime: {}, valuelen: {}, data: {:?} }}",
               self.keylen, self.flags, self.exptime, self.valuelen, self.data)
    }
}

impl PartialEq for Item {
    fn eq(&self, other: &Self) -> bool {
        self.key() == other.key()
    }
}
impl Eq for Item {}

impl Hash for Item {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.key().hash(state);
    }
}
