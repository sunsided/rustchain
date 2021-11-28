use crate::block::Block;
use chrono::Utc;
use log::{error, warn};

/// This is the basis for our very simplistic mining scheme. Essentially, when mining a block,
/// the person mining has to hash the data for the block (with SHA256, in our case) and find a
/// hash, which, in binary, starts with 00 (two zeros).
/// This also denotes our “difficulty” on the network.
///
/// The time to find a suitable hash increases with the number of leading zeroes.
/// In a "real" blockchain system, this difficulty would be a network
/// attribute, which is agreed upon between nodes based on a consensus algorithm and based on
/// the network’s hash-power, so the network can guarantee to produce a new block
/// in a certain amount of time.
const DIFFICULTY_PREFIX: &str = "00";

fn hash_to_binary_representation(hash: &[u8]) -> String {
    let mut res: String = String::default();
    for c in hash {
        res.push_str(&format!("{:b}", c));
    }
    res
}

pub struct App {
    pub blocks: Vec<Block>,
}

impl App {
    fn new() -> Self {
        Self { blocks: vec![] }
    }

    /// Creates the first, hard-coded block in our blockchain.
    /// This is a "special" block in that it doesn’t really adhere to the same
    /// rules as the rest of the blocks. For example, it doesn’t have a valid
    /// `previous_hash`, since there simply was no block before it.
    fn genesis(&mut self) {
        assert!(self.blocks.is_empty());
        let genesis_block = Block {
            id: 0,
            timestamp: Utc::now().timestamp(),
            previous_hash: String::from("genesis"),
            data: String::from("genesis!"),
            nonce: 2836,
            hash: "0000f816a87f806bb0073dcf026a64fb40c946b5abee2573702828694d5b4c43".to_string(),
        };
        self.blocks.push(genesis_block);
    }

    fn try_add_block(&mut self, block: Block) {
        let latest_block = self.blocks.last().expect("there is at least one block");
        if self.is_block_valid(&block, latest_block) {
            self.blocks.push(block);
        } else {
            error!("could not add block - invalid");
        }
    }

    /// Validates a `Block`. This is important because it ensures our blockchain adheres to it’s
    /// chain property and is hard to tamper with. The difficulty of changing something increases
    /// with every block since you’d have to recalculate (i.e., re-mine) the rest of the chain
    /// to get a valid chain again.
    /// This would be expensive enough to disincentivise you in a real blockchain system.
    fn is_block_valid(&self, block: &Block, previous_block: &Block) -> bool {
        if block.previous_hash != previous_block.hash {
            warn!("block with id: {} has wrong previous hash", block.id);
            return false;
        }

        if !hash_to_binary_representation(&hex::decode(&block.hash).expect("can decode from hex"))
            .starts_with(DIFFICULTY_PREFIX)
        {
            warn!("block with id: {} has invalid difficulty", block.id);
            return false;
        }

        if block.id != previous_block.id + 1 {
            warn!(
                "block with id: {} is not the next block after the latest: {}",
                block.id, previous_block.id
            );
            return false;
        }

        if hex::encode(calculate_hash(
            block.id,
            block.timestamp,
            &block.previous_hash,
            &block.data,
            block.nonce,
        )) != block.hash
        {
            warn!("block with id: {} has invalid hash", block.id);
            return false;
        }
        true
    }

    fn is_chain_valid(&self, chain: &[Block]) -> bool {
        // Go through the entire chain but ignore the genesis block.
        for i in 1..chain.len() {
            let first = chain.get(i - 1).expect("has to exist");
            let second = chain.get(i).expect("has to exist");
            if !self.is_block_valid(second, first) {
                return false;
            }
        }
        true
    }

    /// We always choose the longest valid chain
    fn choose_chain(&mut self, local: Vec<Block>, remote: Vec<Block>) -> Vec<Block> {
        let is_local_valid = self.is_chain_valid(&local);
        let is_remote_valid = self.is_chain_valid(&remote);

        if is_local_valid && is_remote_valid {
            if local.len() >= remote.len() {
                local
            } else {
                remote
            }
        } else if is_remote_valid && !is_local_valid {
            remote
        } else if !is_remote_valid && is_local_valid {
            local
        } else {
            panic!("local and remote chains are both invalid");
        }
    }
}
