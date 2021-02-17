# Play Substrate

## Runtime modules

* Template
* Proof of Existence
* Coin flip game 
* Offchain worker (unsigned transaction)
* Offchain worker (signed transaction)
* Weight demo
* Data type
* Genesis config demo

A new FRAME-based Substrate node, ready for hacking.

## Build

Install Rust:

```bash
curl https://sh.rustup.rs -sSf | sh
```

Initialize your Wasm Build environment:

```bash
./scripts/init.sh
```

Build Wasm and native code:

```bash
cargo build --release
```

## Run

### Single Node Development Chain

Purge any existing developer chain state:

```bash
./target/release/node-template purge-chain --dev
```

Start a development chain with:

```bash
./target/release/node-template --dev
```

Detailed logs may be shown by running the node with the following environment variables set: `RUST_LOG=debug RUST_BACKTRACE=1 cargo run -- --dev`.

### Multi-Node Local Testnet

If you want to see the multi-node consensus algorithm in action locally, then you can create a local testnet with two validator nodes for Alice and Bob, who are the initial authorities of the genesis chain that have been endowed with testnet units.

Optionally, give each node a name and expose them so they are listed on the Polkadot [telemetry site](https://telemetry.polkadot.io/#/Local%20Testnet).

You'll need two terminal windows open.

We'll start Alice's substrate node first on default TCP port 30333 with her chain database stored locally at `/tmp/alice`. The bootnode ID of her node is `QmRpheLN4JWdAnY7HGJfWFNbfkQCb6tFf4vvA6hgjMZKrR`, which is generated from the `--node-key` value that we specify below:

```bash
cargo run -- \
  --base-path /tmp/alice \
  --chain=local \
  --alice \
  --node-key 0000000000000000000000000000000000000000000000000000000000000001 \
  --telemetry-url 'ws://telemetry.polkadot.io:1024 0' \
  --validator
```

In the second terminal, we'll start Bob's substrate node on a different TCP port of 30334, and with his chain database stored locally at `/tmp/bob`. We'll specify a value for the `--bootnodes` option that will connect his node to Alice's bootnode ID on TCP port 30333:

```bash
cargo run -- \
  --base-path /tmp/bob \
  --bootnodes /ip4/127.0.0.1/tcp/30333/p2p/QmRpheLN4JWdAnY7HGJfWFNbfkQCb6tFf4vvA6hgjMZKrR \
  --chain=local \
  --bob \
  --port 30334 \
  --telemetry-url 'ws://telemetry.polkadot.io:1024 0' \
  --validator
```

Additional CLI usage options are available and may be shown by running `cargo run -- --help`.

## 1 为 template 模块的 do_something 添加 benchmark 用例（也可以是其它自选模块的可调用函数），并且将 benchmark 运行的结果转换为对应的权重定义

```
# 编译feature需要进入node子目录进行
$ cd benchmark-node-template/node
[Jason@RUAN:~/YiKuaiSubstrate/team1/lesson12/benchmark-node-template/node] (lesson-12)$ cargo build --features runtime-benchmarks --release

[Jason@RUAN:~/YiKuaiSubstrate/team1/lesson12/benchmark-node-template] (lesson-12)$ ./target/release/node-template benchmark --chain dev --execution=wasm --wasm-execution=compiled --pallet benchmark-demo --extrinsic do_something --steps 20 --repeat 50
```

## 2 选择 node-template 或者其它节点程序，生成 Chain Spec 文件（两种格式都需要）

```
Note: 上传 Chain Spec 文件即可
```

## 3（附加题）根据 Chain Spec，部署公开测试网络

```
Note: 上传 telemetry.polkadot.io 上你的网络节点的截图，或者apps上staking页面截图。
```

