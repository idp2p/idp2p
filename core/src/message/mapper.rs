impl Into<idp2p_proto::IdMessage> for IdMessage {
    fn into(self) -> idp2p_proto::IdMessage {
        idp2p_proto::IdMessage {
            id: self.id,
            content: self.body,
        }
    }
}
