use std::env;
use std::sync::{Arc, Mutex};
use std::fs::{self, File};
use std::io::{self, BufReader};
use std::collections::HashMap;

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

const TASKS_FOLDER: &str = "/home/alegottu/Projects/Rust/discord-luh-bot/res/tasks";
const USERS_FOLDER: &str = "/home/alegottu/Projects/Rust/discord-luh-bot/res/users";

const TASKS_CHANNEL: u64 = 1100541860172271697;

const HELP: &str = "!help";
const ADD_TASK: &str = "!add";
//const BUY_ITEM: &str = "!buy";   

const HELP_MESSAGE: &str = "!add <task> adds a task";

fn load_tasks(tasks: &mut HashMap<u64, u8>) -> io::Result<()>
{
    for entry in fs::read_dir(TASKS_FOLDER)?
    {
        let entry = entry?;
        let file = File::open(entry.path())?; // Forward error if unable to open file
        let reader = BufReader::new(file);
        let task: Task = serde_json::from_reader(reader)?;
        tasks.insert(task.message_id, task.value);
    }

    Ok(())
}

fn load_users(users: &mut HashMap<u64, u64>) -> io::Result<()>
{
    for entry in fs::read_dir(USERS_FOLDER)?
    {
        let entry = entry?;
        let file = File::open(entry.path())?;
        let reader = BufReader::new(file);
        let user: User = serde_json::from_reader(reader)?;
        users.insert(user.id, user.points);
    }

    Ok(())
}

// Functions to save JSON files
fn save_task(task: Task, task_num: usize) -> io::Result<()>
{
    let text = serde_json::to_string(&task)?;
    fs::write(TASKS_FOLDER.to_owned() + "/" + &task_num.to_string(), text)?;

    Ok(())
}

fn save_user(user: User, user_num: usize) -> io::Result<()> 
{
    let text = serde_json::to_string(&user)?;
    fs::write(USERS_FOLDER.to_owned() + "/" + &user_num.to_string(), text)?;

    Ok(())
}

fn update_user(user_id: u64, points: u64) -> io::Result<()>
{
    let mut user_num = 0;

    for entry in fs::read_dir(USERS_FOLDER)?
    {
        let entry = entry?;
        let file = File::open(entry.path())?;
        let reader = BufReader::new(file);
        let mut user: User = serde_json::from_reader(reader)?;

        if user.id == user_id
        {
            user.points = points;
            let text = serde_json::to_string(&user)?;
            fs::write(USERS_FOLDER.to_owned() + "/" + &user_num.to_string(), text)?;
        }

        user_num += 1;
    }

    Ok(()) 
}

// HashMap must be in Arcs to be allocated on the heap
struct Handler
{
    tasks: Arc<Mutex<HashMap<u64, u8>>>,
    users: Arc<Mutex<HashMap<u64, u64>>>
}

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
        
        match message
        {
            HELP => 
            {
                if let Err(why) = msg.channel_id.say(&ctx.http, HELP_MESSAGE).await
                {
                    println!("Error sending message: {:?}", why);
                }
            },
            ADD_TASK => // Command takes the form !add <message_id> <point value>
            {
                let mut command_split = msg.content.split(' '); 
                command_split.next();
                let message_id: u64 = command_split.next().expect("Task message ID not provided")
                    .parse().expect("Invalid argument type provided for message ID");
                let value: u8 = command_split.next().expect("Task point value not provided")
                    .parse().expect("Invalid arugment type provided for point value");
                let task = Task { message_id: message_id, value: value }; 

                // Later secure that the message id provided is in the tasks channel
                let tasks_alloc = Arc::clone(&self.tasks);
                let tasks = tasks_alloc.lock().unwrap(); 
                save_task(task, tasks.len()).expect("Unable to save task file");
            },
            &_ => println!("Invalid command")
        }
    }

    async fn reaction_add(&self, _ctx: Context, reaction: Reaction)
    {
        if reaction.channel_id == TASKS_CHANNEL
        {
            if reaction.user_id.is_some()
            {
                let tasks_alloc = Arc::clone(&self.tasks); 
                let users_alloc = Arc::clone(&self.users);
                let tasks = tasks_alloc.lock().unwrap();
                let mut users = users_alloc.lock().unwrap();
                let user_id = reaction.user_id.unwrap().0;
                let message_id = reaction.message_id.as_u64();
                let user_exists = users.contains_key(&user_id);
                
                let points = users.entry(user_id)
                    .or_insert(0); 

                let amount: u8 = 
                    if tasks.contains_key(message_id)
                    {
                        *tasks.get(message_id).unwrap()
                    }
                    else
                    {
                        0
                    };

                *points += amount as u64;

                if user_exists
                {
                    update_user(user_id, *points)
                        .expect("Failed to update user file");
                }
                else
                {
                    let user = User { id: user_id, points: *points };
                    save_user(user, users.len() - 1)
                        .expect("Failed to create user file");
                }
            }
        }
    }
}

#[tokio::main]
async fn main() {
    let mut tasks: HashMap<u64, u8> = HashMap::new();
    let mut users: HashMap<u64, u64> = HashMap::new();
    load_tasks(&mut tasks).expect("Could not load tasks");
    load_users(&mut users).expect("Could not load users");

    let token = env::var("TOKEN")
        .expect("Expected a token in the environment");

    // Sets what bot is notified about
    let intents = GatewayIntents::MESSAGE_CONTENT
        | GatewayIntents::GUILD_MESSAGE_REACTIONS;

    let handler = Handler
    {
        tasks: Arc::new(Mutex::new(tasks)),
        users: Arc::new(Mutex::new(users))
    };
    let mut client = Client::builder(&token, intents)
        .event_handler(handler).await.expect("Error creating client");

    if let Err(why) = client.start().await
    {
        println!("Error with client: {:?}", why);
    }
}

