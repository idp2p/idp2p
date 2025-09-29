pub mod id_verifier {
    use wasmtime::component::bindgen;

    use crate::runtime::{
        HostComponent,
        verifier::id_verifier::host::{Host, IdProof, Idp2pError},
    };
    bindgen!({
        world:"idp2p-id-verifier",
        path:  "./wit/",
        additional_derives: [PartialEq, Eq, Hash, Clone, serde::Serialize, serde::Deserialize],
    });
    impl Host for HostComponent {
        fn verify_proof(&mut self, proof: IdProof, data: Vec<u8>) -> Result<bool, Idp2pError> {
            todo!()
        }
    }
}
