pub struct Aggregate {
    id: Vec<u8>,
    events: Vec<AggregateEvent>,
}

pub struct Code {
    id: Vec<u8>,
    body: Vec<u8>,
}

pub struct Transaction {
    id: Vec<u8>,
    code_id: Vec<u8>,
    input: Vec<u8>,
}

pub struct AggregateEvent {
    name: Vec<u8>,
    payload: Vec<u8>,
}

pub struct AggregateQuery {
    name: Vec<u8>,
    payload: Vec<u8>,
}

pub struct Block {
    id: Vec<u8>,
    prev_hash: Vec<u8>,
    state_hash: Vec<u8>,
    code_hash: Vec<u8>,
    tx_hash: Vec<u8>,
}
