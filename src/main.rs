mod app;
mod betterttv;
mod comments;
mod emotes;
mod seventv;
mod twitch;
mod url_hash;
mod url_query;

use app::*;

fn main() {
    leptos::mount_to_body(App)
}
