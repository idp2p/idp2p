# IDP2P(experimental)

## Description

> Everything about you, nothing about you

> `IDP2P` is a peer-to-peer identity protocol and network which enables a controller to store its own proofs. It is also did method ```did:p2p```. In other words, it is ipfs of decentralized identity.

## Background

See also (related specs):

* [Decentralized Identifiers (DIDs)](https://w3c.github.io/did-core)
* [Verifiable Credentials](https://www.w3.org/TR/vc-data-model/)
* [Key DID](https://github.com/w3c-ccg/did-method-key/)
* [Peer DID](https://identity.foundation/peer-did-method-spec/)
* [Key Event Receipt Infrastructure](https://keri.one//)

## Introduction

Most of did methods tried to solve decentralized identity problems with different ways. Some of them used blockchain or dlt solution, another are based on peer relation. Sidetree used a layer-2 solution. KERI is another solution. 

```
did:p2p:z6MkpTHR8VNsBxYAAWHut2Geadd9jSwuBV8xRoAnwWsdvktH
```

That DID would correspond to the following DID Document:

## Identity Generation

### Example DID Document

```json
{
    "ledger": {
      "id": "bagaaieratxin4o3iclo7ua3s3bbueds2uzfc5gi26mermevzb2etqliwjbla",
      "inception": {
        "signer_key": {
          "type": "Ed25519VerificationKey2020",
          "public": "by5gtwpufy4.."
        },
        "recovery_key": {
          "type": "Ed25519VerificationKey2020",
          "digest": "bmb2cvioxfy65ej.."
        }
      },
      "events": [
        {
          "payload": {
            "previous": "bagaaieratxin4o3iclo7u..",
            "signer_publickey": "by5gtwpufy4zfnog4j..",
            "change": {
              "type": "set_document",
              "value": "bdu3gqtjc6ks52.."
            }
          },
          "proof": "bx6svqb6if5yaflgoumdff7j.."
        }
      ]
    },
    "did_doc": {
      "id": "did:p2p:bagaaieratxin..",
      "controller": "did:p2p:bagaaieratxi..",
      "@context": [
        "https://www.w3.org/ns/did/v1",
        "https://w3id.org/security/suites/ed25519-2020/v1",
        "https://w3id.org/security/suites/x25519-2020/v1"
      ],
      "verificationMethod": [
        {}
      ],
      "assertionMethod": [
        "did:p2p:bagaaieratxib#wtyb2xhyvxolbd.."
      ],
      "authentication": [
        "did:p2p:bagaaieratxib#3txadadmtke6d.."
      ],
      "keyAgreement": [
        "did:p2p:bagaaieratxib#cnzphk5djc3bt64.."
      ]
    }
  }
```

### 
## Consensus

- Libp2p 


## Security

The `keyAgreement` key is a Curve25519 public key (suitable for
Diffie-Hellman key exchange) that is deterministically _derived_ from the source
Ed25519 key, using  [`ed2curve-js`](https://github.com/dchest/ed2curve-js).

Note that this derived key is optional -- there's currently
[no proof](https://crypto.stackexchange.com/questions/3260/using-same-keypair-for-diffie-hellman-and-signing/3311#3311)
that this is safe to do.

## Install

Requires rust and cargo

## Usage

### Create a peer 

### Create idenity

### Subscribe to identity

### Reslove identity

### Change identity


PRs accepted.

## License

[Apache](LICENSE) 

