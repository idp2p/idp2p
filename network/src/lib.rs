mod node;
mod model;

// Generate bindings of the guest and host components.
bindgen!({
    path: "../identity",
    additional_derives: [PartialEq, Eq, Hash, Clone, serde::Serialize, serde::Deserialize],
});