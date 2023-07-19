use std::env;

use serenity::
{
    async_trait,
    model::{prelude::*, channel::Message, gateway::Ready},
    prelude::*,
    Client
};

const TASKS_CHANNEL: u64 = 1131139960129474602;
const REWARDS_CHANNEL: u64 = 1131139932040200343;

const HELP: &str = "!help";
const HELP_MESSAGE: &str = "!add <task_message_id> <point_value> adds a task\n!balance checks your point balance";

struct Handler;

#[async_trait]
impl EventHandler for Handler
{
    async fn ready(&self, _: Context, ready: Ready)
    {
        println!("{} has connected", ready.user.name);
    }

    async fn message(&self, ctx: Context, msg: Message)
    {
        let message = msg.content.as_str();
        println!("Sucess receiving message");
        
        match message
        {
            HELP =>
            {
                if let Err(why) = msg.channel_id.say(&ctx.http, HELP_MESSAGE).await
                {
                    println!("Error sending message: {:?}", why);
                }
            },
            &_ =>
            {
                println!("Invalid command");
            }
        }
    }

    async fn reaction_add(&self, _: Context, reaction: Reaction)
    {
        if reaction.user_id.is_some()
        {
            if reaction.channel_id == TASKS_CHANNEL
            {
                println!("Successfully received reaction from tasks channel");
            }
            else if reaction.channel_id == REWARDS_CHANNEL
            {
                println!("Successfully received reaction from rewards channel");
            }
        }

    }
}

#[tokio::main]
async fn main() 
{
    let token = env::var("TOKEN")
        .expect("Expected a token in the environment");

    // Sets what bot is notified about
    let intents = GatewayIntents::MESSAGE_CONTENT
        | GatewayIntents::GUILD_MESSAGE_REACTIONS;

    let mut client = Client::builder(&token, intents)
        .event_handler(Handler).await.expect("Error creating client");

    if let Err(why) = client.start().await
    {
        println!("Error with client: {:?}", why);
    }
}

