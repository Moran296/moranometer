use crate::buttonable::Buttonable;
use crate::inline_callbacks::CallbackCommands;
use crate::users::{User, Visible};
use anyhow::anyhow;
use teloxide::dispatching2::dialogue::GetChatId;
use teloxide::prelude2::*;
use teloxide::types::InlineKeyboardMarkup;
use trellolon::{Board, Component, List};

pub(crate) struct CbPresentLists {
    lists: Vec<List>,
    query: CallbackQuery,
}

impl CbPresentLists {
    pub async fn new(user: &User, query: CallbackQuery) -> anyhow::Result<CbPresentLists> {
        let mut lists = vec![];

        let boards = Board::get_all_boards()
            .await
            .ok_or(anyhow!("no boards found for user {}", user.name))?;
        for board in boards {
            if !board.is_visible(user).await {
                continue;
            }

            if let Some(lists_) = board.get_all().await {
                lists.extend(lists_.into_iter());
            }
        }

        if lists.is_empty() {
            anyhow::bail!("no lists found for user {}", user.name);
        }

        Ok(CbPresentLists { lists, query })
    }

    pub async fn execute(&self, bot: &AutoSend<Bot>) -> anyhow::Result<()> {
        let mut buttons = vec![vec![]];

        for lists in self.lists.chunks(3) {
            let row = lists
                .iter()
                .map(|list| {
                    CallbackCommands::PresentListsCards(list.id.clone())
                        .as_callback(format!("📜 {}", list.name))
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
