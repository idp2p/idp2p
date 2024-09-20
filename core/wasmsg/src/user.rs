pub struct WasmUser {
    id: String,
    keys: Vec<WasmUserKey>,
    role: String,
    permissions: BTreeMap<String, String>
} 

pub struct WasmUserKey {
    id: String,
    public: Vec<u8>
} 