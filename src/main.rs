mod obtaining_csv_with_users;

use grammers_client::client::dialogs::DialogIter;
use grammers_client::types::Chat;
use grammers_client::{Client, Config, SignInError};
use grammers_session::Session;
use log::{LevelFilter, debug, error, info};
use simple_logging::log_to_file;
use std::env;
use std::fmt::{Display};
use std::io::{BufRead, Write};

fn ret_or_log_err_and_panic<T, E>(res: Result<T, E>, location_of_error: &str) -> T
where
    E: Display,
{
    match res {
        Ok(v) => return v,
        Err(e) => {
            error!("Error while {location_of_error}: \n{e}");
            panic!();
        }
    }
}

#[tokio::main]
async fn main() {
    log_to_file(".log", LevelFilter::Debug).unwrap();

    // 1. Read BOT_TOKEN and chat username
    dotenv::dotenv().expect("Should have a .env file");
    let api_id = env::var("API_ID").unwrap();
    let api_hash = env::var("API_HASH").unwrap();
    let chat_name = env::var("CHAT_NAME_TO_PARSE").unwrap();

    // 2. Initialize session and connect
    let session =
        Session::load_file_or_create("session.session").expect("Failed to create session file");
    let config = Config {
        session,
        api_id: api_id.parse().unwrap(),
        api_hash,
        params: Default::default(),
    };
    let client =
        ret_or_log_err_and_panic(Client::connect(config).await, "connecting to the client");
    info!("Connected to the client");

    let is_authed =
        ret_or_log_err_and_panic(client.is_authorized().await, "checking for authorisation");
    if is_authed {
        info!("Already signed in");
    } else {
        info!("Signing in");
        let phone = ask_user_input("Enter the phone in international format: ");
        let token = ret_or_log_err_and_panic(
            client.request_login_code(&phone).await,
            "requesting login code",
        );
        let code = ask_user_input("Enter the code you received: ");
        match client.sign_in(&token, &code).await {
            Err(SignInError::PasswordRequired(password_token)) => {
                let hint = password_token.hint().unwrap_or("no hint");
                let password = ask_user_input(&format!("Enter the password: (hint: {hint})"));
                let _ = ret_or_log_err_and_panic(
                    client.check_password(password_token, password).await,
                    "checking password",
                );
            }
            Ok(_) => {}
            Err(e) => {
                error!("Got error while trying to log in: \n{e}");
                panic!();
            }
        }
        info!("Successfully logged in");
        client
            .session()
            .save_to_file("session.session")
            .expect("Failed to save session to the file");
    }

    let csv_filepath = format!("{}.csv", &chat_name);
    if !std::fs::exists(&csv_filepath).expect("Somehow can't check whether a path exists") {
        info!("Trying to generate csv");
        ret_or_log_err_and_panic(
            obtaining_csv_with_users::generate_csv(&client, &chat_name).await,
            "trying to generate csv",
        );
    }


    let pings = ret_or_log_err_and_panic(
        obtaining_csv_with_users::get_pings_from_file(&csv_filepath),
        "trying to get pings from csv",
    );
    info!("Pings are: {pings}");

    let packed_chat = match find_chat_by_name(client.iter_dialogs(), &chat_name).await {
        None => {
            error!("Can't find chat: {}", &chat_name);
            panic!();
        }
        Some(ch) => ch.pack()
    };
    ret_or_log_err_and_panic(client.send_message(packed_chat, pings).await, "sending ping for everyone");
    info!("Send all the pings");
    // ping_all_via_bot(&chat_name, &pings).await;
}

fn ask_user_input(prompt: &str) -> String {
    let mut stdout = std::io::stdout().lock();
    stdout
        .write_all(prompt.as_bytes())
        .expect("Failed to write into stdout");
    stdout.flush().expect("Failed to flush into stdout");

    let mut stdin = std::io::stdin().lock();
    let mut user_input = String::new();
    stdin
        .read_line(&mut user_input)
        .expect("Failed to read an input");
    return user_input;
}

#[allow(dead_code, unused_variables)]
async fn ping_all_via_bot(chat_name: &str, all_pings: &str) {
    let bot_token = env::var("BOT_TOKEN").unwrap();
    let bot_session = Session::load_file_or_create("bot_session.session")
        .expect("Failed to created session file");
    let bot_config = Config {
        session: bot_session,
        api_id: 0,
        api_hash: "".into(),
        params: Default::default(),
    };
    let bot_client = ret_or_log_err_and_panic(
        Client::connect(bot_config).await,
        "connecting to bot session",
    );
    let bot = ret_or_log_err_and_panic(
        bot_client.bot_sign_in(&bot_token).await,
        "signing in into the bot",
    );
    if bot.is_bot() {
        info!("Signed in into the bot {}", bot.full_name());
    } else {
        error!("The signed-in account was somehow not a bot");
        panic!();
    }

    let packed_chat = match find_chat_by_name(bot_client.iter_dialogs(), chat_name).await {
        Some(ch) => {
            info!("bot found chat {}", ch.name());
            ch.pack()
        }
        None => {
            error!("bot can't find chat with a following name: {chat_name}");
            panic!();
        }
    };
    ret_or_log_err_and_panic(bot_client.send_message(packed_chat, all_pings).await, "sending all pings");
}

pub async fn find_chat_by_name(mut dialogs: DialogIter, chat_name: &str) -> Option<Chat> {
    while let Some(i_chat) =
        ret_or_log_err_and_panic(dialogs.next().await, "iterating over bot dialogs")
    {
        if i_chat.chat().name() == chat_name {
            return Some(i_chat.chat);
        }
    }
    return None;
}
