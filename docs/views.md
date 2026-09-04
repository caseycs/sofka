# Views and thresholds

## Custom views

Define table columns for any resource. Most useful for custom resources, which
otherwise fall back to NAME/AGE. sofka keys views by apiVersion/plural
(`"cert-manager.io/v1/certificates"`, `"v1/pods"`), group/plural, bare plural, or
lowercase kind. The most specific key wins.

```toml
[views."cert-manager.io/v1/certificates"]
sort = "EXPIRES:desc"     # initial sort column, ":asc" (default) or ":desc";
                          # a sort you pick in the TUI (S/I/header click) is
                          # remembered per kind and wins over this
# replace = true          # replace the curated columns instead of overlaying

[[views."cert-manager.io/v1/certificates".columns]]
name = "READY"
path = "Ready"            # the condition *type* name, found wherever it is
type = "condition"        # in the array — order isn't guaranteed by anything

[[views."cert-manager.io/v1/certificates".columns]]
name = "EXPIRES"
path = "/status/notAfter"
type = "time"             # rendered as elapsed ("3d4h") / "in 30d"

[[views."cert-manager.io/v1/certificates".columns]]
name = "ISSUER"
path = "/spec/issuerRef/name"
wide = true               # only shown in wide mode (`w`)
```

`path` is a JSON Pointer (RFC 6901) into the object as the API serves it:
`/metadata/…`, `/spec/…`, `/status/…`, and array indices like
`/spec/ports/0/port`.

`type` is `text` (default), `status`, `number`, `quantity` (`500m`, `1Gi`),
`time`, or `condition`. Typed columns sort by value, not by text.

For a `condition` column, `path` is the condition **type name** (`Ready`,
`Available`, `Reconciling`, …). sofka finds it in `status.conditions` by name -
never by array index, whose order nothing guarantees - renders its `status`
(`True`/`False`/`Unknown`), and colors the row like a `status` column.

Optional `width` (for fixed columns) and `align` (`left`/`center`/`right`) tune
the layout. By default columns overlay the curated ones: a matching header
replaces it in place, new columns go before AGE. Invalid entries are skipped with
a warning in the app - they never take down the TUI.

### Jumping to a node

A kind whose objects name a node can jump to it: `o` opens the nodes list scoped
to that node, and so does `enter` for a kind with no drill-down of its own. Pods
are built in (`/spec/nodeName`). For anything else, `node` says where the name
lives:

```toml
[views."karpenter.sh/v1/nodeclaims"]
node = "/status/nodeName"   # Karpenter writes the node's name onto the claim
                            # once it registers; before that, `o` warns
```

`node` is a JSON Pointer, like `path`. A row whose pointer is empty warns instead
of opening an empty list; one whose pointer lands on something other than a name
warns that the pointer is wrong.

### Drilling into another kind

`enter` on a workload opens its pods. A view's `drill` gives any kind without a
built-in drill-down the same move: open another kind, scoped by a label
selector (`labels`) and/or a field selector (`fields`) filled in from the
selected row. `{name}` and `{namespace}` are the placeholders.

```toml
[views."karpenter.sh/v1/nodepools"]
drill = { kind = "nodeclaims", labels = "karpenter.sh/nodepool={name}" }

[views.externalsecrets]                 # the Secret it writes shares its name
drill = { kind = "secrets", fields = "metadata.name={name}" }
```

`kind` is anything `:` accepts (alias, plural, or kind) and is resolved when
you press `enter`; an unknown kind warns and stays put. `fields` is for a target
nothing labels back to the row: `metadata.name` and `metadata.namespace` are
selectable on every kind, other fields only where the apiserver indexes them. A namespaced target
opens in the row's namespace, a cluster-scoped one ignores it. `esc` comes back,
like every drill. When a view sets both `drill` and `node`, `enter` drills and
`o` still jumps to the node.

Unlike `columns` and `sort`, which come from the single most specific view,
`node` and `drill` are resolved key by key: a `[views."karpenter.sh/v1/nodeclaims"]`
that only sets columns doesn't hide a `node` set under `[views.nodeclaims]`.

### CRD printer columns

A custom resource with no explicit view picks up its CRD
`additionalPrinterColumns` automatically (columns with `priority > 0` become
wide-only). A condition lookup
(`.status.conditions[?(@.type=="Ready")].status` - how most CRDs express their
READY column) becomes a `condition` column, found by type name. The same filter
selecting another field (`.reason`, `.message`, `.lastTransitionTime`, …) keeps
the column and reads that field from the named condition. Other JSONPath filter
or wildcard expressions aren't representable and those columns are skipped. So
most custom resources get useful columns with zero configuration.

## Thresholds

The warning and critical values behind RESTARTS/CPU/MEM cell color (and the
request/limit utilization in the container picker) are configurable. Anything you
don't set keeps the sofka default, so an empty config colors exactly as before.

Global `[thresholds]` apply everywhere. `[thresholds.resources.<key>]` overrides
per resource (keyed like `[views]`). Like every section, a per-cluster or
per-context override file can retune them for one context. Thresholds re-apply
live on `:reload`.

```toml
[thresholds]
restarts    = { warn = 3, critical = 10 }       # count
cpu         = { warn = "200m", critical = "1" } # absolute usage
memory      = { warn = "256Mi", critical = "1Gi" }
utilization = { warn = 75, critical = 90 }      # percent of request/limit

[thresholds.resources.pods]                     # per-kind override
restarts = { warn = 5, critical = 20 }
```

Omit either bound of a band to disable that level. `warn` is peach, `critical`
is red.
