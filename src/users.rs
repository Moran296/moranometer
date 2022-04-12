use anyhow;
use async_trait::async_trait;
use core::panic;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs::File,
    io::{Read, Write},
};
use trellolon::{Board, Card, List};

const USERS_FILE: &'static str = "users.json";
const BOARDS_FILE: &'static str = "boards.json";

#[derive(Debug, PartialEq, Eq, Hash, Clone, Serialize, Deserialize)]
struct BoardMap {
    pub board_name: String,
    pub board_id: String,
}

static mut BOARD_MAP: Vec<BoardMap> = vec![];

impl BoardMap {
    async fn load_from_file() {
        let file = File::options().read(true).open(BOARDS_FILE);
        if file.is_err() {
            panic!();
        }

        let mut file = file.unwrap();
        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap();

        let boards: Vec<BoardMap> = serde_json::from_str::<Vec<BoardMap>>(&contents).unwrap();
        unsafe {
            BOARD_MAP = boards;
            if BOARD_MAP.is_empty() {
                panic!();
            }
        }

        ()
    }

    pub fn id_to_name(id: &str) -> Option<String> {
        for board in unsafe { &BOARD_MAP } {
            if board.board_id == id {
                return Some(board.board_name.clone());
            }
        }

        None
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BoardPermission {
    Moderator,
    SeeAll,
    ByLabel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub name: String,
    pub id: i64,
    pub admin: bool,
    pub boards: HashMap<String, BoardPermission>,
}

impl User {
    pub fn is_admin(&self) -> bool {
        self.admin
    }

    pub fn is_moderator(&self, board_id: &str) -> bool {
        if let Some(BoardPermission::Moderator) = self.get_permission(board_id) {
            return true;
        }

        false
    }

    pub fn get_permission(&self, board_id: &str) -> Option<BoardPermission> {
        let name = BoardMap::id_to_name(board_id);
        if let Some(name) = name {
            self.boards.get(&name).cloned()
        } else {
            None
        }
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
            let mut map = HashMap::new();
            map.insert("TestBoard".to_string(), BoardPermission::SeeAll);

            let user = User {
                name: name.to_owned(),
                id: id,
                admin: false,
                boards: map,
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
impl Visible for Card {
    async fn is_visible(&self, user: &User) -> bool {
        if let Some(permission) = user.get_permission(&self.id_board) {
            match permission {
                BoardPermission::Moderator => true,
                BoardPermission::SeeAll => true,
                BoardPermission::ByLabel => self.labels.iter().any(|label| label.name == user.name),
            }
        } else {
            false
        }
    }
}

#[async_trait]
impl Visible for Board {
    async fn is_visible(&self, user: &User) -> bool {
        user.get_permission(&self.id).is_some()
    }
}

#[async_trait]
impl Visible for List {
    async fn is_visible(&self, user: &User) -> bool {
        user.get_permission(&self.board_id).is_some()
    }
}
