use anyhow::Result;
use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::store::InMemoryKvStore;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IdUser {
    pub name: String,
    pub id: Option<String>,
}

pub enum UserMode {
    NotReady,
    Ready,
    Connected,
}

pub async fn init_users(current: &str, id: &str, store: Arc<InMemoryKvStore>) -> Result<()> {

    let alice = IdUser {
        name: "Alice".to_string(),
        id: if current == "alice" {
            Some(id.to_string())
        } else {
            None
        },
    };
    let bob = IdUser {
        name: "Bob".to_string(),
        id: if current == "bob" {
            Some(id.to_string())
        } else {
            None
        },
    };
    let dog = IdUser {
        name: "Dog".to_string(),
        id: if current == "dog" {
            Some(id.to_string())
        } else {
            None
        },
    };    
    store.set("/current-user", &current.to_string()).await?;
    store.set_user("alice", &alice).await?;
    store.set_user("bob", &bob).await?;
    store.set_user("dog", &dog).await?;
    Ok(())
}
