#[derive(Default, Debug)]
pub struct QueryParams {
    pub channel: Option<String>,
    pub token: Option<String>,
    pub username: Option<String>,
}

#[derive(Debug)]
pub enum Query {
    Empty,
    Values(QueryParams),
}

pub fn decode_request(window: web_sys::Window) -> Query {
    let query = window
        .location()
        .search()
        .expect("no search exists")
        .trim_start_matches('?')
        .to_owned();

    if !query.is_empty() {
        let mut params = QueryParams::default();

        let pairs: Vec<&str> = query.split('&').collect();

        for pair in pairs {
            let kv: Vec<&str> = pair.split('=').collect();
            if kv.len() == 2 {
                let key = kv[0];
                let value = kv[1];

                if !value.is_empty() {
                    match key {
                        "channel" => params.channel = Some(value.to_string()),
                        "token" => params.token = Some(value.to_string()),
                        "username" => params.username = Some(value.to_string()),
                        _ => {}
                    }
                }
            }
        }

        Query::Values(params)
    } else {
        Query::Empty
    }
}
