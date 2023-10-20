use std::{collections::VecDeque, sync::Arc, time::Duration};

use crate::comments::*;
use crate::twitch::ChatTypeMessage;
use crate::url_query::QueryParams;
use leptos::{
    leptos_dom::logging::{console_error, console_log},
    *,
};
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::{Request, RequestInit, RequestMode, Response};

const CLIENT_ID: &str = "gyy1g5ph81fi3ytmen3uf59oi2xgk9";

#[component]
pub fn App() -> impl IntoView {
    let (messages, set_messages) = create_signal(VecDeque::new());

    let hash = crate::url_hash::decode_request(window());
    match hash {
        crate::url_hash::Hash::Empty => (),
        crate::url_hash::Hash::Value(hash) => {
            return view! {"" <div>{hash}</div> };
        }
    }

    console_log("Creating connection");
    let params = crate::url_query::decode_request(window());

    let query_params = match params {
        crate::url_query::Query::Empty
        | crate::url_query::Query::Values(
            QueryParams { token: None, .. } | QueryParams { channel: None, .. },
        ) => {
            console_error("You have to set a channel and token");
            std::process::exit(1)
        }
        crate::url_query::Query::Values(query_params) => query_params,
    };

    spawn_local(async move {
        let (username, user_id) = {
            let mut opts = RequestInit::new();
            opts.method("GET");
            opts.mode(RequestMode::Cors);

            let url = &format!(
                "https://api.twitch.tv/helix/users?login={}",
                query_params.channel.as_ref().unwrap().clone()
            );
            let request = Request::new_with_str_and_init(url, &opts).unwrap();
            request
                .headers()
                .set(
                    "Authorization",
                    format!("Bearer {}", query_params.token.as_ref().unwrap()).as_str(),
                )
                .unwrap();
            request.headers().set("Client-Id", CLIENT_ID).unwrap();

            let resp_value = JsFuture::from(window().fetch_with_request(&request))
                .await
                .unwrap();
            let resp: Response = resp_value.dyn_into().unwrap();
            let json = JsFuture::from(resp.json().unwrap()).await.unwrap();

            let value = serde_wasm_bindgen::from_value::<serde_json::Value>(json).unwrap();
            let value = value["data"].as_array().unwrap();
            let value = value[0].clone();

            let username = value["display_name"].as_str().unwrap_or("").to_owned();

            let user_id = value["id"].as_str().unwrap_or("").to_owned();

            (username, user_id)
        };

        if username.is_empty() {
            console_error("Error getting username");
            std::process::exit(1)
        }

        let mut client = wasm_sockets::EventClient::new("wss://irc-ws.chat.twitch.tv:443")
            .expect("Cannot connect to the server");

        client.set_on_error(Some(Box::new(|error| {
            console_log(format!("{:#?}", error).as_str());
        })));

        let token = query_params.token.as_ref().unwrap().clone();
        let channel = query_params.channel.as_ref().unwrap().clone();
        client.set_on_connection(Some(Box::new(move |client: &wasm_sockets::EventClient| {
            // console_log(format!("{:#?}", client.status).as_str());
            // console_log(format!("Sending message...").as_str());
            client
                .send_string(format!("PASS oauth:{}", token).as_str())
                .unwrap();
            client
                .send_string("CAP REQ :twitch.tv/commands twitch.tv/tags")
                .unwrap();
            client
                .send_string(format!("NICK {}", username.as_str()).as_str())
                .unwrap();
            client
                .send_string(format!("JOIN #{}", channel).as_str())
                .unwrap();
        })));

        client.set_on_close(Some(Box::new(|_evt| {
            console_log("Connection closed");
        })));

        let mut betterttv = crate::betterttv::BetterTTV::default();
        betterttv.load_global_emotes().await;
        betterttv.load_shared_emotes(user_id.clone()).await;

        let mut seventv = crate::seventv::SevenTv::default();
        seventv.load_global_emotes().await;
        seventv.load_shared_emotes(user_id.clone()).await;

        let mut emotes = crate::emotes::Emotes::default();
        emotes.load_emotes(
            betterttv
                .emotes
                .iter()
                .map(|f| crate::emotes::Emote {
                    id: f.id.clone(),
                    code: f.code.clone(),
                    provider: crate::emotes::Provider::BetterTTV,
                })
                .collect(),
        );
        emotes.load_emotes(
            seventv
                .emotes
                .iter()
                .map(|f| crate::emotes::Emote {
                    id: f.id.clone(),
                    code: f.name.clone(),
                    provider: crate::emotes::Provider::SevenTV,
                })
                .collect(),
        );
        emotes.precompile_emotes();

        let emotes = Arc::new(emotes);

        client.set_on_message(Some(Box::new(
            move |_client: &wasm_sockets::EventClient, message: wasm_sockets::Message| {
                let channel = query_params.channel.as_ref().unwrap().clone();
                let emotes = emotes.clone();
                spawn_local(async move {
                    let msg = match message {
                        wasm_sockets::Message::Text(text) => text,
                        _ => "".to_owned(),
                    };

                    if msg.is_empty() || !msg.contains("PRIVMSG") {
                        return;
                    }
                    // console_log(format!("New Message: {}", &msg).as_str());

                    let twitch_msg: ChatTypeMessage =
                        crate::twitch::parse_twitch_message(&msg, channel);

                    if let ChatTypeMessage::Message(mut twitch_message) = twitch_msg {
                        let message = crate::twitch::parse_emotes(
                            twitch_message.message_body,
                            &twitch_message.emotes,
                        );
                        let message = emotes.parse_emotes(message);

                        twitch_message.message_body = message;

                        set_messages.update(|f| f.push_front(twitch_message));

                        let element = document().get_element_by_id("app").unwrap();
                        element.scroll_into_view_with_bool(false);

                        if messages.get_untracked().len() > 20 {
                            set_messages.update(|f| {
                                (0..9).for_each(|_| {
                                    f.pop_back();
                                });
                            });
                        }
                    }
                });
            },
        )));

        set_interval(
            move || {
                client.send_string("PING").unwrap();
            },
            Duration::from_secs(60),
        );
    });

    view! {
        <div id="app">
            <Comments messages=messages />
        </div>

        <svg xmlns="http://www.w3.org/2000/svg" version="1.1">
            <defs>
                <filter id="fancy-goo">
                <feGaussianBlur in="SourceGraphic" stdDeviation="10" result="blur" />
                <feColorMatrix in="blur" mode="matrix" values="1 0 0 0 0  0 1 0 0 0  0 0 1 0 0  0 0 0 19 -9" result="goo" />
                <feComposite in="SourceGraphic" in2="goo" operator="atop" />
                </filter>
            </defs>
        </svg>
    }
}
