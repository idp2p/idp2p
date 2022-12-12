use std::marker::PhantomData;

use crate::{
    error::Idp2pError,
    idp2p_proto::{
        assertion_mutation_payload::MutationKind, AssertionMutationPayload, VerifiableAssertion,
    },
};
use idp2p_common::multi::id::Idp2pId;
use keccak_hasher::KeccakHasher;
use memory_db::{MemoryDB, HashKey, PrefixedKey};
use hash_db::HashDB;
use trie_db::{proof::{generate_proof}, TrieDBMutBuilder, DBValue, TrieLayout, TrieMut, TrieDBBuilder};
use trie_root::{Hasher};
use prost::Message;
struct Block {
    id: Vec<u8>,
    prev_hash: Vec<u8>,
    state_hash: Vec<u8>,
    tx_hash: Vec<u8>,
}
#[derive(Default, Clone)]
pub struct ReferenceNodeCodecNoExt<H>(PhantomData<H>);

pub struct GenericNoExtensionLayout<H>(PhantomData<H>);
impl<H> Default for GenericNoExtensionLayout<H> {
	fn default() -> Self {
		GenericNoExtensionLayout(PhantomData)
	}
}

impl<H> Clone for GenericNoExtensionLayout<H> {
	fn clone(&self) -> Self {
		GenericNoExtensionLayout(PhantomData)
	}
}

impl<H: Hasher> TrieLayout for GenericNoExtensionLayout<H> {
	const USE_EXTENSION: bool = false;
	const ALLOW_EMPTY: bool = false;
	const MAX_INLINE_VALUE: Option<u32> = None;
	type Hash = H;
	type Codec = ReferenceNodeCodecNoExt<H>;
}
pub struct VerifiableCredentialState {
    pub id: Vec<u8>,
    pub last_mutation_id: Vec<u8>,
    pub next_issuer_pk_hash: Vec<u8>,
    pub next_holder_pk_hash: Vec<u8>,
    pub issuer_assertions: Vec<Vec<u8>>,
    pub owner_assertions: Vec<Vec<u8>>,
}

/// Codec-flavored TrieStream.
#[derive(Default, Clone)]
pub struct TrieStream {
	/// Current node buffer.
	buffer: Vec<u8>,
}

impl TrieStream {
	// useful for debugging but not used otherwise
	pub fn as_raw(&self) -> &[u8] {
		&self.buffer
	}
}

pub fn verify(vas: VerifiableAssertion) -> Result<VerifiableCredentialState, Idp2pError> {
    let (db, root) = {
		let mut db = <MemoryDB<L>>::default();
		let mut root = Default::default();
		{
			let mut trie = <TrieDBMutBuilder<L>>::new(&mut db, &mut root).build();
			for (key, value) in entries.iter() {
				trie.insert(key, value).unwrap();
			}
		}
		(db, root)
	};

	// Generate proof for the given keys..
	let proof = generate_proof::<_, L, _, _>(&db, &root, keys.iter()).unwrap();
	let trie = <TrieDBBuilder<L>>::new(&db, &root).build();
	let items = keys.into_iter().map(|key| (key, trie.get(key).unwrap())).collect();


	let t = TrieDBBuilder::<MyTrieLayout>::new(&memdb, &root).build();
    let proof = generate_proof::<_, MyTrieLayout, _, _>(&t.db(), &root, pairs.iter()).unwrap();
    let id = Idp2pId::from_bytes(&vas.id)?;
    // Check cid is produced with inception
    id.ensure(&vas.inception)?;
    let mut state = VerifiableCredentialState {
        id: id.to_bytes(),
        last_mutation_id: vas.inception,
        next_issuer_pk_hash: vec![],
        next_holder_pk_hash: vec![],
        issuer_assertions: vec![],
        owner_assertions: vec![],
    };
    for mutation in vas.mutations {
        let mutation_id = Idp2pId::from_bytes(&mutation.id)?;
        mutation_id.ensure(&mutation.payload)?;
        let payload = AssertionMutationPayload::decode(&*mutation.payload)?;
        // Previous event_id should match with last_event_id of state.
        if payload.previous != state.last_mutation_id {
            return Err(Idp2pError::InvalidPreviousEventLog);
        }
        match payload.mutation_kind.unwrap() {
            MutationKind::OwnerAssertion(_) => todo!(),
            MutationKind::IssuerAssertion(_) => todo!(),
            MutationKind::ChangeOwner(_) => todo!(),
        }
        state.last_mutation_id = mutation.id;
    }
    Ok(state)
}

// issuer: revoke, change owner and add important proofs etc.
// holder: add proofs
// holder should keep all mutation metadata and issuer change content
// a proof about an event metadata
// how to verify ? holder should present all proofs metadata in wallet
