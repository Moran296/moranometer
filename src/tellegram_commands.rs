use teloxide::utils::command::BotCommand;

#[derive(BotCommand, Clone)]
#[command(rename = "lowercase", description = "These commands are supported:")]
pub enum BasicCommands {
    #[command(description = "welcome to the MoranOmeter")]
    Start,
    #[command(description = "display this text.")]
    Help,
    #[command(description = "see your cards", rename = "get_cards")]
    MyCards,
}

#[derive(BotCommand, Clone)]
#[command(rename = "lowercase", description = "These commands are supported:")]
pub enum CallbackCommands {}