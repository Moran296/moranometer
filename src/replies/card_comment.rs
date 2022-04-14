use crate::buttonable::Buttonable;
use crate::inline_callbacks::CallbackCommands;
use crate::notifier::*;
use crate::users::{User, Visible};
use anyhow::anyhow;
use teloxide::prelude2::*;
use teloxide::types::InlineKeyboardMarkup;
use teloxide::{prelude::Requester, prelude2::Message};
use trellolon::Card;

#[derive(Debug)]
pub(crate) struct CardComment<'a> {
    card: Card,
    comment: &'a str,
    user: &'a User,
}

impl<'a> CardComment<'a> {
    const CARD_ID_LEN: usize = 24;
    const COMMENT_REQUEST: &'static str = "/comment ";

    pub async fn from(msg: &'a Message, user: &'a User) -> Option<CardComment<'a>> {
        let comment = msg.text()?;
        let reply_to_text = msg.reply_to_message()?.text()?;

        let card_id = reply_to_text.strip_prefix(Self::COMMENT_REQUEST)?;
        let is_valid_id = card_id.len() == Self::CARD_ID_LEN;
        if !is_valid_id {
            log::warn!("user trying to comment a non visible card!");
            return None;
        }

        let card = Card::get(card_id).await?;
        if !card.is_visible(user).await {
            log::warn!("user trying to comment a non visible card!");
            return None;
        }

        Some(CardComment {
            card,
            comment,
            user,
        })
    }

    async fn notify(
        users: &Vec<User>,
        user: &User,
        bot: &'a AutoSend<Bot>,
        card: Card,
    ) -> anyhow::Result<String> {
        let notified_keyboard = InlineKeyboardMarkup::new(vec![
            vec![CallbackCommands::PresentCard(card.id.clone())
                .as_callback("🕵🏻‍♀️ show card".to_string())],
            vec![CallbackCommands::CommentCard(card.id.clone())
                .as_callback("🤬 comment".to_string())],
        ]);

        log::info!("comment added to card");

        NotifyOn::Comment(card.clone())
            .for_users(users, user)
            .await
            .with_keyboard(notified_keyboard)
            .execute(bot, &format!("{} commented on: {}", user.name, card.name))
            .await
    }

    pub async fn execute(self, users: &Vec<User>, bot: &'a AutoSend<Bot>) -> anyhow::Result<()> {
        let comment = format!("{}: {}", self.user.name, self.comment);
        let card = self
            .card
            .add_comment(&comment)
            .await
            .ok_or(anyhow!("could not add comment"))?;

        let keyboard =
            InlineKeyboardMarkup::new(vec![vec![
                CallbackCommands::PresentCard(card.id.clone()).as_callback("🚜 back".to_string())
            ]]);

        let notifieds = Self::notify(users, self.user, bot, card).await?;
        let msg = format!("card commented!\n{notifieds}");

        bot.send_message(self.user.id, msg)
            .reply_markup(keyboard)
            .send()
            .await?;

        Ok(())
    }
}
