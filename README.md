# AO Fleet

Fleet control plane for AO daemons, schedules, MCP, and cross-project workflows.

## Why This Exists

AO already solves per-project orchestration well:

- each repo has its own `.ao/` config
- each repo runs its own daemon
- each repo exposes its own AO MCP surface

What is still missing is a true fleet-level control plane:

- one place to register and classify managed repos
- one place to define when projects should be active
- one place to start, pause, stop, and rebalance AO daemons
- one place to run cross-project workflows and operational automations
- one MCP surface for the whole fleet

`ao-fleet` is that layer.

## Product Direction

This repo is intended to become a standalone open-source service and CLI that:

- manages many AO projects from one fleet registry
- schedules project activity windows and operational policies
- supervises AO daemons across the fleet
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

## Proposed Architecture

### 1. Fleet Registry

A persistent registry of managed projects with metadata such as:

- local path or clone URL
- default branch
- stack tags
- desired AO status
- scheduling windows
- daemon policy
- ownership and environment labels

### 2. Fleet Scheduler

A scheduler that decides when a project should be:

- active
- paused
- warmed but idle
- completely stopped

This is where per-project run windows live, such as "run weekdays from 9am to 6pm" or "only run nightly reconciliation".

### 3. Project Adapter Layer

An AO-aware adapter that talks to each managed project through stable AO surfaces:

- AO CLI
- AO MCP
- AO runtime state

It should avoid direct mutation of repo internals unless AO itself explicitly requires it.

### 4. Embedded Fleet AO

`ao-fleet` should also be an AO project itself. That lets it run higher-order workflows such as:

- fleet reconciliation
- crash recovery
- queue pressure rebalancing
- repo provisioning
- stale branch cleanup
- PR and deployment sweeps
- policy drift detection

### 5. Fleet MCP Server

The fleet service should expose its own MCP namespace for control-plane operations:

- `fleet.project.*`
- `fleet.daemon.*`
- `fleet.schedule.*`
- `fleet.policy.*`
- `fleet.workflow.*`
- `fleet.audit.*`

That MCP surface becomes the integration point for dashboards, agents, and external automation.

## Suggested Technical Shape

- Language: Rust
- Primary binary: `ao-fleet`
- Persistence: SQLite for fleet state and history
- Config: YAML or TOML for declarative fleet config
- MCP transport: stdio first, optional HTTP later
- AO integration: spawn AO CLI and consume AO MCP rather than vendoring AO internals

Rust is the right default because AO is already Rust and the operational parts here are process supervision, scheduling, IO, and durable state.

## Initial Roadmap

### Phase 1

- fleet registry
- project discovery/import
- start/stop/status for project daemons
- first-class schedule windows
- fleet MCP read/write tools

### Phase 2

- embedded fleet AO project
- policy engine for pool sizing and active windows
- audit log and fleet event stream
- migration of `ao-fleet-tools` functionality

### Phase 3

- dashboard-facing API surface
- richer topology and dependency awareness
- multi-machine fleet support
- remote runner support

## Open Design Decisions

- Should schedules express desired daemon state, desired workflow activity, or both?
- Should project scheduling be config-first, AO-task-driven, or hybrid?
- How much state should live in declarative files vs SQLite?
- When should `ao-fleet` call AO MCP directly vs shell out to `ao`?
- How should remote hosts and non-local fleets be modeled?

## Status

This repo currently contains the product definition and architecture direction. Implementation should start with the fleet registry, scheduler, and MCP surface.
