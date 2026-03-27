# Operator Guide

This guide shows the current `ao-fleet` surface as a practical company/team workflow.

## Model

- company: `ao-fleet`
- team: one AO instance with a mission and ownership boundary
- project: a repo or workspace owned by a team
- knowledge: company memory, stored as sources, documents, facts, and search results

Use a team for each operational boundary. For example:

- `marketing` can own content, campaign, and launch repos
- `app-one` can own the repo for a single product
- `platform` can own shared tooling and fleet operations

## Bootstrap

Create a local database first:

```bash
cargo run -q -p ao-fleet-cli -- --db-path /tmp/ao-fleet.db db-init
```

List the current team inventory:

```bash
cargo run -q -p ao-fleet-cli -- --db-path /tmp/ao-fleet.db team-list
```

## Create Teams And Projects

Create a marketing team:

```bash
cargo run -q -p ao-fleet-cli -- --db-path /tmp/ao-fleet.db team-create \
  --slug marketing \
  --name Marketing \
  --mission "owns launch campaigns and growth ops" \
  --ownership company \
  --business-priority 80
```

Create an app team:

```bash
cargo run -q -p ao-fleet-cli -- --db-path /tmp/ao-fleet.db team-create \
  --slug app-one \
  --name App One \
  --mission "owns the app codebase and release flow" \
  --ownership product \
  --business-priority 90
```

Attach a repo to a team:

```bash
cargo run -q -p ao-fleet-cli -- --db-path /tmp/ao-fleet.db project-create \
  --team-id <TEAM_ID> \
  --slug marketing-site \
  --root-path /Users/me/marketing-site \
  --ao-project-root /Users/me/marketing-site \
  --default-branch main \
  --enabled
```

Read back the project inventory:

```bash
cargo run -q -p ao-fleet-cli -- --db-path /tmp/ao-fleet.db project-list
```

## Add Schedules

Schedules are expressed as policy kind plus weekday windows.

Policy kinds currently supported:

- `always_on`
- `business_hours`
- `nightly`
- `manual_only`
- `burst_on_backlog`

Weekday numbers use `0 = Monday` and `6 = Sunday`.

Create a business-hours schedule:

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

Create a nightly schedule:

```bash
cargo run -q -p ao-fleet-cli -- --db-path /tmp/ao-fleet.db schedule-create \
  --team-id <TEAM_ID> \
  --timezone America/Mexico_City \
  --policy-kind nightly \
  --window 0,22,6 \
  --enabled
```

List schedules for a team:

```bash
cargo run -q -p ao-fleet-cli -- --db-path /tmp/ao-fleet.db schedule-list
```

## Reconcile Daemons

Run the desired-state evaluator without changing anything:

```bash
cargo run -q -p ao-fleet-cli -- --db-path /tmp/ao-fleet.db daemon-reconcile
```

Apply the reconcile plan:

```bash
cargo run -q -p ao-fleet-cli -- --db-path /tmp/ao-fleet.db daemon-reconcile --apply
```

If you have backlog counts per team, pass them in the `team_id=count` format:

```bash
cargo run -q -p ao-fleet-cli -- --db-path /tmp/ao-fleet.db daemon-reconcile \
  --backlog team_123=4 \
  --backlog team_456=0 \
  --apply
```

## Write Knowledge

Create or update a knowledge source:

```bash
cargo run -q -p ao-fleet-cli -- --db-path /tmp/ao-fleet.db knowledge-source-upsert \
  --scope team \
  --scope-ref <TEAM_ID> \
  --kind manual_note \
  --label launch-notes \
  --uri file:///ops/launch.md \
  --sync-state ready
```

Knowledge source sync states currently are:

- `pending`
- `ready`
- `stale`
- `failed`

Create a knowledge document:

```bash
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
```

Create a knowledge fact:

```bash
cargo run -q -p ao-fleet-cli -- --db-path /tmp/ao-fleet.db knowledge-fact-create \
  --scope team \
  --scope-ref <TEAM_ID> \
  --kind incident \
  --source-kind manual_note \
  --source-id <SOURCE_ID> \
  --statement "Restart failed daemons after two consecutive health check failures." \
  --confidence 90 \
  --tag ops \
  --tag restart
```

The current document kinds include runbooks, decisions, briefs, research notes, team profiles, project profiles, incident reports, and policy notes. Fact kinds include policy, decision, risk, incident, workflow outcome, and schedule observation.

List knowledge records:

```bash
cargo run -q -p ao-fleet-cli -- --db-path /tmp/ao-fleet.db knowledge-source-list --scope team --scope-ref <TEAM_ID>
cargo run -q -p ao-fleet-cli -- --db-path /tmp/ao-fleet.db knowledge-document-list --scope team --scope-ref <TEAM_ID>
cargo run -q -p ao-fleet-cli -- --db-path /tmp/ao-fleet.db knowledge-fact-list --scope team --scope-ref <TEAM_ID>
```

Search knowledge:

```bash
cargo run -q -p ao-fleet-cli -- --db-path /tmp/ao-fleet.db knowledge-search \
  --scope team \
  --scope-ref <TEAM_ID> \
  --text launch \
  --tag marketing \
  --document-kind runbook
```

Search can filter by:

- `--text`
- `--tag`
- `--document-kind`
- `--fact-kind`
- `--source-kind`

## Audit And MCP

Inspect audit history:

```bash
cargo run -q -p ao-fleet-cli -- --db-path /tmp/ao-fleet.db audit-list --team-id <TEAM_ID>
```

Start the fleet MCP server:

```bash
cargo run -q -p ao-fleet-cli -- --db-path /tmp/ao-fleet.db mcp-serve
```

The MCP surface mirrors the CLI and includes:

- `fleet.project.*`
- `fleet.daemon.*`
- `fleet.schedule.*`
- `fleet.audit.*`
- `fleet.knowledge.*`

## Operator Pattern

Use the company layer for policy and memory, and use AO teams for execution.

That means:

- marketing owns marketing work
- the app team owns app delivery
- platform owns fleet policy, shared workflows, and remediation
- `ao-fleet` keeps the durable company view

