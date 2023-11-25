/*

 Mutation: (Id, EventId) -> Hash
 Creation: (Id) -> Hash

*/


pub enum Event{
    Creation(Vec<u8>),
    Mutation(Vec<u8>, Vec<u8>)
}

#[cfg(test)]
mod tests {

    use rs_merkle::{algorithms::Sha256, Hasher, MerkleTree, MerkleProof};

    use super::*;
    #[test]
    fn merkle_test() -> Result<(), Box<dyn std::error::Error>>{
        let leaf_values = ["a", "b", "c", "d", "e", "f"];
        let leaves: Vec<[u8; 32]> = leaf_values
            .iter()
            .map(|x| Sha256::hash(x.as_bytes()))
            .collect();
    
        let merkle_tree = MerkleTree::<Sha256>::from_leaves(&leaves);
        let indices_to_prove = vec![3, 4];
        let leaves_to_prove = leaves.get(3..5).ok_or("can't get leaves to prove")?;
        let merkle_proof = merkle_tree.proof(&indices_to_prove);
        let merkle_root = merkle_tree.root().ok_or("couldn't get the merkle root")?;
        // Serialize proof to pass it to the client
        let proof_bytes = merkle_proof.to_bytes();
    
        // Parse proof back on the client
        let proof = MerkleProof::<Sha256>::try_from(proof_bytes)?;
        eprintln!("Proof len: {:?}", proof.proof_hashes_hex());

        assert!(proof.verify(
            merkle_root,
            &indices_to_prove,
            leaves_to_prove,
            leaves.len()
        ));
        Ok(())
    }
}