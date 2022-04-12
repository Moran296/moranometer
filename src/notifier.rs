use crate::users::{User, Visible};
use anyhow::Result;
use teloxide::prelude2::*;
use teloxide::types::InlineKeyboardMarkup;
use trellolon::Card;

const NOTIFIED_EMOJIS: [&'static str; 5] = ["👲🏻", "🧕🏻", "🧛🏻", "🧟🏻", "🧙🏻"];

#[derive(Debug)]
pub(crate) enum NotifyOn {
    Comment(Card),
    Create(Card),
    MovedToDone(Card),
    // LabelAdded(Card, Label),
}

pub(crate) struct Notifier {
    users: Vec<User>,
    source_user: Option<User>,
    keyboard: Option<InlineKeyboardMarkup>,
}

async fn get_relevant_users(all_users: &Vec<User>, source_user: &User, card: &Card) -> Vec<User> {
    let mut users = Vec::new();

    for user in all_users {
        if !card.is_visible(user).await || user.id == source_user.id {
            continue;
        }

        users.push(user.clone());
    }

    users
}

impl NotifyOn {
    pub(crate) async fn for_users(self, all_users: &Vec<User>, source_user: &User) -> Notifier {
        let mut users = vec![];

        match self {
            NotifyOn::Comment(card) => {
                users.extend(get_relevant_users(all_users, source_user, &card).await)
            }
            NotifyOn::Create(card) => {
                users.extend(get_relevant_users(all_users, source_user, &card).await)
            }
            NotifyOn::MovedToDone(card) => {
                users.extend(get_relevant_users(all_users, source_user, &card).await)
            } // NotifyOn::LabelAdded(card, label) => {
              //     users.extend(get_relevant_users(all_users, source_user, &card).await);
              //     users = users
              //         .into_iter()
              //         .filter(|user| user.name == label.name)
              //         .collect();
              // }
        }

        Notifier::new(users, Some(source_user.clone()), None)
    }

    pub(crate) fn card_name(&self) -> String {
        match self {
            NotifyOn::Comment(card) => card.name.clone(),
            NotifyOn::Create(card) => card.name.clone(),
            NotifyOn::MovedToDone(card) => card.name.clone(),
        }
    }
}

impl Notifier {
    pub fn new(
        users: Vec<User>,
        source_user: Option<User>,
        keyboard: Option<InlineKeyboardMarkup>,
    ) -> Notifier {
        Notifier {
            users,
            source_user,
            keyboard,
        }
    }

    pub fn with_keyboard(mut self, keyboard: InlineKeyboardMarkup) -> Notifier {
        self.keyboard = Some(keyboard);
        self
    }

    pub async fn execute(&self, bot: &AutoSend<Bot>, msg: &str) -> Result<()> {
        let mut notifieds = "notified: ".to_string();
        let mut i = 0;

        log::info!("notifying {} users", self.users.len());

        for user in &self.users {
            notifieds.push_str(&format!("\n{} - {}, ", NOTIFIED_EMOJIS[i], user.name));
            i = i + 1 % NOTIFIED_EMOJIS.len();

            let mut to_send = bot.send_message(user.id, msg);
            if let Some(keyboard) = &self.keyboard {
                to_send = to_send.reply_markup(keyboard.clone());
            }

            tokio::spawn(async move {
                to_send.send().await.unwrap();
            });
        }

        if let Some(source_user) = &self.source_user {
            if self.users.len() > 0 {
                bot.send_message(source_user.id, &notifieds).send().await?;
            } else {
                bot.send_message(source_user.id, "no one to notify..")
                    .send()
                    .await?;
            }
        }

        Ok(())
    }
}
