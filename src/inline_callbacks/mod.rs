use crate::{buttonable, Moranometer};
use anyhow::anyhow;
use serde::{Deserialize, Serialize};
use serde_json;
use teloxide::types::{ForceReply, InlineKeyboardButton, InlineKeyboardMarkup};
use teloxide::{prelude2::*, utils::command::BotCommand};

mod cb_present_lists;
mod move_to_done;
mod present_card;
mod present_list_cards;
use buttonable::Buttonable;
use cb_present_lists::CbPresentLists as PresentLists;
use move_to_done::MoveToDone;
use present_card::PresentCard;
use present_list_cards::PresentListCards;

type CardId = String;
type ListId = String;

#[derive(Debug, BotCommand, Clone, Serialize, Deserialize)]
#[command(rename = "lowercase", description = "These commands are supported:")]
pub enum CallbackCommands {
    #[serde(rename = "pl")]
    PresentLists,
    #[serde(rename = "plc")]
    PresentListsCards(ListId),
    #[serde(rename = "pc")]
    PresentCard(CardId),
    #[serde(rename = "cc")]
    CommentCard(CardId),
    #[serde(rename = "ac")]
    AddCard(ListId),
    //admin commands
    #[serde(rename = "md")]
    MoveToDone(CardId),
}

impl Buttonable for CallbackCommands {
    fn as_callback(self, label: String) -> InlineKeyboardButton {
        InlineKeyboardButton::callback(label, serde_json::to_string(&self).unwrap())
    }
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

    if callback.message.is_none() {
        anyhow::bail!("query is to old");
    }

    if callback.message.is_none() {
        anyhow::bail!("query dont have message id");
    }

    log::info!("{user_name}: {command:?}", user_name = user.name);

    match command {
        CallbackCommands::PresentLists => {
            PresentLists::new(user, callback)
                .await?
                .execute(&bot)
                .await?;
        }

        CallbackCommands::PresentListsCards(list_id) => {
            PresentListCards::new(user, &list_id, callback)
                .await?
                .execute(&bot)
                .await?;
        }

        CallbackCommands::PresentCard(card_id) => {
            PresentCard::new(user, &card_id, callback)
                .await?
                .execute(&bot)
                .await?;
        }

        CallbackCommands::CommentCard(card_id) => {
            bot.send_message(user.id, format!("/comment {card_id}"))
                .reply_markup(
                    ForceReply::new()
                        .input_field_placeholder(Some("comment here senor".to_string())),
                )
                .send()
                .await?;
        }

        CallbackCommands::AddCard(list_id) => {
            bot.edit_message_text(
                user.id,
                callback.message.unwrap().id,
                format!(
                    "to add a card, enter the title on the first line
                            \n and the description (if needed) in the following lines.
                            \n To cancel, just exit the reply."
                ),
            )
            .send()
            .await?;

            bot.send_message(user.id, format!("/add_card {list_id}"))
                .reply_markup(
                    ForceReply::new()
                        .input_field_placeholder(Some("1st line title, 2nd line desc".to_string())),
                )
                .send()
                .await?;
        }

        CallbackCommands::MoveToDone(card_id) => {
            let notify = MoveToDone::new(&card_id).await?.execute().await?;
            let msg = format!("card '{}' marked as done", notify.card_name());


            let keyboard = InlineKeyboardMarkup::new(vec![vec![
                    CallbackCommands::PresentCard(card_id.clone())
                        .as_callback("🕵🏻‍♀️ show card".to_string()),
                ]]);

            bot.send_message(user.id, "card moved!").send().await?;
            notify
                .for_users(&users.db, user)
                .await
                .with_keyboard(keyboard)
                .execute(&bot, &msg)
                .await?;
        }
    };

    Ok(())
}
