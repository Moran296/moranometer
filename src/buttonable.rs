use teloxide::types::InlineKeyboardButton;

pub(crate) trait Buttonable {
    fn as_callback(self, label: String) -> InlineKeyboardButton;
}
