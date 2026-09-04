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

### Pods and nodes

Custom columns also overlay sofka's curated core-resource views. Pods already
show `IP` and `NODE` after toggling wide mode with `w`, and nodes always show
`VERSION`. Extra node topology and provisioning details can come from labels:

```toml
# Karpenter example; adjust provider-specific NODEPOOL and TYPE label names.
[views."v1/nodes"]

[[views."v1/nodes".columns]]
name = "NODEPOOL"
path = "/metadata/labels/karpenter.sh~1nodepool"

[[views."v1/nodes".columns]]
name = "ZONE"
path = "/metadata/labels/topology.kubernetes.io~1zone"

[[views."v1/nodes".columns]]
name = "INSTANCE"
path = "/metadata/labels/node.kubernetes.io~1instance-type"

[[views."v1/nodes".columns]]
name = "TYPE"
path = "/metadata/labels/karpenter.sh~1capacity-type"
```

In a JSON Pointer, `/` inside a label name must be escaped as `~1`. For example,
EKS commonly uses `eks.amazonaws.com~1nodegroup` for `NODEPOOL` and
`eks.amazonaws.com~1capacityType` for `TYPE`. Add `wide = true` to any custom
column that should appear only after pressing `w`.

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
