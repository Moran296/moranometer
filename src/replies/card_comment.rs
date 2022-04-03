use super::comment_notify::CommentNotify;
use crate::inline_callbacks::CallbackCommands;
use crate::users::{User, Visible};
use anyhow::anyhow;
use teloxide::prelude2::*;
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup};
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

    pub async fn execute(self, bot: &'a AutoSend<Bot>) -> anyhow::Result<CommentNotify<'a>> {
        let comment = format!("{}: {}", self.user.name, self.comment);
        let card = self
            .card
            .add_comment(&comment)
            .await
            .ok_or(anyhow!("could not add comment"))?;

        let keyboard =
            InlineKeyboardMarkup::default().append_row(vec![InlineKeyboardButton::callback(
                "🚜 back".to_string(),
                serde_json::to_string(&CallbackCommands::PresentCard(card.id.clone())).unwrap(),
            )]);

        bot.send_message(self.user.id, "card commented!")
            .reply_markup(keyboard)
            .send()
            .await?;

        log::info!("comment added to card");
        Ok(CommentNotify::from(card, comment, self.user))
    }
}
