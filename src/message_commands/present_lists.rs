use crate::inline_callbacks::CallbackCommands;
use crate::users::User;
use crate::{buttonable::Buttonable, users::Visible};
use anyhow::anyhow;
use teloxide::prelude2::*;
use teloxide::types::InlineKeyboardMarkup;
use trellolon::{Board, Component, List};

pub(crate) struct PresentLists<'a> {
    user: &'a User,
    lists: Vec<List>,
}

impl<'a> PresentLists<'a> {
    pub async fn new(user: &'a User) -> anyhow::Result<PresentLists<'a>> {
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

        Ok(PresentLists { user, lists })
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

        bot.send_message(self.user.id, "🤹‍♀️ choose the requested category".to_owned())
            .reply_markup(InlineKeyboardMarkup::new(buttons))
            .send()
            .await?;

        Ok(())
    }
}
