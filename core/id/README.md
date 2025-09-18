# ID Module

> KERI implementation of ID as wasm component.



[!TODO]

- Witness
- Keys(authentication, key-agreement, assertion-method)
- Peers
- Mediators
- Delegations(revoke, recovery, ancestor)


`/idp2p/id/{major}/{minor}/{cid}`

Specifies identifier of identity controller

`/idp2p/event/{major}/{minor}/{cid}`

Specifies identity keri event

`/idp2p/signer/{cid}`

Specifies identity keri signer

-----------------------------------

`/idp2p/message/{major}/{minor}/{cid}`

Specifies idp2p message id

-------------------------------

`/idp2p/authentication/`   -> cid

`/idp2p/key-agreement/`    -> cid 

`/idp2p/assertion-method/` -> cid 

`/idp2p/peer/`             -> peer 

`/idp2p/mediator/`         -> id 

0 -> id
1 -> event
2 -> signer
3 -> message
4 -> mediator
5 -> peer
6 -> authentication
7 -> key-agreement
8 -> assertion-method

Examples:

- `/idp2p/id/1/0/bafkreieq5jui4j25lacwomsqgjeswwl3y5zcdrresptwgmfylxo2depppq`

```json
{
    "inception": "(self:3 || external-\"abc\") && self:6",
}
```

## Verification Rules (Current)

- Timestamps
  - All event and inception timestamps are seconds since Unix epoch.
  - `VALID_FROM` is compared in seconds; events must be at or after this boundary.
  - `state.event_timestamp` updates to the verified event’s time (RFC3339, seconds precision) on every event.

- Proofs
  - Signatures are Ed25519 over the CBOR payload; `receipt.id` must be the CID of that payload.
  - Interaction
    - Requires at least `state.threshold` proofs in `receipt.proofs`.
    - Proofs are checked against allowed signers (currently all `state.signers`).
  - Rotation
    - Let `all_signers = revealed_signers ∪ new_signers`.
    - Requires `all_signers.len() == receipt.proofs.len()` and `all_signers.len() >= threshold`.
    - `revealed_signers.len() >= state.next_threshold`, and all revealed must be in `state.next_signers`.
    - `next_signers.len() >= next_threshold`, and each next signer CID must be ED25519.
    - On success: updates `state.threshold`, `state.next_threshold`, `state.next_signers`.
  - Revocation
    - Requires `revealed_signers.len() == receipt.proofs.len()` and `revealed_signers.len() >= state.next_threshold`.
    - All revealed must be in `state.next_signers`.
    - On success: sets `state.revoked = true` and `state.revoked_at` to event time.
  - Migration
    - Same proof requirements as Revocation on `revealed_signers`.
    - On success: sets `state.next_id`.

- Claims (Interaction)
  - `new_claims` add claim values if they don’t duplicate existing `(key,id)` pairs.
  - `revoked_claims` set `valid_until` for existing `(key,id)` values; if not found, returns `InvalidClaim`.

Notes
- These rules reflect the current implementation and tests in `core/id/src/verifier`. If policy needs to be tightened (e.g., Interaction proofs restricted to `current_signers`), update code and tests accordingly.
