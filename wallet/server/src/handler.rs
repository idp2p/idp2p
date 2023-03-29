/*  

# Consent process

- Server initialize a consent with id(from, to, id, )
- User accept the consent(id) with assertion key. The consent includes server consent. 

# Login 

- When a user wants connecting to a server, firstly it retrieves server configuration from http://server.com/id.json
- Then user verifies configuration to communicate, sends login request to server(state, sdt). If state is not valid sends error.
- Server creates a cookie for the user

*/

pub struct IdServerConfiguration {
    pub id: String,
    pub keys: Sdt
}