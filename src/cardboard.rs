use crate::tellegram_commands::{BasicCommands, CallbackCommands};
use crate::users::{User, Users, Visible};
use anyhow::anyhow;
use serde::{Deserialize, Serialize};
use teloxide::payloads::SendMessageSetters;
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup, Message, ReplyMarkup};
use teloxide::{prelude2::*, utils::command::BotCommand};
use trellolon::{Board, Card, Component, List};

macro_rules! value_or_continue {
    ($opt:expr) => {
        match $opt {
            Some(val) => val,
            None => continue,
        }
    };
}

pub struct Cardboard {
    pub list: List,
    pub cards: Vec<Card>,
}

impl Cardboard {
    pub async fn send(&self, bot: &AutoSend<Bot>, user: &User) -> anyhow::Result<()> {
        let card_keyes: Vec<InlineKeyboardButton> = self
            .cards
            .iter()
            .map(|card| {
                InlineKeyboardButton::callback(
                    format!("{}", card.name),
                    serde_json::to_string(&CallbackCommands::PresentCard(card.id.clone())).unwrap(),
                )
            })
            .collect();

        bot.send_message(user.id, format!("list: {}", self.list.name))
            .reply_markup(InlineKeyboardMarkup::new(vec![card_keyes]))
            .send()
            .await?;

        Ok(())
    }
}

pub struct Cardboards(pub Vec<Cardboard>);

impl Cardboards {
    pub async fn from(user: &User) -> anyhow::Result<Self> {
        let mut cardboards = Vec::new();

        for board_name in user.boards.iter() {
            let board = Board::get(board_name)
                .await
                .ok_or(anyhow!("Could not get board {board_name}"))?;
            let lists = value_or_continue!(board.get_all().await);
            for list in lists {
                let cards = value_or_continue!(list.get_all().await);
                if user.is_moderator(board_name) {
                    cardboards.push(Cardboard {
                        list: list,
                        cards: cards,
                    });
                } else {
                    let mut relevant_cards = Vec::new();
                    for card in cards {
                        if card.is_visible(&user).await {
                            relevant_cards.push(card);
                        }
                    }

                    if !relevant_cards.is_empty() {
                        cardboards.push(Cardboard {
                            list: list,
                            cards: relevant_cards,
                        });
                    }
                }
            }
        }

        Ok(Cardboards(cardboards))
    }

    pub async fn send(&self, bot: &AutoSend<Bot>, user: &User) -> anyhow::Result<()> {
        if self.0.is_empty() {
            anyhow::bail!("no cards for this cardboard")
        }

        for cardboard in &self.0 {
            cardboard.send(bot, user).await?
        }

        Ok(())
    }
}
