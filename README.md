# idp2p

> `Experimental`, inspired by `ipfs`, `did:peer` and `keri`

A self-describing identity protocol on top of libp2p. 

## FAQ

**Question:** Why idp2p?

**Answer:** Idp2p is .

**Question:** Is idp2p a DIDs method?

**Answer:** How idp2p try to solve privacy?


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

