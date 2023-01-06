pub struct Node {
    key: String,
    children: Vec<Proof>,
    // children [key, proof] or raw string
}

pub struct Proof {
    key: String,
    value: String,
}

/*

 Give me /addresses/home/zipcode

   "/": {
        "assertion_keys": "zhassertions",
        "authentication_keys": "zhauths",
        "adresses": "zaddr"
    }

   "/adresses": {
        "home": "zhome",
        "work": "zwork"
    },

    "/adresses/home": {
        "zipcode": "zzip"
    },

    {
        "value": "2020", // may be zkp proof
        "nonce": "123456"
    }
*/
