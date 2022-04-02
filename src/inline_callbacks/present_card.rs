use crate::inline_callbacks::CallbackCommands;
use crate::users::{User, Visible};
use anyhow::anyhow;
use teloxide::payloads::SendMessageSetters;
use teloxide::prelude2::*;
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup};
use trellolon::Card;

pub(crate) struct PresentCard<'a> {
    user: &'a User,
    card: Card,
}

impl<'a> PresentCard<'a> {
    pub async fn new(user: &'a User, card_id: &'a str) -> anyhow::Result<PresentCard<'a>> {
        let card = Card::get(card_id)
            .await
            .ok_or(anyhow!("card does not exist"))?;
        if !card.is_visible(user).await {
            anyhow::bail!("card is not visible to user");
        }

        Ok(Self { user, card })
    }

    pub async fn execute(&self, bot: &AutoSend<Bot>) -> anyhow::Result<()> {
        let comments = self
            .card
            .get_comments()
            .await
            .unwrap_or(vec!["no comments...".to_string()]);
        let mut card_str = format!(
            "*{}*:\n{}\n======Comments======\n",
            self.card.name, self.card.description
        );
        for comment in comments {
            card_str.push_str(&comment);
            card_str.push_str("\n");
        }

        let keyboard =
            InlineKeyboardMarkup::default().append_row(vec![InlineKeyboardButton::callback(
                "comment".to_string(),
                serde_json::to_string(&CallbackCommands::CommentCard(self.card.id.clone()))
                    .unwrap(),
            )]);

        bot.send_message(self.user.id, card_str)
            .reply_markup(keyboard)
            .send()
            .await?;

        Ok(())
    }
}
