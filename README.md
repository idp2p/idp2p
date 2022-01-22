# idp2p-rust

> `Experimental`, inspired by `ipfs`, `did:peer` and `keri`

[See idp2p spec and demo](https://idp2p.github.io)

## Getting Started 

#### Generate peers

- ```cargo run -- -p 5000 -d ../target/alice```
- ```cargo run -- -p 6000 -d ../target/bob```

#### Create identity

- cmd: ```create <name>```
- ex: `create alice` and `create bob`

#### Subscribe to identity

- cmd: ```get <id>```
- ex: `get bagaaieraam4...`

#### Create DID Document

- cmd: ```set-document```

#### Recover

- cmd: ```recover```

## Contributions

The idp2p `rust` implementation is work in progress. 

Contributions are most welcome

## License

[Apache License 2.0](LICENSE) 

