use crate::buttonable::Buttonable;
use crate::inline_callbacks::CallbackCommands;
use anyhow::anyhow;
use teloxide::dispatching2::dialogue::GetChatId;
use teloxide::prelude2::*;
use teloxide::types::InlineKeyboardMarkup;
use trellolon::{Card, Label};

pub(crate) struct AddLabel {
    query: CallbackQuery,
    card: Card,
    label: Label,
}

impl AddLabel {
    pub async fn new(
        query: CallbackQuery,
        card_id: String,
        label_id: String,
    ) -> anyhow::Result<Self> {
        let card = Card::get(&card_id)
            .await
            .ok_or(anyhow!("card does not exist"))?;

        let label = Label::get_from_board(&card.id_board)
            .await
            .ok_or(anyhow!("label does not exist"))?
            .iter()
            .find(|l| l.id == label_id)
            .ok_or(anyhow!("label does not exist"))?
            .clone();

        Ok(Self { query, card, label })
    }

    fn keyboard(&self) -> InlineKeyboardMarkup {
        InlineKeyboardMarkup::new(vec![vec![CallbackCommands::PresentListsCards(
            self.card.id_list.clone(),
        )
        .as_callback("🚜 back".to_owned())]])
    }

    pub async fn execute(&self, bot: &AutoSend<Bot>) -> anyhow::Result<()> {
        let card = self
            .card
            .clone()
            .add_label(&self.label)
            .await
            .ok_or(anyhow!("failed adding label"));

        let msg = match &card {
            Ok(card) => format!("label added successfully to {name}", name = card.name),
            Err(_err) => "failed adding label".to_string(),
        };

        log::info!("{}", &msg);

        let chat_id = *self.query.chat_id().as_ref().unwrap();
        let message_id = self.query.message.as_ref().unwrap().id;

        bot.edit_message_text(chat_id, message_id, msg)
            .reply_markup(self.keyboard())
            .send()
            .await?;

        card?;
        Ok(())
    }
}
