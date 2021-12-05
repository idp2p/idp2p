# idp2p

> Everything about you, nothing about you

## Background

See also (related specs):

* [Decentralized Identifiers (DIDs)](https://w3c.github.io/did-core)
* [Verifiable Credentials](https://www.w3.org/TR/vc-data-model/)

`idp2p` is a peer-to-peer identity protocol and network which enables a controller to store own proofs. It is also did method ```did:p2p```. It is ipfs of decentralized identity.

Most of did methods tried to solve decentralized identity problems with different ways. Some of them used blockchain or dlt solution, another are based on peer relation. Sidetree used a layer-2 solution. KERI is another solution. 


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

Requires Node.js 12+

To install from `npm`:

```
npm install --save @digitalbazaar/did-method-key
```

To install locally (for development):

```
git clone https://github.com/digitalbazaar/did-method-key.git
cd did-method-key
npm install
```

## Usage

### `generate()`

To generate a new key and get its corresponding `did:key` method DID Document:

```js
const didKeyDriver = require('@digitalbazaar/did-method-key').driver();

// generate did:key using Ed25519 key type by default
const {didDocument, keyPairs, methodFor} = await didKeyDriver.generate();

// print the DID Document above
console.log(JSON.stringify(didDocument, null, 2));

// keyPairs will be set like so ->
Map(2) {
  'did:key:z6MkuBLrjSGt1PPADAvuv6rmvj4FfSAfffJotC6K8ZEorYmv#z6MkuBLrjSGt1PPADAvuv6rmvj4FfSAfffJotC6K8ZEorYmv' => Ed25519VerificationKey2020 {
    id: 'did:key:z6MkuBLrjSGt1PPADAvuv6rmvj4FfSAfffJotC6K8ZEorYmv#z6MkuBLrjSGt1PPADAvuv6rmvj4FfSAfffJotC6K8ZEorYmv',
    controller: 'did:key:z6MkuBLrjSGt1PPADAvuv6rmvj4FfSAfffJotC6K8ZEorYmv',
    type: 'Ed25519VerificationKey2020',
    publicKeyMultibase: 'z6MkuBLrjSGt1PPADAvuv6rmvj4FfSAfffJotC6K8ZEorYmv',
    privateKeyMultibase: 'z3zDo1wXuXGcFkJa9SPE7VYpdutmHq8gJsvFRMKJckTWMykoHsAjWNbHXqzrZ8qa7aWdDTjmJNJ1amYEG2mCvZZeY'
  },
  'did:key:z6MkuBLrjSGt1PPADAvuv6rmvj4FfSAfffJotC6K8ZEorYmv#z6LSeRSE5Em5oJpwdk3NBaLVERBS332ULC7EQq5EtMsmXhsM' => X25519KeyAgreementKey2020 {
    id: 'did:key:z6MkuBLrjSGt1PPADAvuv6rmvj4FfSAfffJotC6K8ZEorYmv#z6LSeRSE5Em5oJpwdk3NBaLVERBS332ULC7EQq5EtMsmXhsM',
    controller: 'did:key:z6MkuBLrjSGt1PPADAvuv6rmvj4FfSAfffJotC6K8ZEorYmv',
    type: 'X25519KeyAgreementKey2020',
    publicKeyMultibase: 'z6LSeRSE5Em5oJpwdk3NBaLVERBS332ULC7EQq5EtMsmXhsM',
    privateKeyMultibase: 'z3weeMD56C1T347EmB6kYNS7trpQwjvtQCpCYRpqGz6mcemT'
  }
}
```

`methodFor` is a convenience function that returns a public/private key pair
instance for a given purpose. For example, a verification key (containing a
`signer()` and `verifier()` functions) are frequently useful for
[`jsonld-signatures`](https://github.com/digitalbazaar/jsonld-signatures) or
[`vc-js`](https://github.com/digitalbazaar/vc-js) operations. After generating
a new did:key DID, you can do:

```js
// For signing Verifiable Credentials
const assertionKeyPair = methodFor({purpose: 'assertionMethod'});
// For Authorization Capabilities (zCaps)
const invocationKeyPair = methodFor({purpose: 'capabilityInvocation'});
// For Encryption using `@digitalbazaar/minimal-cipher`
const keyAgreementPair = methodFor({purpose: 'keyAgreement'});
```

Note that `methodFor` returns a key pair that contains both a public and private
key pair (since it has access to the `keyPairs` map from `generate()`).
This makes it useful for _signing_ and _encrypting_ operations (unlike the
`publicMethodFor` that's returned by `get()`, below).

### `publicKeyToDidDoc()`

If you already have an `Ed25519VerificationKey2020` public key object (as an
LDKeyPair instance, or a plain key description object), you can turn it into
a DID Document:

```js
const {didDocument} = await didKeyDriver.publicKeyToDidDoc({publicKeyDescription});
```

### `get()`

#### Getting a full DID Document from a `did:key` DID

To get a DID Document for an existing `did:key` DID:

```js
const did = 'did:key:z6MknCCLeeHBUaHu4aHSVLDCYQW9gjVJ7a63FpMvtuVMy53T';
const didDocument = await didKeyDriver.get({did});
```

(Results in the [example DID Doc](#example-did-document) above).

#### Getting just the key object by key id

You can also use a `.get()` to retrieve an individual key, if you know it's id
already (this is useful for constructing `documentLoader`s for JSON-LD Signature
libs, and the resulting key does include the appropriate `@context`).

```js
const verificationKeyId = 'did:key:z6MknCCLeeHBUaHu4aHSVLDCYQW9gjVJ7a63FpMvtuVMy53T#z6MknCCLeeHBUaHu4aHSVLDCYQW9gjVJ7a63FpMvtuVMy53T';
await didKeyDriver.get({url: verificationKeyId});
// ->
{
  "@context": "https://w3id.org/security/suites/ed25519-2020/v1",
  "id": "did:key:z6MknCCLeeHBUaHu4aHSVLDCYQW9gjVJ7a63FpMvtuVMy53T#z6MknCCLeeHBUaHu4aHSVLDCYQW9gjVJ7a63FpMvtuVMy53T",
  "type": "Ed25519VerificationKey2020",
  "controller": "did:key:z6MknCCLeeHBUaHu4aHSVLDCYQW9gjVJ7a63FpMvtuVMy53T",
  "publicKeyMultibase": "z6MknCCLeeHBUaHu4aHSVLDCYQW9gjVJ7a63FpMvtuVMy53T"
}

const keyAgreementKeyId = 'did:key:z6MknCCLeeHBUaHu4aHSVLDCYQW9gjVJ7a63FpMvtuVMy53T#z6LSotGbgPCJD2Y6TSvvgxERLTfVZxCh9KSrez3WNrNp7vKW';
await didKeyDriver.get({url: keyAgreementKeyId});
// ->
{
  "@context": "https://w3id.org/security/suites/x25519-2020/v1",
  "id": "did:key:z6MknCCLeeHBUaHu4aHSVLDCYQW9gjVJ7a63FpMvtuVMy53T#z6LSotGbgPCJD2Y6TSvvgxERLTfVZxCh9KSrez3WNrNp7vKW",
  "type": "X25519KeyAgreementKey2020",
  "controller": "did:key:z6MknCCLeeHBUaHu4aHSVLDCYQW9gjVJ7a63FpMvtuVMy53T",
  "publicKeyMultibase": "z6LSotGbgPCJD2Y6TSvvgxERLTfVZxCh9KSrez3WNrNp7vKW"
}
```

### `publicMethodFor()`

Often, you have just a `did:key` DID, and you need to get a key for a
particular _purpose_ from it, such as an `assertionMethod` key to verify a
VC signature, or a `keyAgreement` key to encrypt a document for that DID's
controller.

For that purpose, you can use a combination of `get()` and `publicMethodFor`:

```js
// Start with the DID
const didDocument = await didKeyDriver.get({did});
// This lets you use `publicMethodFor()` to get a key for a specific purpose
const keyAgreementData = didKeyDriver.publicMethodFor({
  didDocument, purpose: 'keyAgreement'
});
const assertionMethodData = didKeyDriver.publicMethodFor({
  didDocument, purpose: 'assertionMethod'
});

// If you're using a `crypto-ld` driver harness, you can create key instances
// which allow you to get access to a `verify()` function.
const assertionMethodPublicKey = await cryptoLd.from(assertionMethodData);
const {verify} = assertionMethodPublicKey.verifier();
```

`publicMethodFor` will return `undefined` if no key is found for a given
purpose.

### Backwards Compatibility with the 2018/2019 Crypto Suites

By default, this `did:key` driver returns DID Documents that have the 2020
crypto suites for verification and key agreement.
If you need DID Documents that are using the 2018/2019 crypto suites,
you can customize the driver as follows.

```js
import {
  Ed25519VerificationKey2018
} from '@digitalbazaar/ed25519-verification-key-2018';
import * as didKey from '@digitalbazaar/did-method-key';

const didKeyDriver2018 = didKey.driver({
 verificationSuite: Ed25519VerificationKey2018
});

const did = 'did:key:z6MkpTHR8VNsBxYAAWHut2Geadd9jSwuBV8xRoAnwWsdvktH';
await didKeyDriver2018.get({did});
// ->
{
  '@context': [
    'https://www.w3.org/ns/did/v1',
    'https://w3id.org/security/suites/ed25519-2018/v1',
    'https://w3id.org/security/suites/x25519-2019/v1'
  ],
  id: 'did:key:z6MkpTHR8VNsBxYAAWHut2Geadd9jSwuBV8xRoAnwWsdvktH',
  verificationMethod: [{
    id: 'did:key:z6MkpTHR8VNsBxYAAWHut2Geadd9jSwuBV8xRoAnwWsdvktH#z6MkpTHR8VNsBxYAAWHut2Geadd9jSwuBV8xRoAnwWsdvktH',
    type: 'Ed25519VerificationKey2018',
    controller: 'did:key:z6MkpTHR8VNsBxYAAWHut2Geadd9jSwuBV8xRoAnwWsdvktH',
    publicKeyBase58: 'B12NYF8RrR3h41TDCTJojY59usg3mbtbjnFs7Eud1Y6u'
  }],
  // etc,
  keyAgreement: [{
    id: 'did:key:z6MkpTHR8VNsBxYAAWHut2Geadd9jSwuBV8xRoAnwWsdvktH#z6LSbysY2xFMRpGMhb7tFTLMpeuPRaqaWM1yECx2AtzE3KCc',
    type: 'X25519KeyAgreementKey2019',
    controller: 'did:key:z6MkpTHR8VNsBxYAAWHut2Geadd9jSwuBV8xRoAnwWsdvktH',
    publicKeyBase58: 'JhNWeSVLMYccCk7iopQW4guaSJTojqpMEELgSLhKwRr'
  }]
}
```

## Contribute

See [the contribute file](https://github.com/digitalbazaar/bedrock/blob/master/CONTRIBUTING.md)!

PRs accepted.

If editing the Readme, please conform to the
[standard-readme](https://github.com/RichardLitt/standard-readme) specification.

## Commercial Support

Commercial support for this library is available upon request from
Digital Bazaar: support@digitalbazaar.com

## License

[New BSD License (3-clause)](LICENSE) © Digital Bazaar

