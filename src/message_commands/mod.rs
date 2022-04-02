use crate::Moranometer;
use anyhow::anyhow;
use teloxide::{prelude2::*, utils::command::BotCommand};

mod present_lists;
use present_lists::PresentLists;

#[derive(BotCommand, Clone)]
#[command(rename = "lowercase", description = "These commands are supported:")]
pub enum BasicCommands {
    #[command(description = "welcome to the MoranOmeter")]
    Start,
    #[command(description = "display this text.")]
    Help,
    #[command(description = "see your cards", rename = "get_cards")]
    MyCards,
}

pub(crate) async fn basic_commands_endpoint(
    msg: Message,
    bot: AutoSend<Bot>,
    cmd: BasicCommands,
    cfg: Moranometer,
) -> anyhow::Result<()> {
    let user_id = msg.from().ok_or(anyhow!("No user id"))?.id;
    let user_name = &msg.from().ok_or(anyhow!("No name found"))?.first_name;

    log::info!("got request from {user_name}, {user_id}");

    let mut cfg = cfg.lock().await;
    cfg.users.add(user_name, user_id).await?;
    let user = cfg.users.get_user(user_id).unwrap();

    match cmd {
        BasicCommands::Help => {
            bot.send_message(user_id, BasicCommands::descriptions())
                .send()
                .await?;
        }

        BasicCommands::Start => {
            bot.send_message(user_id, BasicCommands::descriptions())
                .send()
                .await?;
        }

        BasicCommands::MyCards => {
            PresentLists::new(user).await?.execute(&bot).await?;
        }
    }

    Ok(())
}
