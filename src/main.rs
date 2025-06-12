mod helper_functions;
mod obtaining_csv_with_users;

use grammers_client::types::Chat;
use helper_functions::{create_client_from_session, find_chat_by_name, ret_or_log_err_and_panic};
use log::{LevelFilter, debug, error, info};
use simple_logging::log_to_file;
use std::env;

#[tokio::main]
async fn main() {
    log_to_file(".log", LevelFilter::Debug).unwrap();
    dotenv::dotenv().expect("Needed a .env file");

    let chat_name = env::var("CHAT_NAME_TO_PARSE").expect("Need 'chat_name_to_parse' env variable");
    let csv_filepath = format!("{}.csv", &chat_name);

    let client = create_client_from_session("session", false).await;
    let chat = match find_chat_by_name(client.iter_dialogs(), &chat_name).await {
        None => {
            error!("Can't find chat with name {name}", name = &chat_name);
            panic!();
        }
        Some(ch) => ch,
    };
    // TODO: remove in the future
    if !std::fs::exists(&csv_filepath).expect("Somehow can't check whether a path exists") {
        info!("Generating {path}.csv", path = &csv_filepath);
        ret_or_log_err_and_panic(
            obtaining_csv_with_users::generate_csv(&client, &chat).await,
            "trying to generate csv",
        );
    }

    let pings = ret_or_log_err_and_panic(
        obtaining_csv_with_users::parse_pings_from_file(&csv_filepath),
        "trying to get pings from csv",
    );
    info!("Pings are: {pings}");

    ping_all_via_bot(&chat, &pings).await;
    info!("Send all the pings");
}

#[allow(dead_code, unused_variables)]
async fn ping_all_via_bot(chat: &Chat, all_pings: &str) {
    let bot_client = create_client_from_session("bot_session", true).await;

    ret_or_log_err_and_panic(
        bot_client.send_message(chat, all_pings).await,
        "sending all pings",
    );
}
