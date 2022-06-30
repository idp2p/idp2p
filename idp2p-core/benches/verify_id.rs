use criterion::{black_box, criterion_group, criterion_main, Criterion};
use idp2p_common::multi::keypair::Idp2pKeypair;
use idp2p_core::identity::{
    models::{ChangeType, IdEvent},
    ChangeInput, CreateIdentityInput, Identity, IdentityHandler,
};
use idp2p_core::identity::handler::id_handler::ProtoIdentityHandler;

fn create_did() -> Identity {
    let keypair = Idp2pKeypair::from_ed_secret(&[0u8; 32]).unwrap();
    let input = CreateIdentityInput {
        timestamp: 0,
        next_key_digest: keypair.to_key().to_key_digest(),
        recovery_key_digest: keypair.to_key().to_key_digest(),
        events: vec![IdEvent::CreateAuthenticationKey {
            id: vec![1],
            key: keypair.to_key().to_bytes(),
        }],
    };
    let key = keypair.to_key();
    let id_behaviour = ProtoIdentityHandler {};
    let mut did = id_behaviour.new(input).unwrap();
    for i in 2..10 {
        let change_input = ChangeInput {
            next_key_digest: key.to_key_digest(),
            signer_keypair: keypair.clone(),
            change: ChangeType::AddEvents {
                events: vec![IdEvent::CreateAuthenticationKey {
                    id: vec![i],
                    key: key.to_bytes(),
                }],
            },
        };
        id_behaviour.change(&mut did, change_input).unwrap();
    }
    did
}
fn criterion_benchmark(c: &mut Criterion) {
    let id_behaviour = ProtoIdentityHandler {};
    let did = create_did();
    c.bench_function("verify identity", |b| {
        b.iter(|| black_box(id_behaviour.verify(&did, None).unwrap()))
    });
}
criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
