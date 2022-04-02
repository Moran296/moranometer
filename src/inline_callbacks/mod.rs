use crate::Moranometer;
use anyhow::anyhow;
use serde::{Deserialize, Serialize};
use serde_json;
use teloxide::types::ForceReply;
use teloxide::{prelude2::*, utils::command::BotCommand};

mod present_card;
mod present_list_cards;
use present_card::PresentCard;
use present_list_cards::PresentListCards;

#[derive(Debug, BotCommand, Clone, Serialize, Deserialize)]
#[command(rename = "lowercase", description = "These commands are supported:")]
pub enum CallbackCommands {
    #[serde(rename = "plc")]
    PresentListsCards(String),
    #[serde(rename = "pc")]
    PresentCard(String),
    #[serde(rename = "cc")]
    CommentCard(String),
}

pub(crate) async fn callback_command_endpoint(
    callback: CallbackQuery,
    bot: AutoSend<Bot>,
    cfg: Moranometer,
) -> anyhow::Result<()> {
    let data = callback
        .data
        .as_ref()
        .ok_or(anyhow!("no data in callback"))?;
    let command = serde_json::from_str::<CallbackCommands>(data)
        .map_err(|_x| anyhow!("callback is unknown"))?;

    let users = &cfg.lock().await.users;
    let user = users
        .get_user(callback.from.id)
        .ok_or(anyhow!("user does not exist.. what??"))?;

    log::info!("{command:?}");

    match command {
        CallbackCommands::PresentListsCards(list_id) => {
            PresentListCards::new(user, &list_id, callback)
                .await?
                .execute(&bot)
                .await?;
        }

        CallbackCommands::PresentCard(card_id) => {
            PresentCard::new(user, &card_id)
                .await?
                .execute(&bot)
                .await?;
        }

        CallbackCommands::CommentCard(card_id) => {
            bot.send_message(user.id, format!("/comment {card_id}"))
                .reply_markup(
                    ForceReply::new().input_field_placeholder(Some("comment here".to_string())),
                )
                .send()
                .await?;
        }
    };

    Ok(())
}
