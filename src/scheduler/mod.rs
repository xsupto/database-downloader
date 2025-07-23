use crate::upload;
use log::{error, info};
use std::error::Error;
use std::process::Output;
use std::time::Duration;
use tokio::process::Command;
use tokio_cron_scheduler::{Job, JobScheduler};

async fn run_script(script_path: &str) -> Result<Output, Box<dyn Error + Send + Sync>> {
    match Command::new("sh").arg("-c").arg(script_path).output().await {
        Ok(output) => Ok(output),
        Err(e) => {
            error!("Failed to execute script: {}", e);
            Err(Box::new(e))
        }
    }
}

async fn run_job() -> Result<(), Box<dyn Error + Send + Sync>> {
    let script_path = "./backup.sh";

    match run_script(script_path).await {
        Ok(output) => {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let file_name = stdout.replace("./", "").trim().to_string();
                upload::run(&file_name).await;
                info!("Script output: {}", stdout);
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                error!("Script error: {}", stderr);
            }
        }
        Err(e) => {
            error!("Failed to run script: {}", e);
        }
    }

    Ok(())
}

async fn run_delete_job() -> Result<(), Box<dyn Error + Send + Sync>> {
    upload::delete_files().await;
    info!("File deletion successfully completed.");
    Ok(())
}

pub async fn setup_scheduler() -> Result<(), Box<dyn Error>> {
    let sched = JobScheduler::new().await?;

    let backup_corn = std::env::var("BACKUP_CRON").unwrap_or_else(|_| "0 0 */23 * * *".to_string());
    let clean_corn = std::env::var("CLEAN_CRON").unwrap_or_else(|_| "0 0 6 * * Sat".to_string());

    sched
        .add(Job::new_async(backup_corn, |uuid, mut l| {
            Box::pin(async move {
                match run_job().await {
                    Ok(()) => info!("Backup job completed successfully"),
                    Err(e) => error!("Backup job failed: {}", e),
                }

                let next_tick = l.next_tick_for_job(uuid).await;
                match next_tick {
                    Ok(Some(ts)) => info!("Next backup scheduled for: {:?}", ts),
                    _ => error!("Could not determine next backup time"),
                }
            })
        })?)
        .await?;

    sched
        .add(Job::new_async(clean_corn, |uuid, mut l| {
            Box::pin(async move {
                match run_delete_job().await {
                    Ok(()) => info!("Successfully delete old db backups"),
                    Err(e) => error!("Failed to delete old db backup: {}", e),
                }

                let next_tick = l.next_tick_for_job(uuid).await;
                match next_tick {
                    Ok(Some(ts)) => info!("Next scheduled for database delete: {:?}", ts),
                    _ => error!("Could not determine next database delete time"),
                }
            })
        })?)
        .await?;

    sched.start().await?;

    loop {
        tokio::time::sleep(Duration::from_secs(3600)).await;
        info!("Scheduler is running...");
    }
}
