use serde::{Serialize,Deserialize};
use ring::signature::{Ed25519KeyPair, Signature, KeyPair, VerificationAlgorithm, EdDSAParameters, self};
use rand::Rng;

use super::address::Address;
use crate::types::hash::{H256, Hashable};



#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Transaction {
    sender: Address,
    reciever: Address,
    value: i32,

}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct SignedTransaction {
    transaction: Transaction,
    signature: Vec<u8>,
    public_key: Vec<u8>,
    
}

impl Hashable for SignedTransaction{
    fn hash(&self) -> H256 {
        let serial_signed = serde_json::to_string(self);
        ring::digest::digest(&ring::digest::SHA256, serial_signed.unwrap().as_bytes()).into()
    }
}


/// Create digital signature of a transaction
/// Do i need to edit for bigger transactions? -> SHA256 can handle any length automatically
pub fn sign(t: &Transaction, key: &Ed25519KeyPair) -> Signature {
    let serial_transaction = serde_json::to_string(t);
    let sig = key.sign(serial_transaction.unwrap().as_bytes());
    sig
}

pub fn sig_to_vec(signature: Signature) -> Vec<u8>{
    let signature_vector: Vec<u8> = signature.as_ref().to_vec();
    signature_vector
}

/// Verify digital signature of a transaction, using public key instead of secret key
pub fn verify(t: &Transaction, public_key: &[u8], signature: &[u8]) -> bool {
    let serial_transaction = serde_json::to_string(t);
    let good_public_key = ring::signature::UnparsedPublicKey::new(&ring::signature::ED25519, public_key);
    good_public_key.verify(serial_transaction.unwrap().as_bytes(),signature).is_ok()
    
}
use crate::types::address;
#[cfg(any(test, test_utilities))]
pub fn generate_random_transaction() -> Transaction {
    fn generate_random_address() -> Address{
        let rng = ring::rand::SystemRandom::new();
        let pkcs8_thing = ring::signature::Ed25519KeyPair::generate_pkcs8(&rng);
        let key_pair     = ring::signature::Ed25519KeyPair::from_pkcs8(pkcs8_thing.unwrap().as_ref()).unwrap();
        let public_key_bytes = key_pair.public_key();
        let generated = Address::from_public_key_bytes(public_key_bytes.as_ref());
        generated
    }
    let address1 = generate_random_address();
    let address2 = generate_random_address();
    let mut rng = rand::thread_rng();
    let val: i32 = rng.gen();
    let rand_transact = Transaction {sender: address1, reciever: address2, value: val};
    rand_transact
    
}

// DO NOT CHANGE THIS COMMENT, IT IS FOR AUTOGRADER. BEFORE TEST

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::key_pair;
    use ring::signature::KeyPair;


    #[test]
    fn sign_verify() {
        let t = generate_random_transaction();
        let key = key_pair::random();
        let signature = sign(&t, &key);
        assert!(verify(&t, key.public_key().as_ref(), signature.as_ref()));
    }
    #[test]
    fn sign_verify_two() {
        let t = generate_random_transaction();
        let key = key_pair::random();
        let signature = sign(&t, &key);
        let key_2 = key_pair::random();
        let t_2 = generate_random_transaction();
        assert!(!verify(&t_2, key.public_key().as_ref(), signature.as_ref()));
        assert!(!verify(&t, key_2.public_key().as_ref(), signature.as_ref()));
    }
}

// DO NOT CHANGE THIS COMMENT, IT IS FOR AUTOGRADER. AFTER TEST