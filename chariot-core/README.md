# chariot-core

Core common data structures and functions for both chariot-client and chariot-server.
Networking-related structures should also exist here since both the client and server need to
know about both types of packet.

### using this crate

You cannot run this crate directly, but it is already added
as a dependency of the client and server crates.

You should be able to import and use like so:

```rust
use chariot-core::Vehicle; // etc
```


#### how is game state converted to binary for sending over the network?

We use the Rust library [bincode](https://github.com/bincode-org/bincode).
By adding the `#derive(Decode, Encode)` attribute, you can give any struct a binary encoding.