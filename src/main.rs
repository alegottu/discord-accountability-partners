use std::env;
use std::fs::File;
use std::io::BufReader;
use std::vec;
use std::collections::HashMap;
use std::error::Error;

use serenity::
{
    async_trait,
    model::{prelude::*, channel::Message, gateway::Ready},
    prelude::*,
    Client
};
use serde::Deserialize;

#[derive(Deserialize, Debug)]
struct Task
{
    pub message_id: u64, // The message to react to to gain points
    pub value: u8 // How many points the task is worth
}

#[derive(Deserialize, Debug)]
struct User
{
    pub id: u64,
    pub points: u64 // Accumlated points for this user
}

// Message ID mapped to task value
let mut tasks: HashMap<u64, u8> = HashMap::new();
// User ID mapped to accumulated points
let mut users: HashMap<u64, u64> = HashMap::new();
// Must be global to used with interface for event handlers

fn load_tasks(path: const &str) -> Result<(), Box<Error>>
{
    let file = File::open(path)?; // ? = Forward error if unable to open file
    let reader = BufReader::new(file) // Create read-only buffer
    let _tasks: Vec<Task> = serde_json::from_reader(reader)?;

    for task in _tasks.iter()
    {
        tasks.insert(task.message_id, task.value);
    }

    Ok(());
}

fn load_users(path: const &str) -> Result<(), Box<Error>>
{
    let file = File::open(path)?; // ? = Forward error if unable to open file
    let reader = BufReader::new(file) // Create read-only buffer
    let _users: Vec<User> = serde_json::from_reader(reader)?;

    for user in _users.iter()
    {
        users.insert(user.id, user.points);
    }

    Ok(());
}

// If the bot closes, write update json files
fn quit(tasks_path: const &str, users_path: const &str)
{

}

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
        if msg.content == "!test"
        {
            if let Err(why) = msg.channel_id.say(&ctx.http, "hello").await
            {
                println!("Error sending message: {:?}", why);
            }
        }
    }

    async fn reaction_add(&self, ctx: Context, reaction: Reaction)
    {
        if reaction.channel_id == 1100541860172271697 // Channel for tasks
        {
            let points = users.entry(reaction.user_id).or_insert(0);
            *points += tasks.contains_key(reaction.message_id) ?
                tasks.get(reaction.message_id) : 0;
        }
    }
}

#[tokio::main]
async fn main() {
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

