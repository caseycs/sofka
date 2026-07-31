# Sofka future roadmap

This document collects potential future work for Sofka. It includes useful
K9s parity work, ideas that build on Sofka's existing strengths, and features
that should deliberately remain lower priority.

The goal is not to turn Sofka into a line-for-line K9s clone. Sofka should
remain a fast, async-first Kubernetes TUI with a generic object pipeline, while
growing into a particularly strong incident-response and GitOps navigator.

## Product direction

Sofka already covers the core resource-navigation and management loop. Its
strongest differentiators are:

- A generic rendering pipeline that works with previously unknown CRDs.
- Native Flux actions without requiring the Flux CLI.
- Bulk actions through multiselect.
- Background port-forwards that do not block the UI.
- CRD drill-down into custom resources.
- Fast, generation-tagged watches and cached row computation.

Future work should reinforce those qualities. In particular, Sofka should aim
to answer three operator questions especially well:

1. What is unhealthy?
2. Why is it unhealthy?
3. What is the safest useful action I can take now?

## Priority roadmap

### P0: custom columns and views

Add declarative, user-configurable table columns for any resource type.

The current curated columns are useful for common Kubernetes resources, but an
unknown custom resource falls back to `NAME` and `AGE`. This preserves basic
navigability but often hides the status fields that make a CR useful. Custom
views should extend the generic pipeline rather than introduce a separate
renderer per kind.

Suggested capabilities:

- Match views by GVR, plural, kind, alias, namespace, or namespace pattern.
- Extract values using JSON Pointer, JSONPath, or another small documented
  expression language.
- Configure header, alignment, width, visibility, and value type.
- Mark columns as numeric, time, quantity, status, or ordinary text.
- Configure the initial sort column and direction.
- Support normal and wide-only columns.
- Apply user columns as either a complete replacement or an overlay on the
  curated defaults.
- Validate configuration on load and show errors inside Sofka.
- Reload view configuration without restarting the application.

Also read `additionalPrinterColumns` from CRD schemas. These should provide a
better automatic fallback before Sofka resorts to `NAME` and `AGE`.

Example configuration:

```toml
[views."cert-manager.io/v1/certificates"]
sort = "READY"

[[views."cert-manager.io/v1/certificates".columns]]
name = "READY"
path = "/status/conditions/0/status"
type = "status"

[[views."cert-manager.io/v1/certificates".columns]]
name = "EXPIRES"
path = "/status/notAfter"
type = "time"
```

Done when:

- A user can define useful columns for an arbitrary CR without recompiling.
- Sorting treats quantities, numbers, and times by value rather than text.
- Invalid expressions do not crash the TUI and produce an actionable warning.
- CRD printer columns work automatically when no explicit view is configured.

### P0: structured filtering and selectors

Expand fuzzy row filtering into a coherent query language. Fuzzy filtering
should remain the quick default, while structured expressions support precise
operational questions.

Suggested syntax and behavior:

- `/text` for the current fuzzy match.
- `/!text` for inverse matching.
- `/-l app=api,env=prod` for Kubernetes label selectors.
- `/-f spec.nodeName=node-3` for field selectors where supported.
- `/status=CrashLoopBackOff` for column or well-known field matching.
- `/cpu>500m`, `/memory>1Gi`, `/restarts>=5`, and `/age<2h` for typed values.
- Boolean combinations with a deliberately small and understandable grammar.
- Command-palette expressions combining resource, namespace, context, and
  filter.

Label and field selectors should be sent to the Kubernetes API when possible,
instead of downloading an entire resource set and filtering only in memory.
The UI should indicate when a filter is server-side versus local.

Done when:

- Existing fuzzy filtering remains fast and backward compatible.
- Label, field, inverse, status, quantity, and age filters are covered by
  parser and behavior tests.
- Server-side selectors survive refreshes, drill-down, history, and namespace
  changes.
- The active filter is visible and easy to clear or edit.

### P0: explain unhealthy resources

Add an incident-oriented view that explains why a selected object is
unhealthy. Sofka already exposes YAML, describe output, events, logs, owners,
and related pods, but operators must manually assemble those pieces.

The explanation should be deterministic and evidence-backed. It should not
require an AI service.

For a workload, gather and correlate:

- Spec generation and observed generation.
- Conditions and their reason/message fields.
- Desired, updated, available, ready, and unavailable replica counts.
- ReplicaSets, Jobs, Pods, and containers owned by the object.
- Container waiting and termination reasons.
- Restart counts, last termination state, OOM kills, and exit codes.
- Scheduling, image pull, mount, and readiness failures.
- Recent relevant events.
- Recent warning/error log lines where explicitly requested.
- Current CPU/memory pressure and missing resource requests.

Example output:

```text
Deployment/api is unavailable

Rollout
  desired 5 -> updated 3 -> ready 2
  ProgressDeadlineExceeded

Blocking objects
  ReplicaSet/api-7df9    2/5 ready
  Pod/api-7df9-r2m9     ImagePullBackOff
  Pod/api-7df9-x8kf     readiness probe failing

Recent evidence
  14:03 FailedToRetrieveImagePullSecret
  14:04 BackOff pulling ghcr.io/acme/api:1.8.2
```

Done when:

- The view identifies the common rollout, scheduling, image, volume, probe,
  crash-loop, and OOM failure modes.
- Every conclusion links back to the condition, event, pod, container, or
  metric that supports it.
- Missing permissions or unavailable APIs degrade gracefully.
- The view can jump directly to the relevant resource, event stream, or logs.

### P0: requests, limits, and container metrics

Make resource metrics actionable rather than showing only raw pod/node usage.

Add:

- Per-container CPU and memory usage.
- Pod totals derived from container metrics.
- CPU usage as a percentage of request and limit.
- Memory usage as a percentage of request and limit.
- Missing request and limit indicators.
- Kubernetes QoS class.
- Node allocatable, requested, limited, and actual usage.
- Clear handling of init containers and pod overhead.
- GPU capacity, allocation, and usage where a supported metrics source exists.
- Configurable warning and critical thresholds.

The initial implementation may use Metrics Server for live usage. Historical
recommendations should be a separate feature because Metrics Server alone is
not a historical database.

Done when:

- Pod, container, and node views share consistent quantity calculations.
- Percentages distinguish missing requests from a real zero value.
- Sorting works numerically for raw and percentage columns.
- The feature remains usable when Metrics Server is unavailable.

### P1: richer plugins

Generalize the current single-character shell plugins into a safe, expressive
extension system.

Add support for:

- Key combinations such as `ctrl-g`, `shift-b`, and function keys.
- Foreground and background commands.
- Optional confirmation and dangerous-action labeling.
- An explicit `mutating` declaration so read-only mode can allow known
  read-only plugins instead of rejecting every plugin.
- Context, cluster, group, version, kind, plural, namespace, name, container,
  filter, selected column, and displayed cell placeholders.
- Invocation over every marked resource.
- Typed inputs: string, number, boolean, secret input, and dropdown.
- Output modes: inherited terminal, popup document, background notification,
  or a new table view.
- Context-specific plugin collections.
- Loading multiple plugin files from a directory.
- Hot reload with visible validation errors.
- Timeouts, cancellation, exit status, and bounded output capture.

Security requirements:

- Never substitute placeholders through an implicit shell string.
- Preserve argument boundaries and execute an argv array by default.
- Make shell interpretation an explicit opt-in.
- Never print secret inputs into the status bar or logs.
- Show the executable and arguments before dangerous commands.

Done when:

- Existing plugin configuration remains compatible or has a clear migration.
- Read-only plugins can run in read-only mode only when explicitly declared.
- Bulk invocation clearly reports partial failures.
- A failed or hung background plugin cannot freeze Sofka.

### P1: configurable hotkeys, bookmarks, and saved queries

Allow users to bind complete navigation commands rather than only resource
aliases.

Bookmarks should be able to save:

- Resource kind.
- Namespace or all-namespaces scope.
- Context.
- Label/field/local filter.
- Sort column and direction.
- Optional Xray, Pulse, or future incident view.

Example:

```toml
[[bookmarks]]
key = "shift-1"
name = "Prod API failures"
resource = "pods"
context = "prod-eu"
namespace = "checkout"
filter = "status!=Running -l app=api"
```

Bookmarks should appear in help and the command palette and reload without a
restart.

### P1: GitOps ownership and drift

Build on Sofka's native Flux support so GitOps becomes a defining product
strength.

Add:

- Detect the Flux Kustomization or HelmRelease managing a selected object.
- Navigate from an object to its GitOps owner, source, and revision.
- Navigate through Kustomization dependencies.
- Show suspended, reconciling, stalled, and failed reconciliation states.
- Show the last attempted and last successfully applied revision.
- Compare live state with the last applied or inventory-tracked state.
- Warn before editing or deleting an object that Flux will recreate/revert.
- Explain the reconciliation chain when a source or dependency blocks an
  object.

Longer term, support live-versus-rendered-source drift when the required source
artifact is available. This must clearly distinguish API defaulting and
controller-managed fields from meaningful drift.

### P1: RBAC explorer and action-aware authorization

Sofka currently uses `SelfSubjectRulesReview` to hide resource kinds that the
current identity cannot list. Extend this into a dedicated authorization
workflow.

Add:

- `:can-i` overview for the current identity.
- `:can-i <verb> <resource>` checks.
- Per-action authorization checks before showing or executing mutations.
- Role/ClusterRole to RoleBinding/ClusterRoleBinding traversal.
- Subject to bindings and effective rules traversal.
- ServiceAccount-centric views.
- `:who-can` as an optional best-effort reverse lookup, with a warning that
  Kubernetes authorization can involve external authorizers and incomplete
  information.
- Clear distinction between namespace and cluster scope.

Done when:

- The UI does not advertise an action that an authoritative access review says
  is forbidden.
- Authorization failures are explained without being confused with read-only
  mode.
- Reverse lookup describes its limitations rather than presenting guesses as
  certainty.

### P1: node shell and ephemeral debugging

Add controlled debug workflows for pods and nodes.

Capabilities:

- Create an ephemeral debug container in a selected pod.
- Copy a pod into a temporary debug pod when ephemeral containers are not
  available or appropriate.
- Launch a configurable diagnostic pod on a selected node.
- Select a debug image and target container.
- Configure namespace, resource limits, host mounts, privilege, and TTL.
- Track debug resources created by Sofka and clean them up automatically or on
  request.
- Clearly preview privileged settings before creation.
- Disable all debug creation in read-only mode.

This feature needs particularly strong production guardrails because a node
debug pod may expose host namespaces or filesystems.

### P1: operational workspaces

Allow users to save a task-oriented collection of views.

A workspace may contain:

- Context and namespace.
- Several resource views.
- Filters and sorts.
- Column configuration.
- Selected dashboard or relationship root.
- Optional layout when split views are implemented.

Example use cases include checkout operations, cluster upgrades, certificate
renewal, or Flux reconciliation. Workspaces should be plain configuration that
teams can keep in a repository and share.

### P1: shareable diagnostic bundles

Export a bounded, redacted incident bundle for handoff between application and
platform teams.

A bundle may contain:

- Sofka version and collection timestamp.
- Context and cluster identity, with an option to anonymize them.
- Selected resource YAML.
- Related owner and child resources.
- Conditions and recent events.
- A bounded amount of recent logs.
- A current metrics snapshot.
- The generated incident explanation and timeline.

Safety requirements:

- Redact Secret data and `stringData` unconditionally by default.
- Redact known credential annotations and environment values sourced from
  secrets.
- Provide an explicit manifest of included and omitted data.
- Use conservative size and time limits.
- Preview the bundle before saving.

### P2: resource timeline

Record and present state changes as a causal timeline.

Start with a session-local history derived from watch events:

```text
13:58 Deployment generation changed 17 -> 18
13:59 ReplicaSet api-7df9 created
14:00 Pods scheduled
14:01 Readiness probes started failing
14:02 First container restart
14:03 Warning: image pull secret not found
14:05 ProgressDeadlineExceeded
```

Potential sources:

- Resource watch events and condition transitions.
- Kubernetes Events.
- Container restart and last termination state.
- Rollout generation changes.
- Flux reconciliation revisions.
- Optional external metrics/log/trace backends.

Session-local storage avoids operating a database but cannot reconstruct events
that happened before Sofka started. Persisted history should be optional,
bounded, and clearly labeled as incomplete.

### P2: cross-context fleet view

Add an opt-in dashboard that summarizes several kubeconfig contexts without
requiring the user to switch through them one at a time.

Possible columns:

- Connectivity and authentication state.
- Kubernetes version.
- Node readiness.
- Unhealthy pod/workload counts.
- Flux reconciliation failures.
- Expiring certificates or other configured health signals.
- Current read-only/guardrail policy.

Implementation constraints:

- Only query explicitly selected contexts.
- Use strict concurrency, request, and refresh limits.
- Make authentication prompts and exec-plugin failures visible.
- Never let one slow context block the rest of the dashboard.
- Cache only non-sensitive summaries.

Selecting a context should reuse Sofka's context-switch path and open its last
or configured default view.

### P2: richer relationship explorer

Extend Xray beyond owner references. Kubernetes relationships also exist
through names, selectors, references, and policy targets.

Model relationships such as:

- Service selectors to Pods.
- Ingress and Gateway API routes to Services.
- Workloads to ConfigMaps, Secrets, PVCs, and ServiceAccounts.
- PVCs to PVs and StorageClasses.
- HPAs and PodDisruptionBudgets to their targets.
- NetworkPolicies to selected pods and allowed peers.
- Roles to bindings and subjects.
- Flux inventory and dependency relationships.

Support questions such as:

- What depends on this Secret?
- Which route exposes this Service?
- Why can traffic not reach this Pod?
- Which workloads use this ServiceAccount?
- What will be affected if this ConfigMap is deleted?

Every inferred edge must identify the selector or reference that created it.

### P2: safety guardrails and action journal

Go beyond a session-wide read-only switch with declarative policies.

Potential guardrails:

- Match contexts, namespaces, resources, labels, and actions.
- Require ordinary confirmation, typed resource name, or typed context name.
- Deny force deletion, drain, or shell in selected environments.
- Warn when a resource is managed by GitOps.
- Require a reason or ticket identifier.
- Preview patches, propagation policy, and selected targets.
- Detect unexpectedly broad bulk selections.
- Set a maximum bulk-action size unless explicitly overridden.

Example:

```toml
[[guardrails]]
contexts = ["prod-*"]
namespaces = ["kube-system", "payments"]
actions = ["delete", "force-delete", "drain"]
confirmation = "type-resource-name"
```

Maintain a local session journal containing requested action, target, context,
timestamp, result, and optional reason. Never store secret input or decoded
Secret values in the journal.

### P2: historical right-sizing recommendations

When an external metrics backend is configured, estimate workload requests and
limits from historical usage instead of only showing live values.

Possible outputs:

- Current requests and limits.
- P50/P95/P99 CPU and memory over a chosen window.
- Suggested request with configurable headroom.
- OOM and throttling evidence.
- Estimated waste or risk.
- A patch preview, never an automatic mutation by default.

Metrics Server is insufficient for this feature. Define a small provider
interface so Prometheus-compatible systems can be integrated without coupling
the main object pipeline to one vendor.

## Additional K9s parity backlog

### Wide and narrow table modes

Allow users to toggle between a compact operational view and a wide view with
less frequently needed columns. Integrate this with custom-column visibility
instead of maintaining two unrelated renderers.

### Namespace favourites and recent namespaces

Pin important namespaces at the top of the namespace picker and retain a small
per-context recent list. Configuration should support locking a curated team
list while still keeping session-local recents.

### Screen dumps and saved snapshots

Save the current rendered view or structured row data for later inspection.
Possible formats are ANSI text, plain text, JSON, and YAML. Saved snapshots
should be browsable from a `:snapshots` view and clearly marked as stale.

Do not confuse this with the existing one-frame `--snapshot` CI mode; the TUI
feature is an interactive capture and review workflow.

### Mouse support

Optionally support selecting rows, scrolling, switching tabs/views, and
clicking obvious controls. Keep it disabled by default and ensure every action
remains keyboard accessible.

### Port-forward configuration

Expand port-forward controls with:

- Configurable local bind address.
- Named and numeric remote ports.
- Automatic free local-port selection.
- Saved per-resource mappings.
- Restarting a failed forward.
- Copying the local endpoint.
- Clear forward health and process-exit details.
- Optional browser-open action through an explicit plugin or command.

### Richer log controls

Add:

- Configurable initial tail size and maximum buffer.
- `since` duration or timestamp.
- All-containers mode.
- Container prefix visibility.
- Regex and inverse filtering in addition to substring search.
- Clear-buffer action.
- Reconnect status and retry controls.
- Optional full-screen default.
- Download logs for the current container or selected workload.

Preserve the current bounded-buffer behavior and responsiveness for high-volume
streams.

### Native interactive actions

Sofka currently shells out to `kubectl` for edit, describe, exec, and attach.
Reduce external dependencies over time:

- Generate a native structured describe view where practical.
- Use Kubernetes exec/attach APIs directly.
- Allow editing YAML through `$EDITOR` while applying the result through the
  Kubernetes API.
- Preserve kubeconfig context pinning and terminal restoration behavior.

Native implementations should only replace shell-outs when they match
`kubectl` behavior reliably; a predictable shell-out is better than an
incomplete native clone.

### Runtime diagnostics

Add `sofka info` and an in-app diagnostics view showing:

- Version and build information.
- Loaded base and override configuration paths.
- Current context, cluster, API server, and namespace.
- Active skin, plugins, custom views, and warnings.
- Kubernetes discovery and Metrics API status.
- Watch reconnect/error counts and request latency summaries.
- State/log/bundle directories.

Introduce structured application logging with configurable levels and safe
redaction. Diagnostics must not emit kubeconfig credentials, bearer tokens,
decoded Secrets, or plugin secret inputs.

### Packaging and platform coverage

Improve installation reach and release trust:

- Sign and notarize macOS binaries.
- Provide a Homebrew formula or tap.
- Add Windows builds if terminal and interactive subprocess behavior can be
  supported reliably.
- Consider common Linux packages or a maintained package repository.
- Generate checksums and provenance/attestations for releases.
- Publish an SBOM.
- Document the supported Kubernetes version matrix.

### Config reload and validation

Reload configuration layers reactively or through `:reload`, including views,
bookmarks, plugins, skins, and guardrails. Display precise validation errors in
the UI and retain the last known-good configuration when a reload fails.

### Resource aliases for complete commands

Allow aliases to target saved commands, not only canonical resource names. For
example, an alias could open pods in a namespace with a label selector. Prefer
implementing this on top of the bookmark/query model so aliases do not become a
second command language.

### Configurable UI thresholds

Expose CPU, memory, restart, age, and readiness warning thresholds. Allow
global defaults and per-context or per-resource overrides. Thresholds should
affect both cell coloring and incident explanations consistently.

### Image vulnerability integration

Prefer integration over embedding a scanner initially:

- Run Trivy, Grype, or another configured scanner through the plugin/task
  system.
- Parse bounded machine-readable results into a Sofka document/table view.
- Cache results by immutable image digest.
- Show severity counts on workloads and containers.
- Make database freshness visible.

An embedded scanner should only be considered if external-tool integration
cannot provide an acceptable experience; scanner databases and update logic
would add substantial maintenance and supply-chain surface.

### HTTP endpoint benchmarking

Allow an active port-forward or Service endpoint to be benchmarked using an
external tool such as `oha`, `hey`, or `vegeta`.

The richer plugin system should be the first implementation. A dedicated
benchmark result view can be added later if real usage justifies it. Avoid
shipping an HTTP load generator in Sofka solely for feature parity.

## Ideas beyond K9s

### Incident cockpit

Combine the explain view, timeline, related resources, events, logs, and
metrics into a single incident workflow. The first screen should summarize the
problem; subsequent actions should reveal the supporting evidence without
losing the original selection.

Potential workflow:

1. Open an unhealthy workload.
2. See ranked causes and affected child objects.
3. Jump to the relevant event/log/condition.
4. Compare recent generation or GitOps revision changes.
5. Preview a safe action.
6. Export a redacted diagnostic bundle if escalation is required.

Optional AI explanation can be added as an adapter later, but it must cite the
collected evidence, remain opt-in, and never receive Secret data by default.

### GitOps-first resource navigation

Treat desired-state ownership as a first-class relationship throughout Sofka.
Every managed row may expose its manager, revision, drift state, and
reconciliation health. Mutations should explain whether they are durable or
will be reverted by a controller.

### Fleet health without a central server

Use the local kubeconfig and bounded concurrent queries to provide useful
multi-cluster visibility without requiring users to install a controller or
send cluster data to a hosted service.

### Evidence-driven recommendations

Recommendations should always say why they are being made. For example:

```text
Increase memory request from 256Mi to approximately 420Mi
Evidence: 30-day P95 382Mi, 3 OOM kills, current limit 512Mi
```

Avoid generic advice that cannot be traced to a condition, event, metric,
configuration field, or historical observation.

### Safe action previews and rollback hints

Before a mutation, show the exact patch or deletion policy and likely
controller behavior. Where Kubernetes has no true rollback, say so. Where a
Helm revision or GitOps source provides a recovery path, link to it rather than
promising an implicit undo.

### Provider integrations

Define small optional interfaces for historical metrics, logs, traces, GitOps,
and vulnerability data. Keep the core usable with Kubernetes APIs alone and
avoid requiring one observability vendor.

Potential integrations:

- Prometheus-compatible metrics.
- Loki- or VictoriaLogs-compatible log search.
- Tempo-, Jaeger-, or VictoriaTraces-compatible trace search.
- Flux ownership/reconciliation metadata.
- Trivy or Grype image scan results.

Integrations should launch from the currently selected Kubernetes object and
derive service, namespace, pod, container, and time-range context where
possible.

## Deliberate non-goals and low-priority work

### Do not duplicate K9s's per-kind renderer architecture

Curated helpers are valuable for common resources, but the generic object-to-
cells pipeline should remain the default architecture. Custom views and CRD
printer columns provide extensibility without adding one renderer module per
resource kind.

### Do not embed every external operational tool

HTTP load generators, vulnerability scanners, Git clients, and cloud-provider
tunnels evolve independently and can be large dependencies. Build excellent
integration and result presentation before deciding to embed them.

### Do not make AI required

Health explanations, relationships, redaction, and diagnostic bundles must be
deterministic core features. AI may summarize or help explore collected
evidence, but the operator must be able to inspect the raw evidence and use the
feature offline.

### Do not trade responsiveness for exhaustive background collection

Historical, fleet, relationship, and incident features must use bounded work,
cancellation, generation tags, and lazy loading. Opening a normal resource
view must remain fast even when optional integrations are configured.

## Suggested delivery sequence

### Milestone 1: power-user foundation

- [x] Custom columns and view overlays.
- [x] CRD `additionalPrinterColumns` support.
- [x] Structured filter parser.
- [x] Server-side label and field selectors.
- [x] Config reload and validation view.
- [x] Wide/narrow column visibility.

### Milestone 2: actionable health

- [x] Container metrics.
- [x] Request/limit percentages and QoS.
- [ ] Configurable thresholds.
- [ ] Explain-unhealthy view.
- [ ] Direct evidence navigation.
- [ ] Initial session-local timeline.

### Milestone 3: extensibility and repeatable workflows

- [ ] Modifier-aware hotkeys.
- [ ] Rich plugin execution and output modes.
- [ ] Bulk plugins.
- [ ] Bookmarks and saved queries.
- [ ] Operational workspaces.
- [ ] Namespace favourites and recents.

### Milestone 4: GitOps and safety

- [ ] Flux ownership and dependency navigation.
- [ ] Revision and reconciliation-chain visibility.
- [ ] Managed-resource mutation warnings.
- [ ] Action-aware authorization checks.
- [ ] Declarative guardrails.
- [ ] Local action journal.

### Milestone 5: debugging and collaboration

- [ ] Ephemeral container workflow.
- [ ] Node debug pod workflow.
- [ ] Redacted diagnostic bundles.
- [ ] Screen dumps and structured snapshots.
- [ ] Runtime diagnostics and structured logs.
- [ ] Richer log controls.

### Milestone 6: fleet and integrations

- [ ] Opt-in cross-context health dashboard.
- [ ] Historical metrics provider interface.
- [x] Log provider interface (VictoriaLogs: autodiscovery or configured URL).
- [ ] Trace provider interface.
- [ ] Extended relationship graph.
- [ ] Vulnerability scanner integration.
- [ ] Historical right-sizing recommendations.

### Milestone 7: distribution polish

- [ ] Signed and notarized macOS releases.
- [ ] Homebrew distribution.
- [ ] Checksums, attestations, and SBOM.
- [ ] Evaluate Windows support.
- [ ] Document a Kubernetes compatibility matrix.

## Prioritization rule

When deciding between backlog items, prefer work that:

1. Reduces time to diagnose an unhealthy workload.
2. Makes a dangerous action safer or more understandable.
3. Improves arbitrary CRDs without adding per-kind code.
4. Strengthens Flux/GitOps navigation.
5. Is useful with only Kubernetes API access.
6. Preserves responsiveness on large or degraded clusters.

This keeps Sofka's roadmap product-led instead of treating K9s parity as the
definition of completion.
