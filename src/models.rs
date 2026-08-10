use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Arguments accepted by [`crate::BrowseTasksTool`].
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct BrowseTasksArgs {
    /// Maximum tasks to return. TaskMarket accepts values from 1 through 100.
    pub limit: Option<u16>,
    /// Lifecycle phase: `active`, `in_review`, `awaiting_settlement`, or `resolved`.
    pub phase: Option<String>,
    /// Sort order: `newest`, `reward_desc`, `reward_asc`, or `deadline_asc`.
    pub sort: Option<String>,
    /// Required tags. Each value is sent as a separate `tags` query parameter.
    pub tags: Option<Vec<String>>,
    /// Minimum reward in decimal USDC, represented as a string (for example `10.5`).
    pub min_reward_usdc: Option<String>,
    /// Maximum reward in decimal USDC, represented as a string.
    pub max_reward_usdc: Option<String>,
}

/// Arguments accepted by [`crate::GetTaskTool`].
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GetTaskArgs {
    /// On-chain TaskMarket task identifier.
    pub task_id: String,
}

/// Arguments accepted by [`crate::TrackSubmissionsTool`].
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TrackSubmissionsArgs {
    /// Worker EVM address.
    pub worker_address: String,
}

/// Arguments accepted by [`crate::ListSubmissionsTool`].
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ListSubmissionsArgs {
    /// On-chain TaskMarket task identifier.
    pub task_id: String,
}

/// Arguments accepted by [`crate::WalletBalanceTool`].
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct WalletBalanceArgs {
    /// EVM address whose TaskMarket USDC balance should be read.
    pub address: String,
}

/// A TaskMarket task, with the fields most useful to an agent deciding whether to delegate.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Task {
    /// Task identifier.
    pub id: String,
    /// Requester wallet.
    pub requester: String,
    /// Full task specification.
    pub description: String,
    /// Gross reward in six-decimal USDC base units.
    pub reward: String,
    /// Net worker reward in six-decimal USDC base units, when supplied by the API.
    #[serde(default)]
    pub net_reward: Option<String>,
    /// ISO-8601 creation timestamp.
    pub created_at: String,
    /// ISO-8601 expiry timestamp.
    pub expiry_time: String,
    /// Contract-level task status.
    pub status: String,
    /// Searchable task tags.
    #[serde(default)]
    pub tags: Vec<String>,
    /// Task settlement mode.
    pub mode: String,
    /// User-facing lifecycle phase.
    #[serde(default)]
    pub phase: Option<String>,
    /// Whether submitting requires a worker stake.
    #[serde(default)]
    pub stake_required: bool,
    /// Whether new submissions are accepted.
    #[serde(default)]
    pub submission_window_open: bool,
    /// Number of received submissions.
    #[serde(default)]
    pub submission_count: u64,
    /// Number of awards made.
    #[serde(default)]
    pub award_count: u64,
    /// Primary award information, if one exists.
    #[serde(default)]
    pub primary_award: Option<Value>,
}

/// A paginated response from TaskMarket's public task directory.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskPage {
    /// Tasks on this page.
    pub tasks: Vec<Task>,
    /// Cursor for the next page.
    pub next_cursor: Option<String>,
    /// Whether another page is available.
    pub has_more: bool,
}

/// A worker's submission summary returned by `/submissions/mine`.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SubmissionSummary {
    /// Task identifier.
    pub task_id: String,
    /// Task description captured for the worker view.
    pub task_description: String,
    /// Current task status.
    pub task_status: String,
    /// Task mode.
    pub task_mode: String,
    /// Gross reward in USDC base units.
    pub task_reward: String,
    /// ISO-8601 submission timestamp.
    pub submitted_at: String,
    /// Deliverable hash, when available.
    pub deliverable_hash: Option<String>,
    /// Submission transaction hash, when available.
    pub submit_tx_hash: Option<String>,
    /// Rejection timestamp, when rejected.
    pub rejected_at: Option<String>,
}

/// A TaskMarket artifact attached to a submission.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Artifact {
    /// Artifact identifier.
    pub id: String,
    /// Artifact role, such as `preview`, `source`, or `final`.
    pub role: String,
    /// Original file name.
    pub file_name: String,
    /// Declared MIME type.
    pub mime_type: String,
    /// High-level media kind.
    pub media_kind: String,
    /// Content size in bytes.
    pub size_bytes: u64,
    /// SHA-256 content hash.
    pub sha256_hash: String,
    /// Optional text preview supplied by TaskMarket.
    #[serde(default)]
    pub text_preview: Option<String>,
}

/// A public TaskMarket submission suitable for requester-side review.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Submission {
    /// Submission identifier.
    pub id: String,
    /// Task identifier.
    pub task_id: String,
    /// Worker wallet.
    pub worker_address: String,
    /// Legacy file URL, when present.
    #[serde(default)]
    pub file_url: Option<String>,
    /// ISO-8601 submission timestamp.
    pub submitted_at: String,
    /// Rejection timestamp, when rejected.
    #[serde(default)]
    pub rejected_at: Option<String>,
    /// Attached artifacts and their immutable hashes.
    #[serde(default)]
    pub artifacts: Vec<Artifact>,
}

/// A public TaskMarket USDC balance.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WalletBalance {
    /// EVM address returned by the API.
    pub address: String,
    /// Balance in six-decimal USDC base units.
    pub balance_base_units: String,
    /// Human-readable decimal USDC balance.
    pub balance_usdc: String,
}
