use criterion::{black_box, criterion_group, criterion_main, Criterion};
use idp2p_common::multi::{id::Idp2pCodec, keypair::Idp2pKeypair};
use idp2p_core::identity::{
    models::{ChangeInput, ChangeType, CreateIdentityInput, IdEvent},
    Identity,
};
fn create_did(codec: Idp2pCodec) -> Identity {
    let keypair = Idp2pKeypair::new_ed25519(&[0u8; 32]).unwrap();
    let input = CreateIdentityInput {
        next_key_digest: keypair.to_key().to_key_digest(),
        recovery_key_digest: keypair.to_key().to_key_digest(),
        events: vec![IdEvent::CreateAuthenticationKey {
            id: vec![1],
            key: keypair.to_key(),
        }],
    };
    let key = keypair.to_key();

    let mut did = Identity::new(codec, input).unwrap();
    for i in 2..10 {
        let change_input = ChangeInput {
            next_key_digest: key.to_key_digest(),
            signer_keypair: keypair.clone(),
            change: ChangeType::AddEvents {
                events: vec![IdEvent::CreateAuthenticationKey {
                    id: vec![i],
                    key: key.clone(),
                }],
            },
        };
        did.change(change_input).unwrap();
    }
    did
}
fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("verify json identity", |b| {
        let did = create_did(Idp2pCodec::Json);
        b.iter(|| black_box(did.verify(None)))
    });
    c.bench_function("verify protobuf identity", |b| {
        let did = create_did(Idp2pCodec::Protobuf);
        b.iter(|| black_box(did.verify(None)))
    });
}
criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
