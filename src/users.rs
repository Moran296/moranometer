use anyhow;
use serde::{Deserialize, Serialize};
use std::{
    fs::File,
    io::{Read, Write},
};

const USERS_FILE: &'static str = "users.json";

#[derive(Debug, PartialEq, Eq, Hash, Clone, Serialize, Deserialize)]
struct User {
    name: String,
    id: i64,
    boards: Vec<String>,
}

#[derive(Clone)]
pub struct Users {
    db: Vec<User>,
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

    pub fn get_boards(&self, name: &str) -> Option<Vec<String>> {
        let user = self.db.iter().find(|u| u.name == name)?;

        if user.boards.is_empty() {
            return None;
        }

        Some(user.boards.clone())
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
