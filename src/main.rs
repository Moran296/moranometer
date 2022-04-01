use std::env;
use std::sync::Arc;
use teloxide::{prelude2::*, utils::command::BotCommand};
use tokio::sync::Mutex;
extern crate pretty_env_logger;
#[macro_use]
extern crate log;

mod handlers;
mod inline_callbacks;
mod message_commands;
mod replies;
mod users;

use message_commands::BasicCommands;
use users::Users;

type MyHandlerType = Handler<'static, DependencyMap, anyhow::Result<()>>;
type Moranometer = Arc<Mutex<MoranometerInner>>;

async fn set_command(bot: &AutoSend<Bot>) {
    bot.set_my_commands(BasicCommands::bot_commands())
        .send()
        .await
        .unwrap();
}

async fn dispatch<'a>(bot: AutoSend<Bot>, handler: MyHandlerType, dep: Moranometer) {
    Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![dep])
        .default_handler(|upd| async move {
            log::warn!("Unhandled update: {:#?}", upd);
        })
        .error_handler(LoggingErrorHandler::with_custom_text(
            "An error has occurred in the dispatcher",
        ))
        .build()
        .setup_ctrlc_handler()
        .dispatch()
        .await;
}

struct MoranometerInner {
    users: Users,
}

impl MoranometerInner {
    async fn create() -> Arc<Mutex<Self>> {
        Arc::new(Mutex::new(Self {
            users: Users::load().await,
        }))
    }
}

#[tokio::main]
async fn main() {
    pretty_env_logger::init();

    info!("~**==**MORANOMETER**==**~");

    let token = env::var("TELOXIDE_MORANOMETER").expect("TELEGRAM_BOT_TOKEN not found");
    let bot = Bot::new(token).auto_send();

    set_command(&bot).await;

    let handler = dptree::entry()
        .branch(handlers::basic_commands_handler())
        .branch(handlers::callbacks_handler())
        .branch(handlers::reply_message_handler());

    let moranometer = MoranometerInner::create().await;

    dispatch(bot, handler, moranometer).await;
}
