use std::env;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;

use serenity::
{
    async_trait,
    model::
    {
        prelude::*, 
        channel::Message,
        gateway::Ready
    },
    prelude::*,
    futures::StreamExt,
    Client
};

// Make it so that these can be set after runtime using commands in a DM with Admin
const TASKS_CHANNEL: u64 = 1131139960129474602;
const REWARDS_CHANNEL: u64 = 1131139932040200343;
const USERS_CHANNEL: u64 = 1135811709936861195;
const BOT_ID: u64 = 1131137661998993420;

const HELP: &str = "!help";
const CHECK_BALANCE: &str = "!balance";
const RELOAD: &str = "!reload";

// Below requires thorough checks or otherwise infinite money
// const SELL_REWARD: &str = "!sell";

const HELP_MESSAGE: &str = "!balance to check your current LP balance";

// Expects users to write all objects in the correct format through their own messages
async fn load_objects(map: &mut HashMap<u64, u64>, ctx: Context, channel_id: u64) 
{
    let channel_id = ChannelId(channel_id);
    let mut messages = channel_id.messages_iter(&ctx).boxed(); 

    while let Some(message_result) = messages.next().await 
    {
        match message_result
        {
            Ok(message) =>
            {
                let point_arg = message.content
                    .split(" ").nth(1)
                    .expect("Not enough arguments in message")
                    .parse().expect("Invalid type given for point argument");
                map.insert(message.id.0, point_arg);
            },
            Err(error) => 
            {
                println!("{:?}", error);
            }
        }
    }
}

// Returns message ID of message sent if successful
async fn send_message(text: &String, ctx: Context, channel_id: ChannelId) -> Result<u64, ()>
{
    let message_result = channel_id.say(&ctx.http, text).await;
    
    match message_result
    {
        Ok(message) => return Ok(message.id.0),
        Err(error) => return Err(()) 
    }
}

// Create a message for a user upon their first reaction
async fn save_user(user_id: u64, starting_points: u64, ctx: Context, users: &mut HashMap<u64, u64>, user_posts: &mut HashMap<u64, u64>)
{
    let text = format!("{} {}", user_id, starting_points);
    let channel_id = ChannelId(USERS_CHANNEL);
    let message_id = send_message(&text, ctx, channel_id)
        .await.expect("Unable to send message"); 

    users.insert(user_id, starting_points);
    user_posts.insert(user_id, message_id);
}

// Edit user channel messages
async fn update_user(user_id: u64, message_id: u64,  points: u64, ctx: Context, users: &mut HashMap<u64, u64>)
{
    let channel_id = ChannelId(USERS_CHANNEL);

    if let Err(why) = channel_id.edit_message(&ctx.http, message_id,
        |message| message.content(points.to_string())).await
    {
        println!("{:?}", why);
    }
    
    *users.get_mut(&user_id).expect("User does not exist") = points;
}

// HashMap must be in Arcs to be allocated on the heap
struct Handler
{
    tasks: Arc<Mutex<HashMap<u64, u64>>>,
    users: Arc<Mutex<HashMap<u64, u64>>>,
    user_posts: Arc<Mutex<HashMap<u64, u64>>>, // Maps each user to their coresponding tracking post
    rewards: Arc<Mutex<HashMap<u64, u64>>>
}

async fn send_private(text: &String, ctx: Context, user_id: UserId)
{
    let dm = user_id.create_dm_channel(&ctx.http).await;
    
    if let Err(why) = dm
    {
        println!("Error creating private channel with user: {:?}", why);
    }
    else if let Err(why) = dm.unwrap().say(&ctx.http, text).await
    {
        println!("Error sending private message to user: {:?}", why); 
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
        if msg.author.id.0 == BOT_ID { return; } 

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
                let message_to_send = format!("{}{}{}", "You have ", points, " LP");
                send_message(&message_to_send, ctx, msg.channel_id).await;
            },
            RELOAD =>
            {
                // Load objects x3
            }
            &_ =>
            {
                println!("Invalid command");
            }
        }
    }

    async fn reaction_add(&self, ctx: Context, reaction: Reaction)
    {
        if reaction.user_id.is_some()
        {
            if reaction.channel_id == TASKS_CHANNEL
            {
                let points;

                {
                    let tasks_alloc = Arc::clone(&self.tasks); 
                    let users_alloc = Arc::clone(&self.users);
                    let user_posts_alloc = Arc::clone(&self.user_posts);
                    let tasks = tasks_alloc.lock().unwrap();
                    let mut users = users_alloc.lock().unwrap();
                    let user_posts = user_posts_alloc.lock().unwrap();
                    let user_id = reaction.user_id.unwrap().0;
                    let message_id = reaction.message_id.as_u64();
                    let user_exists = users.contains_key(&user_id);

                    let points_entry = users.entry(user_id)
                        .or_insert(0); 

                    let amount: u64 = 
                        if tasks.contains_key(message_id)
                        {
                            *tasks.get(message_id).unwrap()
                        }
                        else
                        {
                            0
                        };

                    *points_entry += amount as u64;
                    points = *points_entry;

                    if user_exists
                    {
                        *users.get_mut(&user_id).unwrap() = points;
                        let user_post_id = user_posts.get(user_id);
                        update_user(user_id, user_post_id, points, ctx, &mut users)
                            .await
                            .expect("Failed to update user file");
                    }
                    else
                    {
                        users.insert(user_id, points);
                        let user_post_id = save_user(user_id, points, ctx, &mut users, &mut user_posts)
                            .await?
                            .expect("Failed to create user file");
                    }
                }

                // Make sure bot has "MANAGE_MESSAGES" permission
                if let Err(why) = reaction.delete(&ctx.http).await
                {
                    println!("Error deleting recent task reaction {:?}", why);
                }

                let message_to_send = format!("Task complete! You now have a total of {} LP", points); 
                send_private(&message_to_send, ctx, reaction.user_id.unwrap()).await;
            }
            else if reaction.channel_id == REWARDS_CHANNEL
            {
                let mut fail = false;
                let mut points: u64 = 0;

                {
                    let rewards_alloc = Arc::clone(&self.rewards); 
                    let users_alloc = Arc::clone(&self.users);
                    let rewards = rewards_alloc.lock().unwrap();
                    let mut users = users_alloc.lock().unwrap();
                    let user_id = reaction.user_id.unwrap().0;
                    let message_id = reaction.message_id.as_u64();

                    let points_entry = users.entry(user_id)
                        .or_insert(0); 

                    if rewards.contains_key(message_id)
                    {
                        let cost = *rewards.get(message_id).unwrap() as u64;

                        if *points_entry >= cost
                        {
                            *points_entry -= cost;
                            points = *points_entry;
                            update_user(user_id, points).expect("Failed to update user");
                        }
                        else
                        {
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
                    let _ctx = ctx.clone();
                    send_message(&"Insufficient points to purchase this reward".to_owned(), _ctx, reaction.user_id.unwrap()).await;

                    if let Err(why) = reaction.delete(&ctx.http).await
                    {
                        println!("Error deleting recent reward reaction {:?}", why);
                    }
                }
                else
                {
                    let message_to_send = format!(
                        "Reward purchased! Remove your reaction oonce you have used this reward. Your balance is now {} LP",
                        points);
                    send_message(&message_to_send, 
                        ctx, reaction.user_id.unwrap()).await;
                }
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
        | GatewayIntents::GUILD_MESSAGE_REACTIONS
        | GatewayIntents::DIRECT_MESSAGES;

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

