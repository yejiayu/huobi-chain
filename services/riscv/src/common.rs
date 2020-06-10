use protocol::{types::Hash, BytesMut};

pub fn combine_key(addr: &[u8], key: &[u8]) -> Hash {
    let mut buf = BytesMut::from(addr);
    buf.extend(key);
    Hash::digest(buf.freeze())
}
