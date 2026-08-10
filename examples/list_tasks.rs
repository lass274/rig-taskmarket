//! Lists active TaskMarket tasks worth at least 10 USDC.

use rig_taskmarket::{BrowseTasksArgs, TaskmarketClient};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = TaskmarketClient::new();
    let page = client
        .browse_tasks(&BrowseTasksArgs {
            limit: Some(5),
            phase: Some("active".to_owned()),
            sort: Some("reward_desc".to_owned()),
            min_reward_usdc: Some("10".to_owned()),
            ..BrowseTasksArgs::default()
        })
        .await?;

    for task in page.tasks {
        println!(
            "{} | {} base units | {} submissions | {}",
            task.id, task.reward, task.submission_count, task.expiry_time
        );
    }
    Ok(())
}
