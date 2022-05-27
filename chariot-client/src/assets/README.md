
# assets

This directory stores the client's non-code assets, but includes them
as bytes or strings in the binary itself. They can then be referenced
like so:

```rust
let beanbag: &[u8] = &assets::models::BEANBAG;
```

When you add a resource to the project, be sure to include it via
this method.

When the file would benefit from compression, use the `include-flate`
macro instead of `include_bytes`. These files include:

* plain text or wgsl code
* fonts
* glb models

Files that do NOT benefit from additional compression include:

* PNGs and most other images
* Oog and most other audio