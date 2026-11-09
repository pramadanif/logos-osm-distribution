# Prize text vs plan: conflicts and ambiguities

Rule: the upstream prize text wins over the plan.

## C1. Two country names have no Geofabrik id of the same name (2026-11-09)

The prize lists countries by display name. Plan §2 says "exactly the 72 leaf ids in the spec",
but for these two the spec gives a name, not an id, and the index has no exact match:

| Prize name | Index candidates | Chosen | Why |
|------------|------------------|--------|-----|
| Ireland | `ireland-and-northern-ireland`, `britain-and-ireland` | `ireland-and-northern-ireland` | Only extract that is Ireland without Great Britain; `britain-and-ireland` overlaps `great-britain` (also in the set) and would break non-overlap |
| Malaysia | `malaysia-singapore-brunei` | `malaysia-singapore-brunei` | Only extract that contains Malaysia |

Northern Ireland is not part of Great Britain, so `ireland-and-northern-ireland` and
`great-britain` do not overlap. This choice goes into the registry-layout ADR and README; it is
an interpretation, so it is also a line in docs/limitations.md once that file exists.

No other conflict between docs/plan/prize-LP-0018.md and the plan found so far.
