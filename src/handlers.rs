use crate::MyHandlerType;
use teloxide::dispatching2::UpdateFilterExt;
use teloxide::prelude2::*;
use teloxide::types::Message;

//================MESSAGES=====================
use crate::message_commands::*;

pub fn basic_commands_handler() -> MyHandlerType {
    Update::filter_message().branch(
        dptree::entry()
            .filter_command::<BasicCommands>()
            .endpoint(basic_commands_endpoint),
    )
}

use crate::admin_commands::*;

pub fn admin_commands_handler() -> MyHandlerType {
    Update::filter_message().branch(
        dptree::entry()
            .filter_command::<AdminCommands>()
            .endpoint(admin_commands_endpoint),
    )
}

//================MESSAGES REPLIES=====================
use crate::replies::*;

pub fn reply_message_handler() -> MyHandlerType {
    Update::filter_message()
        //.branch(Message::filter_reply_to_message().endpoint(reply_message_endpoint))
        .branch(
            dptree::filter(|x: Message| x.reply_to_message().is_some())
                .endpoint(reply_message_endpoint),
        )
}

//================CALLBACKS=====================
use crate::inline_callbacks::*;

pub fn callbacks_handler() -> MyHandlerType {
    Update::filter_callback_query().branch(dptree::entry().endpoint(callback_command_endpoint))
}
