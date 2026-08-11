use rig_agent::tool::{Tool, ToolContext};
use serde_json::{Value, json};

use crate::{
    BrowseTasksArgs, GetTaskArgs, ListSubmissionsArgs, ScreenTasksArgs, ScreenedTaskPage,
    Submission, SubmissionSummary, Task, TaskPage, TaskmarketClient, TaskmarketError,
    TrackSubmissionsArgs, WalletBalance, WalletBalanceArgs,
};

/// Factory for a consistent set of read-only TaskMarket tools.
#[derive(Clone, Debug, Default)]
pub struct TaskmarketTools {
    client: TaskmarketClient,
}

impl TaskmarketTools {
    /// Uses TaskMarket's production API.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Uses a caller-supplied read-only client.
    #[must_use]
    pub fn with_client(client: TaskmarketClient) -> Self {
        Self { client }
    }

    /// Creates the task-discovery tool.
    #[must_use]
    pub fn browse_tasks(&self) -> BrowseTasksTool {
        BrowseTasksTool::new(self.client.clone())
    }

    /// Creates the task-discovery and policy-screening tool.
    #[must_use]
    pub fn screen_tasks(&self) -> ScreenTasksTool {
        ScreenTasksTool::new(self.client.clone())
    }

    /// Creates the task-detail tool.
    #[must_use]
    pub fn get_task(&self) -> GetTaskTool {
        GetTaskTool::new(self.client.clone())
    }

    /// Creates the worker-submission tracking tool.
    #[must_use]
    pub fn track_submissions(&self) -> TrackSubmissionsTool {
        TrackSubmissionsTool::new(self.client.clone())
    }

    /// Creates the requester-side submission review tool.
    #[must_use]
    pub fn list_submissions(&self) -> ListSubmissionsTool {
        ListSubmissionsTool::new(self.client.clone())
    }

    /// Creates the public balance tool.
    #[must_use]
    pub fn wallet_balance(&self) -> WalletBalanceTool {
        WalletBalanceTool::new(self.client.clone())
    }
}

/// Rig tool that discovers public TaskMarket tasks without signing or spending.
#[derive(Clone, Debug)]
pub struct BrowseTasksTool {
    client: TaskmarketClient,
}

impl BrowseTasksTool {
    /// Creates the tool from a client.
    #[must_use]
    pub fn new(client: TaskmarketClient) -> Self {
        Self { client }
    }
}

impl Tool for BrowseTasksTool {
    const NAME: &'static str = "taskmarket_browse_tasks";
    type Args = BrowseTasksArgs;
    type Output = TaskPage;
    type Error = TaskmarketError;

    fn description(&self) -> String {
        "Discover real public TaskMarket work using reward, lifecycle, tag, and sort filters. Read-only: never signs, submits, stakes, or spends funds.".to_owned()
    }

    fn parameters(&self) -> Value {
        browse_args_schema()
    }

    async fn call(
        &self,
        _context: &mut ToolContext,
        args: Self::Args,
    ) -> Result<Self::Output, Self::Error> {
        self.client.browse_tasks(&args).await
    }
}

/// Rig tool that discovers tasks and applies caller-defined eligibility checks.
#[derive(Clone, Debug)]
pub struct ScreenTasksTool {
    client: TaskmarketClient,
}

impl ScreenTasksTool {
    /// Creates the tool from a client.
    #[must_use]
    pub fn new(client: TaskmarketClient) -> Self {
        Self { client }
    }
}

impl Tool for ScreenTasksTool {
    const NAME: &'static str = "taskmarket_screen_tasks";
    type Args = ScreenTasksArgs;
    type Output = ScreenedTaskPage;
    type Error = TaskmarketError;

    fn description(&self) -> String {
        "Discover TaskMarket work and annotate every result against explicit local policy: stake, open submission window, competition, and blocked terms. Read-only and auditable; excluded tasks remain visible with reasons.".to_owned()
    }

    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "browse": browse_args_schema(),
                "max_submission_count": {"type": "integer", "minimum": 0},
                "exclude_stake": {"type": "boolean", "default": true},
                "require_open_window": {"type": "boolean", "default": true},
                "blocked_terms": {
                    "type": "array",
                    "maxItems": 20,
                    "items": {"type": "string", "minLength": 1, "maxLength": 64}
                }
            }
        })
    }

    async fn call(
        &self,
        _context: &mut ToolContext,
        args: Self::Args,
    ) -> Result<Self::Output, Self::Error> {
        self.client.screen_tasks(&args).await
    }
}

/// Rig tool that retrieves one public TaskMarket task.
#[derive(Clone, Debug)]
pub struct GetTaskTool {
    client: TaskmarketClient,
}

impl GetTaskTool {
    /// Creates the tool from a client.
    #[must_use]
    pub fn new(client: TaskmarketClient) -> Self {
        Self { client }
    }
}

impl Tool for GetTaskTool {
    const NAME: &'static str = "taskmarket_get_task";
    type Args = GetTaskArgs;
    type Output = Task;
    type Error = TaskmarketError;

    fn description(&self) -> String {
        "Inspect a TaskMarket task's specification, reward, deadline, stake requirement, phase, and competition before deciding whether to delegate or work.".to_owned()
    }

    fn parameters(&self) -> Value {
        task_id_schema()
    }

    async fn call(
        &self,
        _context: &mut ToolContext,
        args: Self::Args,
    ) -> Result<Self::Output, Self::Error> {
        self.client.get_task(&args.task_id).await
    }
}

/// Rig tool that tracks a worker wallet's TaskMarket submissions.
#[derive(Clone, Debug)]
pub struct TrackSubmissionsTool {
    client: TaskmarketClient,
}

impl TrackSubmissionsTool {
    /// Creates the tool from a client.
    #[must_use]
    pub fn new(client: TaskmarketClient) -> Self {
        Self { client }
    }
}

impl Tool for TrackSubmissionsTool {
    const NAME: &'static str = "taskmarket_track_submissions";
    type Args = TrackSubmissionsArgs;
    type Output = Vec<SubmissionSummary>;
    type Error = TaskmarketError;

    fn description(&self) -> String {
        "Track TaskMarket submissions for an EVM worker address, including status, reward, rejection time, and transaction or deliverable hashes. Read-only.".to_owned()
    }

    fn parameters(&self) -> Value {
        address_schema("worker_address")
    }

    async fn call(
        &self,
        _context: &mut ToolContext,
        args: Self::Args,
    ) -> Result<Self::Output, Self::Error> {
        self.client.track_submissions(&args.worker_address).await
    }
}

/// Rig tool that presents public task submissions for requester-side review.
#[derive(Clone, Debug)]
pub struct ListSubmissionsTool {
    client: TaskmarketClient,
}

impl ListSubmissionsTool {
    /// Creates the tool from a client.
    #[must_use]
    pub fn new(client: TaskmarketClient) -> Self {
        Self { client }
    }
}

impl Tool for ListSubmissionsTool {
    const NAME: &'static str = "taskmarket_list_submissions";
    type Args = ListSubmissionsArgs;
    type Output = Vec<Submission>;
    type Error = TaskmarketError;

    fn description(&self) -> String {
        "Present public submissions and immutable artifact hashes for human review. Read-only: this tool cannot accept, reject, download private previews, or pay workers.".to_owned()
    }

    fn parameters(&self) -> Value {
        task_id_schema()
    }

    async fn call(
        &self,
        _context: &mut ToolContext,
        args: Self::Args,
    ) -> Result<Self::Output, Self::Error> {
        self.client.list_submissions(&args.task_id).await
    }
}

/// Rig tool that reads a public TaskMarket USDC balance.
#[derive(Clone, Debug)]
pub struct WalletBalanceTool {
    client: TaskmarketClient,
}

impl WalletBalanceTool {
    /// Creates the tool from a client.
    #[must_use]
    pub fn new(client: TaskmarketClient) -> Self {
        Self { client }
    }
}

impl Tool for WalletBalanceTool {
    const NAME: &'static str = "taskmarket_wallet_balance";
    type Args = WalletBalanceArgs;
    type Output = WalletBalance;
    type Error = TaskmarketError;

    fn description(&self) -> String {
        "Read the confirmed TaskMarket USDC balance of an EVM address. This tool cannot withdraw, transfer, sign, or spend.".to_owned()
    }

    fn parameters(&self) -> Value {
        address_schema("address")
    }

    async fn call(
        &self,
        _context: &mut ToolContext,
        args: Self::Args,
    ) -> Result<Self::Output, Self::Error> {
        self.client.wallet_balance(&args.address).await
    }
}

fn task_id_schema() -> Value {
    json!({
        "type": "object",
        "properties": {"task_id": {"type": "string", "minLength": 1}},
        "required": ["task_id"]
    })
}

fn browse_args_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "limit": {"type": "integer", "minimum": 1, "maximum": 100},
            "phase": {"type": "string", "enum": ["active", "in_review", "awaiting_settlement", "resolved"]},
            "sort": {"type": "string", "enum": ["newest", "reward_desc", "reward_asc", "deadline_asc"]},
            "tags": {"type": "array", "items": {"type": "string"}},
            "min_reward_usdc": {"type": "string", "pattern": "^[0-9]+(?:\\.[0-9]{1,6})?$"},
            "max_reward_usdc": {"type": "string", "pattern": "^[0-9]+(?:\\.[0-9]{1,6})?$"}
        }
    })
}

fn address_schema(name: &str) -> Value {
    json!({
        "type": "object",
        "properties": {(name): {"type": "string", "pattern": "^0x[0-9a-fA-F]{40}$"}},
        "required": [name]
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn address_schema_uses_the_requested_argument_name() {
        let schema = address_schema("worker_address");
        assert!(schema["properties"]["worker_address"].is_object());
        assert!(schema["properties"]["name"].is_null());
        assert_eq!(schema["required"][0], "worker_address");
    }

    #[test]
    fn tool_names_are_stable_and_distinct() {
        let names = [
            BrowseTasksTool::NAME,
            ScreenTasksTool::NAME,
            GetTaskTool::NAME,
            TrackSubmissionsTool::NAME,
            ListSubmissionsTool::NAME,
            WalletBalanceTool::NAME,
        ];
        let unique: std::collections::HashSet<_> = names.into_iter().collect();
        assert_eq!(unique.len(), names.len());
    }
}
