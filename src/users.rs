use anyhow;
use async_trait::async_trait;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs::File,
    io::{Read, Write},
};
use tokio::sync::Mutex;
use trellolon::{Board, Card, List};

const USERS_FILE: &'static str = "users.json";

#[derive(Debug, PartialEq, Eq, Hash, Clone, Serialize, Deserialize)]
struct BoardMap {}

lazy_static! {
    static ref BOARD_MAP: Mutex<HashMap<String, String>> = Mutex::new(HashMap::new());
}

impl BoardMap {
    pub async fn load() {
        let mut board_map = BOARD_MAP.lock().await;
        let boards = Board::get_all_boards().await.unwrap();

        for board in boards {
            board_map.insert(board.id, board.name);
        }
    }

    pub async fn id_to_name(id: &str) -> Option<String> {
        let mut board_map = BOARD_MAP.lock().await;
        if let Some(name) = board_map.get(id) {
            return Some(name.clone());
        } else {
            if let Some(board) = Board::get_by_id(id).await {
                board_map.insert(board.id, board.name.clone());
                return Some(board.name);
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

    pub async fn is_moderator(&self, board_id: &str) -> bool {
        if let Some(BoardPermission::Moderator) = self.get_permission(board_id).await {
            return true;
        }

        false
    }

    pub async fn get_permission(&self, board_id: &str) -> Option<BoardPermission> {
        let name = BoardMap::id_to_name(board_id).await;
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
        BoardMap::load().await;

        let file = File::options().read(true).open(USERS_FILE);
        if file.is_err() {
            return users;
        }

        let mut file = file.unwrap();
        let mut contents = String::new();

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

#[async_trait]
impl Visible for Card {
    async fn is_visible(&self, user: &User) -> bool {
        if let Some(permission) = user.get_permission(&self.id_board).await {
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
        user.get_permission(&self.id).await.is_some()
    }
}

#[async_trait]
impl Visible for List {
    async fn is_visible(&self, user: &User) -> bool {
        user.get_permission(&self.board_id).await.is_some()
    }
}
