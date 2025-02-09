use std::error::Error;

mod scheduler;
mod upload;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();
    scheduler::setup_scheduler().await?;
    Ok(())
}
