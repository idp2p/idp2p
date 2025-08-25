mod receipt;
mod signer;
mod state;
mod error;

pub use error::*;
pub use receipt::*;
pub use signer::IdSigner;
pub use state::*;


#[cfg(test)]
mod tests {
    use crate::VALID_FROM;
    use chrono::*;
    #[test]
    fn abc() {
        let valid_from: DateTime<Utc> = VALID_FROM.parse().unwrap();
        let valid_from = valid_from.to_rfc3339_opts(SecondsFormat::Secs, true);
        println!("{}", valid_from);
    }
}
