
pub struct User {
    id: String,
    state: String,
    consents: Vec<>
}

pub trait UserRepository {
    fn get() -> Result<User, Error>;
    fn put(user: User) -> Result<(), Error>;
}