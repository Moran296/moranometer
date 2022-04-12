use crate::buttonable::Buttonable;
use crate::inline_callbacks::CallbackCommands::*;
use crate::users::{User, Visible};
use anyhow::anyhow;
use teloxide::dispatching2::dialogue::GetChatId;
use teloxide::prelude2::*;
use teloxide::requests::HasPayload;
use teloxide::types::{InlineKeyboardMarkup, ParseMode};
use trellolon::Card;

pub(crate) struct PresentCard {
    card: Card,
    query: CallbackQuery,
    is_moderator: bool,
}

impl<'a> PresentCard {
    pub async fn new(
        user: &'a User,
        card_id: &'a str,
        query: CallbackQuery,
    ) -> anyhow::Result<PresentCard> {
        let card = Card::get(card_id)
            .await
            .ok_or(anyhow!("card does not exist"))?;
        if !card.is_visible(user).await {
            anyhow::bail!("card is not visible to user");
        }

        Ok(Self {
            card,
            query,
            is_moderator: user.is_moderator(),
        })
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

        let mut keyboard = InlineKeyboardMarkup::default()
            .append_row(vec![
                CommentCard(self.card.id.clone()).as_callback("🤬 comment".to_string())
            ])
            .append_row(vec![
                PresentListsCards(self.card.id_list.clone()).as_callback("🚜 back".to_string())
            ]);

        keyboard = if self.is_moderator {
            keyboard
                .append_row(vec![
                    MoveToDone(self.card.id.clone()).as_callback("😍 mark as done".to_string())
                ])
                .append_row(vec![
                    ShowLabels(self.card.id.clone()).as_callback("👹 Add label".to_string())
                ])
        } else {
            keyboard
        };

        bot.edit_message_text(
            *self.query.chat_id().as_ref().unwrap(),
            self.query.message.as_ref().unwrap().id,
            card_str,
        )
        .with_payload_mut(|p| p.parse_mode = Some(ParseMode::Html))
        .reply_markup(keyboard)
        .send()
        .await?;

        Ok(())
    }
}
