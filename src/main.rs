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
    pub value: u16 // How many points the task is worth
}

#[derive(Serialize, Deserialize, Debug)]
struct User
{
    pub id: u64,
    pub points: u64 // Accumulated points for this user
}

#[derive(Serialize, Deserialize, Debug)]
struct Reward
{
    pub message_id: u64, // The message to react to buy this reward
    pub cost: u16 // How many points the reward costs
}

const TASKS_FOLDER: &str = "/home/alegottu/Projects/Rust/discord-luh-bot/res/tasks";
const USERS_FOLDER: &str = "/home/alegottu/Projects/Rust/discord-luh-bot/res/users";
const REWARDS_FOLDER: &str = "/home/alegottu/Projects/Rust/discord-luh-bot/res/rewards";

const TASKS_CHANNEL: u64 = 1100541860172271697;
const REWARDS_CHANNEL: u64 = 1100541860172271697; // INNACURATE CHANGE LATER

const HELP: &str = "!help";
const ADD_TASK: &str = "!add task";
// const DELETE_TASK: &str = "!delete task";
const ADD_REWARD: &str = "!add reward";
// const DELETE_REWARD: &str = "!delete reward";
const CHECK_BALANCE: &str = "!balance";

// Below requires thorough checks or otherwise infinite money
// const SELL_REWARD: &str = "!sell";

const HELP_MESSAGE: &str = "!add <task_message_id> <point_value> adds a task\n!balance checks your point balance";

fn load_tasks(tasks: &mut HashMap<u64, u16>) -> io::Result<()>
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

fn load_rewards(rewards: &mut HashMap<u64, u16>) -> io::Result<()>
{
    for entry in fs::read_dir(REWARDS_FOLDER)?
    {
        let entry = entry?;
        let file = File::open(entry.path())?;
        let reader = BufReader::new(file);
        let reward: Reward = serde_json::from_reader(reader)?;
        rewards.insert(reward.message_id, reward.cost);
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

fn save_reward(reward: Reward, reward_num: usize) -> io::Result<()>
{
    let text = serde_json::to_string(&reward)?;
    fs::write(REWARDS_FOLDER.to_owned() + "/" + &reward_num.to_string(), text)?;

    Ok(())
}

// HashMap must be in Arcs to be allocated on the heap
struct Handler
{
    tasks: Arc<Mutex<HashMap<u64, u16>>>,
    users: Arc<Mutex<HashMap<u64, u64>>>,
    rewards: Arc<Mutex<HashMap<u64, u16>>>
}

async fn send_message(text: &String, ctx: Context, channel_id: ChannelId)
{
    if let Err(why) = channel_id.say(&ctx.http, text).await
    {
        println!("Error sending message: {:?}", why);
    }
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
                send_message(&HELP_MESSAGE.to_owned(), ctx, msg.channel_id).await; 
            },
            CHECK_BALANCE =>
            {
                let points = Arc::clone(&self.users).lock().unwrap()
                    [&msg.author.id.0].to_string();
                let message_to_send = format!("{}{}{}", "You have ", points, " total");
                send_message(&message_to_send, ctx, msg.channel_id).await;
            },
            &_ =>
            {
                if message.contains(ADD_TASK)
                {
                    // Command takes the form !add task <message_id> <point value>
                    let mut command_split = msg.content.split(' '); 
                    command_split.nth(1);
                    let message_id: u64 = command_split.next().expect("Task message ID not provided")
                        .parse().expect("Invalid argument type provided for message ID");
                    let value: u16 = command_split.next().expect("Task point value not provided")
                        .parse().expect("Invalid arugment type provided for point value");
                    let task = Task { message_id: message_id, value: value }; 

                    let tasks_alloc = Arc::clone(&self.tasks);
                    let tasks = tasks_alloc.lock().unwrap(); 
                    save_task(task, tasks.len()).expect("Unable to save task file");
                }
                else if message.contains(ADD_REWARD) // Possibly to try to merge some of this with above case, command split the same
                {
                    // Command takes the form !add reward <message_id> <cost>
                    let mut command_split = msg.content.split(' '); 
                    command_split.nth(1);
                    let message_id: u64 = command_split.next().expect("Reward message ID not provided")
                        .parse().expect("Invalid argument type provided for message ID");
                    let cost: u16 = command_split.next().expect("Reward cost not provided")
                        .parse().expect("Invalid arugment type provided for cost");
                    let reward = Reward { message_id: message_id, cost: cost };

                    let rewards_alloc = Arc::clone(&self.rewards);
                    let rewards = rewards_alloc.lock().unwrap(); 
                    save_reward(reward, rewards.len()).expect("Unable to save reward file");
                }
                else 
                {
                    println!("Invalid command");
                }
            }
        }
    }

    async fn reaction_add(&self, ctx: Context, reaction: Reaction)
    {
        if reaction.user_id.is_some()
        {
            if reaction.channel_id == TASKS_CHANNEL
            {
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

                    let amount: u16 = 
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

                // Make sure bot has "MANAGE_MESSAGES" permission
                if let Err(why) = reaction.delete(&ctx.http).await
                {
                    println!("Error deleting recent reaction {:?}", why);
                }
            }
            else if reaction.channel_id == REWARDS_CHANNEL
            {
                let mut fail = false;

                {
                    let rewards_alloc = Arc::clone(&self.rewards); 
                    let users_alloc = Arc::clone(&self.users);
                    let rewards = rewards_alloc.lock().unwrap();
                    let mut users = users_alloc.lock().unwrap();
                    let user_id = reaction.user_id.unwrap().0;
                    let message_id = reaction.message_id.as_u64();

                    let points = users.entry(user_id)
                        .or_insert(0); 

                    if rewards.contains_key(message_id)
                    {
                        let cost = *rewards.get(message_id).unwrap() as u64;

                        if *points >= cost
                        {
                            *points -= cost;
                        }
                        else
                        {
                            // To indicate that a message should be sent later once futures are dropped
                            fail = true;
                        }
                    }
                    else
                    {
                        println!("Reaction sent from invalid message in rewards channel");
                    }
                }
                
                if fail
                {
                    send_message(&"Insufficient points to purchase this reward".to_owned(), ctx, reaction.channel_id).await;
                }

                // User can remove reaction themselves to indicate the reward has been used
            }
        }

    }
}

#[tokio::main]
async fn main() 
{
    let mut tasks: HashMap<u64, u16> = HashMap::new();
    let mut users: HashMap<u64, u64> = HashMap::new();
    let mut rewards: HashMap<u64, u16> = HashMap::new();
    load_tasks(&mut tasks).expect("Could not load tasks");
    load_users(&mut users).expect("Could not load users");
    load_rewards(&mut rewards).expect("Could not load rewards");

    let token = env::var("TOKEN")
        .expect("Expected a token in the environment");

    // Sets what bot is notified about
    let intents = GatewayIntents::MESSAGE_CONTENT
        | GatewayIntents::GUILD_MESSAGE_REACTIONS;

    let handler = Handler
    {
        tasks: Arc::new(Mutex::new(tasks)),
        users: Arc::new(Mutex::new(users)),
        rewards: Arc::new(Mutex::new(rewards))
    };
    let mut client = Client::builder(&token, intents)
        .event_handler(handler).await.expect("Error creating client");

    if let Err(why) = client.start().await
    {
        println!("Error with client: {:?}", why);
    }
}

