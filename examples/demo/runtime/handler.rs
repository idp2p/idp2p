pub mod message_handler {
    use crate::runtime::{HostComponent, handler::message_handler::idp2p::core::*};
    use wasmtime::component::bindgen;
    bindgen!({
        world:"idp2p-message-handler",
        path:  "./wit/",
        additional_derives: [PartialEq, Eq, Hash, Clone, serde::Serialize, serde::Deserialize],
    });

    impl store::Host for HostComponent {
        #[doc = " Gets a value from the store."]
        fn get(&mut self, key: String) -> Result<Option<Vec<u8>>, types::Idp2pError> {
            Ok(self.store.get(&key))
        }

        #[doc = " Put a value in the store."]
        fn put(&mut self, key: String, value: Vec<u8>) -> Result<(), types::Idp2pError> {
             Ok(self.store.set(&key, &value))
        }
    }

    impl p2p_sender::Host for HostComponent {
        
        fn subscribe(&mut self, topic: String) -> Result<(), types::Idp2pError> {
            todo!()
        }

        fn publish(&mut self, topic: String, payload: Vec<u8>) -> Result<(), types::Idp2pError> {
            // Send network command
            todo!()
        }

        fn send(&mut self, addr: String, payload: Vec<u8>) -> Result<(), types::Idp2pError> {
            todo!()
        }
    }

    impl id_verifier::Host for HostComponent {
        #[doc = " Verifies an initial identity inception event."]
        fn verify_inception(
            &mut self,
            inception: types::IdEventReceipt,
        ) -> Result<types::IdState, types::Idp2pError> {
            self.runtime.verify_inception(&inception.into()).unwrap();
            todo!()
        }

        #[doc = " Verifies an identity update event against the existing identity state."]
        fn verify_event(
            &mut self,
            state: types::IdState,
            event: types::IdEventReceipt,
        ) -> Result<types::IdState, types::Idp2pError> {
            todo!()
        }
    }

}
