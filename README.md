# ForgeDesk

> **Implementation status (2026-11-18):** ForgeDesk has substantial working local functionality and a real GitHub integration, but it is not yet a finished product. The complete remaining backlog, phase plan, acceptance criteria and DeepSeek handoff are in [docs/specs/PRODUCT_COMPLETION_SPEC.md](docs/specs/PRODUCT_COMPLETION_SPEC.md). The narrower [P0 sync specification](docs/specs/P0_REMAINING_SYNC_SPEC.md) covers only one subsystem.

ForgeDesk is a local-first desktop workspace for issues, projects, planning, repositories, GitHub Issues, Pull Requests, code review, CI and release workflows.

## Normative project documents

- 01_PRODUCT_PROJECT.md — product scope and user workflows.
- 02_TECHNICAL_SPEC.md — architecture, priorities, stages and Definition of Done.
- 03_GITHUB_API_BUDGET_AND_SYNC.md — GitHub GraphQL/REST, budgets and sync.
- 04_TOKEN_EFFICIENCY_RULES.md — mandatory DeepSeek API context/token efficiency.
- 05_DEEPSEEK_PROMPTS.md — stable warm prompt and starter templates.
- 06_UI_UX_SPEC.md — mandatory UI geometry, components and visual acceptance.
- docs/specs/PRODUCT_COMPLETION_SPEC.md — **master plan to finish all ForgeDesk functionality**.
- docs/PROJECT_STATE.md — compact current state.
- docs/PRODUCT_GAPS_AUDIT.md — full current gap matrix.

PR #12 is merged as e380c58098878080b7d746f3cad7889effdc2fb6. CI run 44166546258 passed 7/7 jobs. GraphQL dispatch and background sync remain unfinished.
