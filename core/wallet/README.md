# Wallet core


#### Request

When a client needs to connect user wallet it should send the following info:

- Subject: This consent is valid for that subject
- Context: This consent is valid for that root context
- Disclosure: Reveal personal info
- Authentication
- Assertions(Payment?)


#### Admin 

- login(pwd) -> token
- update(token, data)
- message(msg)
- logout()
