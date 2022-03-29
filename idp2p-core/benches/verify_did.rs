use criterion::{black_box, criterion_group, criterion_main, Criterion};
use idp2p_common::ed_secret::EdSecret;
use idp2p_common::hash;
use idp2p_core::did::Identity;
use idp2p_core::did_doc::CreateDocInput;
use idp2p_core::did_doc::IdDocument;
use idp2p_core::eventlog::DocumentDigest;
use idp2p_core::eventlog::EventLogChange;

fn save_doc(did: &mut Identity, secret: EdSecret) {
    let ed_key = secret.to_publickey();
    let x_key = secret.to_key_agreement();
    let input = CreateDocInput {
        id: did.id.clone(),
        assertion_key: ed_key.to_vec(),
        authentication_key: ed_key.to_vec(),
        keyagreement_key: x_key.to_vec(),
    };
    let doc = IdDocument::new(input);
    let doc_digest = doc.get_digest();
    did.document = Some(doc);
    let change = EventLogChange::SetDocument(DocumentDigest { value: doc_digest });
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
criterion_main!(benches);
