//! A crate designed to batch multiple requests into a single request.
//!
//! ## Why do we need this?
//!
//! Often when we are building applications we need to load multiple items from
//! a database or some other external resource. It is often expensive to load
//! each item individually, and this is typically why most drivers have some
//! form of multi-item loading or executing. This crate provides an improved
//! version of this functionality by combining multiple calls from different
//! scopes into a single batched request.
//!
//! ## Tradeoffs
//!
//! Because we are buffering requests for a short period of time we do see
//! higher latencies when there are not many requests. This is because the
//! overhead from just processing the requests is lower then the time we spend
//! buffering.
//!
//! However, this is often negated when we have a large number of requests as we
//! see on average lower latencies due to more efficient use of resources.
//! Latency is also more consistent as we are doing fewer requests to the
//! external resource.
//!
//! ## Usage
//!
//! Here is an example of how to use the `DataLoader` interface to batch
//! multiple reads from a database.
//!
//! ```rust
//! # use std::collections::{HashSet, HashMap};
//! # use scuffle_batching::{DataLoaderFetcher, DataLoader, dataloader::DataLoaderBuilder};
//! # tokio_test::block_on(async {
//! # #[derive(Clone, Hash, Eq, PartialEq)]
//! # struct User {
//! #     pub id: i64,
//! # }
//! # struct SomeDatabase;
//! # impl SomeDatabase {
//! #    fn fetch(&self, query: &str) -> Fetch {
//! #       Fetch
//! #    }
//! # }
//! # struct Fetch;
//! # impl Fetch {
//! #     async fn bind(&self, user_ids: HashSet<i64>) -> Result<Vec<User>, &'static str> {
//! #         Ok(vec![])
//! #     }
//! # }
//! # let database = SomeDatabase;
//! struct MyUserLoader(SomeDatabase);
//!
//! impl DataLoaderFetcher for MyUserLoader {
//!     type Key = i64;
//!     type Value = User;
//!
//!     async fn load(&self, keys: HashSet<Self::Key>) -> Option<HashMap<Self::Key, Self::Value>> {
//!         let users = self.0.fetch("SELECT * FROM users WHERE id IN ($1)").bind(keys).await.map_err(|e| {
//!             eprintln!("Failed to fetch users: {}", e);
//!         }).ok()?;
//!
//!         Some(users.into_iter().map(|user| (user.id, user)).collect())
//!     }
//! }
//!
//! let loader = DataLoaderBuilder::new().build(MyUserLoader(database));
//!
//! // Will only make a single request to the database and load both users
//! // You can also use `loader.load_many` if you have more then one item to load.
//! let (user1, user2): (Result<_, _>, Result<_, _>) = tokio::join!(loader.load(1), loader.load(2));
//! # });
//! ```
//!
//! Another use case might be to batch multiple writes to a database.
//!
//! ```rust
//! # use std::collections::HashSet;
//! # use scuffle_batching::{DataLoaderFetcher, BatchExecutor, Batcher, batch::{BatchResponse, BatcherBuilder}, DataLoader};
//! # tokio_test::block_on(async move {
//! # #[derive(Clone, Hash, Eq, PartialEq)]
//! # struct User {
//! #     pub id: i64,
//! # }
//! # struct SomeDatabase;
//! # impl SomeDatabase {
//! #    fn update(&self, query: &str) -> Update {
//! #       Update
//! #    }
//! # }
//! # struct Update;
//! # impl Update {
//! #     async fn bind(&self, users: Vec<User>) -> Result<Vec<User>, &'static str> {
//! #         Ok(vec![])
//! #     }
//! # }
//! # let database = SomeDatabase;
//! struct MyUserUpdater(SomeDatabase);
//!
//! impl BatchExecutor for MyUserUpdater {
//!     type Request = User;
//!     type Response = bool;
//!
//!     async fn execute(&self, requests: Vec<(Self::Request, BatchResponse<Self::Response>)>) {
//!         let (users, responses): (Vec<Self::Request>, Vec<BatchResponse<Self::Response>>) = requests.into_iter().unzip();
//!
//!         // You would need to build the query somehow, this is just an example
//!         if let Err(e) = self.0.update("INSERT INTO users (id, name) VALUES ($1, $2), ($3, $4)").bind(users).await {
//!             eprintln!("Failed to insert users: {}", e);
//!
//!             for response in responses {
//!                 // Reply back saying we failed
//!                 response.send(false);
//!             }
//!
//!             return;
//!         }
//!
//!         // Reply back to the client that we successfully inserted the users
//!         for response in responses {
//!             response.send(true);
//!         }
//!     }
//! }
//!
//! let batcher = BatcherBuilder::new().build(MyUserUpdater(database));
//! # let user1 = User { id: 1 };
//! # let user2 = User { id: 2 };
//! // Will only make a single request to the database and insert both users
//! // You can also use `batcher.execute_many` if you have more then one item to insert.
//! let (success1, success2) = tokio::join!(batcher.execute(user1), batcher.execute(user2));
//!
//! if success1.is_some_and(|s| !s) {
//!     eprintln!("Failed to insert user 1");
//! }
//!
//! if success2.is_some_and(|s| !s) {
//!     eprintln!("Failed to insert user 2");
//! }
//! # });
//! ```
//!
//! ## License
//!
//! This project is licensed under the [MIT](./LICENSE.MIT) or
//! [Apache-2.0](./LICENSE.Apache-2.0) license. You can choose between one of
//! them if you use this work.
//!
//! `SPDX-License-Identifier: MIT OR Apache-2.0`
#![cfg_attr(all(coverage_nightly, test), feature(coverage_attribute))]
#![deny(missing_docs)]
#![deny(unsafe_code)]

pub mod batch;
pub mod dataloader;

pub use batch::{BatchExecutor, Batcher};
pub use dataloader::{DataLoader, DataLoaderFetcher};
