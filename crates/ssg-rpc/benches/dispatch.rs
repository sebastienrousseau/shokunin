// Copyright © 2023 - 2026 Static Site Generator (SSG). All rights reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Dispatch-overhead bench backing issue #548 AC7.
//!
//! AC7 requires p99 dispatch overhead (worker entry → Rust function
//! start) ≤ 5 ms on Cloudflare Workers. Cloudflare's V8 isolate is
//! roughly 2-3× slower than a bare-metal native run; we set the
//! native bench target at **≤ 500 µs p99** which gives a comfortable
//! 10× headroom.

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use ssg_rpc::{
    dispatch, schema_for, schema_for_result, RpcDescriptor, RpcDescriptorRef,
    RpcError, RpcSchema,
};

#[derive(Serialize, Deserialize, JsonSchema)]
struct EchoIn {
    msg: String,
}

#[derive(Serialize, Deserialize, JsonSchema)]
struct EchoOut {
    msg: String,
}

fn echo_dispatch(payload: &str) -> Result<String, RpcError> {
    let inp: EchoIn = serde_json::from_str(payload)
        .map_err(|e| RpcError::BadRequest(e.to_string()))?;
    let out = EchoOut { msg: inp.msg };
    serde_json::to_string(&out).map_err(|e| RpcError::Internal(e.to_string()))
}

fn echo_schema() -> RpcSchema {
    RpcSchema {
        name: "bench_echo",
        input: schema_for::<EchoIn>(),
        output: schema_for_result::<Result<EchoOut, RpcError>>(),
    }
}

static BENCH_ECHO: RpcDescriptor = RpcDescriptor {
    name: "bench_echo",
    dispatch: echo_dispatch,
    schema: echo_schema,
};

ssg_rpc::inventory::submit! { RpcDescriptorRef(&BENCH_ECHO) }

fn bench_dispatch(c: &mut Criterion) {
    let payload = "{\"msg\":\"hello\"}";
    let _ = c.bench_function("rpc_dispatch_echo", |b| {
        b.iter(|| {
            let out = dispatch(black_box("bench_echo"), black_box(payload));
            let _ = black_box(out);
        });
    });
}

criterion_group!(benches, bench_dispatch);
criterion_main!(benches);
