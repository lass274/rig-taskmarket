//! Integration tests for the public TaskMarket client surface.

use rig_taskmarket::{BrowseTasksArgs, ScreenTasksArgs, TaskmarketClient};
use serde_json::json;
use wiremock::{
    Mock, MockServer, ResponseTemplate,
    matchers::{method, path, query_param},
};

#[tokio::test]
async fn browse_tasks_converts_usdc_and_deserializes_results() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/tasks"))
        .and(query_param("limit", "3"))
        .and(query_param("phase", "active"))
        .and(query_param("sort", "reward_desc"))
        .and(query_param("minReward", "10500000"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "tasks": [{
                "id": "0xabc",
                "requester": "0x0000000000000000000000000000000000000001",
                "description": "Integrate a real tool",
                "reward": "12000000",
                "netReward": "11100000",
                "createdAt": "2026-08-09T00:00:00Z",
                "expiryTime": "2026-08-20T00:00:00Z",
                "status": "open",
                "tags": ["rust"],
                "mode": "bounty",
                "phase": "active",
                "stakeRequired": false,
                "submissionWindowOpen": true,
                "submissionCount": 2,
                "awardCount": 0,
                "primaryAward": null
            }],
            "nextCursor": null,
            "hasMore": false
        })))
        .mount(&server)
        .await;

    let client = TaskmarketClient::with_base_url(server.uri()).expect("mock URL is valid");
    let page = client
        .browse_tasks(&BrowseTasksArgs {
            limit: Some(3),
            min_reward_usdc: Some("10.5".to_owned()),
            ..BrowseTasksArgs::default()
        })
        .await
        .expect("mock request succeeds");

    assert_eq!(page.tasks.len(), 1);
    assert_eq!(page.tasks[0].net_reward.as_deref(), Some("11100000"));
    assert!(!page.has_more);
}

#[tokio::test]
async fn non_success_status_is_bounded_and_typed() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/tasks/0xmissing"))
        .respond_with(ResponseTemplate::new(404).set_body_string("not found"))
        .mount(&server)
        .await;

    let client = TaskmarketClient::with_base_url(server.uri()).expect("mock URL is valid");
    let error = client
        .get_task("0xmissing")
        .await
        .expect_err("404 is an error");
    assert!(error.to_string().contains("HTTP 404"));
}

#[tokio::test]
async fn screen_tasks_keeps_exclusion_reasons_auditable() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/tasks"))
        .and(query_param("limit", "20"))
        .and(query_param("phase", "active"))
        .and(query_param("sort", "reward_desc"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "tasks": [
                task_json("0xsafe", "Write a Rust adapter", false, true, 2),
                task_json("0xstake", "Stake to enter", true, true, 1),
                task_json("0xhuman", "Complete this HUMAN_ONLY interview", false, true, 1),
                task_json("0xcrowded", "Write another adapter", false, true, 12)
            ],
            "nextCursor": "next-page",
            "hasMore": true
        })))
        .mount(&server)
        .await;

    let client = TaskmarketClient::with_base_url(server.uri()).expect("mock URL is valid");
    let page = client
        .screen_tasks(&ScreenTasksArgs {
            max_submission_count: Some(5),
            blocked_terms: Some(vec!["human_only".to_owned()]),
            ..ScreenTasksArgs::default()
        })
        .await
        .expect("screening succeeds");

    assert!(page.tasks[0].eligible);
    assert_eq!(page.tasks[1].reasons, ["worker stake required"]);
    assert_eq!(page.tasks[2].reasons, ["blocked term matched: human_only"]);
    assert_eq!(
        page.tasks[3].reasons,
        ["submission count 12 exceeds limit 5"]
    );
    assert_eq!(page.next_cursor.as_deref(), Some("next-page"));
    assert!(page.has_more);
}

fn task_json(
    id: &str,
    description: &str,
    stake: bool,
    window: bool,
    submissions: u64,
) -> serde_json::Value {
    json!({
        "id": id,
        "requester": "0x0000000000000000000000000000000000000001",
        "description": description,
        "reward": "12000000",
        "netReward": "11100000",
        "createdAt": "2026-08-09T00:00:00Z",
        "expiryTime": "2026-08-20T00:00:00Z",
        "status": "open",
        "tags": ["rust"],
        "mode": "bounty",
        "phase": "active",
        "stakeRequired": stake,
        "submissionWindowOpen": window,
        "submissionCount": submissions,
        "awardCount": 0,
        "primaryAward": null
    })
}
