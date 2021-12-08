# IDP2P(experimental)

## Description

> Everything about you, nothing about you

> `IDP2P` is a peer-to-peer identity protocol and network which enables a controller to store its own proofs. It is also did method `did:p2p`. The protocol is developed based on `libp2p`, in other words, it is ipfs of decentralized identity.

## Background

See also (related specs):

* [Decentralized Identifiers (DIDs)](https://w3c.github.io/did-core)
* [Verifiable Credentials](https://www.w3.org/TR/vc-data-model/)
* [Key DID](https://github.com/w3c-ccg/did-method-key/)
* [Peer DID](https://identity.foundation/peer-did-method-spec/)
* [Key Event Receipt Infrastructure](https://keri.one//)

## Introduction

Each did method tried to solve decentralized identity problems with different ways. Some of them used `blockchain` or `dlt` for storing did documents. Sidetree used a layer-2 solution to solve cost and efficiency problem. Others are simple, self-describing methods and aren't depend on any ledger technology  e.g. `did:peer`, `did:key`, `did:keri`. Each method chooses some of following features: 

- Decentralization
- Easy to use
- Resolve
- Cost and Efficiency


## Identity Generation

Identity is microledger and document. Microledger is inception(recovery key, signer key) and events. 

### Example DID Document

```did:p2p:z6MkpTHR8VNsBxYAAWHut2Geadd9jSwuBV8xRoAnwWsdvktH```

```json
{
    "microledger": {
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

## Consensus

![w:1000](idp2p.drawio.png) 


## Install

Requires rust and cargo

## Usage

### Create a peer

- ```cargo run -p <port>```

### Create idenity

- ```create-id <name>```

### Subscribe to identity

- ```get <id>```

### Reslove identity

- ```resolve <id>```

### Create new doc

- ```create-doc <name>```


## License

[Apache](LICENSE) 

