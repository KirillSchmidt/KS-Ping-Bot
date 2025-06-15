mod csv_related;
mod helper_functions;

use grammers_client::types::Chat;
use helper_functions::{create_client_from_session, find_chat_by_name, ret_or_log_err_and_panic};
use log::{LevelFilter, error, info};
use simple_logging::log_to_file;
use std::env;

#[tokio::main]
async fn main() {
    log_to_file(".log", LevelFilter::Debug).unwrap();
    dotenv::dotenv().expect("Needed a .env file");

    let client = create_client_from_session("session", false).await;

    let chat_name = env::var("CHAT_NAME").expect("Need 'CHAT_NAME' env variable");
    let chat = match find_chat_by_name(client.iter_dialogs(), &chat_name).await {
        None => {
            error!("Can't find chat with name {name}", name = &chat_name);
            panic!();
        }
        Some(ch) => ch,
    };
    let chat_id = chat.id();

    ret_or_log_err_and_panic(
        csv_related::generate_csv(&client, &chat).await,
        "trying to generate csv",
    );
    info!("Updated {chat_id}");

    let pings = ret_or_log_err_and_panic(
        csv_related::parse_pings_of_chat(&chat),
        "trying to get pings from csv",
    );
    info!("Generated pings for {chat_id}");

    ping_all_via_bot(&chat, &pings).await;
    info!("Sent all pings for {chat_id}");
}

async fn ping_all_via_bot(client_chat: &Chat, all_pings: &str) {
    let bot_client = create_client_from_session("bot_session", true).await;

    let required_chat = match client_chat.username() {
        None => client_chat.pack(),
        Some(chat_username) => bot_client
            .resolve_username(chat_username)
            .await
            .expect("Can't resolve username")
            .expect("No such chat found")
            .pack(),
    };
    ret_or_log_err_and_panic(
        bot_client.send_message(required_chat, all_pings).await,
        "sending all pings",
    );
}
