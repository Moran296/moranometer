use crate::users::{User, Users};
use crate::Moranometer;
use anyhow::anyhow;
use serde::{Deserialize, Serialize};
use teloxide::{prelude2::*, utils::command::BotCommand};
use tokio::spawn;

#[derive(Debug, BotCommand, Clone, Serialize, Deserialize)]
#[command(rename = "lowercase", description = "admin commands")]
pub enum AdminCommands {
    #[command(description = "update changes from users file")]
    Update,
    #[command(description = "exit the program")]
    Exit,
}

async fn get_user<'a>(msg: &'a Message, cfg: &'a Moranometer) -> anyhow::Result<User> {
    let user_id = msg.from().ok_or(anyhow!("No user id"))?.id;
    let cfg = cfg.lock().await;
    let user = cfg
        .users
        .get_user(user_id)
        .ok_or(anyhow!("No user found"))?;
    Ok(user.clone())
}

pub(crate) async fn admin_commands_endpoint(
    msg: Message,
    bot: AutoSend<Bot>,
    cmd: AdminCommands,
    cfg: Moranometer,
) -> anyhow::Result<()> {
    let user = get_user(&msg, &cfg).await?;
    if !user.is_admin() {
        return Err(anyhow!("You are not a admin"));
    }

    match cmd {
        AdminCommands::Update => {
            let mut cfg = cfg.lock().await;
            cfg.users = Users::load().await;
            bot.send_message(user.id, "Users updated").send().await?;
        }

        AdminCommands::Exit => {
            bot.send_message(user.id, "Exiting....").send().await?;
            spawn(async move {
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                std::process::exit(0);
            });
        }
    }

    Ok(())
}
