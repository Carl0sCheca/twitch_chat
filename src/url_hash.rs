pub enum Hash {
    Empty,
    Value(String),
}

pub fn decode_request(window: web_sys::Window) -> Hash {
    let hash = window
        .location()
        .hash()
        .expect("no search exists")
        .trim_start_matches('#')
        .to_owned();

    if !hash.is_empty() {
        let mut access_token = "";

        let pairs: Vec<&str> = hash.split('&').collect();

        for pair in pairs {
            let kv: Vec<&str> = pair.split('=').collect();
            if kv.len() == 2 {
                let key = kv[0];
                let value = kv[1];

                if !value.is_empty() && key == "access_token" {
                    access_token = value;
                }
            }
        }

        if !access_token.is_empty() {
            window.location().set_hash("").unwrap();
            return Hash::Value(access_token.to_string());
        }
    }

    Hash::Empty
}
