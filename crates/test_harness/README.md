<div align="center">
<img src="../../.doc/test-harness.png" height=300 alt="Illustration of a common robin with worms in its mouth. Text, 'bevy test harness'" title="test harness logo" />
</div>

This is a simple test harness for bevy projects.

## Features

- [x] Utility functions for easy, step-based testing.
- [x] Timeout functionality
- [x] Logging utilities
  - [x] Log the world hierarchy in a simple and readable format
  - [ ] Add names for common types
  - [x] Log capturing
- [x] Utilities for finding specific entities (by name) and testing their properties.
    - `find_entity` 
    - `find_no_entity` 
    - `find_entity_filtered<QueryFilter>`
    - `find_no_entity_filtered<QueryFilter>`
    - `find_entity_with<Component>`
  
## Stretch goals

- [ ] Headless rendering support
- [ ] Screenshots
- [ ] Scene snapshots
- [ ] Replay
- [ ] Reporting

## Non-goals

- Advanced trace viewer (a la playwright)
- Benchmark functionality

## Compatibility table

| bevy_test_harness | bevy |
| ----------------- | ---- |
| main              | 0.18 |
