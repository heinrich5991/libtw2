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

#[cfg(feature = "serde")]
mod serialize {
    use std::fmt;
    use std::iter;

    use super::Sha256;

    impl serde::Serialize for Sha256 {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where S: serde::Serializer,
        {
            serializer.serialize_str(&format!("{}", self))
        }
    }

    struct HexSha256Visitor;

    impl<'de> serde::de::Visitor<'de> for HexSha256Visitor {
        type Value = Sha256;

        fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
            f.write_str("64 character hex value")
        }
        fn visit_str<E: serde::de::Error>(self, v: &str) -> Result<Sha256, E> {
            let len = v.chars().count();
            if len != 64 {
                return Err(E::invalid_length(len, &self));
            }
            let mut result = [0; 32];
            // I just want to get string slices with two characters each. :(
            // Sorry for this monstrosity.
            let starts = v.char_indices().map(|(i, _)| i).chain(iter::once(v.len())).step_by(2);
            let ends = { let mut e = starts.clone(); e.next(); e };
            for (i, (s, e)) in starts.zip(ends).enumerate() {
                result[i] = u8::from_str_radix(&v[s..e], 16).map_err(|_| {
                    E::invalid_value(serde::de::Unexpected::Str(v), &self)
                })?;
            }
            Ok(Sha256(result))
        }
    }

    impl<'de> serde::Deserialize<'de> for Sha256 {
        fn deserialize<D>(deserializer: D) -> Result<Sha256, D::Error>
            where D: serde::de::Deserializer<'de>,
        {
            deserializer.deserialize_str(HexSha256Visitor)
        }
    }
}
