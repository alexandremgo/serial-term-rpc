# Overview

Enables the communication through a serial port over RPC.

# Installation and run

```
cargo run --bin server -- 127.0.1.1:3333
```

# Dependencies

- gRPC: [tonic](https://github.com/hyperium/tonic)
- Serial port communication: [serialport-rs](https://gitlab.com/susurrus/serialport-rs)
