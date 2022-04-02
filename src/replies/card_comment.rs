use teloxide::prelude2::Message;

#[derive(Debug)]
pub struct CardComment<'a> {
    card_id: &'a str,
    comment: &'a str,
}

impl<'a> CardComment<'a> {
    const CARD_ID_LEN: usize = 24;
    const COMMENT_REQUEST: &'static str = "/comment ";

    pub fn from(msg: &'a Message) -> Option<Self> {
        log::info!("{}", line!());
        let comment = msg.text()?;
        log::info!("{}", line!());
        let reply_to_text = msg.reply_to_message()?.text()?;
        log::info!("{}", line!());

        let card_id = reply_to_text.strip_prefix(Self::COMMENT_REQUEST)?;
        log::info!("{}", line!());
        let is_valid_id = card_id.len() == Self::CARD_ID_LEN;
        log::info!("{} is valid id {is_valid_id}", line!());
        match is_valid_id {
            true => Some(CardComment { card_id, comment }),

            false => None,
        }
    }
}
