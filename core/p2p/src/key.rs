#[macro_export]
macro_rules! id_key {
    ($id:expr) => {
        &format!("/id/{}", $id)
    };
}

#[macro_export]
macro_rules! id_providers_key {
    ($id:expr) => {
        &format!("/id/{}/providers", $id)
    };
}

#[macro_export]
macro_rules! id_messages_key {
    ($id:expr) => {
        &format!("/id/{}/messages", $id)
    };
}

#[macro_export] 
macro_rules! id_peers_key {
    ($id:expr) => {
        &format!("/id/{}/peers", $id)
    };
}

#[macro_export] 
macro_rules! id_pending_messages_key {
    ($id:expr) => {
        &format!("/id/{}/pending/messages", $id)
    };
}


