# Seed Commands

These commands create the same small-company shape shown in `company.snapshot.json`.

```bash
cargo run -q -p ao-fleet-cli -- --db-path ./ao-fleet.db db-init

TEAM_MARKETING=$(cargo run -q -p ao-fleet-cli -- --db-path ./ao-fleet.db team-create \
  --slug marketing \
  --name Marketing \
  --mission "Own launch messaging, campaigns, and the marketing site." \
  --ownership company \
  --business-priority 80 | jq -r '.id')

TEAM_AURORA_APP=$(cargo run -q -p ao-fleet-cli -- --db-path ./ao-fleet.db team-create \
  --slug aurora-app \
  --name "Aurora App" \
  --mission "Own the product app and its release cadence." \
  --ownership company \
  --business-priority 95 | jq -r '.id')

TEAM_PLATFORM=$(cargo run -q -p ao-fleet-cli -- --db-path ./ao-fleet.db team-create \
  --slug platform \
  --name Platform \
  --mission "Own AO runtime, shared fleet tooling, and cross-team reliability." \
  --ownership company \
  --business-priority 100 | jq -r '.id')

cargo run -q -p ao-fleet-cli -- --db-path ./ao-fleet.db project-create \
  --team-id "$TEAM_MARKETING" \
  --slug marketing-site \
  --root-path /Users/samishukri/brain/repos/marketing-site \
  --ao-project-root /Users/samishukri/brain/repos/marketing-site/.ao \
  --default-branch main \
  --enabled

cargo run -q -p ao-fleet-cli -- --db-path ./ao-fleet.db project-create \
  --team-id "$TEAM_AURORA_APP" \
  --slug aurora-web \
  --root-path /Users/samishukri/brain/repos/aurora-web \
  --ao-project-root /Users/samishukri/brain/repos/aurora-web/.ao \
  --default-branch main \
  --enabled

cargo run -q -p ao-fleet-cli -- --db-path ./ao-fleet.db project-create \
  --team-id "$TEAM_PLATFORM" \
  --slug platform-ops \
  --root-path /Users/samishukri/brain/repos/ao-fleet \
  --ao-project-root /Users/samishukri/brain/repos/ao-fleet/.ao \
  --default-branch main \
  --enabled

cargo run -q -p ao-fleet-cli -- --db-path ./ao-fleet.db schedule-create \
  --team-id "$TEAM_MARKETING" \
  --timezone America/Mexico_City \
  --policy-kind business_hours \
  --window 1,9,17 \
  --window 2,9,17 \
  --window 3,9,17 \
  --window 4,9,17 \
  --window 5,9,17 \
  --enabled

cargo run -q -p ao-fleet-cli -- --db-path ./ao-fleet.db schedule-create \
  --team-id "$TEAM_AURORA_APP" \
  --timezone America/Mexico_City \
  --policy-kind nightly \
  --window 1,20,24 \
  --window 2,20,24 \
  --window 3,20,24 \
  --window 4,20,24 \
  --window 5,20,24 \
  --enabled

cargo run -q -p ao-fleet-cli -- --db-path ./ao-fleet.db schedule-create \
  --team-id "$TEAM_PLATFORM" \
  --timezone America/Mexico_City \
  --policy-kind burst_on_backlog \
  --window 1,8,18 \
  --window 2,8,18 \
  --window 3,8,18 \
  --window 4,8,18 \
  --window 5,8,18 \
  --enabled

SOURCE_COMPANY=$(cargo run -q -p ao-fleet-cli -- --db-path ./ao-fleet.db knowledge-source-upsert \
  --scope global \
  --kind manual_note \
  --label company-policy \
  --uri file:///Users/samishukri/brain/repos/ao-fleet/docs/examples/company-policy.md \
  --sync-state ready | jq -r '.id')

SOURCE_MARKETING=$(cargo run -q -p ao-fleet-cli -- --db-path ./ao-fleet.db knowledge-source-upsert \
  --scope team \
  --scope-ref "$TEAM_MARKETING" \
  --kind manual_note \
  --label marketing-notes \
  --uri file:///Users/samishukri/brain/repos/ao-fleet/docs/examples/marketing-notes.md \
  --sync-state ready | jq -r '.id')

SOURCE_AURORA=$(cargo run -q -p ao-fleet-cli -- --db-path ./ao-fleet.db knowledge-source-upsert \
  --scope team \
  --scope-ref "$TEAM_AURORA_APP" \
  --kind manual_note \
  --label aurora-runbook \
  --uri file:///Users/samishukri/brain/repos/ao-fleet/docs/examples/aurora-runbook.md \
  --sync-state ready | jq -r '.id')

cargo run -q -p ao-fleet-cli -- --db-path ./ao-fleet.db knowledge-document-create \
  --scope team \
  --scope-ref "$TEAM_MARKETING" \
  --kind runbook \
  --source-kind manual_note \
  --source-id "$SOURCE_MARKETING" \
  --title "Launch checklist for campaign days" \
  --summary "How marketing prepares a launch day and coordinates with the company layer." \
  --body "Check the launch brief, publish the campaign assets before business hours, and keep the AO team on business-hours scheduling so it can respond to feedback during the day." \
  --tag marketing \
  --tag launch \
  --tag business-hours

cargo run -q -p ao-fleet-cli -- --db-path ./ao-fleet.db knowledge-document-create \
  --scope team \
  --scope-ref "$TEAM_AURORA_APP" \
  --kind project_profile \
  --source-kind manual_note \
  --source-id "$SOURCE_AURORA" \
  --title "Aurora app operating profile" \
  --summary "Aurora runs on a nightly schedule so daytime work stays focused on product and demos." \
  --body "The aurora-app team uses nightly execution windows to keep the app moving while avoiding peak collaboration hours. Platform monitors the fleet and escalates when a nightly run misses its window." \
  --tag aurora \
  --tag nightly \
  --tag app

cargo run -q -p ao-fleet-cli -- --db-path ./ao-fleet.db knowledge-fact-create \
  --scope operational \
  --kind risk \
  --statement "Platform should keep the ao-fleet project available for burst-on-backlog scheduling because it handles shared runtime work." \
  --confidence 88 \
  --source-kind manual_note \
  --source-id "$SOURCE_COMPANY" \
  --tag platform \
  --tag risk \
  --tag fleet

cargo run -q -p ao-fleet-cli -- --db-path ./ao-fleet.db knowledge-search \
  --scope team \
  --scope-ref "$TEAM_MARKETING" \
  --text launch \
  --tag marketing \
  --document-kind runbook
```

The search command returns the matching documents and facts together, which is the shape the company-level knowledge layer uses for retrieval.
