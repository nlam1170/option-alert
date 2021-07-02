mod deribit;
mod discord;

#[tokio::main]
async fn main() {
    discord::launch_discord().await;
}
