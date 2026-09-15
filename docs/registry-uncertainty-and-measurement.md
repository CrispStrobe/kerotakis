# Giving numeric records an uncertainty, and marking which are measured

Status: 2026-09-15. IN PROGRESS — this file is the deliverable and is being
written as the work proceeds. See PLAN.md's scoped task of the same name.

## The starting state

1917 numeric records in `data/registry/registry-source-v1.json`:

| uncertainty kind | records |
|---|---|
| `not_reported` | 1093 |
| `exact` | 824 |
| `absolute` / `relative` / `interval` | 0 |

| method kind | records |
|---|---|
| `imported` | 852 |
| `derived` | 848 |
| `curated` | 111 |
| `editorial` | 106 |
| `measured` | **0** |
| `calculated` | 0 |
