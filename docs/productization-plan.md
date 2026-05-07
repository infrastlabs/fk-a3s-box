# A3S Box Productization Plan

This plan tracks the gap between the current implementation and the product target:
a production-grade MicroVM runtime with a Docker-like CLI, Kubernetes CRI support,
portable networking, and verifiable confidential-computing workflows.

## Release Gates

### Gate 1: Honest MVP

Goal: make the documented surface match working behavior and protect users from
silent partial implementations.

Acceptance criteria:

- `README.md` marks experimental and planned capabilities clearly.
- CLI rejects unsupported combinations instead of silently degrading.
- Pure unit tests run without host virtualization, network access, or privileged
  socket/mount operations.
- Core path is verified on macOS and Linux: `pull`, `run`, `exec`, `logs`,
  `stop`, and `rm`.

### Gate 2: Runtime Correctness

Goal: make one-container MicroVM execution reliable enough for local development.

Acceptance criteria:

- OCI entrypoint, cmd, env, workdir, user, volumes, and exposed ports are applied
  inside the guest.
- Foreground, detached, PTY, and non-PTY execution have deterministic exit-code
  and log behavior.
- Rootfs preparation supports copy everywhere and overlayfs where available, with
  cleanup tests for failure paths.
- Health checks and restart policy behavior are covered by integration tests.

### Gate 3: Docker-Compatible Build MVP

Goal: support a truthful subset of Dockerfile builds.

Acceptance criteria:

- Supported instructions are listed explicitly in docs and CLI help.
- Unsupported instructions fail with contextual errors.
- `RUN` executes in an isolated Linux environment, not directly on the macOS host.
- `--platform` supports one target platform; multi-platform OCI indexes are added
  only after per-platform builds are real.

### Gate 4: Kubernetes CRI MVP

Goal: pass a focused kubelet/crictl smoke suite for CRI Pods.

Acceptance criteria:

- `RunPodSandbox`, `CreateContainer`, and non-TTY `StartContainer` map to a real
  workload inside the MicroVM.
- `ExecSync`, streaming `Exec`, `Attach`, logs, stop/remove, and image status work
  through CRI.
- Pod sandbox status reports meaningful network information.
- Multi-container Pod behavior is either implemented or explicitly rejected.

Current notes:

- `CreateContainer` persists CRI command, args, env, workdir, TTY/stdin flags,
  and Linux `run_as_user`/`run_as_group` user overrides. Non-empty CRI image
  references must now exist in the local image store, and the resolved digest
  and OCI layout path are persisted with the container alongside OCI defaults
  for entrypoint, cmd, env, workdir, and user. The pod sandbox must be `Ready`.
- `PodSandboxStatus` now uses the CRI `verbose` channel to report sandbox
  lifecycle state, whether a VM manager is currently present, and the number of
  tracked containers in that sandbox.
- `RunPodSandbox` no longer requires every Pod to carry the
  `a3s.box/agent-image` annotation. The runtime has a default CRI agent image,
  and RuntimeClass-specific image overrides can be configured by runtime handler.
- The CRI proto surface now includes the newer streaming list, stats, runtime
  config, checkpoint, events, metrics, and image streaming RPCs. Stats/runtime
  config return minimal safe data, unsupported checkpointing is explicit,
  container lifecycle events stream from in-process CRI operations, and metrics
  remain empty until runtime collectors are implemented.
- `Status` now uses the CRI `verbose` channel to report runtime-level sandbox,
  container, VM-manager, and warm-pool counts.
- `ContainerStatus` now uses the CRI `verbose` channel to report container
  lifecycle state, sandbox ownership, VM presence, command/env sizing, and
  stream/TTY flags.
- `StartContainer` now builds a guest exec or PTY request from the persisted CRI
  config, wires stdin containers to the streaming exec/PTY stdin channel,
  verifies the sandbox VM is ready, rejects duplicate starts/restarts, starts the
  workload asynchronously, and records the eventual exit code from a background
  stream supervisor.
- `StartContainer` now fails fast when a container with a non-empty image
  reference lacks resolved local image metadata or its resolved OCI layout path
  has disappeared. The next CRI image-rootfs step is to launch the workload
  against that resolved container image rootfs inside the guest, not only the
  sandbox agent/rootfs image.
- `RunPodSandbox` now mounts a managed CRI container-rootfs directory into the
  sandbox VM, `CreateContainer` extracts the resolved OCI image into a
  per-container rootfs under that directory, and `StartContainer`, `ExecSync`,
  and streaming `Exec` carry the guest-visible rootfs path in the exec/PTY
  request. The guest exec server now chroots into that rootfs on Linux before
  spawning the command. TTY/PTY `StartContainer` and `Exec` carry the same rootfs
  through `PtyRequest`, and the guest PTY server chroots before exec on Linux.
  `Attach` now follows the supervised `StartContainer` stdout/stderr or TTY
  stream instead of opening an unrelated shell, and stdin attach forwards bytes
  to the main workload stdin.
- The guest exec server now speaks the streaming exec wire format used by
  `StartContainer` supervision and emits stdout/stderr chunks while the
  workload is still running, instead of buffering all output until exit. The
  same stream now accepts live stdin data frames plus a stdin-close control frame
  for `stdin_once` attach sessions.
- The async `StartContainer` success path now has unit coverage for the
  `Created -> Running -> Exited` transition sequence.
- The pure CRI create/start smoke path now verifies that `CreateContainer`
  resolves OCI image defaults, prepares a per-container rootfs, and that
  `StartContainer` sends the image command, env, workdir, user, and
  guest-visible rootfs path to the sandbox exec server.
- A pure one-container CRI smoke flow now exercises
  `RunPodSandbox -> PodSandboxStatus -> CreateContainer -> StartContainer ->
  StopPodSandbox -> RemovePodSandbox` against a fake ready VM and fake exec
  server. It verifies bridge network IP status, local-image rootfs handoff,
  container exit supervision, CRI log records, and cleanup of sandbox/container
  state without requiring host virtualization.
- Multi-container Pods are now accepted at `CreateContainer`: each container gets
  its own prepared rootfs, independent workload supervision, attach/log stream,
  and exit status while sharing the pod sandbox VM. `StopContainer` refuses the
  destructive VM-teardown fallback when other containers in the same sandbox are
  still running.
- An ignored `crictl` smoke harness now exists for the real CRI socket path. It
  starts `a3s-box-cri`, drives `crictl runp/create/start/logs/inspectp` for a
  two-container Pod, and cleans up all containers plus the pod sandbox. It
  remains opt-in because it needs `crictl`, image availability, and host
  virtualization.
- `RunPodSandbox` now maps CRI TCP `port_mappings` into Box VM `port_map`
  entries so sandbox-level host ports are not silently dropped. Unsupported
  UDP/SCTP mappings, bind-specific `host_ip` values, and invalid port numbers
  now fail fast during sandbox config conversion.
- `PodSandboxStatus.network` now reports runtime-known Pod IP metadata when an
  integration supplies `a3s.box/pod-ip` and optional comma-separated
  `a3s.box/additional-pod-ips` annotations. Invalid IP annotations fail fast,
  and legacy persisted sandboxes default to an empty network status until real
  CNI/bridge allocation is wired in.
- `RunPodSandbox` now accepts the `a3s.box/network` annotation to join an
  existing A3S bridge network before boot. The runtime preallocates a stable
  sandbox ID, registers a network endpoint in `NetworkStore`, uses the allocated
  IPv4 address as `PodSandboxStatus.network.ip`, and disconnects the endpoint on
  sandbox/container teardown or boot failure. Explicit `a3s.box/pod-ip` values
  must match the allocated address. The CRI runtime service can now use an
  injected `NetworkStore`, keeping unit tests isolated from the user's default
  `~/.a3s/networks.json`, and unit coverage now verifies cleanup for IP
  mismatch, VM acquisition failures, `StopContainer`, `StopPodSandbox`, and
  `RemovePodSandbox`.
- `StartContainer` supervision now writes stdout/stderr events to the CRI
  container log path using Kubernetes CRI log records, including parent
  directory creation and final partial-line flushing.
- Non-interactive CRI `Exec` streaming now bridges to the guest through the
  same frame-based exec client protocol as `ExecSync`, instead of the older
  HTTP-over-Unix-socket stub. Non-TTY `Exec` requests with stdin now switch to
  a streaming exec bridge so HTTP input bytes reach the guest process stdin and
  stdout/stderr chunks are returned while the process runs.
- The guest exec server accepts workload connections concurrently, so one
  long-running CRI container no longer blocks starting another container in the
  same pod sandbox.
- `StopContainer` now reports missing container IDs instead of silently returning
  success, preserves already-exited status instead of overwriting the original
  exit code, and first asks the supervised guest workload to stop through the
  streaming exec control channel so the sandbox VM can remain `Ready`.
- `ContainerStatus` reports `Completed` for exit code 0 and `Error` for non-zero
  exits, with a short message that includes the exit code.
- `ListContainers` has unit coverage for ID, sandbox, state, and label-selector
  filters.
- `ExecSync` and streaming `Exec` reject empty command vectors before attempting
  VM lookup or session registration.
- `Attach` rejects requests with no streams selected, requires the TTY flag to
  match the container configuration, requires CRI stdin to be enabled before
  accepting stdin attach, and `PortForward` rejects empty port lists before
  attempting VM lookup.
- `ExecSync`, streaming `Exec`, and `Attach` now require the target container to
  be running before attempting VM lookup or session registration.
- `UpdateContainerResources` now requires a running container when Linux
  resource changes are requested; only the explicit no-op `linux = None` path is
  accepted for non-running containers.
- `RemoveContainer` is now idempotent for missing containers, but rejects
  deletion of running containers until they have been stopped.
- `PortForward` now requires a `Ready` pod sandbox in addition to a non-empty
  port list before it attempts VM lookup.
- CRI `PortForward` now uses a dedicated guest control socket instead of the
  older broken HTTP-over-Unix-socket stub, and it can bridge one requested
  guest TCP port per streaming session on Unix hosts.
- CRI streaming session URLs now reject operation-kind mismatches without
  consuming the token, and PortForward keeps reading guest responses after the
  client half-closes its upload side.
- CRI streaming server startup now binds before constructing runtime services,
  so ephemeral streaming ports advertise the actual listener address instead of
  returning unusable `:0` URLs.
- CRI streaming session tokens now expire after a short TTL and are pruned during
  registration and connection handling, so stale one-shot URLs cannot remain
  usable indefinitely.
- `StartContainer`, `ExecSync`, streaming `Exec`, `Attach`, `PortForward`, and
  `UpdateContainerResources` now share a VM health gate and fail fast when the
  sandbox VM exists but is not ready.
- `StopPodSandbox` now returns `NotFound` for missing sandboxes and is
  idempotent for already-`NotReady` sandboxes; `RemovePodSandbox` is idempotent
  for missing sandboxes but rejects removal while the sandbox is still `Ready`.
- Stopped or removed CRI sandboxes now destroy any lingering VM manager instead
  of recycling a potentially dirty workload VM back into the warm pool.
- `StopContainer` now has workload-level stop plumbing for `StartContainer`
  processes: the runtime sends a streaming exec cancel control frame, the guest
  kills the child process group and reports exit `137`, and the runtime falls
  back to sandbox VM teardown only when no active workload control exists or the
  stop times out.
- `StopPodSandbox` now fans out workload-level stop controls to all running
  containers in the sandbox before tearing down the shared VM, preserving
  supervisor-reported container exit codes when those workload stops complete
  and only marking remaining non-exited containers with the forced-stop `137`
  fallback.
- `GetContainerEvents` now provides a live stream for container created,
  started, stopped, and deleted events emitted by CRI lifecycle operations,
  including supervisor-reported workload exits.

### Gate 5: Portable Networking

Goal: make published ports and outbound access predictable.

Acceptance criteria:

- Linux and macOS have documented backend selection and diagnostics.
- macOS outbound NAT is implemented or explicitly unsupported by mode.
- Port publishing has end-to-end tests for HTTP services.
- Network policy behavior is enforced, not only stored.

### Gate 6: Confidential Computing

Goal: separate simulated TEE development from hardware-backed production claims.

Acceptance criteria:

- Simulated attestation is clearly marked in CLI and audit output.
- SEV-SNP hardware attestation, RA-TLS, sealing, and secret injection have an
  automated or documented hardware acceptance suite.
- TDX remains hidden or documented as planned until runtime support exists.
- KDS/KBS network-dependent tests are isolated from pure unit tests.

### Gate 7: Cross-Platform Packaging

Goal: make installation behavior match platform claims.

Acceptance criteria:

- macOS and Linux packages ship matching binaries and guest assets.
- Windows clearly chooses one path: native WHPX or WSL launcher, with command
  support matrix and tests.
- Version numbers and package metadata are aligned across workspace crates,
  Homebrew, winget, and docs.

## Immediate Development Queue

1. Reject unsupported multi-platform builds instead of creating a single-platform
   image while accepting multiple platforms.
2. Split network/host-dependent tests from pure unit tests. The cosign keyless
   registry lookup test is now ignored by default and should be run explicitly
   when registry network access is available.
3. Run and harden the opt-in kubelet/crictl CRI smoke suite on a host with
   `crictl`, image availability, and microVM support. Pure unit coverage now
   verifies one-container and multi-container CRI lifecycle paths through a fake
   ready VM and exec server, and `src/cri/tests/crictl_smoke.rs` provides the
   real CRI socket harness.
4. Replace macOS host-side Dockerfile `RUN` execution with an isolated execution
   path. It now fails by default and requires `A3S_BOX_UNSAFE_HOST_RUN=1` for
   explicit unsafe local experiments.
5. Add a Windows command support matrix and make unsupported commands hidden or
   explicitly documented.
