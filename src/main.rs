use std::collections::HashMap;
use std::sync::Arc;
use async_std::sync::Mutex;
use securestore::{KeySource, SecretsManager};
use std::path::Path;
use once_cell::sync::Lazy;
use std::env;

use serenity::
{
    async_trait,
    model::
    {
        prelude::*, 
        channel::Message,
        gateway::Ready
    },
    builder::EditMessage,
    prelude::*,
    futures::StreamExt,
    Client
};

const HELP: &str = "!help";
const CHECK_BALANCE: &str = "!balance";

// Below requires thorough checks or otherwise infinite money
// const SELL_REWARD: &str = "!sell";

const HELP_MESSAGE: &str = "!balance to check your current AP balance";

// Expects users to write all objects in the correct format through their own messages
async fn load_objects(map: &mut HashMap<u64, u64>, user_posts: &mut Option<&mut HashMap<u64, u64>>, ctx: Context, channel_id: u64) 
{
    let channel_id = ChannelId::new(channel_id);
    let mut messages = channel_id.messages_iter(&ctx).boxed(); 

    while let Some(message_result) = messages.next().await 
    {
        match message_result
        {
            Ok(message) =>
            {
                let mut args = message.content.split(" - ");
                let wild = args.next();
                let point_arg: u64 = args.next()
                    .expect("Not enough arguments in message")
                    .parse().expect("Invalid type given for point argument");

                if user_posts.is_some()
                {
                    let user_id: u64 = wild.expect("Not enough arguments in message")
                        .parse().expect("Invalid type given for point argument");
                    map.insert(user_id, point_arg);
                    user_posts.as_mut().unwrap().insert(user_id, message.id.get());
                }
                else
                {
                    map.insert(message.id.get(), point_arg);
                }
            },
            Err(error) => 
            {
                println!("{:?}", error);
            }
        }
    }
}

async fn create_user(user_id: u64, points: u64, channel: u64, ctx: Context, users: &mut HashMap<u64, u64>, user_posts: &mut HashMap<u64, u64>)
{
    let text = format!("{} - {}", user_id, points);
    let channel_id = ChannelId::new(channel);
    let message_id = send_message(&text, ctx, channel_id)
        .await.expect("Unable to send message"); 
    let user_post_id = message_id;

    users.insert(user_id, points);
    user_posts.insert(user_id, user_post_id); 
}

// Edit user channel messages
async fn update_user(message_id: u64, user_id: u64, points: u64, channel: u64, ctx: Context)
{
    let channel_id = ChannelId::new(channel);
    let text = format!("{} - {}", user_id, points);
    let builder = EditMessage::new().content(text);

    if let Err(why) = channel_id.edit_message(&ctx.http, message_id, builder).await
    {
        println!("{:?}", why);
    }
    
    // Update user map during reaction response
}

// HashMap must be in Arcs to be allocated on the heap
struct Handler
{
    rewards: Arc<Mutex<HashMap<u64, u64>>>,
    tasks: Arc<Mutex<HashMap<u64, u64>>>,
    users: Arc<Mutex<HashMap<u64, u64>>>,
    user_posts: Arc<Mutex<HashMap<u64, u64>>>, // Maps each user to their coresponding tracking post
    message: String, // Secret
    rewards_channel: u64,
    tasks_channel: u64,
    users_channel: u64,
    bot_id: u64,
    contact_id: u64,
    self_id: u64
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

// Returns message ID of message sent if successful
async fn send_message(text: &String, ctx: Context, channel_id: ChannelId) -> Result<u64, ()>
{
    let message_result = channel_id.say(&ctx.http, text).await;
    
    match message_result
    {
        Ok(message) => return Ok(message.id.get()),
        Err(_error) => return Err(()) 
    }
}

#[async_trait]
impl EventHandler for Handler
{
    async fn ready(&self, ctx: Context, ready: Ready)
    {
        if self.contact_id != 0
        {
            send_private(&self.message, ctx.clone(), UserId::new(self.contact_id)).await;
            send_private(&self.message, ctx.clone(), UserId::new(self.self_id)).await;
        }

        let tasks_alloc = Arc::clone(&self.tasks); 
        let users_alloc = Arc::clone(&self.users);
        let user_posts_alloc = Arc::clone(&self.user_posts);
        let rewards_alloc = Arc::clone(&self.rewards);
        let mut tasks = tasks_alloc.lock().await;
        let mut users = users_alloc.lock().await;
        let mut user_posts = user_posts_alloc.lock().await;
        let mut rewards = rewards_alloc.lock().await;

        load_objects(&mut rewards, &mut None, ctx.clone(), self.rewards_channel).await;
        load_objects(&mut tasks, &mut None, ctx.clone(), self.tasks_channel).await;
        load_objects(&mut users, &mut Some(&mut user_posts), ctx.clone(), self.users_channel).await;

        println!("{} has connected", ready.user.name);
    }

    async fn message(&self, ctx: Context, msg: Message)
    {
        if msg.author.id.get() == self.bot_id { return; } 

        let message = msg.content.as_str();
        
        match message
        {
            HELP =>
            {
                let _ = send_message(&HELP_MESSAGE.to_owned(), ctx.clone(), msg.channel_id).await; 
            },
            CHECK_BALANCE =>
            {
                let points = Arc::clone(&self.users).lock().await
                    [&msg.author.id.get()].to_string();
                let message_to_send = format!("{}{}{}", "You have ", points, " AP");
                send_private(&message_to_send, ctx.clone(), msg.author.id).await;
            },
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
            if reaction.channel_id == self.rewards_channel
            {
                let rewards_alloc = Arc::clone(&self.rewards); 
                let users_alloc = Arc::clone(&self.users);
                let user_posts_alloc = Arc::clone(&self.user_posts);
                let rewards = rewards_alloc.lock().await;
                let mut users = users_alloc.lock().await;
                let user_posts = user_posts_alloc.lock().await;
                let user_id = reaction.user_id.unwrap().get();
                let message_id: &u64 = &reaction.message_id.get();

                let points_entry = users.entry(user_id)
                    .or_insert(0); 

                // if user does not exist or has 0 points, quit early
                if rewards.contains_key(message_id)
                {
                    let cost = *rewards.get(message_id).unwrap() as u64;

                    if *points_entry >= cost
                    {
                        *points_entry -= cost;
                        let points = *points_entry;
                        *users.get_mut(&user_id).expect("User does not exist") = points;

                        let user_post_id = *user_posts.get(&user_id)
                            .expect("Unable to find user post");
                        update_user(user_post_id, user_id, points, self.users_channel, ctx.clone()).await;
                        let message_to_send = format!(
                            "Reward purchased! Remove your reaction once you have used this reward. Your balance is now {} AP",
                            points);
                        send_private(&message_to_send, 
                            ctx.clone(), reaction.user_id.unwrap()).await;
                    }
                    else
                    {
                        send_private(&"Insufficient points to purchase this reward".to_owned(), ctx.clone(), reaction.user_id.unwrap()).await;

                        if let Err(why) = reaction.delete(&ctx.http).await
                        {
                            println!("Error deleting recent reward reaction {:?}", why);
                        }
                    }
                }
                else
                {
                    println!("Reaction sent from invalid message in rewards channel");
                }
            }
            else if reaction.channel_id == self.tasks_channel
            {
                let tasks_alloc = Arc::clone(&self.tasks); 
                let users_alloc = Arc::clone(&self.users);
                let user_posts_alloc = Arc::clone(&self.user_posts);
                let tasks = tasks_alloc.lock().await;
                let mut users = users_alloc.lock().await;
                let mut user_posts = user_posts_alloc.lock().await;
                let user_id = reaction.user_id.unwrap().get();
                let message_id: &u64 = &reaction.message_id.get();
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

                *points_entry += amount;
                let points = *points_entry;

                if user_exists
                {
                    let user_post_id = user_posts.get_mut(&user_id)
                        .expect("Unable to find user's post");
                    update_user(*user_post_id, user_id, points, self.users_channel, ctx.clone())
                        .await;
                }
                else
                {
                    create_user(user_id, points, self.users_channel, ctx.clone(), &mut users, &mut user_posts)
                        .await;
                }

                let message_to_send = format!("Task complete! You now have a total of {} AP", points); 
                send_private(&message_to_send, ctx.clone(), reaction.user_id.unwrap()).await;

                // Make sure bot has "MANAGE_MESSAGES" permission
                if let Err(why) = reaction.delete(&ctx.http).await
                {
                    println!("Error deleting recent task reaction {:?}", why);
                }
            }
        }
    }
}

#[tokio::main]
async fn main() 
{
    // might not need to include channel IDs in secret store
    let secrets: Lazy<SecretsManager> = Lazy::new(|| {
        let keyfile = Path::new("secure/secrets.key");
        SecretsManager::load("secure/secrets.json", KeySource::File(keyfile))
            .expect("Failed to load SecureStore vault!")
    });
    let token = secrets.get("token").expect("Could not find secret 'token'");
    let mut contact_id: u64 = 0;
    let mut self_id: u64 = 0;
    let mut message = String::new();

    // NOTE: For now handling the actual logic that would go here in a bash script
    if env::args().count() > 0
    {
        contact_id = secrets.get("contact_id")
            .expect("Could not find secret 'contact_id'")
            .parse().expect("Contact was not a valid ID");
        self_id = secrets.get("self_id")
            .expect("Could not find secret 'self_id'")
            .parse().expect("Self was not a valid ID");
        message = secrets.get("contact_message")
            .expect("Could not find secret message");
    }

    let tasks: HashMap<u64, u64> = HashMap::new();
    let users: HashMap<u64, u64> = HashMap::new();
    let user_posts: HashMap<u64, u64> = HashMap::new();
    let rewards: HashMap<u64, u64> = HashMap::new();

    let rewards_channel: u64 = secrets.get("rewards_channel").expect("Could not find secret 'rewards_channel'")
        .parse().unwrap();
    let tasks_channel: u64 = secrets.get("tasks_channel").expect("Could not find secret 'tasks_channel'")
        .parse().unwrap();
    let users_channel: u64 = secrets.get("users_channel").expect("Could not find secret 'users_channel'")
        .parse().unwrap();
    let bot_id: u64 = secrets.get("bot_id").expect("Could not find secret 'bot_id'")
        .parse().unwrap();

    // Sets what bot is notified about
    let intents = GatewayIntents::MESSAGE_CONTENT
        | GatewayIntents::GUILD_MESSAGE_REACTIONS
        | GatewayIntents::DIRECT_MESSAGES;

    let handler = Handler
    {
        rewards: Arc::new(Mutex::new(rewards)),
        tasks: Arc::new(Mutex::new(tasks)),
        users: Arc::new(Mutex::new(users)),
        user_posts: Arc::new(Mutex::new(user_posts)),
        message,
        rewards_channel,
        tasks_channel,
        users_channel,
        bot_id,
        contact_id,
        self_id
    };
    let mut client = Client::builder(&token, intents)
        .event_handler(handler).await.expect("Error creating client");

    if let Err(why) = client.start().await
    {
        println!("Error with client: {:?}", why);
    }
}

