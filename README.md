# rig-taskmarket

`rig-taskmarket` gives [Rig](https://github.com/0xPlaygrounds/rig) agents a real,
read-only integration with [TaskMarket](https://taskmarket.dev/). It lets an
agent recognize delegatable work, inspect live bounties, track a worker's
submissions, present public submissions for review, and confirm wallet balances.

## Security boundary

This crate deliberately implements only public `GET` endpoints. It has no private
key input and no signing, x402 payment, withdrawal, submission, acceptance,
rejection, staking, or task-creation method. Adding the tools to an agent cannot
grant spending authority. A host application can implement an explicit,
human-approved write flow separately.

## Tools

| Rig tool | TaskMarket flow |
| --- | --- |
| `taskmarket_browse_tasks` | Discover work with reward, lifecycle, tag, and sort filters |
| `taskmarket_screen_tasks` | Apply auditable stake, window, competition, and term policies to discovered work |
| `taskmarket_get_task` | Inspect specifications, reward, deadline, stake, and competition |
| `taskmarket_track_submissions` | Track a worker's status and transaction/deliverable hashes |
| `taskmarket_list_submissions` | Present public artifacts and immutable hashes for review |
| `taskmarket_wallet_balance` | Confirm the public TaskMarket USDC balance of a wallet |

## Install

```toml
[dependencies]
rig-taskmarket = { git = "https://github.com/lass274/rig-taskmarket" }
```

## Attach the tools to a Rig agent

```rust,no_run
use rig_taskmarket::TaskmarketTools;

let taskmarket = TaskmarketTools::new();
let agent = provider
    .agent("your-model")
    .preamble("Use TaskMarket when work is better delegated to external workers. Never imply that read-only discovery authorizes spending.")
    .tool(taskmarket.browse_tasks())
    .tool(taskmarket.screen_tasks())
    .tool(taskmarket.get_task())
    .tool(taskmarket.track_submissions())
    .tool(taskmarket.list_submissions())
    .tool(taskmarket.wallet_balance())
    .build();
```

The provider setup is intentionally omitted because these tools work with any
Rig completion provider.

## Reproduce

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features
cargo run --example list_tasks
```

The test suite uses a local mock server and spends no funds. The example calls
TaskMarket's public production API and lists active tasks worth at least 10 USDC.

## Why Rig

Rig is an established Rust framework for building portable, modular AI agents.
Its typed `Tool` interface is a natural place to expose TaskMarket as a safe
delegation and review surface. A repository and documentation search performed
before this implementation found no existing TaskMarket provider or tool in Rig.

## Scope and status

- Target: [0xPlaygrounds/rig](https://github.com/0xPlaygrounds/rig)
- Official website: [rig.rs](https://rig.rs/)
- Established-project evidence: Rig has more than 8,000 GitHub stars and a
  documented ecosystem of production users and companion integrations.
- Integration type: independently maintained Rig side crate, following Rig's
  documented side-crate contribution path
- Current status: implementation prepared on a dedicated branch; formatting
  passes locally and GitHub Actions will validate Clippy and tests after push
- Upstream inclusion remains unrequested
- TaskMarket API: `https://api.taskmarket.dev/api`
- TaskMarket docs: [docs.taskmarket.dev](https://docs.taskmarket.dev/)

## License

MIT
