# 019-02 — SSH orchestration and deploy
**Epic:** 019-L1-two-machine-harness · **Effort:** M · **Depends on:** 019-01, epic 015, epic 016, epic 017

## Context
Binary `hadronyche-l1` (crate `hadronyche-tools`, created in 019-01) runs L1 experiments on two Linux hosts on one wire through one switch. The epic requires that it "orchestrates two hosts over SSH: deploys the same build, starts reference modules (from `hadronyche-reference`), runs a named scenario, collects both hosts' record files and stamps, produces one report."

This ticket delivers the orchestration layer: the transport to each host, deployment of one identical build to both, starting and stopping reference modules (crate `hadronyche-reference`, task 017) on the hosted runtime (task 015) with real-time setup (task 016), and copying back each host's output files.

The harness must also have a dry-run mode that passes in CI on loopback, unprivileged. This ticket therefore provides two transports behind one trait: real SSH, and a local loopback transport that runs both "hosts" as local processes in separate work directories.

## Work
- Module `hadronyche_tools::orchestrate`:
  - Trait `Transport` with `exec(cmd) -> Output`, `spawn(cmd) -> RemoteProcess`, `put(local, remote)`, `get(remote, local)`.
  - `SshTransport` (uses the system `ssh`/`scp` or an SSH library already accepted in the workspace; key-based auth only, no password prompts).
  - `LoopbackTransport` (local processes, per-host work directory under the run's `--out` directory); used when `--dry-run` is set.
- Deploy step: build once, compute a content hash of the deployed artefacts, copy to both hosts, verify the hash on each host after copy. Abort the run if the two hashes differ.
- Module start/stop: start the reference modules a scenario's `Plan` names, with their arguments, on host `a` or `b`; capture stdout/stderr to files; stop them with SIGTERM then SIGKILL after a timeout; record exit status.
- Collection: after a run, `get` every file the plan declares (record files, logs) into `<out>/<host>/`.
- Run directory layout: `<out>/plan.json`, `<out>/a/`, `<out>/b/`, `<out>/run.log`.
- Clean abort: on Ctrl-C or any step failure, stop all started processes on both hosts and still collect whatever exists.
- Tests run on `LoopbackTransport` only.

## Acceptance criteria
1. With `--dry-run`, scenario `noop` deploys to two loopback work directories, starts one process per host, stops both and collects their output into `<out>/a/` and `<out>/b/`; exit code 0.
2. A deliberately corrupted artefact on one side causes the run to abort before any module starts, with an error naming the host and both hashes.
3. A started process that ignores SIGTERM is killed within the configured timeout (test with a 2 s timeout).
4. Interrupting a run (simulated failure in step N) leaves no orphan process on either side (test checks the process table of the loopback "hosts").
5. Against a real SSH pair (manual check, logged in the ticket), `noop` deploys and collects from both hosts with identical hashes.

## Definition of done
- [ ] Tests green in CI, unprivileged
- [ ] No new open item without an owner and a falsifier
- [ ] The frozen POC is not modified (task 003 is the only exception)
- [ ] Every new factual claim carries an evidence tag
- [ ] Orchestration has a loopback transport usable by the dry-run mode

## Out of scope
- What a stamp contains (019-03); parsing record files and writing the report (019-04).
- The CI job that runs the whole dry-run (019-05).
- Host provisioning and real-time configuration of the hosts themselves (task 016).
