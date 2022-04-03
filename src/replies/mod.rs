use crate::Moranometer;
use anyhow::anyhow;
use teloxide::prelude2::*;

mod add_card;
mod card_comment;
use add_card::AddCard;
use card_comment::CardComment;
mod comment_notify;
mod created_notify;

pub(crate) async fn reply_message_endpoint(
    msg: Message,
    bot: AutoSend<Bot>,
    cfg: Moranometer,
) -> anyhow::Result<()> {
    log::info!("got reply");

    let users = &cfg.lock().await.users;
    let user = users
        .get_user(msg.from().ok_or(anyhow!("No user id"))?.id)
        .unwrap();

    if let Some(card_comment) = CardComment::from(&msg, &user).await {
        log::info!("commenting on card {card_comment:?}");
        card_comment
            .execute(&bot)
            .await?
            .execute(&bot, &users.db)
            .await?;
    } else if let Some(add_card) = AddCard::from(&msg, &user).await {
        log::info!("add card {add_card:?}");
        add_card
            .execute(&bot)
            .await?
            .execute(&bot, &users.db)
            .await?;
    } else {
        anyhow::bail!("unhadled reply");
    }

    Ok(())
}
