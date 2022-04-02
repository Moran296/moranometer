use crate::inline_callbacks::CallbackCommands;
use crate::users::{User, Visible};
use anyhow::anyhow;
use teloxide::prelude2::Message;
use teloxide::prelude2::*;
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup};
use trellolon::Card;

pub(crate) struct CommentNotify<'a> {
    card: Card,
    comment: String,
    commenter: &'a User,
}

impl<'a> CommentNotify<'a> {
    pub fn from(card: Card, comment: String, commenter: &'a User) -> CommentNotify<'a> {
        CommentNotify {
            card,
            comment,
            commenter,
        }
    }

    async fn get_relevant_users(
        all_users: &'a Vec<User>,
        commenting_user: &'a User,
        card: &'a Card,
    ) -> Vec<&'a User> {
        let mut users = Vec::new();

        for user in all_users {
            if !card.is_visible(user).await {
                continue;
            }

            if user == commenting_user {
                continue;
            }

            users.push(user);
        }

        users
    }

    pub async fn execute(
        &'a self,
        bot: &'a AutoSend<Bot>,
        users: &'a Vec<User>,
    ) -> anyhow::Result<()> {
        let notified_users = Self::get_relevant_users(&users, self.commenter, &self.card).await;
        if notified_users.is_empty() {
            log::info!("no users to notify");
            return Ok(());
        }

        let notify = format!("new comment on: {}:\n {}", self.card.name, self.comment);

        let keyboard = InlineKeyboardMarkup::default().append_row(vec![
            InlineKeyboardButton::callback(
                "show card".to_string(),
                serde_json::to_string(&CallbackCommands::PresentCard(self.card.id.clone()))
                    .unwrap(),
            ),
            InlineKeyboardButton::callback(
                "comment".to_string(),
                serde_json::to_string(&CallbackCommands::CommentCard(self.card.id.clone()))
                    .unwrap(),
            ),
        ]);

        let mut notifieds = "notified: ".to_string();
        for user in notified_users {
            log::info!("notifying user: {}", user.id);

            bot.send_message(user.id, &notify)
                .reply_markup(keyboard.clone())
                .send()
                .await?;

            notifieds.push_str(&format!("\n{}, ", user.name));
        }

        log::info!("{notifieds}");
        bot.send_message(self.commenter.id, &notifieds)
            .send()
            .await?;

        Ok(())
    }
}
