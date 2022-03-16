# idp2p-rust

> `Experimental`, inspired by `ipfs`, `did:peer` and `keri`

[See idp2p spec and demo](https://idp2p.github.io)

## Getting Started 

#### Generate peers

- ```cargo run --example demo```
- ```cargo run --example demo -- -p 5000```

#### Create identity

- cmd: ```create <name>```
- ex: `create alice` and `create bob`

#### Subscribe to identity

- cmd: ```get <id>```
- ex: `get did:p2p:bagaaieraam4`

#### Send message

- cmd: ```send <message> to did:p2p:bagaaieraam4```

## Contributions

The idp2p `rust` implementation is work in progress. 

Contributions are most welcome

## License

[Apache License 2.0](LICENSE) 

