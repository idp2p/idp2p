pub fn verify_inception_inner(inception: Vec<u8>) -> anyhow::Result<Vec<u8>> {
    let inception: PersistedIdInception = decode(inception.as_slice())?;
    let view = inception.verify()?;
    Ok(encode(&view)?)
}

pub fn verify_event(view: IdView, pevent: PersistedIdEvent) -> anyhow::Result<IdView> {
    /*let event_id = Cid::from_bytes(&pevent.id)?;
    event_id.ensure(pevent.payload.as_slice())?;
    let mut signers: Vec<IdSigner> = vec![];
    for proof in pevent.proofs {
        let signer_id = Cid::from_bytes(&proof.id)?;
        signer_id.ensure(&proof.pk)?;
        if let Some(signer) = snapshot.next_signers.iter().find(|x| x.id == signer_id.to_bytes()){
            verify(&proof.pk, &pevent.payload, &proof.sig)?;
            signers.push(signer.to_owned());
        }else{
            anyhow::bail!("invalid signer")
        }
    }

    let event = IdEvent::from_bytes(&pevent.payload)?;
    event.validate()?;
    let mut snapshot = snapshot;

    if event.previous != Cid::from_bytes(&snapshot.event_id)? {
        anyhow::bail!("invalid previous")
    }

    // Check signer quorum
    match event.payload {
        IdEventPayload::ChangeState(state) => {
            if snapshot.used_states.contains(&state.to_bytes()) {
                anyhow::bail!("duplicated state")
            }
            snapshot.state = state.to_bytes();
            snapshot.used_states.push(state.to_bytes());
        },
        IdEventPayload::ChangeConfig(id_config) => {
            id_config.validate()?;
            snapshot.config = id_config;
        },
        IdEventPayload::RevokeEvent => todo!(),
    }
    for signer in signers.iter() {
        snapshot.used_signers.push(signer.id.clone());
    }
    snapshot.next_signers = event.next_signers;
    Ok(snapshot)*/
    todo!()
}
