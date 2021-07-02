use serenity::{async_trait, 
    model::gateway::Ready, 
    prelude::*, 
    framework::standard::StandardFramework
};
use std::time::Instant;
use std::error::Error;

use crate::deribit;
use crate::deribit::Instrument;

async fn manage_alerts(ctx: &Context) -> Result<(), Box<dyn Error>> {
    let mut info = deribit::get_instrument_info()?;
    let mut now = Instant::now();
    loop {

        if now.elapsed().as_secs_f64() >= 3600.0 {
            let new_info = deribit::get_instrument_info()?;

            for i in 0..info.len() {
                if deribit::alert_event(&info[i], &new_info[i]) {
                    send_alert(ctx, &info[i], &new_info[i]).await
                }
            }
            info = new_info;
            now = Instant::now();
        }
    }
}

async fn send_alert(ctx: &Context, old: &Instrument, new: &Instrument) {
    let target = serenity::model::id::ChannelId(860552501785657429);
    let vol_change = new.volume - old.volume;
    let oi_change = new.oi - old.oi;
    let name = &new.name;

    let msg = format!("```Name: {}\nOI Change: {}\nVolume Change: {}\ngvol.io```", name, oi_change, vol_change);

    target.say(ctx, &msg).await.unwrap();

}

pub async fn launch_discord() {
    let token = "NzIzNTU4MzY1MjYzMDM2NDI4.XuzYPQ.6l-RAW7zOby7UazklzkBm9r59us";
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