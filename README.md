# toad-core

Core data models, workspace management, and shared types for the
[Primatif Toad](https://github.com/Primatif/Primatif_Toad) ecosystem.

## What It Does

`toad-core` is the **foundation crate** that every other Toad module depends on.
It defines the shared vocabulary of the entire system — the data structures,
traits, error types, and configuration that all crates speak.

- **Data Models** — `ProjectDetail`, `SubmoduleDetail`, `ActivityTier`,
  `VcsStatus`, `StackStrategy`, `EcosystemChangelog`, and more. All derive
  `Serialize + Deserialize` for JSON output (Schema-First Contract).
- **Workspace Management** — `Workspace` struct handles discovery of `~/.toad/`
  global context, active context resolution, fingerprinting, and shadow
  directory management.
- **Project Registry** — `ProjectRegistry` and `TagRegistry` for persisting and
  loading discovered project metadata.
- **Report Types** — `StatusReport`, `AnalyticsReport`, `BatchOperationReport`,
  `BatchCleanReport`, `SearchResult`, and all multi-repo git report types.
- **Error Surface** — `ToadError` enum for typed, programmatic error handling
  across all crates.
- **Traits** — `ProgressReporter` for decoupling terminal progress bars from
  business logic (enabling headless/MCP execution).
- **Configuration** — `GlobalConfig` with token budgets, context types, and
  project-level overrides.

## Role in the Ecosystem

```text
toad-core (this crate)
  ├── toad-git        (depends on core)
  ├── toad-discovery  (depends on core, git, ops)
  ├── toad-manifest   (depends on core)
  ├── toad-ops        (depends on core)
  ├── toad-scaffold   (depends on core, git)
  ├── bin/toad        (CLI — depends on all)
  └── bin/toad-mcp    (MCP server — depends on all)
```

`toad-core` is MIT-licensed so that the open-source CLI and any community
tooling can freely depend on it.

## License

MIT
