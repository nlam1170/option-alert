use serenity::{async_trait, 
    model::gateway::Ready, 
    prelude::*, 
    framework::standard::StandardFramework
};
use std::time::Instant;
use std::error::Error;

use crate::deribit;
use crate::deribit::{Instruments, Data};


async fn manage_alerts(ctx: &Context) -> Result<(), Box<dyn Error>> {
    let mut info = Instruments::new()?;
    let mut now = Instant::now();
    loop {
        if now.elapsed().as_secs() >= 3600 {
            let new_info = Instruments::new()?;

            for (name, data) in &new_info.0 {
                if info.0.contains_key(name) {
                    if deribit::check_alert_event(info.0.get(name).unwrap(), &data) {
                        send_alert(ctx, name, info.0.get(name).unwrap(), &data).await;
                    }
                }

            }

            info = new_info;
            now = Instant::now();
            
        }
    }
}

async fn send_alert(ctx: &Context, name: &str, old: &Data, new: &Data) {
    let target = serenity::model::id::ChannelId(860552501785657429);
    let oi_change = new.oi - old.oi;
    let vol_change = new.volume - old.volume;

    let msg = format!("<@&860619201765310495>```Name: {}\n1hr OI Change: {}\n1hr Volume Change: {}\n```", name, oi_change, vol_change);

    target.say(ctx, &msg).await.unwrap();
    
    let _ = target.send_message(&ctx.http, |m| {
        m.embed(|e| {
            e.field("Laevitas", "https://app.laevitas.ch/dashboard/btc/deribit/options/activity?flow=OFlow", false);
            e
        });
        m
    }).await;

}

pub async fn launch_discord() {
    let token = "ODYwNjA2ODEyNTAzODAxOTE2.YN9sjQ.ZJj18cHbSXdYgTXZJjI_UUkJJSg";
    let framework = StandardFramework::new();
    let mut client = Client::builder(&token)
    .framework(framework)
        .event_handler(Handler).await
        .expect("Error creating client");
    
    if let Err(why) = client.start().await {
        println!("{:?}", why);
    }
}

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, bot_data: Ready) {
        println!("{} is connected", bot_data.user.name);

        loop {
            match manage_alerts(&ctx).await {
                Err(_) => continue,
                _ => unreachable!()
            }
        }
    }
}