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

## Founder Bootstrap And Service Mode

For Phase 1, run `ao-fleet` like a founder-operated company control plane:

1. Initialize one persistent database path.
2. Register the company teams.
3. Attach the repos or workspaces each team owns.
4. Add schedules and knowledge records.
5. Export a config snapshot before major changes or upgrades.
6. Run `mcp-serve` as the long-lived service process.
7. Run `daemon-status --refresh` and `daemon-reconcile --apply` on a timer. Local projects use direct AO CLI control; placed remote projects should target a host-scoped AO web API or MCP endpoint.

In practice, that means a service manager such as `systemd` or `launchd` should own the MCP server process, while a separate timer or cron entry can refresh daemon state and reconcile schedules. Keep the database path and snapshot export path stable so backup and restore stay simple.

Export a snapshot whenever you want a recoverable seed of the company state:

```bash
cargo run -q -p ao-fleet-cli -- --db-path /tmp/ao-fleet.db config-snapshot-export --output /tmp/ao-fleet.snapshot.json
```

A minimal service unit looks like this:

```ini
[Service]
ExecStart=/usr/local/bin/ao-fleet-cli --db-path /var/lib/ao-fleet/ao-fleet.db mcp-serve
Restart=always
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

Discover repos and `.ao` projects under configured workspace roots:

```bash
cargo run -q -p ao-fleet-cli -- --db-path /tmp/ao-fleet.db project-discover \
  --search-root /Users/me/repos \
  --search-root /Users/me/workspaces
```

By default, discovery skips AO-only shell directories whose only entry is `.ao`. Include them explicitly when you want placeholder AO workspaces in the result:

```bash
cargo run -q -p ao-fleet-cli -- --db-path /tmp/ao-fleet.db project-discover \
  --search-root /Users/me/repos \
  --include-ao-shells
```

Register discovered projects into a team:

```bash
cargo run -q -p ao-fleet-cli -- --db-path /tmp/ao-fleet.db project-discover \
  --search-root /Users/me/repos \
  --register \
  --team-id <TEAM_ID>
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

## Register Hosts And Placement Intent

Phase 1 host support is about placement intent and transport selection. It lets the founder model which machine should own which project and choose whether a project uses local AO CLI control or a host-scoped AO web API or MCP endpoint.

Register a host. Use a plain machine name for local intent, or a base URL when the host exposes an AO web API for remote control:

```bash
cargo run -q -p ao-fleet-cli -- --db-path /tmp/ao-fleet.db host-create \
  --slug founder-mac \
  --name "Founder Mac" \
  --address http://founder.local:7444 \
  --platform macos \
  --status healthy \
  --capacity-slots 6
```

List hosts:

```bash
cargo run -q -p ao-fleet-cli -- --db-path /tmp/ao-fleet.db host-list
```

Assign a project to a host:

```bash
cargo run -q -p ao-fleet-cli -- --db-path /tmp/ao-fleet.db project-host-assign \
  --project-id <PROJECT_ID> \
  --host-id <HOST_ID> \
  --assignment-source founder
```

Inspect or clear the placement map:

```bash
cargo run -q -p ao-fleet-cli -- --db-path /tmp/ao-fleet.db project-host-list

cargo run -q -p ao-fleet-cli -- --db-path /tmp/ao-fleet.db project-host-clear \
  --project-id <PROJECT_ID>
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

Record the last observed daemon state for the fleet:

```bash
cargo run -q -p ao-fleet-cli -- --db-path /tmp/ao-fleet.db daemon-status --refresh
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

The currently implemented MCP surface centers on:

- fleet overview for inventory and reconcile preview
- daemon status for observed state
- knowledge search for company memory

The next additions should be:

- `fleet.project.*`
- `fleet.schedule.*`
- `fleet.audit.*`
- `fleet.host.*`
- `fleet.policy.*`
- `fleet.workflow.*`

Those are the pieces `ao-dashboard` should read once Phase 1 expands beyond the current local-fleet model.

## Operator Pattern

Use the company layer for policy and memory, and use AO teams for execution.

That means:

- marketing owns marketing work
- the app team owns app delivery
- platform owns fleet policy, shared workflows, and remediation
- `ao-fleet` keeps the durable company view
