use leptos::wasm_bindgen::JsCast;
use leptos::*;
use regex::{Captures, Regex};
use serde::Deserialize;
use wasm_bindgen_futures::JsFuture;
use web_sys::{Request, RequestInit, RequestMode, Response};

#[derive(Default, Debug, Clone)]
pub struct SevenTv {
    global_emotes: Vec<SevenTvEmote>,
    shared_emotes: Vec<SevenTvEmote>,
    combined_emotes: Vec<SevenTvEmote>,
    regex_precompiled: Option<Regex>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SevenTvEmote {
    pub id: String,
    pub name: String,
}

impl SevenTv {
    pub async fn load_global_emotes(&mut self) {
        let mut opts = RequestInit::new();
        opts.method("GET");
        opts.mode(RequestMode::Cors);

        let url = "https://api.7tv.app/v2/emotes/global";
        let request = Request::new_with_str_and_init(url, &opts).unwrap();
        let window = web_sys::window().unwrap();
        let resp_value = JsFuture::from(window.fetch_with_request(&request))
            .await
            .unwrap();
        let resp: Response = resp_value.dyn_into().unwrap();
        let json = JsFuture::from(resp.json().unwrap()).await.unwrap();
        let emotes = serde_wasm_bindgen::from_value::<Vec<SevenTvEmote>>(json).unwrap();

        emotes.iter().for_each(|emote| {
            self.global_emotes.push(emote.clone());
        });
    }

    pub async fn load_shared_emotes(&mut self, user_id: String) {
        let mut opts = RequestInit::new();
        opts.method("GET");
        opts.mode(RequestMode::Cors);

        let url = &format!("https://api.7tv.app/v2/users/{}/emotes", user_id);
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

        let emotes = serde_wasm_bindgen::from_value::<Vec<SevenTvEmote>>(json).unwrap();

        emotes.iter().for_each(|emote| {
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
                .map(|emote| regex::escape(&emote.name))
                .collect::<Vec<_>>()
                .join("|")
        ))
        .unwrap();

        self.regex_precompiled = Some(re);
        self.combined_emotes = emotes;
    }

    pub fn parse_emotes(&self, message: String) -> String {
        let re = self.regex_precompiled.as_ref().unwrap();

        re.replace_all(&message, |caps: &Captures| {
            let matched_emote = &caps[0]; // Use caps[0] to access the matched emote name
            let emote = self
                .combined_emotes
                .iter()
                .find(|emote| emote.name == matched_emote)
                .unwrap();
            format!(
                "<img src=\"https://cdn.7tv.app/emote/{}/2x.webp\" />",
                emote.id
            )
        })
        .into_owned()
    }
}
