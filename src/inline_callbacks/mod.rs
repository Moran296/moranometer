use crate::Moranometer;
//use anyhow::anyhow;
use serde::{Deserialize, Serialize};
use teloxide::{prelude2::*, utils::command::BotCommand};

#[derive(BotCommand, Clone, Serialize, Deserialize)]
#[command(rename = "lowercase", description = "These commands are supported:")]
pub enum CallbackCommands {
    PresentCard(String),
    CommentCard(String),
}

pub(crate) async fn callback_command_endpoint(
    _callback: CallbackQuery,
    _bot: AutoSend<Bot>,
    _cfg: Moranometer,
) -> anyhow::Result<()> {
    Ok(())
}
