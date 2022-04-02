use crate::Moranometer;
use anyhow::anyhow;
use teloxide::prelude2::*;

mod card_comment;
use card_comment::CardComment;
mod comment_notify;
use comment_notify::CommentNotify;

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
    } else {
        anyhow::bail!("unhadled reply");
    }

    Ok(())
}
