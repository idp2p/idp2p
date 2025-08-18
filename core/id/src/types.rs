mod envelope;
mod signer;
mod state;

use chrono::SubsecRound;
pub use envelope::{IdEventEnvelope, IdEventProof, IdProof};
pub use signer::IdSigner;
pub use state::{IdClaimEvent, IdDelegator, IdState};


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
