pub struct Idp2pMessage{
   id: Vec<u8>,
   from: Vec<u8>,
   to: Vec<u8>,
   body: Vec<u8>,
   created_at: i64
}
 
pub mod codec;