use std::{collections::VecDeque, time::Duration};

use crate::comments::*;
use crate::twitch::ChatTypeMessage;
use crate::url_query::QueryParams;
use leptos::{
    leptos_dom::logging::{console_error, console_log},
    *,
};

#[component]
pub fn App() -> impl IntoView {
    let (messages, set_messages) = create_signal(VecDeque::new());

    console_log("Creating connection");
    let params = crate::url_query::decode_request(window());

    let query_params = match params {
        crate::url_query::Query::Empty
        | crate::url_query::Query::Values(
            QueryParams { channel: None, .. }
            | QueryParams { token: None, .. }
            | QueryParams { username: None, .. },
        ) => {
            console_error("You have to set a channel, token and username");
            std::process::exit(1)
        }
        crate::url_query::Query::Values(query_params) => query_params,
    };

    let mut client = wasm_sockets::EventClient::new("wss://irc-ws.chat.twitch.tv:443")
        .expect("Cannot connect to the server");

    client.set_on_error(Some(Box::new(|error| {
        console_log(format!("{:#?}", error).as_str());
    })));

    let channel = query_params.channel.as_ref().unwrap().clone();
    client.set_on_connection(Some(Box::new(move |client: &wasm_sockets::EventClient| {
        // console_log(format!("{:#?}", client.status).as_str());
        // console_log(format!("Sending message...").as_str());
        client
            .send_string(
                format!(
                    "PASS oauth:{}",
                    query_params.token.as_ref().unwrap().as_str()
                )
                .as_str(),
            )
            .unwrap();
        client
            .send_string("CAP REQ :twitch.tv/commands twitch.tv/tags")
            .unwrap();
        client
            .send_string(
                format!("NICK {}", query_params.username.as_ref().unwrap().as_str()).as_str(),
            )
            .unwrap();
        client
            .send_string(format!("JOIN #{}", channel).as_str())
            .unwrap();
    })));

    client.set_on_close(Some(Box::new(|_evt| {
        console_log(format!("Connection closed").as_str());
    })));

    client.set_on_message(Some(Box::new(
        move |_client: &wasm_sockets::EventClient, message: wasm_sockets::Message| {
            let msg = match message {
                wasm_sockets::Message::Text(text) => text,
                _ => "".to_owned(),
            };

            if msg.is_empty() || !msg.contains("PRIVMSG") {
                return;
            }

            let twitch_msg = crate::twitch::parse_twitch_message(&msg);

            // console_log(format!("New Message: {}", &msg).as_str());

            if let ChatTypeMessage::Message(twitch_message) = twitch_msg {
                set_messages.update(|f| f.push_back(twitch_message));

                let element = document().get_element_by_id("app").unwrap();
                element.scroll_into_view_with_bool(false);

                    if messages.get_untracked().len() > 14 {
                    set_messages.update(|f| {
                        (0..6).for_each(|_| {
                            f.pop_front();
                        });
                    });
                } else {
                    // set_timeout(
                    //     move || {
                    //         if messages.get().len() > 0 {
                    //             set_messages.update(|f| {
                    //                 f.pop_front();
                    //             });
                    //         }
                    //     },
                    //     Duration::from_secs(10),
                    // );
                }
            }
        },
    )));

    set_interval(
        move || {
            client.send_string("PING").unwrap();
        },
        Duration::from_secs(60),
    );

    view! {
        <div id="app">
            <Comments messages=messages />
        </div>

        <svg xmlns="http://www.w3.org/2000/svg" version="1.1">
        <defs>
            <filter id="old-goo">
            <feGaussianBlur in="SourceGraphic" stdDeviation="10" result="blur" />
            <feColorMatrix in="blur" mode="matrix" values="1 0 0 0 0  0 1 0 0 0  0 0 1 0 0  0 0 0 18 -7" result="goo" />
            <feBlend in="SourceGraphic" in2="goo" />
            </filter>
            <filter id="fancy-goo">
            <feGaussianBlur in="SourceGraphic" stdDeviation="10" result="blur" />
            <feColorMatrix in="blur" mode="matrix" values="1 0 0 0 0  0 1 0 0 0  0 0 1 0 0  0 0 0 19 -9" result="goo" />
            <feComposite in="SourceGraphic" in2="goo" operator="atop" />
            </filter>
        </defs>
        </svg>
    }
}
