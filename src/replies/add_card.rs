use super::created_notify::CreatedNotify;
use crate::buttonable::Buttonable;
use crate::inline_callbacks::CallbackCommands;
use crate::users::User;
use anyhow::anyhow;
use teloxide::prelude2::*;
use teloxide::types::InlineKeyboardMarkup;
use teloxide::{prelude::Requester, prelude2::Message};
use trellolon::{Card, Creatable, Label, List};

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

    async fn add_label(&self, card: &Card) -> anyhow::Result<()> {
        if self.user.is_moderator() {
            return Ok(());
        }

        let labels = Label::get_from_board(&card.id_board).await;
        if let Some(labels) = labels {
            if let Some(label) = labels.iter().find(|l| l.name == self.user.name) {
                card.clone()
                    .add_label(&label)
                    .await
                    .ok_or(anyhow!("could not add label"))?;
            } else {
                log::warn!("user added card but has no label");
            }
        }

        Ok(())
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

        let keyboard = InlineKeyboardMarkup::new(vec![
            vec![CallbackCommands::PresentCard(card.id.clone())
                .as_callback(" 🕵🏻‍♀️ show card".to_owned())],
            vec![CallbackCommands::PresentListsCards(self.list.id.clone())
                .as_callback("🚜 back".to_owned())],
        ]);

        self.add_label(&card).await?;

        bot.send_message(self.user.id, "card created!")
            .reply_markup(keyboard)
            .send()
            .await?;

        log::info!("card added");
        Ok(CreatedNotify::from(card, self.user))
    }
}
