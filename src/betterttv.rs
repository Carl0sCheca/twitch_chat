use leptos::wasm_bindgen::JsCast;
use leptos::*;
use regex::{Captures, Regex};
use serde::Deserialize;
use wasm_bindgen_futures::JsFuture;
use web_sys::{Request, RequestInit, RequestMode, Response};

#[derive(Default, Debug, Clone)]
pub struct BetterTTV {
    global_emotes: Vec<BetterTTVEmote>,
    shared_emotes: Vec<BetterTTVEmote>,
    combined_emotes: Vec<BetterTTVEmote>,
    regex_precompiled: Option<Regex>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BetterTTVEmote {
    pub id: String,
    pub code: String,
    pub animated: bool,
}

impl BetterTTV {
    pub async fn load_global_emotes(&mut self) {
        let mut opts = RequestInit::new();
        opts.method("GET");
        opts.mode(RequestMode::Cors);

        let url = "https://api.betterttv.net/3/cached/emotes/global";
        let request = Request::new_with_str_and_init(url, &opts).unwrap();
        let window = web_sys::window().unwrap();
        let resp_value = JsFuture::from(window.fetch_with_request(&request))
            .await
            .unwrap();
        let resp: Response = resp_value.dyn_into().unwrap();
        let json = JsFuture::from(resp.json().unwrap()).await.unwrap();
        let emotes = serde_wasm_bindgen::from_value::<Vec<BetterTTVEmote>>(json).unwrap();

        emotes.iter().for_each(|emote| {
            self.global_emotes.push(emote.clone());
        });
    }

    pub async fn load_shared_emotes(&mut self, user_id: String) {
        let mut opts = RequestInit::new();
        opts.method("GET");
        opts.mode(RequestMode::Cors);

        let url = &format!(
            "https://api.betterttv.net/3/cached/users/twitch/{}",
            user_id
        );
        let request = Request::new_with_str_and_init(url, &opts).unwrap();
        let window = web_sys::window().unwrap();
        let resp_value = JsFuture::from(window.fetch_with_request(&request))
            .await
            .unwrap();
        let resp: Response = resp_value.dyn_into().unwrap();

        if resp.status() != 200 {
            return;
        }

        let json = JsFuture::from(resp.json().unwrap()).await.unwrap();

        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct SharedEmotes {
            shared_emotes: Vec<BetterTTVEmote>,
        }

        let emotes = serde_wasm_bindgen::from_value::<SharedEmotes>(json).unwrap();

        emotes.shared_emotes.iter().for_each(|emote| {
            self.shared_emotes.push(emote.clone());
        });
    }

    pub fn precompile_regex(&mut self) {
        let mut emotes = vec![];
        emotes.extend(self.global_emotes.clone());
        emotes.extend(self.shared_emotes.clone());
        let re = Regex::new(&format!(
            r"\b({})\b",
            emotes
                .iter()
                .map(|emote| regex::escape(&emote.code))
                .collect::<Vec<_>>()
                .join("|")
        ))
        .unwrap();

        self.regex_precompiled = Some(re);
        self.combined_emotes = emotes;
    }

    pub fn parse_emotes(&self, message: String) -> String {
        self.regex_precompiled
            .as_ref()
            .unwrap()
            .replace_all(&message, |caps: &Captures| {
                let matched_emote = &caps[0]; // Use caps[0] to access the matched emote name
                let emote = self
                    .combined_emotes
                    .iter()
                    .find(|emote| emote.code == matched_emote)
                    .unwrap();
                format!(
                    "<img src=\"https://cdn.betterttv.net/emote/{}/2x\" />",
                    emote.id
                )
            })
            .into_owned()
    }
}
