use crate::find_chat_by_name;
use grammers_client::{Client, types::PackedChat};
use log::{error, info};
use serde::{Deserialize, Serialize};
use std::error::Error;
use tokio::time::{Duration, sleep};

#[derive(Serialize, Deserialize)]
struct MyUser {
    id: i64,
    full_name: String,
    tg_username: Option<String>,
}
impl MyUser {
    fn get_mention(&self) -> String {
        return match &self.tg_username {
            None => format!("@{}({})", self.id, &self.full_name),
            Some(username) => format!("@{}", &username),
        };
    }
}

pub async fn generate_csv(client: &Client, chat_name: &str) -> Result<(), Box<dyn Error>> {
    // let chat_name = env::var("CHAT_NAME_TO_PARSE").unwrap();
    // 3. Resolve chat username into PackedChat
    let packed_chat = match find_chat_by_name(client.iter_dialogs(), chat_name).await {
        None => {
            error!("Can't find chat with name {chat_name}");
            panic!();
        }
        Some(ch) => {
            info!("Found chat {}", ch.name());
            ch.pack()
        }
    };

    // 4. Export participants to CSV
    let csv_filepath = format!("{chat_name}.csv");
    let mut wtr = csv::Writer::from_path(&csv_filepath)?;

    // the csv header is used automatically when serializing from a struct
    let mut iter = client.iter_participants(packed_chat);
    while let Some(part) = iter.next().await? {
        let user = &part.user;
        let my_user = MyUser {
            id: user.id(),
            full_name: user.full_name(),
            tg_username: user.username().map(String::from),
        };
        wtr.serialize(my_user)?;
        sleep(Duration::from_millis(25)).await;
    }
    wtr.flush()?;
    info!("Exported participants to {path}", path = &csv_filepath);

    return Ok(());
}

pub fn get_pings_from_file(filepath: &str) -> Result<String, Box<dyn Error>> {
    // 5. Read CSV and ping each user
    let mut result_message = String::new(); // the final message with space-separated mentions
    let mut rdr = csv::Reader::from_path(filepath)?;
    let iter = rdr.deserialize();

    for result in iter {
        let user: MyUser = result?;
        result_message.push_str(&user.get_mention());
        result_message.push(' ');
    }

    return Ok(result_message);
}
