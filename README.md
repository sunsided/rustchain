# rustchain - a blockchain example in Rust

This repo is purely an exploration of current ideas communicated in various 
blog posts. It is not meant to produce a practically useful blockchain solution,
nor does it attempt to be particularly smart about it. 

- [How to build a blockchain in Rust] by Mario Zupan
  - Essentially builds upon [libp2p], [Tokio] and sha2
  - Data is purely in-memory, i.e. the blockchain is not persisted
- [How to Build a Blockchain from Scratch with Rust Programming Language] by Nandu Singh
- [Rusty Chains: A Basic Blockchain Implementation Written in Pure Rust] on hackernoon

[How to build a blockchain in Rust]: https://blog.logrocket.com/how-to-build-a-blockchain-in-rust/
[How to Build a Blockchain from Scratch with Rust Programming Language]: https://morioh.com/p/d554ac13bad3
[Rusty Chains: A Basic Blockchain Implementation Written in Pure Rust]: https://hackernoon.com/rusty-chains-a-basic-blockchain-implementation-written-in-pure-rust-gk2m3uri

[libp2p]: https://github.com/libp2p/rust-libp2p
[Tokio]: https://github.com/tokio-rs/tokio