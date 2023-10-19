use leptos::wasm_bindgen::JsCast;
use serde::Deserialize;
use wasm_bindgen_futures::JsFuture;
use web_sys::{Request, RequestInit, RequestMode, Response};

#[derive(Default, Debug, Clone)]
pub struct BetterTTV {
    global_emotes: Vec<BetterTTVEmote>,
    shared_emotes: Vec<BetterTTVEmote>,
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

    pub async fn parse_emotes(&self, mut message: String) -> String {
        self.global_emotes.iter().for_each(|emote| {
            message = message.replace(
                &emote.code,
                &format!(
                    "<img src=\"https://cdn.betterttv.net/emote/{}/2x\" />",
                    emote.id
                ),
            );
        });

        self.shared_emotes.iter().for_each(|emote| {
            message = message.replace(
                &emote.code,
                &format!(
                    "<img src=\"https://cdn.betterttv.net/emote/{}/2x\" />",
                    emote.id
                ),
            );
        });

        message
    }
}
