use dotenv;
use log::{LevelFilter, debug, error, info};
use simple_logging::log_to_file;
use teloxide::{prelude::*, types::ChatId, utils::command::BotCommands};

trait Replyable {
    fn chat_id(&self) -> ChatId;

    async fn reply(&self, bot: &Bot, text: &str) -> ResponseResult<()> {
        bot.send_message(self.chat_id(), text).await?;
        return Ok(());
    }
}

impl Replyable for Message {
    fn chat_id(&self) -> ChatId {
        return self.chat.id;
    }
}

#[tokio::main]
async fn main() {
    assert!(log_to_file(".log", LevelFilter::Debug).is_ok());
    assert!(dotenv::dotenv().is_ok());

    info!("Booting up the bot");
    let bot = Bot::from_env();
    Command::repl(bot, handle_msg).await;
}

// type GroupToPing = String;

#[derive(BotCommands, Clone)]
#[command(
    rename_rule = "lowercase",
    description = "Following commands are supported: "
)]
enum Command {
    #[command(description = "Display this text")]
    Help,
    #[command(description = "Ping all the members of the chat")]
    PingAll,
    // #[command(description="Ping a specific group of people")]
    // Ping(GroupToPing)
}

async fn handle_msg(bot: Bot, msg: Message, cmd: Command) -> ResponseResult<()> {
    match cmd {
        Command::Help => {
            bot.send_message(msg.chat.id, Command::descriptions().to_string())
                .await?;
        }
        Command::PingAll => {
            ping_all(bot, msg).await?;
        }
    }
    return Ok(());
}

async fn ping_all(bot: Bot, msg: Message) -> ResponseResult<()> {
    let user_id = msg.from.clone().unwrap().id; // this bot will not work with channels, so can unwrap
    let passing = msg.chat.is_group() || (msg.chat.is_supergroup() && !msg.chat.is_channel());
    if !passing {
        msg.reply(&bot, "This bot is only useful in group chats, sorry!")
            .await?;
        return Ok(());
    }

    info!("{} pinged everyone in group chat {} ({})", user_id, msg.chat.id, msg.chat.title().unwrap_or("No Title"));
    msg.reply(&bot, "Done!").await?;
    return Ok(());
}
