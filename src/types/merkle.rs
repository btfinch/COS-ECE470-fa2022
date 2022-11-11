use super::hash::{Hashable, H256};

/// A Merkle tree.
#[derive(Debug, Default)]
pub struct MerkleTree {
    tree_vector: Vec<Vec<H256>>,
    levels: u8, //the highest level of Merkel Tree, (index starts at 0)
}

use ring::{digest};
fn hash_pairs(old_vec:&Vec<H256>) -> Vec<H256>{
    let mut new_vec: Vec<H256> = vec!();
    let mut temp = &old_vec[0];
    let mut counter: u8 = 0;
    
    for x in old_vec { // Check types and referencing
        if counter%2 ==0 {
            temp = x;
        }
        else {
            //hash temp and current and then add it to the new vector
            let x_array = x.as_ref();
            let temp_array= temp.as_ref();
            let both = [temp_array, x_array].concat();
            let both_slice = both.as_slice();
            let both_hash = digest::digest(&digest::SHA256, both_slice);
            let hash_H256 = H256::from(both_hash);
            // println!("{:?}",hash_H256);

            // let hash_H256 = both_slice.hash(); ? Why doesn't this work?
            new_vec.push(hash_H256);

        }
        counter = counter+1;

    }
    new_vec

}


impl MerkleTree {
    pub fn new<T>(data: &[T]) -> Self where T: Hashable, {
        let mut lev: u8 = 0;
        let mut length: usize = data.len(); // does this output 4? for for elements -> yes
        let mut vector: Vec<H256> = vec!();
        let mut output: Vec<Vec<H256>> = vec!();
        

        for i in data{
            vector.push(i.hash()); //is this ok?
        }
        if length == 0{
            let zeros: [u8; 32] = [0;32];
            let h_zeros: H256 = H256::from(zeros);
            vector.push(h_zeros);

        }
        while length >= 2{
            if length%2 ==1{
                vector.push(vector[length-1]);
                length = length +1;
            
            } 
            let mut out_push = vec!();
            for element in vector.iter(){
                out_push.push(*element);
            }
            output.push(out_push);
            vector = hash_pairs(&vector); // is this ok?
            length = length/2;
            lev = lev + 1;

        }
        let mut out_push2 = vec!();
            for element in vector.iter(){
                out_push2.push(*element);
            }
            output.push(out_push2);

        MerkleTree { tree_vector: output, levels: lev }

        }
    

    pub fn root(&self) -> H256 {
        self.tree_vector[self.levels as usize][0] //figure out how to use self
        // println!("{}",self.tree_vector[self.levels as usize][0])
    }

    /// Returns the Merkle Proof of data at index i
    /// 
    /// Might need to modify proof so that it can handle an input with a levels value of 0 (this means there is only 1 level with one H256)
    pub fn proof(&self, index: usize) -> Vec<H256> { // is self a merkel tree (where self T: MerkelTree)
        let mut output: Vec<H256> = vec!(); // is it vec!() or vec![] and does it matter?
        let mut ind = index;
        for i in 0..self.levels{ // probably will have to change i type
            if ind%2 == 1{
                output.push(self.tree_vector[i as usize][(ind-1) as usize]);
                ind = ind -1
            }
            else{
                output.push(self.tree_vector[i as usize][ind+1]);
            }
            ind = ind/2
        }
        output
    }

}
/// Verify that the datum hash with a vector of proofs will produce the Merkle root. Also need the
/// index of datum and `leaf_size`, the total number of leaves.
pub fn verify(root: &H256, datum: &H256, proof: &[H256], index: usize, leaf_size: usize) -> bool {
    // start with datum, based on index value, hash left or hash right until you get the root
    let mut gen_root: H256 = *datum;
    let mut ind = index;
    for x in proof{
        if ind%2 == 1{ //right side
            let both = [x.as_ref(), gen_root.as_ref()].concat();
            let both_slice = both.as_slice();
            let both_hash = digest::digest(&digest::SHA256, both_slice);
            let hash_H256 = H256::from(both_hash);
            gen_root = hash_H256;
        }
        else { // left side
            let both = [gen_root.as_ref(), x.as_ref()].concat();
            let both_slice = both.as_slice();
            let both_hash = digest::digest(&digest::SHA256, both_slice);
            let hash_H256 = H256::from(both_hash);
            gen_root = hash_H256;
        }
    }

    gen_root == *root
    
}
// DO NOT CHANGE THIS COMMENT, IT IS FOR AUTOGRADER. BEFORE TEST

#[cfg(test)]
mod tests {
    use crate::types::hash::H256;
    use super::*;

    macro_rules! gen_merkle_tree_data {
        () => {{
            vec![
                (hex!("0a0b0c0d0e0f0e0d0a0b0c0d0e0f0e0d0a0b0c0d0e0f0e0d0a0b0c0d0e0f0e0d")).into(),
                (hex!("0101010101010101010101010101010101010101010101010101010101010202")).into(),
            ]
        }};
    }

    #[test]
    fn merkle_root() {
        let input_data: Vec<H256> = gen_merkle_tree_data!();
        let merkle_tree = MerkleTree::new(&input_data);
        let root = merkle_tree.root();
        assert_eq!(
            root,
            (hex!("6b787718210e0b3b608814e04e61fde06d0df794319a12162f287412df3ec920")).into()
        );
        // "b69566be6e1720872f73651d1851a0eae0060a132cf0f64a0ffaea248de6cba0" is the hash of
        // "0a0b0c0d0e0f0e0d0a0b0c0d0e0f0e0d0a0b0c0d0e0f0e0d0a0b0c0d0e0f0e0d"
        // "965b093a75a75895a351786dd7a188515173f6928a8af8c9baa4dcff268a4f0f" is the hash of
        // "0101010101010101010101010101010101010101010101010101010101010202"
        // "6b787718210e0b3b608814e04e61fde06d0df794319a12162f287412df3ec920" is the hash of
        // the concatenation of these two hashes "b69..." and "965..."
        // notice that the order of these two matters
    }

    #[test]
    fn merkle_proof() {
        let input_data: Vec<H256> = gen_merkle_tree_data!();
        let merkle_tree = MerkleTree::new(&input_data);
        let proof = merkle_tree.proof(0);
        assert_eq!(proof,
                   vec![hex!("965b093a75a75895a351786dd7a188515173f6928a8af8c9baa4dcff268a4f0f").into()]
        );
        // "965b093a75a75895a351786dd7a188515173f6928a8af8c9baa4dcff268a4f0f" is the hash of
        // "0101010101010101010101010101010101010101010101010101010101010202"
    }

    #[test]
    fn merkle_verifying() {
        let input_data: Vec<H256> = gen_merkle_tree_data!();
        let merkle_tree = MerkleTree::new(&input_data);
        let proof = merkle_tree.proof(0);
        assert!(verify(&merkle_tree.root(), &input_data[0].hash(), &proof, 0, input_data.len()));
    }
}

// DO NOT CHANGE THIS COMMENT, IT IS FOR AUTOGRADER. AFTER TEST