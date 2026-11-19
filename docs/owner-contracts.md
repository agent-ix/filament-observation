# Owner contracts

Besides admission, filament-observation publishes fourteen canonical, bounded
owner contracts. Each one has a JSON Schema in [`schemas/`](../schemas/) and a
module under `authority::` that either derives the canonical bytes or reads
bytes back against independently supplied state. A reader returns a validated
view; there is no parser that hands back unchecked data.

| Module | Contract | Public reader |
| --- | --- | --- |
| `authority::activation` | `filament.observation.activation-scope-authority/v1` | `activation::read` |
| `authority::observation` | `filament.observation.record/v3` | `observation::read` |
| `authority::population` | `filament.observation.population/v2` | `population::read` |
| `authority::position` | `filament.observation.position-ledger/v1` | `position::read` |
| `authority::clock` | `filament.observation.clock-binding/v1` | `clock::read` |
| `authority::partial` | `filament.observation.partial-fact/v1` | `partial::read` |
| `authority::capture` | `filament.observation.capture-environment/v1` | `capture::read` |
| `authority::progress` | `filament.observation.progress-assertion/v1` | `progress::read` |
| `authority::closure` | `filament.observation.closure-assertion/v1` | `closure::read` |
| `authority::completeness` | `filament.observation.completeness-assertion/v1` | `completeness::read` |
| `authority::availability` | `filament.observation.result-availability/v1` | `availability::read` |
| `authority::bundle` | `filament.observation-authority/v1` | `bundle::read` / `bundle::read_revision` |
| `authority::bundle` | `filament.observation-authority/v2` | `bundle::read_v2` / `bundle::read_revision_v2` |
| `authority::bundle` | `filament.observation-authority-lineage/v1` | `bundle::read_lineage` |

Population admission additionally strict-reads
`filament.observation.explicit-members/v1` through
`population::read_membership`.

The record v1 and v2 schema files remain committed for their original
contracts. The active qualified record owner document is v3 because it permits
an absent ingestion instant; population v2 retains authority, revision and
opaque-byte members.
The authority-bundle v1 schema likewise remains immutable. Its v2 successor is
restricted to the exact seven-role structurally empty population profile; it
does not manufacture record-dependent sentinel facts.
