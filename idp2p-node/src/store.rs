use idp2p_core::did::Identity;

pub trait IdStore {
    fn put(&self, id: &str, value: Identity);
    fn get(&self, id: &str) -> Option<Identity>;
}