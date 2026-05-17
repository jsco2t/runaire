Suggested /new-eng-feature commands (in order)

Run these one at a time, reviewing the output of each before the next. The features build on each other, so later runs benefit from earlier
artifacts.

1. Vault core (foundation; do this first)

/new-eng-feature /Users/jason/Developer/sources/personal/notebook/projects/runaire/features/vault-core

Specs to read (all required reading for this feature):

- PRD: /Users/jason/Developer/sources/personal/notebook/projects/runaire/prd.md
  Sections in scope: §6.1 Vault Management (FR-001..006), §6.6 Security FR-054 (atomic writes), §7 NFR-006/NFR-007/NFR-009 (reliability + concurrent
  access + KDBX interop), §8 Constraints (vault dir = $HOME/.local/state/runaire/).
- Verifications: .../verifications/vaults.md (US-001..007), .../verifications/security.md (US-054, US-055), .../verifications/interop.md
  (US-090..092).
- KB: .../kb/kdbx-format.md, .../kb/keepass-rs-library.md, .../kb/memory-hygiene.md, .../kb/supply-chain-policy.md.

Codebase to analyze: /Users/jason/Developer/sources/personal/runaire (empty / greenfield — analysis will note this and recommend initial crate
layout).

Scope: KDBX read/write wrapper around keepass-rs; vault-registration config (TOML at $HOME/.local/state/runaire/vaults.toml); master-password unlock
flow; atomic file writes; advisory file locking. Does NOT cover entry CRUD UX or sync — those are separate features.

2. Entry management

/new-eng-feature /Users/jason/Developer/sources/personal/notebook/projects/runaire/features/entry-management

Specs:

- PRD §6.2 (FR-010..017): entry CRUD across credentials, secure notes, TOTP, SSH keys, attachments; stable UUIDs; history; groups; search;
  expiration.
- Verifications: .../verifications/entries.md (US-010..018).
- KB: .../kb/kdbx-format.md (TOTP otp-uri convention, attachment model, history semantics).

Codebase: /Users/jason/Developer/sources/personal/runaire (will build on the vault-core feature once it lands).

Scope: Entry-type model and CRUD operations against an unlocked vault. Search and group management included. CLI/TUI surface is OUT of scope — those
wrap this. SSH-agent integration is OUT of scope (its own feature). Depends on: vault-core.

3. Password generation

/new-eng-feature /Users/jason/Developer/sources/personal/notebook/projects/runaire/features/password-generation

Specs:

- PRD §6.3 (FR-020..023): random passwords with length/classes/ambiguous exclusion; EFF-diceware passphrases; CSPRNG-backed.
- Verifications: .../verifications/password-and-ssh.md (US-020..023).

Codebase: /Users/jason/Developer/sources/personal/runaire.

Scope: A standalone generator crate/module. EFF wordlist embedded as a const. No dependency on vault-core; this is a leaf module. Has no dependencies
on other features.

4. CLI skeleton

/new-eng-feature /Users/jason/Developer/sources/personal/notebook/projects/runaire/features/cli-skeleton

Specs:

- PRD §6.7 (FR-060..064): one-shot subcommand surface; secure master-password input; JSON output; stable exit codes; shell completions.
- Verifications: .../verifications/surfaces.md (US-060..064).

Codebase: /Users/jason/Developer/sources/personal/runaire.

Scope: clap-based CLI binary; subcommand routing; secure stdin prompt; JSON formatter; exit-code conventions; completion-script generation.
Subcommands are wired up to core features as those land. Depends on: vault-core, entry-management, password-generation.

5. TUI skeleton

/new-eng-feature /Users/jason/Developer/sources/personal/notebook/projects/runaire/features/tui-skeleton

Specs:

- PRD §6.8 (FR-070..074): full TUI surface, vault list / unlock / navigation / detail; search; generate; copy with auto-clear; edit; history; sync
  trigger; auto-lock countdown; themability.
- Verifications: .../verifications/surfaces.md (US-070..074).

Codebase: /Users/jason/Developer/sources/personal/runaire.

Scope: ratatui-based TUI; event loop; key-binding system; screens (vault picker, unlock, browse, detail, edit, history); auto-lock countdown widget.
Depends on: vault-core, entry-management, password-generation. Sync trigger UX hooks into sync feature when it lands.

6. Sync (git)

/new-eng-feature /Users/jason/Developer/sources/personal/notebook/projects/runaire/features/sync-git

Specs:

- PRD §4.1 Goal 6, §6.5 (FR-040..046), §9.4 Workflow, §11 Risk #3.
- Verifications: .../verifications/sync.md (US-040..046).
- KB: .../kb/three-way-merge.md (algorithm + properties), .../kb/gitoxide-library.md (transport notes), .../kb/kdbx-format.md (history semantics for
  collision preservation).

Codebase: /Users/jason/Developer/sources/personal/runaire.

Scope: (a) a sync trait abstraction; (b) gitoxide-based git transport (fetch, push, SSH/HTTPS auth, merge-base detection, commit creation); (c) the
entry-UUID three-way merge engine. Property-based testing of merge invariants is a top-tier deliverable. Depends on: vault-core, entry-management.

7. Security behaviors

/new-eng-feature /Users/jason/Developer/sources/personal/notebook/projects/runaire/features/security-behaviors

Specs:

- PRD §6.6 (FR-050..053): zeroize on drop; auto-lock idle timer; OS lock-event hooks; clipboard auto-clear.
- Verifications: .../verifications/security.md (US-050..053).
- KB: .../kb/memory-hygiene.md.

Codebase: /Users/jason/Developer/sources/personal/runaire.

Scope: Zeroize types and audit; idle-timer machinery (shared between TUI and agent); platform-specific OS-lock hooks (macOS launchd/IOKit, Linux
systemd-logind); clipboard module with auto-clear across macOS / X11 / Wayland. Touches vault-core (lock/unlock state) and tui-skeleton (idle
interaction signal) — design will note cross-cutting concerns. Depends on: vault-core.

8. Optional agent

/new-eng-feature /Users/jason/Developer/sources/personal/notebook/projects/runaire/features/agent

Specs:

- PRD §6.9 (FR-080..082): optional runaire-agent; Unix-domain socket with 0700 perms and peer-UID validation; auto-lock; CLI fallback to per-command
  prompt.
- Verifications: .../verifications/surfaces.md (US-080..081).
- KB: .../kb/memory-hygiene.md (the agent's threat model).

Codebase: /Users/jason/Developer/sources/personal/runaire.

Scope: runaire-agent binary; socket protocol; CLI client integration with fallback. Likely a Phase-0-follow-on (after the agentless MVP is solid).
Depends on: vault-core, entry-management, security-behaviors.

After each run

/new-eng-feature is itself an orchestrator — it sequences /eng-plan-creator → /eng-design-creator → /eng-test-planning → /eng-task-planning →
/eng-verification-creator and asks you questions between phases. So a single command per feature drives the whole pipeline; you don't need to invoke
the sub-skills separately unless you want to re-run a specific phase later (which is what /eng-feature-followup is for).

I'd recommend doing #1 (vault-core) first and seeing how the output feels before kicking off the others — the artifacts and Q&A patterns will tell
you whether the spec inputs are sufficient or whether you want to expand the PRD/KB before driving deeper feature work.
