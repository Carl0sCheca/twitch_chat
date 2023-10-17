#![allow(unused)]

use leptos::*;

#[derive(Default, Clone)]
pub struct TwitchChatMessage {
    pub id: usize,
    pub badge_info: Option<String>,
    pub badges: Vec<TwitchBadge>,
    pub client_nonce: Option<String>,
    pub color: Option<String>,
    pub display_name: String,
    pub emotes: Vec<TwitchEmote>,
    pub first_msg: u8,
    pub mod_flag: u8,
    // pub flags: String,
    // pub id: String,
    // pub reply_parent_display_name: Option<String>,
    // pub reply_parent_msg_body: Option<String>,
    // pub reply_parent_msg_id: Option<String>,
    // pub reply_parent_user_id: Option<u32>,
    // pub reply_parent_user_login: Option<String>,
    // pub reply_thread_parent_msg_id: Option<String>,
    // pub reply_thread_parent_user_login: Option<String>,
    // pub returning_chatter: u8,
    // pub room_id: u32,
    // pub subscriber: u8,
    // pub tmi_sent_ts: u64,
    // pub turbo: u8,
    // pub user_id: u32,
    // pub user_type: String,
    // pub username: String,
    // pub message_type: String,
    // pub channel_name: String,
    pub message_body: String,
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Hash)]
pub struct TwitchEmote {
    pub emote_id: String,
    pub range: Vec<TwitchEmoteRange>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TwitchEmoteRange {
    pub start: u32,
    pub end: u32,
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Hash)]
pub struct TwitchBadge {
    pub badge: String,
    pub version: u32,
}

pub enum ChatTypeMessage {
    Message(TwitchChatMessage),
    Other(String),
}

pub static mut VALUE: usize = 0;

pub fn parse_twitch_message(data: &String) -> ChatTypeMessage {
    let is_message = if let Some(first_char) = data.chars().next() {
        first_char == '@'
    } else {
        false
    };

    if !is_message {
        return ChatTypeMessage::Other(data.to_owned());
    }

    let pairs: Vec<&str> = data.split(';').collect();

    let mut message = TwitchChatMessage::default();

    for pair in pairs {
        let kv: Vec<&str> = pair.split('=').collect();
        if kv.len() == 2 {
            let key = kv[0];
            let value = kv[1];

            if !value.is_empty() {
                match key {
                    "badge_info" => message.badge_info = Some(value.to_string()),
                    "badges" => {
                        let badges: Vec<&str> = value.split(',').collect();
                        message.badges = badges
                            .iter()
                            .map(|f| {
                                let badge: Vec<&str> = f.split('/').collect();
                                TwitchBadge {
                                    badge: badge[0].to_owned(),
                                    version: badge[1].parse::<u32>().unwrap_or(0),
                                }
                            })
                            .collect::<Vec<_>>();
                    }
                    "client-nonce" => message.client_nonce = Some(value.to_string()),
                    "color" => message.color = Some(value.to_string()),
                    "display-name" => message.display_name = value.to_string(),
                    "emotes" => {
                        let emotes: Vec<&str> = value.split('/').collect();

                        let mut emotes_out: Vec<TwitchEmote> = vec![];

                        for emote in emotes {
                            let parts: Vec<&str> = emote.split(':').collect();
                            let emote_id = parts[0];
                            let positions: Vec<&str> = parts[1].split(',').collect();

                            for position in positions {
                                let range: Vec<&str> = position.split('-').collect();
                                let start = range[0].parse::<u32>().unwrap();
                                let end = range[1].parse::<u32>().unwrap();

                                emotes_out.push(TwitchEmote {
                                    emote_id: emote_id.to_owned(),
                                    range: vec![TwitchEmoteRange { start, end }],
                                });
                            }
                        }
                        message.emotes = emotes_out;
                    }
                    "mod" => message.mod_flag = value.parse::<u8>().unwrap_or(0),
                    _ => {}
                }
            }
        }

        if let Some(last_space_index) = data.rfind(" :") {
            let mut msg = data[last_space_index + 2..].to_string();

            let mut replacements = vec![];
            for emote in &message.emotes {
                replacements.push((emote.range[0].start as usize..emote.range[0].end as usize + 1, format!("<img src=\"https://static-cdn.jtvnw.net/emoticons/v2/{}/static/light/2.0\" />", emote.emote_id)));
            }

            replacements.sort_by(|a, b| b.0.start.cmp(&a.0.start));

            for (range, replacement) in replacements {
                msg.replace_range(range, replacement.as_str());
            }

            message.message_body = msg;
        }

        unsafe {
            message.id = VALUE;
            VALUE += 1;
        }
    }

    ChatTypeMessage::Message(message)
}
