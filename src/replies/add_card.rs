use super::created_notify::CreatedNotify;
use crate::inline_callbacks::CallbackCommands;
use crate::users::User;
use anyhow::anyhow;
use teloxide::prelude2::*;
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup};
use teloxide::{prelude::Requester, prelude2::Message};
use trellolon::{Card, Creatable, List};

#[derive(Debug)]
pub(crate) struct AddCard<'a> {
    list: List,
    card_txt: &'a str,
    user: &'a User,
}

impl<'a> AddCard<'a> {
    const LIST_ID_LEN: usize = 24;
    const ADD_REQUEST: &'static str = "/add_card ";

    pub async fn from(msg: &'a Message, user: &'a User) -> Option<AddCard<'a>> {
        let card_txt = msg.text()?;
        let reply_to_text = msg.reply_to_message()?.text()?;

        let list_id = reply_to_text.strip_prefix(Self::ADD_REQUEST)?;
        let is_valid_id = list_id.len() == Self::LIST_ID_LEN;
        if !is_valid_id {
            log::warn!("user trying to comment a non valid list id {list_id}");
            return None;
        }

        let list = List::get(list_id).await?;

        Some(AddCard {
            list,
            card_txt,
            user,
        })
    }

    pub async fn execute(self, bot: &'a AutoSend<Bot>) -> anyhow::Result<CreatedNotify<'a>> {
        let mut lines = self.card_txt.lines();
        let title = lines.next().ok_or(anyhow!("no title"))?;
        let description = lines.collect::<String>();

        log::info!("title: {title}");
        log::info!("desc: {description}");
        let card = Card::new(title, description);
        let card = card
            .create(&self.list)
            .await
            .ok_or(anyhow!("card creation failed"))?;

        let keyboard = InlineKeyboardMarkup::default().append_row(vec![
            InlineKeyboardButton::callback(
                " 🕵🏻‍♀️ show card".to_string(),
                serde_json::to_string(&CallbackCommands::PresentCard(card.id.clone())).unwrap(),
            ),
            InlineKeyboardButton::callback(
                "🚜 back".to_string(),
                serde_json::to_string(&CallbackCommands::PresentListsCards(self.list.id.clone()))
                    .unwrap(),
            ),
        ]);

        bot.send_message(self.user.id, "card created!")
            .reply_markup(keyboard)
            .send()
            .await?;

        log::info!("comment added to card");
        Ok(CreatedNotify::from(card, self.user))
    }
}
