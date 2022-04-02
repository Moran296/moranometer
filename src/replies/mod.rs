use crate::Moranometer;
use anyhow::anyhow;
use teloxide::prelude2::*;

mod card_comment;
use card_comment::CardComment;

pub(crate) async fn reply_message_endpoint(
    msg: Message,
    _bot: AutoSend<Bot>,
    _cfg: Moranometer,
) -> anyhow::Result<()> {
    log::info!("got reply");
        log::info!("{msg:#?}");

    if let Some(card_comment) = CardComment::from(&msg) {
        log::info!("commenting on card {card_comment:?}");
    } else {
        anyhow::bail!("reply is unhadled");
    }

    Ok(())
}
