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

