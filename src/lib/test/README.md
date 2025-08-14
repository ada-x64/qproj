TODO:

- Is it possible to truly run headless?
  - Right now some features depend on a window. This is set to be invisible but still exists, and still requires WGPU.
- Look into bevy_ci for frame-based events.
- Integrate with cargo_nextest for a custom test runner.
  - Allows for specifying test configuration with builder-like syntax.
