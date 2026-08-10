//! Integration tests for the public TaskMarket client surface.

use rig_taskmarket::{BrowseTasksArgs, TaskmarketClient};
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
