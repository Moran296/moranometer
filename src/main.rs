use std::env;
use teloxide::{prelude2::*, utils::command::BotCommand};
extern crate pretty_env_logger;
#[macro_use] extern crate log;

mod users;
mod handlers;
mod tellegram_commands;
use users::Users;
use tellegram_commands::{BasicCommands};

type MyHandlerType = Handler<'static, DependencyMap, anyhow::Result<()>>;

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

#[derive(Clone)]
struct Moranometer {
    users: Box<Users>,
}

#[tokio::main]
async fn main() {
    pretty_env_logger::init();

    info!("~**==**MORANOMETER**==**~");

    let token = env::var("TELOXIDE_MORANOMETER").expect("TELEGRAM_BOT_TOKEN not found");
    let bot = Bot::new(token).auto_send();

    set_command(&bot).await;

    let handler = dptree::entry()
        .branch(handlers::message_handler())
        .branch(handlers::callback_handler());

    let moranometer = Moranometer {
        users: Box::new(Users::load().await),
    };

    dispatch(bot, handler, moranometer).await;
}
