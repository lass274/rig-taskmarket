use reqwest::{Client, StatusCode};
use serde::de::DeserializeOwned;
use thiserror::Error;

use crate::{
    BrowseTasksArgs, ScreenTasksArgs, ScreenedTask, ScreenedTaskPage, Submission,
    SubmissionSummary, Task, TaskPage, WalletBalance,
};

/// TaskMarket's production API origin.
pub const DEFAULT_BASE_URL: &str = "https://api.taskmarket.dev";

/// Errors returned by the TaskMarket client and Rig tools.
#[derive(Debug, Error)]
pub enum TaskmarketError {
    /// An argument would produce an invalid or unsafe request.
    #[error("invalid TaskMarket request: {0}")]
    InvalidArgument(String),
    /// The HTTP transport failed.
    #[error("TaskMarket transport error: {0}")]
    Transport(#[from] reqwest::Error),
    /// TaskMarket returned a non-success status.
    #[error("TaskMarket returned HTTP {status}: {message}")]
    Api {
        /// HTTP status code.
        status: StatusCode,
        /// Bounded response text useful for diagnostics.
        message: String,
    },
}

/// Minimal, read-only client for TaskMarket's public API.
#[derive(Clone, Debug)]
pub struct TaskmarketClient {
    http: Client,
    base_url: String,
}

impl Default for TaskmarketClient {
    fn default() -> Self {
        Self::new()
    }
}

impl TaskmarketClient {
    /// Creates a production TaskMarket client.
    #[must_use]
    pub fn new() -> Self {
        Self {
            http: Client::new(),
            base_url: DEFAULT_BASE_URL.to_owned(),
        }
    }

    /// Creates a client with a custom API origin, useful for tests and proxies.
    pub fn with_base_url(base_url: impl Into<String>) -> Result<Self, TaskmarketError> {
        let base_url = base_url.into();
        let trimmed = base_url.trim_end_matches('/');
        let parsed = reqwest::Url::parse(trimmed).map_err(|error| {
            TaskmarketError::InvalidArgument(format!("invalid base URL: {error}"))
        })?;
        if !matches!(parsed.scheme(), "http" | "https") {
            return Err(TaskmarketError::InvalidArgument(
                "base URL must use http or https".to_owned(),
            ));
        }
        Ok(Self {
            http: Client::new(),
            base_url: trimmed.to_owned(),
        })
    }

    /// Lists public tasks using safe discovery filters.
    pub async fn browse_tasks(&self, args: &BrowseTasksArgs) -> Result<TaskPage, TaskmarketError> {
        let limit = args.limit.unwrap_or(20);
        if !(1..=100).contains(&limit) {
            return Err(TaskmarketError::InvalidArgument(
                "limit must be between 1 and 100".to_owned(),
            ));
        }

        let phase = args.phase.as_deref().unwrap_or("active");
        if !matches!(
            phase,
            "active" | "in_review" | "awaiting_settlement" | "resolved"
        ) {
            return Err(TaskmarketError::InvalidArgument(format!(
                "unsupported phase `{phase}`"
            )));
        }

        let sort = args.sort.as_deref().unwrap_or("reward_desc");
        if !matches!(
            sort,
            "newest" | "reward_desc" | "reward_asc" | "deadline_asc"
        ) {
            return Err(TaskmarketError::InvalidArgument(format!(
                "unsupported sort `{sort}`"
            )));
        }

        let mut query = vec![
            ("limit".to_owned(), limit.to_string()),
            ("phase".to_owned(), phase.to_owned()),
            ("sort".to_owned(), sort.to_owned()),
        ];
        if let Some(value) = &args.min_reward_usdc {
            query.push(("minReward".to_owned(), usdc_to_base_units(value)?));
        }
        if let Some(value) = &args.max_reward_usdc {
            query.push(("maxReward".to_owned(), usdc_to_base_units(value)?));
        }
        if let Some(tags) = &args.tags {
            for tag in tags {
                let tag = tag.trim();
                if !tag.is_empty() {
                    query.push(("tags".to_owned(), tag.to_owned()));
                }
            }
        }

        self.get("/api/tasks", &query).await
    }

    /// Lists public tasks and annotates them with deterministic, read-only policy checks.
    pub async fn screen_tasks(
        &self,
        args: &ScreenTasksArgs,
    ) -> Result<ScreenedTaskPage, TaskmarketError> {
        let blocked_terms = normalize_blocked_terms(args.blocked_terms.as_deref())?;
        let page = self.browse_tasks(&args.browse).await?;
        let exclude_stake = args.exclude_stake.unwrap_or(true);
        let require_open_window = args.require_open_window.unwrap_or(true);

        let tasks = page
            .tasks
            .into_iter()
            .map(|task| {
                let mut reasons = Vec::new();
                if exclude_stake && task.stake_required {
                    reasons.push("worker stake required".to_owned());
                }
                if require_open_window && !task.submission_window_open {
                    reasons.push("submission window is closed".to_owned());
                }
                if let Some(limit) = args.max_submission_count {
                    if task.submission_count > limit {
                        reasons.push(format!(
                            "submission count {} exceeds limit {limit}",
                            task.submission_count
                        ));
                    }
                }

                let searchable =
                    format!("{} {}", task.description, task.tags.join(" ")).to_lowercase();
                for term in &blocked_terms {
                    if searchable.contains(term) {
                        reasons.push(format!("blocked term matched: {term}"));
                    }
                }

                ScreenedTask {
                    eligible: reasons.is_empty(),
                    reasons,
                    task,
                }
            })
            .collect();

        Ok(ScreenedTaskPage {
            tasks,
            next_cursor: page.next_cursor,
            has_more: page.has_more,
        })
    }

    /// Fetches one public task by identifier.
    pub async fn get_task(&self, task_id: &str) -> Result<Task, TaskmarketError> {
        validate_path_id(task_id, "task_id")?;
        self.get(&format!("/api/tasks/{task_id}"), &[]).await
    }

    /// Lists a worker's TaskMarket submissions.
    pub async fn track_submissions(
        &self,
        worker_address: &str,
    ) -> Result<Vec<SubmissionSummary>, TaskmarketError> {
        validate_address(worker_address)?;
        self.get(
            "/api/submissions/mine",
            &[("workerAddress".to_owned(), worker_address.to_owned())],
        )
        .await
    }

    /// Lists the public submissions made to a task.
    pub async fn list_submissions(
        &self,
        task_id: &str,
    ) -> Result<Vec<Submission>, TaskmarketError> {
        validate_path_id(task_id, "task_id")?;
        self.get(
            &format!("/api/tasks/{task_id}/submissions"),
            &[("includePreviewUrls".to_owned(), "none".to_owned())],
        )
        .await
    }

    /// Reads the public TaskMarket USDC balance for an EVM address.
    pub async fn wallet_balance(&self, address: &str) -> Result<WalletBalance, TaskmarketError> {
        validate_address(address)?;
        self.get(
            "/api/wallet/balance",
            &[("address".to_owned(), address.to_owned())],
        )
        .await
    }

    async fn get<T: DeserializeOwned>(
        &self,
        path: &str,
        query: &[(String, String)],
    ) -> Result<T, TaskmarketError> {
        let response = self
            .http
            .get(format!("{}{path}", self.base_url))
            .query(query)
            .send()
            .await?;
        let status = response.status();
        if !status.is_success() {
            let message = response.text().await?;
            return Err(TaskmarketError::Api {
                status,
                message: bounded_message(&message),
            });
        }
        Ok(response.json().await?)
    }
}

fn validate_path_id(value: &str, name: &str) -> Result<(), TaskmarketError> {
    if value.is_empty() || value.contains(['/', '?', '#']) {
        return Err(TaskmarketError::InvalidArgument(format!(
            "{name} must be a non-empty path-safe identifier"
        )));
    }
    Ok(())
}

fn validate_address(address: &str) -> Result<(), TaskmarketError> {
    let valid = address.len() == 42
        && address.starts_with("0x")
        && address[2..].bytes().all(|byte| byte.is_ascii_hexdigit());
    if !valid {
        return Err(TaskmarketError::InvalidArgument(
            "address must be a 20-byte 0x-prefixed EVM address".to_owned(),
        ));
    }
    Ok(())
}

fn normalize_blocked_terms(terms: Option<&[String]>) -> Result<Vec<String>, TaskmarketError> {
    let terms = terms.unwrap_or_default();
    if terms.len() > 20 {
        return Err(TaskmarketError::InvalidArgument(
            "blocked_terms accepts at most 20 entries".to_owned(),
        ));
    }

    terms
        .iter()
        .map(|term| {
            let term = term.trim();
            if term.is_empty() || term.chars().count() > 64 {
                return Err(TaskmarketError::InvalidArgument(
                    "blocked terms must contain 1 to 64 characters".to_owned(),
                ));
            }
            Ok(term.to_lowercase())
        })
        .collect()
}

fn usdc_to_base_units(value: &str) -> Result<String, TaskmarketError> {
    let value = value.trim();
    if value.is_empty() || value.starts_with('-') || value.starts_with('+') {
        return Err(TaskmarketError::InvalidArgument(format!(
            "invalid USDC amount `{value}`"
        )));
    }
    let mut parts = value.split('.');
    let whole = parts.next().unwrap_or_default();
    let fraction = parts.next().unwrap_or_default();
    if parts.next().is_some()
        || whole.is_empty()
        || !whole.bytes().all(|byte| byte.is_ascii_digit())
        || !fraction.bytes().all(|byte| byte.is_ascii_digit())
        || fraction.len() > 6
    {
        return Err(TaskmarketError::InvalidArgument(format!(
            "invalid USDC amount `{value}`; use at most six decimals"
        )));
    }
    let whole: u128 = whole.parse().map_err(|_| {
        TaskmarketError::InvalidArgument(format!("USDC amount `{value}` is too large"))
    })?;
    let padded_fraction = format!("{fraction:0<6}");
    let fraction: u128 = padded_fraction
        .parse()
        .map_err(|_| TaskmarketError::InvalidArgument(format!("invalid USDC amount `{value}`")))?;
    whole
        .checked_mul(1_000_000)
        .and_then(|base| base.checked_add(fraction))
        .map(|base| base.to_string())
        .ok_or_else(|| {
            TaskmarketError::InvalidArgument(format!("USDC amount `{value}` is too large"))
        })
}

fn bounded_message(message: &str) -> String {
    const MAX_CHARS: usize = 1_024;
    let mut chars = message.chars();
    let bounded: String = chars.by_ref().take(MAX_CHARS).collect();
    if chars.next().is_some() {
        format!("{bounded}…")
    } else {
        bounded
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converts_usdc_without_floating_point() {
        assert_eq!(
            usdc_to_base_units("10.5").expect("valid amount"),
            "10500000"
        );
        assert_eq!(usdc_to_base_units("0.000001").expect("valid amount"), "1");
        assert_eq!(usdc_to_base_units("12").expect("valid amount"), "12000000");
    }

    #[test]
    fn rejects_ambiguous_usdc_amounts() {
        assert!(usdc_to_base_units("1.0000001").is_err());
        assert!(usdc_to_base_units("-1").is_err());
        assert!(usdc_to_base_units("1e3").is_err());
    }
}
