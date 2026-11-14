# Model map

Amended by the design owner, 2026-11-14: "Reconfigure the swarm so the
coding, testing and review agents use GLM 5.3 flash to save tokens."

| Tier | Model | Provider id | Used for |
|---|---|---|---|
| Fast | `glm-5.3-flash` | `melious/glm-5.3-flash` | **Every tester, every coder (routine AND hard tickets), every reviewer** |
| Strong | `glm-5.3` | `melious/glm-5.3` | Team leads, escalations (a ticket that fails two reviews escalates its coder to glm-5.3), the constants steward, the integrator |

Consequences, recorded honestly:
- The kickoff's tester/coder family-separation invariant (one deepseek, one
  glm per ticket) is **suspended by this directive** — every worker is one
  model, so a shared misreading of a ticket is no longer caught by family
  diversity. The first-pass failure rates below are the remaining signal.
- The kickoff's hard-ticket rule (servo, seqlock, unsafe, state machines,
  spec-writing → glm-5.3) is **suspended** for the coding role; those tickets
  now go to glm-5.3-flash. Escalation after two failed reviews still reaches
  glm-5.3.
- The deepseek endpoints are no longer used.

## First-pass review failure rates

Per team and role (all glm-5.3-flash since 2026-11-14; historical rates from
the previous map kept for comparison — tester was deepseek-v4.1-flash-1017,
hard coders and reviewers were glm-5.3):

| Team | Tester | Coder | Reviewer |
|---|---|---|---|
| Baseline | 0/7 | 2/4 (flash) 0/0 (glm) | — (glm-5.3 era) |
| Specification | 0/0 | 1/2 (flash) 2/3 (glm) | — (glm-5.3 era) |

(rates as first-pass failures / reviews completed; the glm-5.3-era reviewer
caught real defects — falsified bounds, false [code] claims, citation
discipline — the quality bar to watch under the new map)
