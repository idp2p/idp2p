# idp2p(experimental)

> Everything about you, nothing about you

> `idp2p` is a peer-to-peer identity protocol and network which enables a controller to store own proofs. It is also did method ```did:p2p```.In another words, it is ipfs of decentralized identity.

## Background

See also (related specs):

* [Decentralized Identifiers (DIDs)](https://w3c.github.io/did-core)
* [Verifiable Credentials](https://www.w3.org/TR/vc-data-model/)
* [Key DID](https://github.com/w3c-ccg/did-method-key/)
* [Peer DID](https://identity.foundation/peer-did-method-spec/)
* [Key Event Receipt Infrastructure](https://keri.one//)

## Problem

Most of did methods tried to solve decentralized identity problems with different ways. Some of them used blockchain or dlt solution, another are based on peer relation. Sidetree used a layer-2 solution. KERI is another solution. 

## Solution

```
did:p2p:z6MkpTHR8VNsBxYAAWHut2Geadd9jSwuBV8xRoAnwWsdvktH
```

That DID would correspond to the following DID Document:

### Example DID Document

```json
{
  "@context": [
    "https://www.w3.org/ns/did/v1",
    "https://w3id.org/security/suites/ed25519-2020/v1",
    "https://w3id.org/security/suites/x25519-2020/v1"
  ],
  "id": "did:p2p:z6MkhaXgBZD...",
  "verificationMethod": [{
    "id": "did:p2p:z6MkhaXgBZD...",
    "type": "Ed25519VerificationKey2020",
    "controller": "did:p2p:z6MkhaXgBZD...",
    "publicKeyMultibase": "z6MkhaXgBZD..."
  }],
  "authentication": [
    "did:key:z6MkhaXgBZD..."
  ],
  "assertionMethod": [
    "did:key:z6MkhaXgBZD..."
  ],
  "capabilityDelegation": [
    "did:key:z6MkhaXgBZD..."
  ],
  "capabilityInvocation": [
    "did:key:z6MkhaXgBZD..."
  ],
  "keyAgreement": [{
    "id": "did:key:z6MkhaXgBZD...",
    "type": "X25519KeyAgreementKey2020",
    "controller": "did:key:z6MkhaXgBZD...",
    "publicKeyMultibase": "z6LSj72tK8..."
  }]
}
```

## Security

The `keyAgreement` key is a Curve25519 public key (suitable for
Diffie-Hellman key exchange) that is deterministically _derived_ from the source
Ed25519 key, using  [`ed2curve-js`](https://github.com/dchest/ed2curve-js).

Note that this derived key is optional -- there's currently
[no proof](https://crypto.stackexchange.com/questions/3260/using-same-keypair-for-diffie-hellman-and-signing/3311#3311)
that this is safe to do.

## Install

Requires Rust

To install from `cargo`:

```

## Usage

### `create-identity <name>`


PRs accepted.

If editing the Readme, please conform to the
[standard-readme](https://github.com/RichardLitt/standard-readme) specification.

## License

[Apache](LICENSE) 

