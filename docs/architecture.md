# Architecture Notes

## Research Summary

The current LaunchApp/AO ecosystem already contains four useful signals:

1. `brain` proves the operating model.
2. `ao-fleet-tools` proves there is demand for fleet-level scripts and MCP.
3. `ao-fleet-pack` proves there is value in fleet-specific workflows and agents.
4. `ao-dashboard` proves operators want a real-time fleet view.

What none of those repos currently provide is a durable, standalone fleet control plane.

## What The New Service Should Own

`ao-fleet` should own:

- inventory of managed repos
- desired-state scheduling for projects
- daemon lifecycle policy
- fleet audit history
- fleet-wide MCP tools
- fleet-native AO workflows

It should not own AO's per-project execution semantics.

## Why It Should Sit Above AO

AO's daemon is intentionally a dumb per-project scheduler. It manages dispatches, capacity, subprocesses, and execution facts, but it does not own org-level policy or cross-project orchestration.

That separation is useful. It means `ao-fleet` can stay focused on:

- which repos should run
- when they should run
- with what pool size or runtime policy
- what cross-project workflows should fire

AO can keep doing:

- executing workflows
- handling project-local schedules
- supervising agent subprocesses
- managing task and requirement state

## Recommended System Model

```text
                    +----------------------+
                    |      ao-fleet        |
                    |----------------------|
                    | registry             |
                    | scheduler            |
                    | policy engine        |
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

## MCP Shape

The MCP surface should be split into:

- inventory: `fleet.project.list`, `fleet.project.get`, `fleet.project.import`
- daemon ops: `fleet.daemon.start`, `fleet.daemon.stop`, `fleet.daemon.pause`, `fleet.daemon.resume`, `fleet.daemon.rebalance`
- scheduling: `fleet.schedule.list`, `fleet.schedule.set`, `fleet.schedule.enable`, `fleet.schedule.disable`
- policy: `fleet.policy.preview`, `fleet.policy.apply`
- workflows: `fleet.workflow.run`, `fleet.workflow.list`
- observability: `fleet.events.tail`, `fleet.audit.list`, `fleet.health.overview`

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

## External Pattern Check

Adjacent systems suggest the right direction:

- HashiCorp Nomad treats periodic work as distributed cron and explicitly models overlap and timezone concerns.
- Temporal separates schedules from workflow executions and treats schedules as first-class resources.
- Rundeck emphasizes operator-managed job scheduling and runbook-style control.

The implication for `ao-fleet` is clear:

- schedules should be explicit resources
- overlap and pause semantics should be first-class
- auditability matters as much as execution

## Recommended First Build Slice

Build this first:

1. Fleet registry backed by SQLite.
2. Config import from a declarative file.
3. Project daemon desired-state controller.
4. Schedule engine for project activity windows.
5. MCP server exposing read/write fleet operations.

Only after that should richer AI automation and dashboard coupling expand.
