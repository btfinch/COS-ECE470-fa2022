use serde::{Serialize, Deserialize};


// 20-byte address
#[derive(Eq, PartialEq, Serialize, Deserialize, Clone, Hash, Default, Copy)]
pub struct Address([u8; 20]);

// create Address from a slice of length 20, type u8
impl std::convert::From<&[u8; 20]> for Address {
    fn from(input: &[u8; 20]) -> Address {
        let mut buffer: [u8; 20] = [0; 20];
        buffer[..].copy_from_slice(input);
        Address(buffer)
    }
}
// create Address from an array
impl std::convert::From<[u8; 20]> for Address {
    fn from(input: [u8; 20]) -> Address {
        Address(input)
    }
}

impl std::fmt::Display for Address {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let start = if let Some(precision) = f.precision() {
            if precision >= 40 {
                0
            } else {
                20 - precision / 2
            }
        } else {
            0
        };
        for byte_idx in start..20 {
            write!(f, "{:>02x}", &self.0[byte_idx])?;
        }
        Ok(())
    }
}

impl std::fmt::Debug for Address {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{:>02x}{:>02x}..{:>02x}{:>02x}",
            &self.0[0], &self.0[1], &self.0[18], &self.0[19]
        )
    }
}
// use SHA256 (from ring crate) to hash the input bytes, and takes the last 20 bytes and convert them into a Address struct. 
// questions: 
// how long can bytes be? Do I need to perform multiple hashes if the public key is too long?
// -> I think bytes can be 768 bits and digest output will be 265 bits
// 
use ring::{digest};
impl Address {
    pub fn from_public_key_bytes(bytes: &[u8]) -> Address {
        let big_hash = digest::digest(&digest::SHA256, bytes);
        let mut big_hash_slice: &[u8] = big_hash.as_ref();
        let mut hashArray: [u8; 20] = [0; 20];
        let mut counter: usize = 12;
        for i in hashArray.iter_mut(){
            *i = *big_hash_slice.get(counter).unwrap(); // would using match fix this?
            counter = counter+1;
        }
        Address::from(hashArray)



    }
}
// DO NOT CHANGE THIS COMMENT, IT IS FOR AUTOGRADER. BEFORE TEST

#[cfg(test)]
mod test {
    use super::Address;

    #[test]
    fn from_a_test_key() {
        let test_key = hex!("0a0b0c0d0e0f0e0d0a0b0c0d0e0f0e0d0a0b0c0d0e0f0e0d0a0b0c0d0e0f0e0d");
        let addr = Address::from_public_key_bytes(&test_key);
        let correct_addr: Address = hex!("1851a0eae0060a132cf0f64a0ffaea248de6cba0").into();
        assert_eq!(addr, correct_addr);
        // "b69566be6e1720872f73651d1851a0eae0060a132cf0f64a0ffaea248de6cba0" is the hash of
        // "0a0b0c0d0e0f0e0d0a0b0c0d0e0f0e0d0a0b0c0d0e0f0e0d0a0b0c0d0e0f0e0d"
        // take the last 20 bytes, we get "1851a0eae0060a132cf0f64a0ffaea248de6cba0"
    }
}

// DO NOT CHANGE THIS COMMENT, IT IS FOR AUTOGRADER. AFTER TEST