<!-- SPDX-License-Identifier: Apache-2.0 OR MIT -->

# ssg-rpc

Edge RPC layer for [SSG](https://crates.io/crates/ssg) — JSON-over-POST
dispatcher + `#[ssg_rpc]` macro + TypeScript schema emitter (issue
[#548](https://github.com/sebastienrousseau/static-site-generator/issues/548)).

This crate is part of the SSG workspace. Documentation lives on
[docs.rs](https://docs.rs/ssg-rpc/) and the canonical README is the
[repository root](https://github.com/sebastienrousseau/static-site-generator#readme).

## What it does

- A `dispatch(name, json)` runtime that resolves RPC method names to
  their typed handlers via a static `inventory`-based registry.
- The companion proc-macro [`ssg-rpc-macro`](https://crates.io/crates/ssg-rpc-macro)
  exposes `#[ssg_rpc]` to register a function with the dispatcher.
- A JSON-Schema → TypeScript emitter so site authors get a typed
  `.d.ts` for their RPC methods at build time.

## Quick start

```rust,ignore
use ssg_rpc::{ssg_rpc, RpcError};
use serde::{Deserialize, Serialize};
use schemars::JsonSchema;

#[derive(Deserialize, JsonSchema)]
struct LikeInput { post_id: String }

#[derive(Serialize, JsonSchema)]
struct LikeOutput { likes: u64 }

#[ssg_rpc]
fn like_post(input: LikeInput) -> Result<LikeOutput, RpcError> {
    Ok(LikeOutput { likes: input.post_id.len() as u64 + 1 })
}
```

The build then emits `site/rpc.d.ts`; the JS client calls
`POST /__rpc/like_post` with the JSON body.

## License

Dual-licensed under [Apache 2.0](https://www.apache.org/licenses/LICENSE-2.0)
or [MIT](https://opensource.org/licenses/MIT), at your option.
