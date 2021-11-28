# rustchain - a blockchain example in Rust

This repo is purely an exploration of current ideas communicated in various 
blog posts. It is not meant to produce a practically useful blockchain solution,
nor does it attempt to be particularly smart about it. Most if not all credit
goes to the authors of the posts below:

- [How to build a blockchain in Rust] by Mario Zupan
  - Proof-of-work mining
  - Data is purely in-memory, i.e. the blockchain is not persisted
  - Essentially builds upon [libp2p], [Tokio] and sha2
  - GitHub repo at [zupzup/rust-blockchain-example]
- [How to Build a Blockchain from Scratch with Rust Programming Language] by Nandu Singh
- [Rusty Chains: A Basic Blockchain Implementation Written in Pure Rust] on hackernoon

## Example

To run the example, execute

```shell
$ RUST_LOG=info cargo run
```

After the console output settles, you can execute these commands:

- `ls p`: prints all discovered peers
- `ls c`: prints the current blockchain
- `create b <DATA>`: creates (mines) a new block containing the payload `<DATA>`

## Known flaws

- If two peers disagree on the next block in a chain because they are mining a 
  successor block to the current state, the first one wins and the second one
  is now in a failed state from which it will not recover.
- Race conditions can be created by increasing the difficulty and creating
  a block on multiple application instances simultaneously.

[How to build a blockchain in Rust]: https://blog.logrocket.com/how-to-build-a-blockchain-in-rust/
[How to Build a Blockchain from Scratch with Rust Programming Language]: https://morioh.com/p/d554ac13bad3
[Rusty Chains: A Basic Blockchain Implementation Written in Pure Rust]: https://hackernoon.com/rusty-chains-a-basic-blockchain-implementation-written-in-pure-rust-gk2m3uri

[libp2p]: https://github.com/libp2p/rust-libp2p
[Tokio]: https://github.com/tokio-rs/tokio

[zupzup/rust-blockchain-example]: https://github.com/zupzup/rust-blockchain-example