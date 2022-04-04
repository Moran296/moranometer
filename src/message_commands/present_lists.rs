use crate::inline_callbacks::CallbackCommands;
use crate::users::User;
use anyhow::anyhow;
use teloxide::prelude2::*;
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup};
use trellolon::{Board, Component, List};

pub(crate) struct PresentLists<'a> {
    user: &'a User,
    lists: Vec<List>,
}

impl<'a> PresentLists<'a> {
    pub async fn new(user: &'a User) -> anyhow::Result<PresentLists<'a>> {
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

        Ok(PresentLists { user, lists })
    }

    pub async fn execute(&self, bot: &AutoSend<Bot>) -> anyhow::Result<()> {
        let mut buttons = vec![vec![]];

        for lists in self.lists.chunks(3) {
            let row = lists
                .iter()
                .filter_map(|list| {
                    // consider only visible lists, this function cannot be called inside iterator
                    // if !list.is_visible(self.user) {
                    //     return None;
                    // }

                    let callback = serde_json::to_string::<CallbackCommands>(
                        &CallbackCommands::PresentListsCards(list.id.clone()),
                    )
                    .unwrap();

                    Some(InlineKeyboardButton::callback(
                        format!("📜 {}", list.name),
                        callback,
                    ))
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
