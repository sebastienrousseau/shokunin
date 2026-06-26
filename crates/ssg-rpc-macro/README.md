<!-- SPDX-License-Identifier: Apache-2.0 OR MIT -->

# ssg-rpc-macro

Proc-macro backing the `#[ssg_rpc]` attribute (split out so
[`ssg-rpc`](https://crates.io/crates/ssg-rpc) itself stays a normal
library without a `proc-macro = true` toggle).

You typically don't depend on this crate directly — it's re-exported
from `ssg-rpc` as the `#[ssg_rpc]` attribute. See the
[ssg-rpc README](https://crates.io/crates/ssg-rpc) for usage.

The macro:

1. Re-emits the original function untouched, so direct callers in Rust
   keep working.
2. Generates a sibling `__SSG_RPC_<fn_name>` static descriptor.
3. Wires the static into the `inventory`-based dispatch registry so
   `ssg_rpc::dispatch(name, json)` resolves it at runtime.

## License

Dual-licensed under [Apache 2.0](https://www.apache.org/licenses/LICENSE-2.0)
or [MIT](https://opensource.org/licenses/MIT), at your option.
