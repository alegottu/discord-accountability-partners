A discord bot with a productivity system that you can use with your partner or friends to keep each other accountable and productive!
The idea is simple: you and your accountability partner(s) can set up tasks and rewards, each with their own point values and costs, respectively. 
When you react to a message that represents a task, you'll gain the corresponding points for it, and your accountability partner(s) will know about it!
When you react to a message that represents a reward, you'll use the sum of points equal to the cost from your total saved up, and you and your accountability partners know you can reward yourself, or perhaps even they'll be rewarding you!

The bot that I'm hosting is for my own personal use, so I won't be sharing any info like a bot invite link.
However, you're welcome to use the source code as you please, of course!

If you want to set up and host the bot for yourself, first you'll have to go through the usual steps to set up any discord bot. You can find out more [here](https://discord.com/developers/docs/quick-start/getting-started).
Once you have your own discord app set up, invite it to your server, and make sure it has Administrator priveleges. On that server, you'll want to have a few different channels.
One channel for tasks, one channel for rewards, and one channel for users. Personally I've made the users channel private, so that only the bot can see it, as it uses it to track and store each users point bank.
You can leave the users channel alone, but you'll have to populate the tasks and rewards channels of course. 
For each task you want to track, the format is:
`Task Description - X` where 'X' is the amount of points someone will recieve for completeing this task.
For each reward you want to track, the format is:
`Reward Description - X` where 'X' is the amount of points required to obtain this reward.
When that's finished, you'll need to set up the proper secrets, namely the IDs of those three channels, and the bot token, using SecureStore. A tutorial for that step can be found [here](https://neosmart.net/blog/securestore-open-secrets-format/).
You can see my secrets store file as an example, but yours should ultimately replace mine at the same location: `"secure/secrets.json"`
Once you have all of this set up, just run the build and run the bot using Rust! That is, use `cargo build` then `cargo run`. Simple as that! If you don't have Rust installed on your machine, you can start [here](https://rustup.rs/).
The bot takes up very little resources so I've found no problems just keeping it running or starting it up and leaving it in the background when I begin my day, but you of course could also host it with a cloud service permanently.

Currently this bot is very bare bones, but I'll plan on adding more features to track user stats besides point totals. Please let me know if you enjoy the bot or have any requests!
