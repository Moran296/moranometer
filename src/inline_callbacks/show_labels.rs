use crate::buttonable::Buttonable;
use crate::inline_callbacks::CallbackCommands;
use crate::users::User;
use anyhow::anyhow;
use teloxide::dispatching2::dialogue::GetChatId;
use teloxide::prelude2::*;
use teloxide::types::InlineKeyboardMarkup;
use trellolon::{Card, Component, Label};

pub(crate) struct ShowLabels {
    query: CallbackQuery,
    card: Card,
    card_labels: Vec<Label>,
    possible_labels: Vec<Label>,
}

impl ShowLabels {
    pub async fn new(query: CallbackQuery, card_id: String) -> anyhow::Result<Self> {
        let card = Card::get(&card_id)
            .await
            .ok_or(anyhow!("card does not exist"))?;

        let card_labels = card.get_all().await.unwrap_or(vec![]);

        let possible_labels = card
            .get_board()
            .await
            .ok_or(anyhow!("board cant be found"))?
            .get_labels()
            .await
            .unwrap_or(vec![])
            .iter()
            .filter(|label| {
                card_labels
                    .iter()
                    .find(|card_label| card_label.id == label.id)
                    .is_none()
            })
            .cloned()
            .collect();

        Ok(Self {
            query,
            card,
            card_labels,
            possible_labels,
        })
    }

    fn keyboard(&self) -> InlineKeyboardMarkup {
        let mut buttons = vec![];
        for label in &self.possible_labels {
            buttons.push(vec![CallbackCommands::AddLabel(
                self.card.id.clone(),
                label.id.clone(),
            )
            .as_callback(label.name.clone())]);
        }

        buttons
            .push(vec![CallbackCommands::PresentCard(self.card.id.clone())
                .as_callback("🚜 back".to_owned())]);

        InlineKeyboardMarkup::new(buttons)
    }

    fn message(&self) -> String {
        if self.card_labels.is_empty() {
            format!("Choose a label for {}", self.card.name)
        } else {
            let mut msg = format!(
                "following labels are already added to {}:\n",
                self.card.name
            );
            for label in &self.card_labels {
                msg.push_str(&format!("🦴 {}\n", label.name));
            }

            msg
        }
    }

    pub async fn execute(&self, bot: &AutoSend<Bot>, user: &User) -> anyhow::Result<()> {
        if self.possible_labels.is_empty() {
            bot.send_message(user.id, "🤷‍♀️ no possible labels found".to_owned())
                .send()
                .await?;

            log::info!("no possible labels found for card {}", self.card.name);
            return Ok(());
        }

        let markup = self.keyboard();

        let chat_id = *self.query.chat_id().as_ref().unwrap();
        let message_id = self.query.message.as_ref().unwrap().id;

        log::info!("sending {} labels to user", self.possible_labels.len());
        bot.edit_message_text(chat_id, message_id, self.message())
            .reply_markup(markup)
            .send()
            .await?;

        Ok(())
    }
}
