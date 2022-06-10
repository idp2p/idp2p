@0x9e1812b9b1d9d4e5;

struct IdentityEvent { 
    union{
        createAssertionKey @0 :Idp2pVerificationKey;
        createAuthenticationKey @1 :Idp2pVerificationKey;
        createAgreementKey @2 :Idp2pVerificationKey;
        setProof @3 :Idp2pProof;
        revokeAssertionKey @4 :Data;
        revokeAuthenticationKey @5 :Data;
        revokeAgreementKey @6 :Data;
    }
}

struct Idp2pProof{
    key @0 :Data;
    value @1 :Data;
}

struct Idp2pVerificationKey{
    id @0 :Data;
    value @1 :Data;
}

struct IdentityInception {
    timestamp @0 :Int64;
    nextKeyDigest @1 :Data;
    recoveryKeyDigest @2 :Data;
    events @3 :List(IdentityEvent);
}

struct Microledger {
    inception @0 :Data;
    eventLogs @1 :List(EventLog);
}

struct EventLog{
    eventId @0 :Data;
    payload @1 :Data;
    proof @2 :Data;
}

struct EventLogPayload{
    previous @0 :Data;
    signerKey @1 :Data; 
    nextKeyDigest @2 :Data;
    timestamp @3 :Int64;
    change :union {
        recover @4 :Data;
        events @5 :List(IdentityEvent);
    }
}