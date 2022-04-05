use crate::buttonable::Buttonable;
use crate::inline_callbacks::CallbackCommands;
use crate::users::{User, Visible};
use teloxide::prelude2::*;
use teloxide::types::InlineKeyboardMarkup;
use trellolon::Card;

const NOTIFIED_EMOJIS: [&'static str; 5] = ["👲🏻", "🧕🏻", "🧛🏻", "🧟🏻", "🧙🏻"];

pub(crate) struct CreatedNotify<'a> {
    card: Card,
    creator: &'a User,
}

impl<'a> CreatedNotify<'a> {
    pub fn from(card: Card, creator: &'a User) -> CreatedNotify<'a> {
        CreatedNotify { card, creator }
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
        let notified_users = Self::get_relevant_users(&users, self.creator, &self.card).await;
        if notified_users.is_empty() {
            log::info!("no users to notify");
            return Ok(());
        }

        let notify = format!(
            "new card by{user_name}:\n {card_name} ",
            user_name = self.creator.name,
            card_name = self.card.name
        );

        let keyboard = InlineKeyboardMarkup::new(vec![
            vec![CallbackCommands::PresentCard(self.card.id.clone())
                .as_callback("🕵🏻‍♀️ show card".to_string())],
            vec![CallbackCommands::CommentCard(self.card.id.clone())
                .as_callback("🤬 comment".to_string())],
        ]);

        let mut notifieds = "notified: ".to_string();
        let mut i = 0;
        for user in notified_users {
            log::info!("notifying user: {}", user.id);

            bot.send_message(user.id, &notify)
                .reply_markup(keyboard.clone())
                .send()
                .await?;

            notifieds.push_str(&format!("\n{} - {}, ", NOTIFIED_EMOJIS[i], user.name));
            i = i + 1 % NOTIFIED_EMOJIS.len();
        }

        log::info!("{notifieds}");
        bot.send_message(self.creator.id, &notifieds).send().await?;

        Ok(())
    }
}
