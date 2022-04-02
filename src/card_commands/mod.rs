use crate::{users::Visible, Moranometer};
use anyhow::anyhow;
use teloxide::{prelude2::*, utils::command::BotCommand};
use trellolon::Card;

#[derive(BotCommand, Clone)]
#[command(rename = "lowercase", description = "These commands are supported:")]
pub enum CardCommands {
    #[command(description = "card")]
    Card(String),
}

pub(crate) async fn card_commands_endpoint(
    msg: Message,
    bot: AutoSend<Bot>,
    cmd: CardCommands,
    cfg: Moranometer,
) -> anyhow::Result<()> {
    let user_id = msg.from().ok_or(anyhow!("No user id"))?.id;

    let cfg = cfg.lock().await;
    let user = cfg.users.get_user(user_id).unwrap();

    match cmd {
        CardCommands::Card(card_id) => {
            let card = Card::get(&card_id)
                .await
                .ok_or(anyhow!("requested card does not exist"))?;
            if card.is_visible(user).await {
                let comments = card
                    .get_comments()
                    .await
                    .unwrap_or(vec!["no comments...".to_string()]);
                let mut card_str = format!(
                    "*{}*:\n{}\n======Comments======\n",
                    card.name, card.description
                );
                for comment in comments {
                    card_str.push_str(&comment);
                    card_str.push_str("\n");
                }

                bot.send_message(user.id, card_str).send().await?;
            }
        }
    }

    Ok(())
}
