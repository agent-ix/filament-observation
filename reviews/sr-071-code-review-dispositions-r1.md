<!-- reviewer-dispositions repo=agent-ix/filament-observation visibility=public quoin=0.36.7 module=spec-artifacts-process@v0.36.0 id=SR-071 round=1 model=gpt-6.2-astra run=d99ac6d0-5d05-4e3e-b67d-8ee124ccd814 -->

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-001 | fixed | Digest constants and pin assertions were removed. |
| FND-002 | new finding | The lineage closure check was removed along with the digest pin. |

+++ [reviewer data]

```yaml
dispositions:
  - fnd: FND-001
    outcome: fixed
    after_excerpt: |-
      /// JSON Schema bytes for [`CONTRACT`].
      pub const SCHEMA_BYTES: &[u8] =
          include_bytes!("../../schemas/observation-authority-v1.schema.json");
findings:
  - fnd: FND-002
    severity: medium
    confidence: high
    method: code-review
    check: test-intent
    artifact_id: "src/authority/bundle.rs::schemas_are_closed"
    related_ids: [LINEAGE_CONTRACT]
    path: "src/authority/bundle.rs"
    lines: "1896-1900"
    excerpt: |-
      fn schemas_are_closed() {
          common::assert_closed_schema(SCHEMA_BYTES, CONTRACT);
          common::assert_closed_schema(V2_SCHEMA_BYTES, V2_CONTRACT);
      }
    finding: "The follow-up removes the lineage schema from the closure-validation test along with the digest pin, reducing schema-contract coverage."
```

+++
