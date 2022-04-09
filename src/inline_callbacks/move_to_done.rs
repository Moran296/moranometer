use crate::notifier::NotifyOn;
use anyhow::anyhow;
use trellolon::{Card, Component};

pub(crate) struct MoveToDone {
    card: Card,
}

impl MoveToDone {
    pub async fn new(card_id: &str) -> anyhow::Result<MoveToDone> {
        Ok(Self {
            card: Card::get(card_id)
                .await
                .ok_or(anyhow!("card does not exist"))?,
        })
    }

    pub async fn execute(self) -> anyhow::Result<NotifyOn> {
        let board = self
            .card
            .get_board()
            .await
            .ok_or(anyhow!("board cant be found"))?;
        let done = board
            .get_by_name("Done")
            .await
            .ok_or(anyhow!("list cant be found"))?;
        let card = self
            .card
            .move_to_list(done)
            .await
            .ok_or(anyhow!("card cant be moved"))?;

        Ok(NotifyOn::MovedToDone(card))
    }
}
