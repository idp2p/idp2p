pub struct User {
    id: DigestId,
    role: String,
    assertion_keys: Vec<DigestId>,
    authentication_keys: Vec<DigestId>,
}