# `ao-fleet` Examples

This directory holds concrete example artifacts for a small company managed by `ao-fleet`.

The examples are intentionally aligned with the current CLI and domain model:

- `company.snapshot.json` shows a company with multiple AO teams, projects, schedules, and knowledge records.
- `seed-commands.md` shows one way to create the same shape with the implemented CLI commands.
- `company-policy.md`, `marketing-notes.md`, and `aurora-runbook.md` provide the source notes referenced by the knowledge records.

Conceptually, the example company is organized like this:

- `marketing` team owns launch copy, campaigns, and the marketing site.
- `aurora-app` team owns a single product app and runs on a nightly schedule.
- `platform` team owns the shared AO runtime and runs with a burst-on-backlog policy.

The example data uses the same field names and enum spellings that the code already accepts:

- `SchedulePolicyKind`: `always_on`, `business_hours`, `nightly`, `manual_only`, `burst_on_backlog`
- `KnowledgeScope`: `global`, `team`, `project`, `operational`
- `KnowledgeDocumentKind`: `brief`, `decision`, `runbook`, `research_note`, `team_profile`, `project_profile`, `incident_report`, `policy_note`
- `KnowledgeFactKind`: `policy`, `decision`, `risk`, `incident`, `workflow_outcome`, `schedule_observation`
- `KnowledgeSourceKind`: `ao_event`, `git_commit`, `github_issue`, `github_pull_request`, `manual_note`, `incident`, `schedule_change`, `workflow_run`

Use these files as a bootstrap reference when you want a small but realistic fleet shape.
