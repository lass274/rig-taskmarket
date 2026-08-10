//! Read-only [TaskMarket](https://taskmarket.dev/) tools for
//! [Rig](https://github.com/0xPlaygrounds/rig) agents.
//!
//! The crate intentionally exposes no signing, payment, submission, acceptance,
//! or other state-changing endpoint. This gives an agent useful market context
//! without granting it spending authority or access to a private key.

mod client;
mod models;
mod tools;

pub use client::{DEFAULT_BASE_URL, TaskmarketClient, TaskmarketError};
pub use models::{
    Artifact, BrowseTasksArgs, GetTaskArgs, ListSubmissionsArgs, Submission, SubmissionSummary,
    Task, TaskPage, TrackSubmissionsArgs, WalletBalance, WalletBalanceArgs,
};
pub use tools::{
    BrowseTasksTool, GetTaskTool, ListSubmissionsTool, TaskmarketTools, TrackSubmissionsTool,
    WalletBalanceTool,
};
