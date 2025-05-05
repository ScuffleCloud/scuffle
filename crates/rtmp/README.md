# scuffle-rtmp

> [!WARNING]  
> This crate is under active development and may not be stable.

[![crates.io](https://img.shields.io/crates/v/scuffle-rtmp.svg)](https://crates.io/crates/scuffle-rtmp) [![docs.rs](https://img.shields.io/docsrs/scuffle-rtmp)](https://docs.rs/scuffle-rtmp)

---

A crate for handling RTMP server connections.

## Specifications

| Name | Version | Link | Comments |
| --- | --- | --- | --- |
| Adobe’s Real Time Messaging Protocol | `1.0` | <https://github.com/veovera/enhanced-rtmp/blob/main/docs/legacy/rtmp-v1-0-spec.pdf> | Refered to as 'Legacy RTMP spec' in this documentation |
| Enhancing RTMP, FLV | `v1-2024-02-29-r1` | <https://github.com/veovera/enhanced-rtmp/blob/main/docs/enhanced/enhanced-rtmp-v1.pdf> | |
| Enhanced RTMP | `v2-2024-10-22-b1` | <https://github.com/veovera/enhanced-rtmp/blob/main/docs/enhanced/enhanced-rtmp-v2.pdf> | Refered to as 'Enhanced RTMP spec' in this documentation |

## Example

```rust
struct Handler;

impl SessionHandler for Handler {
    async fn on_data(&mut self, stream_id: u32, data: SessionData) -> Result<(), ServerSessionError> {
        // Handle incoming video/audio/meta data
        Ok(())
    }

    async fn on_publish(&mut self, stream_id: u32, app_name: &str, stream_name: &str) -> Result<(), ServerSessionError> {
        // Handle the publish event
        Ok(())
    }

    async fn on_unpublish(&mut self, stream_id: u32) -> Result<(), ServerSessionError> {
        // Handle the unpublish event
        Ok(())
    }
}

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("[::]:1935").await.unwrap();
    // listening on [::]:1935

    while let Ok((stream, addr)) = listener.accept().await {
        let session = ServerSession::new(stream, Handler);

        tokio::spawn(async move {
            if let Err(err) = session.run().await {
                // Handle the session error
            }
        });
    }
}
```

## Status

This crate is currently under development and is not yet stable.

Unit tests are not yet fully implemented. Use at your own risk.

## License

This project is licensed under the [MIT](./LICENSE.MIT) or [Apache-2.0](./LICENSE.Apache-2.0) license.
You can choose between one of them if you use this work.

`SPDX-License-Identifier: MIT OR Apache-2.0`
