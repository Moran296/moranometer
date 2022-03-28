use crate::tellegram_commands::{BasicCommands, CallbackCommands};
use crate::{Moranometer, MyHandlerType, Users};
use anyhow::anyhow;
use teloxide::dispatching2::UpdateFilterExt;
use teloxide::types::Message;
use teloxide::{prelude2::*, utils::command::BotCommand};

//================MESSAGES=====================

pub fn message_handler() -> MyHandlerType {
    Update::filter_message().branch(
        dptree::entry()
            .filter_command::<BasicCommands>()
            .endpoint(message_command_handler),
    )
}

async fn message_command_handler(
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

    let response = match cmd {
        BasicCommands::Help => Some(BasicCommands::descriptions()),

        BasicCommands::Start => Some(BasicCommands::descriptions()),

        BasicCommands::MyCards => Some(BasicCommands::descriptions()),
    };

    if let Some(response) = response {
        bot.send_message(msg.chat.id, &response).send().await?;
    }

    Ok(())
}

//================CALLBACKS=====================

pub fn callback_handler() -> MyHandlerType {
    Update::filter_callback_query().branch(
        dptree::entry()
            .filter_command::<CallbackCommands>()
            .endpoint(callback_command_handler),
    )
}

async fn callback_command_handler(
    _callback: CallbackQuery,
    _bot: AutoSend<Bot>,
    _cmd: CallbackCommands,
    _cfg: Moranometer,
) -> anyhow::Result<()> {
    Ok(())
}
