use crate::app::App;
use crate::block::Block;
use libp2p::floodsub::{Floodsub, FloodsubEvent, Topic};
use libp2p::futures::future::Lazy;
use libp2p::mdns::{Mdns, MdnsEvent};
use libp2p::swarm::NetworkBehaviourEventProcess;
use libp2p::{identity, PeerId};
use log::{error, info};
use std::sync::mpsc;

pub static KEYS: Lazy<identity::Keypair> = Lazy::new(identity::Keypair::generate_ed25519);
pub static PEER_ID: Lazy<PeerId> = Lazy::new(|| PeerId::from(KEYS.public()));
pub static CHAIN_TOPIC: Lazy<Topic> = Lazy::new(|| Topic::new("chains"));
pub static BLOCK_TOPIC: Lazy<Topic> = Lazy::new(|| Topic::new("blocks"));

/// Response to `LocalChainRequest`.
#[derive(Debug, Serialize, Deserialize)]
pub struct ChainResponse {
    pub blocks: Vec<Block>,
    pub receiver: String,
}

/// Request of a `ChainResponse` from a specific peer.
#[derive(Debug, Serialize, Deserialize)]
pub struct LocalChainRequest {
    pub from_peer_id: String,
}

pub enum EventType {
    LocalChainResponse(ChainResponse),
    Input(String),
    Init,
}

#[derive(NetworkBehaviour)]
pub struct AppBehaviour {
    // Note that Floodsub broadcasts all requests. This is horribly inefficient,
    // but it is trivial to set up. See e.g. `Gossipsub`.
    pub floodsub: Floodsub,
    pub mdns: Mdns,
    #[behaviour(ignore)]
    pub response_sender: mpsc::UnboundedSender,
    #[behaviour(ignore)]
    pub init_sender: mpsc::UnboundedSender,
    #[behaviour(ignore)]
    pub app: App,
}

impl AppBehaviour {
    pub async fn new(
        app: App,
        response_sender: mpsc::UnboundedSender,
        init_sender: mpsc::UnboundedSender,
    ) -> Self {
        let mut behaviour = Self {
            app,
            floodsub: Floodsub::new(*PEER_ID),
            mdns: Mdns::new(Default::default())
                .await
                .expect("can create mdns"),
            response_sender,
            init_sender,
        };
        behaviour.floodsub.subscribe(CHAIN_TOPIC.clone());
        behaviour.floodsub.subscribe(BLOCK_TOPIC.clone());

        behaviour
    }
}

impl NetworkBehaviourEventProcess<MdnsEvent> for AppBehaviour {
    fn inject_event(&mut self, event: MdnsEvent) {
        match event {
            MdnsEvent::Discovered(discovered_list) => {
                for (peer, _addr) in discovered_list {
                    self.floodsub.add_node_to_partial_view(peer);
                }
            }
            MdnsEvent::Expired(expired_list) => {
                for (peer, _addr) in expired_list {
                    if !self.mdns.has_node(&peer) {
                        self.floodsub.remove_node_from_partial_view(&peer);
                    }
                }
            }
        }
    }
}

// incoming event handler
impl NetworkBehaviourEventProcess<EventType> for AppBehaviour {
    fn inject_event(&mut self, event: FloodsubEvent) {
        if let FloodsubEvent::Message(msg) = event {
            if let Ok(resp) = serde_json::from_slice::<ChainResponse>(&msg.data) {
                // If it’s a ChainResponse, it means we got sent a local blockchain by another node.
                // We check wether we’re actually the receiver of said piece of data and, if so,
                // log the incoming blockchain and attempt to execute our consensus. If it’s valid
                // and longer than our chain, we replace our chain with it.
                // Otherwise, we keep our own chain.
                if resp.receiver == PEER_ID.to_string() {
                    info!("Response from {}:", msg.source);
                    resp.blocks.iter().for_each(|r| info!("{:?}", r));
                    self.app.blocks = self.app.choose_chain(self.app.blocks.clone(), resp.blocks);
                }
            } else if let Ok(resp) = serde_json::from_slice::<LocalChainRequest>(&msg.data) {
                // If the incoming data is a LocalChainRequest, we check whether we’re the ones
                // they want the chain from, checking the from_peer_id. If so, we simply send them
                // a JSON version of our local blockchain. The actual sending part is in another
                // part of the code, but for now, we simply send it through our event
                // channel for responses.
                info!("sending local chain to {}", msg.source.to_string());
                let peer_id = resp.from_peer_id;
                if PEER_ID.to_string() == peer_id {
                    if let Err(e) = self.response_sender.send(ChainResponse {
                        blocks: self.app.blocks.clone(),
                        receiver: msg.source.to_string(),
                    }) {
                        error!("error sending response via channel, {}", e);
                    }
                }
            } else if let Ok(block) = serde_json::from_slice::<Block>(&msg.data) {
                // Finally, if it’s a Block that’s incoming, that means someone else mined a block
                // and wants us to add it to our local chain.
                // We check whether the block is valid and, if it is, add it.
                info!("received new block from {}", msg.source.to_string());
                self.app.try_add_block(block);
            }
        }
    }
}
