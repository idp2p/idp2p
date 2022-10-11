use criterion::{black_box, criterion_group, criterion_main, Criterion};
use idp2p_common::multi::ledgerkey::Idp2pLedgerKeypair;
use idp2p_core::{identity::{
    ChangeType, IdEvent,
    ChangeInput, CreateIdentityInput, Identity, IdentityDecoder,
}, codec::proto::id_decoder::ProtoIdentityDecoder};

/*fn create_did() -> Identity {
    let keypair = Idp2pKeySecret::from_bytes(&[0u8; 32]).unwrap();
    let input = CreateIdentityInput {
        timestamp: 0,
        next_key_digest: keypair.to_key().unwrap().to_key_digest(),
        recovery_key_digest: keypair.to_key().unwrap().to_key_digest(),
        events: vec![IdEvent::CreateAuthenticationKey {
            id: vec![1],
            key: keypair.to_key().unwrap().to_bytes(),
        }],
    };
    let key = keypair.to_key().unwrap();
    let id_behaviour = ProtoIdentityHandler {};
    let mut did = id_behaviour.new(input).unwrap();
    for i in 2..10 {
        let change_input = ChangeInput {
            next_key_digest: key.to_key_digest(),
            signer_secret: keypair.clone(),
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
}*/
fn criterion_benchmark(c: &mut Criterion) {
    let id_behaviour = ProtoIdentityDecoder {};
    //let did = create_did();
    c.bench_function("verify identity", |b| {
        //b.iter(|| black_box(id_behaviour.verify(&did, None).unwrap()))
    });
}
criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
