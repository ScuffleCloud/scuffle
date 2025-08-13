<!-- cargo-sync-rdme title [[ -->
# scuffle-batching
<!-- cargo-sync-rdme ]] -->

> [!WARNING]  
> This crate is under active development and may not be stable.

<!-- cargo-sync-rdme badge [[ -->
![License: MIT OR Apache-2.0](https://img.shields.io/crates/l/scuffle-batching.svg?style=flat-square)
[![docs.rs](https://img.shields.io/docsrs/scuffle-batching.svg?logo=docs.rs&style=flat-square)](https://docs.rs/scuffle-batching)
[![crates.io](https://img.shields.io/crates/v/scuffle-batching.svg?logo=rust&style=flat-square)](https://crates.io/crates/scuffle-batching)
[![GitHub Actions: ci](https://img.shields.io/github/actions/workflow/status/scufflecloud/scuffle/ci.yaml.svg?label=ci&logo=github&style=flat-square)](https://github.com/scufflecloud/scuffle/actions/workflows/ci.yaml)
[![Codecov](https://img.shields.io/codecov/c/github/scufflecloud/scuffle.svg?label=codecov&logo=codecov&style=flat-square)](https://codecov.io/gh/scufflecloud/scuffle)
<!-- cargo-sync-rdme ]] -->

---

<!-- cargo-sync-rdme rustdoc [[ -->
A crate designed to batch multiple requests into a single request.

See the [changelog](./CHANGELOG.md) for a full release history.

### Feature flags

* **`docs`** —  Enables changelog and documentation of feature flags

### Why do we need this?

Often when we are building applications we need to load multiple items from
a database or some other external resource. It is often expensive to load
each item individually, and this is typically why most drivers have some
form of multi-item loading or executing. This crate provides an improved
version of this functionality by combining multiple calls from different
scopes into a single batched request.

### Tradeoffs

Because we are buffering requests for a short period of time we do see
higher latencies when there are not many requests. This is because the
overhead from just processing the requests is lower then the time we spend
buffering.

However, this is often negated when we have a large number of requests as we
see on average lower latencies due to more efficient use of resources.
Latency is also more consistent as we are doing fewer requests to the
external resource.

### Usage

Here is an example of how to use the `DataLoader` interface to batch
multiple reads from a database.

````rust
struct MyUserLoader(SomeDatabase);

impl DataLoaderFetcher for MyUserLoader {
    type Key = i64;
    type Value = User;

    async fn load(&self, keys: HashSet<Self::Key>) -> Option<HashMap<Self::Key, Self::Value>> {
        let users = self.0.fetch("SELECT * FROM users WHERE id IN ($1)").bind(keys).await.map_err(|e| {
            eprintln!("Failed to fetch users: {}", e);
        }).ok()?;

        Some(users.into_iter().map(|user| (user.id, user)).collect())
    }
}

let loader = DataLoaderBuilder::new().build(MyUserLoader(database));

// Will only make a single request to the database and load both users
// You can also use `loader.load_many` if you have more then one item to load.
let (user1, user2): (Result<_, _>, Result<_, _>) = tokio::join!(loader.load(1), loader.load(2));
````

Another use case might be to batch multiple writes to a database.

````rust
struct MyUserUpdater(SomeDatabase);

impl BatchExecutor for MyUserUpdater {
    type Request = User;
    type Response = bool;

    async fn execute(&self, requests: Vec<(Self::Request, BatchResponse<Self::Response>)>) {
        let (users, responses): (Vec<Self::Request>, Vec<BatchResponse<Self::Response>>) = requests.into_iter().unzip();

        // You would need to build the query somehow, this is just an example
        if let Err(e) = self.0.update("INSERT INTO users (id, name) VALUES ($1, $2), ($3, $4)").bind(users).await {
            eprintln!("Failed to insert users: {}", e);

            for response in responses {
                // Reply back saying we failed
                response.send(false);
            }

            return;
        }

        // Reply back to the client that we successfully inserted the users
        for response in responses {
            response.send(true);
        }
    }
}

let batcher = BatcherBuilder::new().build(MyUserUpdater(database));
// Will only make a single request to the database and insert both users
// You can also use `batcher.execute_many` if you have more then one item to insert.
let (success1, success2) = tokio::join!(batcher.execute(user1), batcher.execute(user2));

if success1.is_some_and(|s| !s) {
    eprintln!("Failed to insert user 1");
}

if success2.is_some_and(|s| !s) {
    eprintln!("Failed to insert user 2");
}
````

### License

This project is licensed under the MIT or Apache-2.0 license.
You can choose between one of them if you use this work.

`SPDX-License-Identifier: MIT OR Apache-2.0`
<!-- cargo-sync-rdme ]] -->
