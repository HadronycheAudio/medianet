# Model map

Kickoff section 5 names the models. What exists in this environment differs in
two places; substitutions are recorded here and are otherwise faithful to the
split.

| Tier | Model | Provider id | Used for | Notes |
|---|---|---|---|---|
| Strong | `glm-5.3` | `melious/glm-5.3` | Team leads, every reviewer, coders on hard tickets, constants steward, integrator, escalations | as specified |
| Fast | `glm-5.3-flash` | `melious/glm-5.3-flash` | Coders on routine tickets | as specified |
| Fast | `deepseek-v4.1-flash-1017` | `melious/deepseek-v4.1-flash-1017` | Testers | as specified |
| Fast | `deepseek-v4.1-flash` | `opencode/deepseek-v4.1-flash-free` | Second opinions, re-tests after a dispute, spare tester or coder capacity | **substituted**: `melious` exposes no plain `deepseek-v4.1-flash`; the free endpoint of the same model is used |
| Fast | `deepseek-v4-flash-0731` | `opencode/glm-5.3-flash-free` | Mechanical work: index and report updates, renames, formatting, fixture generation | **substituted**: no v4-flash-0731 endpoint exists here; mechanical work is the risk-free role for a substitution |

Invariants kept: tester and coder on one ticket are always one deepseek and
one glm; every reviewer and every team lead is `glm-5.3`.

## First-pass review failure rates

Per team and model, updated by the orchestrator as reviews complete. A model
is moved out of a role where it fails clearly more often than another would.

| Team | Tester (`deepseek-v4.1-flash-1017`) | Coder (`glm-5.3-flash`) | Coder (`glm-5.3`) | Reviewer (`glm-5.3`) |
|---|---|---|---|---|
| Baseline | 0/0 | 0/0 | 0/0 | — |
| Specification | 0/0 | 0/0 | 1/2 | — |
| Clocking | 0/0 | 0/0 | 0/0 | — |
| Datagramming | 0/0 | 0/0 | 0/0 | — |
| Control plane | 0/0 | 0/0 | 0/0 | — |
| Measurement | 0/0 | 0/0 | 0/0 | — |
| Networking and real time | 0/0 | 0/0 | 0/0 | — |
| Audio edges | 0/0 | 0/0 | 0/0 | — |
| DSL | 0/0 | 0/0 | 0/0 | — |
| Verification | 0/0 | 0/0 | 0/0 | — |
| Embedded | 0/0 | 0/0 | 0/0 | — |
| Tools | 0/0 | 0/0 | 0/0 | — |

(cells: first-pass review failures / reviews completed)
