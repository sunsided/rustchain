use crate::app::App;
use crate::block::Block;
use libp2p::floodsub::{Floodsub, Topic};
use libp2p::futures::future::Lazy;
use libp2p::mdns::{Mdns, MdnsEvent};
use libp2p::swarm::NetworkBehaviourEventProcess;
use libp2p::{identity, PeerId};
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
