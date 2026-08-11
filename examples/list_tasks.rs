//! Lists active TaskMarket tasks worth at least 10 USDC.

use rig_taskmarket::{BrowseTasksArgs, ScreenTasksArgs, TaskmarketClient};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = TaskmarketClient::new();
    let page = client
        .screen_tasks(&ScreenTasksArgs {
            browse: BrowseTasksArgs {
                limit: Some(5),
                phase: Some("active".to_owned()),
                sort: Some("reward_desc".to_owned()),
                min_reward_usdc: Some("10".to_owned()),
                ..BrowseTasksArgs::default()
            },
            max_submission_count: Some(20),
            blocked_terms: Some(vec!["human_only".to_owned(), "stake".to_owned()]),
            ..ScreenTasksArgs::default()
        })
        .await?;

    for screened in page.tasks.into_iter().filter(|task| task.eligible) {
        let task = screened.task;
        println!(
            "{} | {} base units | {} submissions | {}",
            task.id, task.reward, task.submission_count, task.expiry_time
        );
    }
    Ok(())
}
