use crate::inline_callbacks::CallbackCommands;
use crate::users::{User, Visible};
use anyhow::anyhow;
use teloxide::dispatching2::dialogue::GetChatId;
use teloxide::prelude2::*;
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup};
use trellolon::{Component, List};

pub(crate) struct PresentListCards {
    list: List,
    query: CallbackQuery,
}

impl<'a> PresentListCards {
    pub async fn new(
        user: &'a User,
        list_id: &'a str,
        query: CallbackQuery,
    ) -> anyhow::Result<PresentListCards> {
        if query.message.is_none() {
            anyhow::bail!("query is to old");
        }

        if query.message.is_none() {
            anyhow::bail!("query dont have message id");
        }

        let list = List::get(list_id)
            .await
            .ok_or(anyhow!("list does not exist"))?;
        if !list.is_visible(user).await {
            anyhow::bail!("list is not visible to user");
        }

        Ok(Self { list, query })
    }

    pub async fn execute(&self, bot: &AutoSend<Bot>) -> anyhow::Result<()> {
        let cards = self.list.get_all().await;
        if cards.is_none() {
            //
        }

        let mut buttons = vec![vec![]];
        let cards = cards.unwrap();
        for cards in cards.chunks(3) {
            let row = cards
                .iter()
                .map(|card| {
                    let callback = serde_json::to_string::<CallbackCommands>(
                        &CallbackCommands::PresentCard(card.id.clone()),
                    )
                    .unwrap();

                    InlineKeyboardButton::callback(card.name.clone(), callback)
                })
                .collect();

            buttons.push(row);
        }

        bot.edit_message_text(
            *self.query.chat_id().as_ref().unwrap(),
            self.query.message.as_ref().unwrap().id,
            &self.list.name,
        )
        .reply_markup(InlineKeyboardMarkup::new(buttons))
        .send()
        .await?;

        Ok(())
    }
}