use std::env;
use std::fs::{self, File};
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
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
struct Task
{
    pub message_id: u64, // The message to react to to gain points
    pub value: u8 // How many points the task is worth
}

#[derive(Serialize, Deserialize, Debug)]
struct User
{
    pub id: u64,
    pub points: u64 // Accumulated points for this user
}

// Message ID mapped to task value
let mut tasks: HashMap<u64, u8> = HashMap::new();
// User ID mapped to accumulated points
let mut users: HashMap<u64, u64> = HashMap::new();
// Must be global to used with interface for event handlers

let const str& TASKS_FOLDER = "../../res/tasks";
let const str& USERS_FOLDER = "../../res/users";

let const u64 TASKS_CHANNEL = 1100541860172271697;

let const str& ADD_TASK = "!add";
let const str& RM_TASK = "!remove";
let const str& BUY_ITEM = "!buy";

fn load_tasks(path: const &str) -> Result<(), Box<Error>>
{
    for task in fs::read_dir(path)?
    {
        let _task = task?
        let file = File::open(task)?; // Forward error if unable to open file
        let reader = BufReader::new(file);
        let task: Task = serde_json::from_reader(reader)?;
        tasks.push(task);
    }

    Ok(());
}

fn load_users(path: const &str) -> Result<(), Box<Error>>
{
    for user in fs::read_dir(path)?
    {
        let _user = user?
        let file = File::open(user)?; // Forward error if unable to open file
        let reader = BufReader::new(file);
        let user: User = serde_json::from_reader(reader)?;
        users.push(user);
    }

    Ok(());
}

fn save_item()
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
        if reaction.channel_id == TASKS_CHANNEL
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

