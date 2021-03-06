use crate::buttonable::Buttonable;
use crate::inline_callbacks::CallbackCommands;
use crate::users::{User, Visible};
use anyhow::anyhow;
use teloxide::dispatching2::dialogue::GetChatId;
use teloxide::prelude2::*;
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup};
use trellolon::{Card, Component, List};

pub(crate) struct PresentListCards {
    list: List,
    query: CallbackQuery,
    cards: Option<Vec<Card>>,
}

impl<'a> PresentListCards {
    pub async fn new(
        user: &'a User,
        list_id: &'a str,
        query: CallbackQuery,
    ) -> anyhow::Result<PresentListCards> {
        let list = List::get(list_id)
            .await
            .ok_or(anyhow!("list does not exist"))?;
        if !list.is_visible(user).await {
            anyhow::bail!("list is not visible to user");
        }

        let cards = list.get_all().await.unwrap_or(vec![]);

        let mut relevants = vec![];
        for card in cards {
            if card.is_visible(user).await {
                relevants.push(card);
            }
        }

        let relevants = if relevants.is_empty() {
            None
        } else {
            Some(relevants)
        };

        Ok(Self {
            list,
            query,
            cards: relevants,
        })
    }

    pub async fn execute(&self, bot: &AutoSend<Bot>) -> anyhow::Result<()> {
        let mut buttons = vec![vec![]];

        if let Some(cards) = self.cards.as_ref() {
            for cards in cards.chunks(2) {
                let row = cards
                    .iter()
                    .map(|card| {
                        CallbackCommands::PresentCard(card.id.clone())
                            .as_callback(format!("🎬 {}", card.name))
                    })
                    .collect();

                buttons.push(row);
            }
        } else {
            buttons.push(vec![InlineKeyboardButton::url(
                "🤷‍♀️ no cards found".to_owned(),
                reqwest::Url::parse("https://www.youtube.com/watch?v=dQw4w9WgXcQ").unwrap(),
            )]);
        }

        buttons.push(vec![
            CallbackCommands::PresentLists.as_callback("🚜 back".to_owned())
        ]);
        buttons
            .push(vec![CallbackCommands::AddCard(self.list.id.clone())
                .as_callback("✋🏼 add card".to_owned())]);

        bot.edit_message_text(
            *self.query.chat_id().as_ref().unwrap(),
            self.query.message.as_ref().unwrap().id,
            format!("🤽 {}", &self.list.name),
        )
        .reply_markup(InlineKeyboardMarkup::new(buttons))
        .send()
        .await?;

        Ok(())
    }
}
