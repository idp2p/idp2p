/*use criterion::{black_box, criterion_group, criterion_main, Criterion};
use idp2p_core::did::identity::Identity;
use idp2p_core::did::eventlog::{EventLogChange};
use idp2p_core::did::eventlog::ProofStatement;

fn save_doc(did: &mut Identity, secret: EdSecret) {
    let set_change = EventLogChange::SetProof(ProofStatement {
        key: vec![],
        value: vec![],
    });
    let change = vec![set_change];
    let signer = secret.to_publickey();
    let payload = did
        .microledger
        .create_event(&signer, &hash(&signer), change);
    let proof = secret.sign(&payload);
    did.microledger.save_event(payload, &proof);
}

fn criterion_benchmark(c: &mut Criterion) {
    let secret_str = "beilmx4d76udjmug5ykpy657qa3pfsqbcu7fbbtuk3mgrdrxssseq";
    let secret = EdSecret::from_str(secret_str).unwrap();
    let ed_key_digest = secret.to_publickey_digest().unwrap();
    let mut did = Identity::new(&ed_key_digest, &ed_key_digest);
    for _ in 1..10 {
        save_doc(&mut did, secret.clone());
    }
    c.bench_function("verify identity", |b| {
        b.iter(|| black_box(did.verify()))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);*/
