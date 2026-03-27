# AO Fleet

`ao-fleet` is the company layer above AO teams.

It is the control plane for:

- team and project inventory
- schedule policy and daemon intent
- fleet audit history
- knowledge capture and retrieval
- fleet-native MCP tools
- company-wide AO workflows

## Mental Model

Think of the system like this:

- `ao-fleet` is the company
- an AO instance is a team
- a project is a repo or workspace owned by a team
- the knowledge base is company memory

That means a marketing team can own marketing repos, while an app team can own one product repo or several repos. `ao-fleet` coordinates the teams; AO still executes the work inside each repo.

## Why This Exists

AO already solves per-project orchestration well:

- each repo has its own `.ao/` config
- each repo runs its own daemon
- each repo exposes its own AO MCP surface

What is still missing is a true fleet-level control plane:

- one place to register and classify managed repos
- one place to define when teams and projects should be active
- one place to start, pause, stop, and rebalance AO daemons
- one place to store company knowledge and operational history
- one MCP surface for the whole fleet

`ao-fleet` is that layer.

## Current Surface

The repository currently exposes:

- CLI commands for team, project, schedule, audit, daemon, and knowledge operations
- CLI commands for host registration and project placement intent
- a stdio MCP server for `fleet.*` tools
- SQLite-backed fleet state
- a company knowledge base with sources, documents, facts, and search
- persisted daemon status and config snapshot import/export for founder bootstrap

## Quick Start

Create a local database:

```bash
cargo run -q -p ao-fleet-cli -- --db-path /tmp/ao-fleet.db db-init
```

Create a company team:

```bash
cargo run -q -p ao-fleet-cli -- --db-path /tmp/ao-fleet.db team-create \
  --slug marketing \
  --name Marketing \
  --mission "owns launch campaigns and growth ops" \
  --ownership company \
  --business-priority 80
```

Create a repo/project under that team:

```bash
cargo run -q -p ao-fleet-cli -- --db-path /tmp/ao-fleet.db project-create \
  --team-id <TEAM_ID> \
  --slug marketing-site \
  --root-path /Users/me/marketing-site \
  --ao-project-root /Users/me/marketing-site \
  --default-branch main \
  --enabled
```

Register a founder-managed host and assign a project to it:

```bash
cargo run -q -p ao-fleet-cli -- --db-path /tmp/ao-fleet.db host-create \
  --slug founder-mac \
  --name "Founder Mac" \
  --address founder.local \
  --platform macos \
  --status healthy \
  --capacity-slots 6

cargo run -q -p ao-fleet-cli -- --db-path /tmp/ao-fleet.db project-host-assign \
  --project-id <PROJECT_ID> \
  --host-id <HOST_ID> \
  --assignment-source founder
```

Create a business-hours schedule. Weekday numbers use `0 = Monday` and `6 = Sunday`:

```bash
cargo run -q -p ao-fleet-cli -- --db-path /tmp/ao-fleet.db schedule-create \
  --team-id <TEAM_ID> \
  --timezone America/Mexico_City \
  --policy-kind business_hours \
  --window 0,9,17 \
  --window 1,9,17 \
  --window 2,9,17 \
  --window 3,9,17 \
  --window 4,9,17 \
  --enabled
```

Write company knowledge:

```bash
cargo run -q -p ao-fleet-cli -- --db-path /tmp/ao-fleet.db knowledge-source-upsert \
  --scope team \
  --scope-ref <TEAM_ID> \
  --kind manual_note \
  --label launch-notes \
  --uri file:///ops/launch.md \
  --sync-state ready

cargo run -q -p ao-fleet-cli -- --db-path /tmp/ao-fleet.db knowledge-document-create \
  --scope team \
  --scope-ref <TEAM_ID> \
  --kind runbook \
  --source-kind manual_note \
  --source-id <SOURCE_ID> \
  --title "Campaign launch checklist" \
  --summary "Operational checklist for launches" \
  --body "Verify the launch checklist before enabling the campaign." \
  --tag marketing

cargo run -q -p ao-fleet-cli -- --db-path /tmp/ao-fleet.db knowledge-search \
  --scope team \
  --scope-ref <TEAM_ID> \
  --text launch \
  --tag marketing \
  --document-kind runbook
```

Reconcile daemon intent with observed state:

```bash
cargo run -q -p ao-fleet-cli -- --db-path /tmp/ao-fleet.db daemon-reconcile --apply
```

Start the MCP server:

```bash
cargo run -q -p ao-fleet-cli -- --db-path /tmp/ao-fleet.db mcp-serve
```

## Founder Bootstrap

Phase 1 is meant for a single founder or a very small founding team running a company, not a large ops org.

The practical bootstrap loop is:

1. Initialize the fleet database.
2. Create the company teams.
3. Attach the repos or workspaces each team owns.
4. Add schedules and company knowledge.
5. Export a snapshot for backup or seed replication.
6. Run `mcp-serve` as the long-lived control-plane process.
7. Use `daemon-status --refresh` and `daemon-reconcile --apply` on a timer until remote host control exists in AO CLI.

For a founder-run deployment, treat `ao-fleet` as a service with one persistent database path and one long-lived MCP endpoint. Use a service manager such as `systemd` or `launchd` to keep `mcp-serve` running, then schedule reconciliation separately until remote execution lands.

## Repository Map

- `README.md`: product overview and operator entry point
- `docs/architecture.md`: system model and implementation shape
- `docs/operator-guide.md`: concrete CLI workflows and examples

## Product Direction

This repo is intended to become a standalone open-source service and CLI that:

- manages many AO projects from one fleet registry
- schedules project activity windows and operational policies
- supervises AO daemons across the fleet
- stores company knowledge and makes it searchable
- exposes a fleet-native MCP server
- runs its own AO instance for smart workflow automation across repos

The design target is "Brain as a product" rather than "a few shell scripts".

## Non-Goals

- replacing AO inside individual repositories
- moving workflow execution logic into a fleet daemon
- duplicating AO's task, requirement, queue, or workflow runtime internals
- becoming a desktop dashboard

`ao-fleet` should orchestrate AO, not absorb it.

## Relationship To Existing Repos

- `ao`: execution kernel and per-project daemon
- `ao-dashboard`: visual client that should eventually consume `ao-fleet`
- `ao-fleet-tools`: early scripts and MCP experiments to fold into this repo
- `ao-fleet-pack`: workflow ideas and fleet agent patterns to migrate here
- `brain`: private operator workspace that proved the operating model

## Suggested Technical Shape

- Language: Rust
- Current CLI binary: `ao-fleet-cli`
- Persistence: SQLite for fleet state and history
- Config: YAML or TOML for declarative fleet config
- MCP transport: stdio first, optional HTTP later
- AO integration: spawn AO CLI and consume AO MCP rather than vendoring AO internals
- Multi-host control: depends on AO CLI gaining remote execution and host-targeted daemon operations

Rust is the right default because AO is already Rust and the operational parts here are process supervision, scheduling, IO, and durable state.

## Status

The repo has a working core surface for registry, scheduling, daemon reconciliation, MCP, knowledge operations, daemon status, and config snapshots. The next operator-facing layers live in `docs/operator-guide.md` and `docs/architecture.md`.
