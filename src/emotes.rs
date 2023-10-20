pub struct Emote {
    pub id: String,
    pub code: String,
    pub provider: Provider,
}

pub enum Provider {
    BetterTTV,
    SevenTV,
}

impl Provider {
    fn get_url(&self, emote_id: &String) -> String {
        match self {
            Provider::BetterTTV => format!(
                "<img src=\"https://cdn.betterttv.net/emote/{}/2x\" />",
                emote_id
            ),
            Provider::SevenTV => format!(
                "<img src=\"https://cdn.7tv.app/emote/{}/2x.webp\" />",
                emote_id
            ),
        }
    }
}

#[derive(Default)]
pub struct Emotes {
    emotes: Vec<Emote>,
    precompiled_regex: Option<regex::Regex>,
}

impl Emotes {
    pub fn load_emotes(&mut self, emotes: Vec<Emote>) {
        self.emotes.extend(emotes);
    }

    pub fn precompile_emotes(&mut self) {
        let re = regex::Regex::new(&format!(
            r"\b({})\b",
            self.emotes
                .iter()
                .map(|emote| regex::escape(&emote.code))
                .collect::<Vec<_>>()
                .join("|")
        ))
        .unwrap();

        self.precompiled_regex = Some(re);
    }

    pub fn parse_emotes(&self, message: String) -> String {
        let re = self.precompiled_regex.as_ref().unwrap();

        re.replace_all(&message, |caps: &regex::Captures| {
            let matched_emote = &caps[0];
            let emote = self
                .emotes
                .iter()
                .find(|emote| emote.code == matched_emote)
                .unwrap();

            emote.provider.get_url(&emote.id)
        })
        .into_owned()
    }
}
