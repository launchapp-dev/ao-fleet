# Architecture Notes

## Research Summary

The current LaunchApp/AO ecosystem already contains four useful signals:

1. `brain` proves the operating model.
2. `ao-fleet-tools` proves there is demand for fleet-level scripts and MCP.
3. `ao-fleet-pack` proves there is value in fleet-specific workflows and agents.
4. `ao-dashboard` proves operators want a real-time fleet view.

What none of those repos currently provide is a durable, standalone fleet control plane.

## Company, Teams, Projects

The product model is:

- company: `ao-fleet`
- team: one AO instance with a mission and ownership boundary
- project: a repo or workspace owned by a team
- knowledge: shared company memory, scoped to company, team, project, or operational context

Operators think in teams, not raw repos. A marketing team can own campaign repos, while an app team can own one or more application repos. `ao-fleet` coordinates those teams and keeps the company-level view.

## What The New Service Should Own

`ao-fleet` should own:

- inventory of managed teams
- inventory of managed repos
- desired-state scheduling for teams and projects
- daemon lifecycle policy
- fleet audit history
- company knowledge capture and retrieval
- fleet-wide MCP tools
- fleet-native AO workflows

It should not own AO's per-project execution semantics.

## Why It Should Sit Above AO

AO's daemon is intentionally a dumb per-project scheduler. It manages dispatches, capacity, subprocesses, and execution facts, but it does not own org-level policy or cross-project orchestration.

That separation is useful. It means `ao-fleet` can stay focused on:

- which teams and repos should run
- when they should run
- with what pool size or runtime policy
- what cross-project workflows should fire
- what knowledge should be captured and surfaced to operators

AO can keep doing:

- executing workflows
- handling project-local schedules
- supervising agent subprocesses
- managing task and requirement state

## AO-CLI Dependency For Multi-Host

Phase 1 can model host placement intent inside `ao-fleet`, but actual multi-host daemon control is blocked on AO CLI capabilities that do not exist in the current repo surface.

`ao-cli` needs to provide:

- remote daemon lifecycle operations that can target a specific host or agent
- machine-readable status and command results for start, stop, pause, resume, and health checks
- a secure transport or agent mode for executing AO commands off-box
- explicit host enrollment or identity so fleet assignments are stable
- deterministic error codes and structured payloads for reconciliation and recovery
- project-root resolution that works against remote filesystems or mapped workspaces

Until those exist, `ao-fleet` should treat host placement as intent and keep execution local to known AO roots.

## Recommended System Model

```text
                    +----------------------+
                    |      ao-fleet        |
                    |----------------------|
                    | company registry     |
                    | team/project store   |
                    | scheduler            |
                    | policy engine        |
                    | knowledge base       |
                    | audit/event log      |
                    | fleet MCP server     |
                    | embedded AO project  |
                    +----------+-----------+
                               |
                +--------------+--------------+
                |                             |
         +------+-------+              +------+-------+
         | managed repo |              | managed repo |
         |--------------|              |--------------|
         | .ao config   |              | .ao config   |
         | AO daemon    |              | AO daemon    |
         | AO MCP       |              | AO MCP       |
         +--------------+              +--------------+
```

## Knowledge Flow

The company knowledge base should ingest from:

- operator-authored notes
- schedule changes
- daemon reconcile results
- fleet audit events
- AO workflow runs
- Git commits, issues, and pull requests when integrated later

Knowledge should be stored with provenance and separated into:

- company knowledge for org-wide decisions and policy
- team knowledge for mission-specific context
- operational knowledge for incidents and execution history

The current CLI surface already reflects that shape:

- `knowledge-source-upsert`
- `knowledge-source-list`
- `knowledge-document-create`
- `knowledge-document-list`
- `knowledge-fact-create`
- `knowledge-fact-list`
- `knowledge-search`

The MCP surface mirrors it with `fleet.knowledge.*` tools.

## Dashboard Contract

`ao-dashboard` should read from `ao-fleet`, not from repo-local AO state directly.

Current read models:

- `fleet.overview` for company inventory plus desired-vs-observed reconcile preview
- `fleet.daemon.status` for persisted daemon status by project and team
- `fleet.knowledge.source.list`, `fleet.knowledge.document.list`, `fleet.knowledge.fact.list`, and `fleet.knowledge.search` for company memory

Phase 1 additions to expose next:

- host registry and host health
- founder overrides and policy previews
- incident and event history
- workflow runs and company-level automation outputs
- fleet snapshot export or seeded config summaries for bootstrap flows

## Scheduling Model

The most important product feature is not cron alone. It is desired operating windows.

Example policy types:

- `always_on`
- `business_hours`
- `nightly`
- `manual_only`
- `burst_on_backlog`

Each policy can map to concrete behavior:

- start or stop the repo daemon
- pause or resume dispatch
- adjust pool size
- enqueue or suppress specific fleet workflows

This is better than only storing cron strings because operators think in desired availability, not raw scheduler syntax.

Current policy kinds are:

- `always_on`
- `business_hours`
- `nightly`
- `manual_only`
- `burst_on_backlog`

Weekday windows use numeric weekdays with `0 = Monday` and `6 = Sunday`.

## MCP Shape

The MCP surface should be split into:

- inventory: `fleet.project.list`, `fleet.project.get`, `fleet.project.import`
- daemon ops: `fleet.daemon.start`, `fleet.daemon.stop`, `fleet.daemon.pause`, `fleet.daemon.resume`, `fleet.daemon.rebalance`
- scheduling: `fleet.schedule.list`, `fleet.schedule.set`, `fleet.schedule.enable`, `fleet.schedule.disable`
- policy: `fleet.policy.preview`, `fleet.policy.apply`
- workflows: `fleet.workflow.run`, `fleet.workflow.list`
- observability: `fleet.events.tail`, `fleet.audit.list`, `fleet.health.overview`
- knowledge: `fleet.knowledge.source.list`, `fleet.knowledge.source.upsert`, `fleet.knowledge.document.list`, `fleet.knowledge.document.create`, `fleet.knowledge.fact.list`, `fleet.knowledge.fact.create`, `fleet.knowledge.search`

In the current codebase, the implemented MCP read surface is narrower:

- `fleet.overview`
- `fleet.daemon.status`
- `fleet.knowledge.source.list`
- `fleet.knowledge.source.upsert`
- `fleet.knowledge.document.list`
- `fleet.knowledge.document.create`
- `fleet.knowledge.fact.list`
- `fleet.knowledge.fact.create`
- `fleet.knowledge.search`

The remaining groups are the Phase 1 expansion path.

## Embedded AO Strategy

Running AO inside `ao-fleet` is the right move, but it should be used for intelligence and automation, not for core state storage.

Use embedded AO for:

- health reconciliation
- anomaly detection
- schedule optimization
- repo onboarding
- issue and PR sweeps
- fleet maintenance workflows

Do not make embedded AO the only source of truth for fleet inventory or schedules.

## Existing Repo Migration Plan

### `ao-fleet-tools`

Migrate:

- monitor/watchdog logic
- cleanup logic
- basic MCP verbs

Do not preserve the current script-first architecture as the final product.

### `ao-fleet-pack`

Migrate:

- conductor-style fleet workflows
- reconciler patterns
- reviewer and syncer workflow ideas

Convert these from brain-centric examples into first-class fleet workflows and templates.

### `ao-dashboard`

Move it from direct AO filesystem discovery toward `ao-fleet` as the control-plane source for:

- inventory
- schedule state
- daemon intent vs actual state
- fleet audit history
- company knowledge search

## External Pattern Check

Adjacent systems suggest the right direction:

- HashiCorp Nomad treats periodic work as distributed cron and explicitly models overlap and timezone concerns.
- Temporal separates schedules from workflow executions and treats schedules as first-class resources.
- Rundeck emphasizes operator-managed job scheduling and runbook-style control.

The implication for `ao-fleet` is clear:

- schedules should be explicit resources
- overlap and pause semantics should be first-class
- auditability matters as much as execution

## Current Operator Surface

The docs should match the implemented CLI and MCP surface:

- `db-init`
- `team-create`, `team-get`, `team-list`, `team-update`, `team-delete`
- `project-create`, `project-get`, `project-list`, `project-update`, `project-delete`
- `schedule-create`, `schedule-get`, `schedule-list`, `schedule-update`, `schedule-delete`
- `audit-list`
- `daemon-reconcile`
- `mcp-list`
- `mcp-serve`
- knowledge source, document, fact, and search commands

## Recommended First Build Slice

Build this first:

1. Fleet registry backed by SQLite.
2. Config import from a declarative file.
3. Project daemon desired-state controller.
4. Schedule engine for project activity windows.
5. MCP server exposing read/write fleet operations.
6. Knowledge write and search flows.

Only after that should richer AI automation and dashboard coupling expand.
