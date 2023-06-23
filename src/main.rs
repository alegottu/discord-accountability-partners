use std::env;
use serenity::
{
    async_trait,
    model::{prelude::*, channel::Message, gateway::Ready},
    prelude::*,
    Client
};

const TEST_MESSAGE: &str = "Testing";
const TEST_COMMAND: &str = "!test";

struct Handler;

#[async_trait]
impl EventHandler for Handler
{
    async fn message(&self, ctx: Context, msg: Message)
    {
        if msg.content == TEST_COMMAND
        {
            if let Err(why) = msg.channel_id.say(&ctx.http, TEST_MESSAGE).await
            {
                println!("Error sending message: {:?}", why);
            }
        }
    }

    async fn ready(&self, _: Context, ready: Ready)
    {
        println!("{} has connected", ready.user.name);
    }
}

#[tokio::main]
async fn main() {
    let token = env::var("TOKEN")
        .expect("Expected a token in the environment");

    // Sets what bot is notified about
    let intents = GatewayIntents::MESSAGE_CONTENT;

    let mut client = Client::builder(&token, intents)
        .event_handler(Handler).await.expect("Error creating client");

    if let Err(why) = client.start().await
    {
        println!("Error with client: {:?}", why);
    }
}

