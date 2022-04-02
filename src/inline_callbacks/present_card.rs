use crate::inline_callbacks::CallbackCommands;
use crate::users::{User, Visible};
use anyhow::anyhow;
use teloxide::dispatching2::dialogue::GetChatId;
use teloxide::prelude2::*;
use teloxide::requests::HasPayload;
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup, ParseMode};
use trellolon::Card;

pub(crate) struct PresentCard {
    card: Card,
    query: CallbackQuery,
}

impl<'a> PresentCard {
    pub async fn new(user: &'a User, card_id: &'a str, query: CallbackQuery) -> anyhow::Result<PresentCard> {
        let card = Card::get(card_id)
            .await
            .ok_or(anyhow!("card does not exist"))?;
        if !card.is_visible(user).await {
            anyhow::bail!("card is not visible to user");
        }

        Ok(Self {card, query })
    }

    pub async fn execute(&self, bot: &AutoSend<Bot>) -> anyhow::Result<()> {
        let comments = self
            .card
            .get_comments()
            .await
            .unwrap_or(vec!["no comments...".to_string()]);

        let mut card_str = format!(
            "🦊 <u><b>{}</b></u>:\n
            <i>{}</i>\n
            ======Comments======\n",
            self.card.name, self.card.description
        );

        for comment in comments {
            card_str.push_str(&format!("👉🏽 <em> {}</em> \n", comment));
        }

        let keyboard = InlineKeyboardMarkup::default()
            .append_row(vec![InlineKeyboardButton::callback(
                "🤬 comment".to_string(),
                serde_json::to_string(&CallbackCommands::CommentCard(self.card.id.clone()))
                    .unwrap(),
            )])
            .append_row(vec![InlineKeyboardButton::callback(
                "🚜 back".to_string(),
                serde_json::to_string(&CallbackCommands::PresentListsCards(
                    self.card.id_list.clone(),
                ))
                .unwrap(),
            )]);
        // .append_row(vec![InlineKeyboardButton::callback(
        //     "archive".to_string(),
        //     serde_json::to_string(&CallbackCommands::ArchiveCard(self.card.id.clone()))
        //         .unwrap(),
        // )]);

        bot.edit_message_text(
            *self.query.chat_id().as_ref().unwrap(),
            self.query.message.as_ref().unwrap().id,
             card_str,
        ) .with_payload_mut(|p| p.parse_mode = Some(ParseMode::Html))
            .reply_markup(keyboard)
            .send()
            .await?;

        Ok(())
    }
}
