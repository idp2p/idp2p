//pub mod types;
pub mod error;

wit_bindgen::generate!({
    world: "idp2p-p2p",
    additional_derives: [PartialEq, Eq, Hash, Clone, serde::Serialize, serde::Deserialize],
});

struct GuestComponent;

export!(GuestComponent);

impl Guest for GuestComponent {
    fn handle(msg: Vec<u8>,) -> Result<(), String> {
        /*
        
        let msg = types::IdNetworkEvent::decode(msg);
        match msg {
            Request => {}
            Response => {}
            Pubsub(msg) => {
               match msg {
                   Resolve => {}
                   Provide => {}
                   NotifyEvent => {}
                   NotifyMessage => {}
               }
            }
        }*/
        todo!()
    }
}
