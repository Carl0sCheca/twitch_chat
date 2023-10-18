mod app;
mod comments;
mod twitch;
mod url_hash;
mod url_query;

use app::*;

fn main() {
    leptos::mount_to_body(App)
}
