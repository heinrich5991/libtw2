use std::fmt;

#[derive(Debug)]
pub struct InvalidSliceLength;

#[derive(Clone, Copy)]
pub struct Sha256(pub [u8; 32]);

impl Sha256 {
    pub fn from_slice(bytes: &[u8])
        -> Result<Sha256, InvalidSliceLength>
    {
        let mut result = [0; 32];
        if bytes.len() != result.len() {
            return Err(InvalidSliceLength);
        }
        result.copy_from_slice(bytes);
        Ok(Sha256(result))
    }
}

impl fmt::Debug for Sha256 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for &b in &self.0 {
            write!(f, "{:02x}", b)?;
        }
        Ok(())
    }
}

impl fmt::Display for Sha256 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}
