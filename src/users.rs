use anyhow;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use core::panic;
use std::{
    fs::File,
    io::{Read, Write},
};
use trellolon::{Board, Card, Creatable, List};

const USERS_FILE: &'static str = "users.json";

#[derive(Debug, PartialEq, Eq, Hash, Clone, Serialize, Deserialize)]
struct BoardMap {
    pub board_name: String,
    pub board_id: String,
}

static mut BOARD_MAP: Vec<BoardMap> = vec![];

impl BoardMap {
    async fn load_from_file() {
        let file = File::options().read(true).open("boards.json");
        if file.is_err() {
            panic!();
        }

        let mut file = file.unwrap();
        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap();

        let boards: Vec<BoardMap> = serde_json::from_str::<Vec<BoardMap>>(&contents).unwrap();
        unsafe{
            BOARD_MAP = boards;
            if BOARD_MAP.is_empty() {
                panic!();
            }

        }


        ()
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Serialize, Deserialize)]
pub struct User {
    pub name: String,
    pub id: i64,
    pub boards: Vec<String>,
}

impl User {
    pub fn is_moderator(&self) -> bool {
        self.name == "Moran" || self.name == "Jenny"
    }
}

#[derive(Clone)]
pub(crate) struct Users {
    pub(crate) db: Vec<User>,
}

impl Users {
    pub async fn load() -> Self {
        let mut users = Self { db: vec![] };

        let file = File::options().read(true).open(USERS_FILE);
        if file.is_err() {
            return users;
        }

        let mut file = file.unwrap();
        let mut contents = String::new();

        BoardMap::load_from_file().await;

        file.read_to_string(&mut contents).unwrap();
        let file_users: Vec<User> = serde_json::from_str::<Vec<User>>(&contents).unwrap();
        users.db = file_users;

        users
    }

    pub async fn add(&mut self, name: &str, id: i64) -> anyhow::Result<()> {
        if !self.db.iter().any(|user| user.id == id) {
            let user = User {
                name: name.to_owned(),
                id: id,
                boards: vec!["TestBoard".to_owned()],
            };

            self.db.push(user.clone());
            write_to_file(&self.db)?;
        }

        Ok(())
    }

    pub fn get_user(&self, id: i64) -> Option<&User> {
        self.db.iter().find(|user| user.id == id)
    }

}

fn write_to_file(users: &Vec<User>) -> anyhow::Result<()> {
    let mut file = File::options()
        .truncate(true)
        .write(true)
        .create(true)
        .open(USERS_FILE)?;

    let user_json = serde_json::to_string(users).unwrap();
    file.write_all(user_json.as_bytes())?;

    Ok(())
}

#[async_trait]
pub trait Visible {
    async fn is_visible(&self, user: &User) -> bool;
}

struct BoardId<'a>(&'a str);

#[async_trait]
impl<'a> Visible for BoardId<'a> {
    async fn is_visible(&self, user: &User) -> bool {
        for visible_board_name in &user.boards {
            unsafe {
                for map in BOARD_MAP.iter() {
                    if visible_board_name == &map.board_name && &self.0 == &map.board_id {
                        return true;
                    }
                }
            }

        }

        false
    }
}


#[async_trait]
impl Visible for Card {
    async fn is_visible(&self, user: &User) -> bool {
        let is_board_visible = BoardId(&self.id_board).is_visible(user).await;

        if !is_board_visible {
            return false;
        }

        if user.is_moderator() {
            return true;
        }

        self.labels.iter().any(|label| label.name == user.name)
    }
}

#[async_trait]
impl Visible for Board {
    async fn is_visible(&self, user: &User) -> bool {
        BoardId(&self.id).is_visible(user).await
    }
}

#[async_trait]
impl Visible for List {
    async fn is_visible(&self, user: &User) -> bool {
        BoardId(&self.board_id).is_visible(user).await
    }
}
