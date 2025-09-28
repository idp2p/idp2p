
pub mod message_handler {
    use wasmtime::component::bindgen;
    use crate::runtime::{
        HostComponent,
        message_handler::idp2p::message,
    };
    bindgen!({
        world:"idp2p-message-handler",
        path:  "./wit/msg-handler/",
        additional_derives: [PartialEq, Eq, Hash, Clone, serde::Serialize, serde::Deserialize],
    });

    impl message::store::Host for HostComponent {
        #[doc = " Gets a value from the store."]
        fn get(&mut self,key:wasmtime::component::__internal::String,) -> Result<Option<wasmtime::component::__internal::Vec<u8>>,message::types::Idp2pError> {
            todo!()
        }
    
        #[doc = " Put a value in the store."]
        fn put(&mut self,key:wasmtime::component::__internal::String,value:wasmtime::component::__internal::Vec<u8>,) -> Result<(),message::types::Idp2pError> {
            todo!()
        }
    
        #[doc = " Checks if a key exists in the store."]
        fn exists(&mut self,key:wasmtime::component::__internal::String,) -> Result<bool,message::types::Idp2pError> {
            todo!()
        }
    }

    impl message::p2p_sender::Host for HostComponent {
        fn publish(&mut self,topic:wasmtime::component::__internal::String,payload:wasmtime::component::__internal::Vec<u8>,) -> Result<(),message::types::Idp2pError> {
            todo!()
        }
    
        fn subscribe(&mut self,topic:wasmtime::component::__internal::String,) -> Result<(),message::types::Idp2pError> {
            todo!()
        }
    
        fn request(&mut self,addr:wasmtime::component::__internal::String,payload:wasmtime::component::__internal::Vec<u8>,) -> Result<(),message::types::Idp2pError> {
            todo!()
        }
    
        fn response(&mut self,payload:wasmtime::component::__internal::Vec<u8>,) -> Result<(),message::types::Idp2pError> {
            todo!()
        }
    }

    impl message::id_verifier::Host for HostComponent {
        #[doc = " Verifies an initial identity inception event."]
        fn verify_inception(&mut self,inception:message::types::IdEventReceipt,) -> Result<message::types::IdState,message::types::Idp2pError> {
            todo!()
        }
    
        #[doc = " Verifies an identity update event against the existing identity state."]
        fn verify_event(&mut self,state:message::types::IdState,event:message::types::IdEventReceipt,) -> Result<message::types::IdState,message::types::Idp2pError> {
            todo!()
        }
    }
}