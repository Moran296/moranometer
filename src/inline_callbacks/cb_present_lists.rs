use crate::inline_callbacks::CallbackCommands;
use crate::users::User;
use anyhow::anyhow;
use teloxide::dispatching2::dialogue::GetChatId;
use teloxide::prelude2::*;
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup};
use trellolon::{Board, Component, List};

pub(crate) struct CbPresentLists {
    lists: Vec<List>,
    query: CallbackQuery,
}

impl CbPresentLists {
    pub async fn new(user: &User, query: CallbackQuery) -> anyhow::Result<CbPresentLists> {
        let mut lists = vec![];

        for board in &user.boards {
            let board = Board::get(&board)
                .await
                .ok_or(anyhow!("failed retrieving board"))?;
            let new_lists = board.get_all().await;
            if let Some(new_lists) = new_lists {
                lists.extend(new_lists);
            }
        }

        if lists.is_empty() {
            anyhow::bail!("no lists found");
        }

        Ok(CbPresentLists { lists, query })
    }

    pub async fn execute(&self, bot: &AutoSend<Bot>) -> anyhow::Result<()> {
        let mut buttons = vec![vec![]];

        for lists in self.lists.chunks(3) {
            let row = lists
                .iter()
                .map(|list| {
                    let callback = serde_json::to_string::<CallbackCommands>(
                        &CallbackCommands::PresentListsCards(list.id.clone()),
                    )
                    .unwrap();

                    InlineKeyboardButton::callback(format!("📜 {}", list.name), callback)
                })
                .collect();

            buttons.push(row);
        }

        bot.edit_message_text(
            *self.query.chat_id().as_ref().unwrap(),
            self.query.message.as_ref().unwrap().id,
            "🤹 choose the requested category".to_owned(),
        )
        .reply_markup(InlineKeyboardMarkup::new(buttons))
        .send()
        .await?;

        Ok(())
    }
}
