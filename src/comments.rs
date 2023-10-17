use leptos::*;

use crate::twitch::TwitchChatMessage;
use std::collections::VecDeque;

#[component]
pub fn Comments(messages: ReadSignal<VecDeque<TwitchChatMessage>>) -> impl IntoView {
    view! {
        <For
            each=move || messages.get()
            key=|i| i.id
            children=|item| view! {
                <div class="dialogue">
                    <div class="dialogue-blobs">
                        <div class="dialogue-blob-top"></div>
                        <div class="dialogue-blob-bottom"></div>
                        <div class="dialogue-text">
                            {item.message_body}
                        </div>
                    </div>
                    <div class="dialogue-character-wrap">
                        <div class="dialogue-character" style={format!("background-color:{}", item.color.unwrap_or("".to_owned()))}>
                            <span>{item.display_name}</span>
                        </div>
                    </div>
                </div>
             }
        />
    }
}
