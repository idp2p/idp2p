#[derive(Debug)]
pub struct TrieNode{
    path: String,
    key: String,
    proof: String,
    children: TrieNodeKind
}

#[derive(Debug)]
pub enum TrieNodeKind{
    Branch(Vec<TrieNode>),
    Leaf(RawClaim)
}

#[derive(Debug)]
pub struct RawClaim{
    raw: String,
    salt: String
}

#[skip_serializing_none]
#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
pub struct IdentityEvent {
    pub assertion_keys: Option<HashMap<String, String>>,
    pub authentication_keys: Option<HashMap<String, String>>,
    pub agreement_keys: Option<HashMap<String, String>>,
    pub proofs: Option<HashMap<String, String>>,
}