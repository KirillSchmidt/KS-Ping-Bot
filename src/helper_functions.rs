use grammers_client::client::dialogs::DialogIter;
use grammers_client::types::Chat;
use grammers_client::{Client, Config, SignInError};
use grammers_session::Session;
use log::{error, info};
use std::env;
use std::fmt::Display;
use std::io::{BufRead, Write};

pub fn ret_or_log_err_and_panic<T, E>(res: Result<T, E>, location_of_error: &str) -> T
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

pub async fn create_client_from_session(session_name: &str, is_bot: bool) -> Client {
    dotenv::dotenv().expect("Should have a .env file");
    let api_id = env::var("API_ID").unwrap();
    let api_hash = env::var("API_HASH").unwrap();

    // 2. Initialize session and connect
    let session = Session::load_file_or_create(format!("{session_name}.session"))
        .expect("Failed to create session file");
    let config = Config {
        session,
        api_id: api_id.parse().unwrap(),
        api_hash,
        params: Default::default(),
    };

    let client =
        ret_or_log_err_and_panic(Client::connect(config).await, "connecting to the client");
    info!("Connected to the client");

    if is_bot {
        let bot_token = env::var("BOT_TOKEN").expect("should have BOT_TOKEN env variable");
        let bot_user =
            ret_or_log_err_and_panic(client.bot_sign_in(&bot_token).await, "signing into the bot");
        assert!(bot_user.is_bot());
    } else {
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
                    ret_or_log_err_and_panic(
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
        }
    }
    info!("saving {}", session_name);
    client
        .session()
        .save_to_file(format!("{session_name}.session"))
        .expect("Failed to save session to the file");
    return client;
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
