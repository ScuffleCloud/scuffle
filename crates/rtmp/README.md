<!-- dprint-ignore-file -->
<!-- sync-readme title [[ -->
# scuffle-rtmp
<!-- sync-readme ]] -->

> [!WARNING]  
> This crate is under active development and may not be stable.

<!-- sync-readme badge [[ -->
[![docs.rs](https://img.shields.io/docsrs/scuffle-rtmp/0.2.3.svg?logo=docs.rs&label=docs.rs&style=flat-square)](https://docs.rs/scuffle-rtmp/0.2.3)
[![crates.io](https://img.shields.io/badge/crates.io-v0.2.3-orange?style=flat-square&logo=rust&logoColor=white)](https://crates.io/crates/scuffle-rtmp/0.2.3)
![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-purple.svg?style=flat-square)
![Crates.io Size](https://img.shields.io/crates/size/scuffle-rtmp/0.2.3.svg?style=flat-square)
![Crates.io Downloads](https://img.shields.io/crates/dv/scuffle-rtmp/0.2.3.svg?&label=downloads&style=flat-square)
[![Codecov](https://img.shields.io/codecov/c/github/scufflecloud/scuffle.svg?label=codecov&logo=codecov&style=flat-square)](https://app.codecov.io/gh/scufflecloud/scuffle)
<!-- sync-readme ]] -->

---

<!-- sync-readme rustdoc [[ -->
A crate for handling RTMP server connections.

See the [changelog](./CHANGELOG.md) for a full release history.

### Specifications

|Name|Version|Link|Comments|
|----|-------|----|--------|
|Adobe’s Real Time Messaging Protocol|`1.0`|<https://github.com/veovera/enhanced-rtmp/blob/main/docs/legacy/rtmp-v1-0-spec.pdf>|Refered to as ‘Legacy RTMP spec’ in this documentation|
|Enhancing RTMP, FLV|`v1-2024-02-29-r1`|<https://github.com/veovera/enhanced-rtmp/blob/main/docs/enhanced/enhanced-rtmp-v1.pdf>||
|Enhanced RTMP|`v2-2024-10-22-b1`|<https://github.com/veovera/enhanced-rtmp/blob/main/docs/enhanced/enhanced-rtmp-v2.pdf>|Refered to as ‘Enhanced RTMP spec’ in this documentation|

### Feature flags

* **`docs`** —  Enables changelog and documentation of feature flags

### Example

````rust,no_run
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
````

### License

This project is licensed under the MIT or Apache-2.0 license.
You can choose between one of them if you use this work.

`SPDX-License-Identifier: MIT OR Apache-2.0`
<!-- sync-readme ]] -->
