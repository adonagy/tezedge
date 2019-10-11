use super::events::{EventStorage, EventPayloadStorage, Event};
use networking::p2p::network_channel::NetworkChannelRef;
use rocksdb::DB;
use std::{sync::Arc, time::Instant};
use riker::actor::*;
use networking::p2p::encoding::peer::PeerMessageResponse;
use networking::p2p::binary_message::BinaryMessage;

#[derive(Clone, Debug)]
/// Empty message, to control reading from the database, to prevent blocking the thread, during the
/// reading events from the database
pub enum PlayerSignal {
    Start,
    Continue,
}

type NetworkChannelPlayerRef = ActorRef<PlayerSignal>;

/// An actor class for replaying recorded Network Channel messages
pub struct NetworkChannelPlayer {
    events: EventStorage,
    payloads: EventPayloadStorage,
    network_channel: NetworkChannelRef,
    /// Start of replaying, for more  accurate message delivery simulation.
    /// Counting start, when all event headers are loaded, not when actor is created.
    start: Instant,
    /// Recorded history of messages.
    history: Vec<(u64, Event)>,
}

impl NetworkChannelPlayer {
    pub fn name() -> &'static str { "network-channel-player" }

    pub fn new((network_channel, db): (NetworkChannelRef, Arc<DB>)) -> Self {
        Self {
            events: EventStorage::new(db.clone()),
            payloads: EventPayloadStorage::new(db),
            network_channel,
            start: Instant::now(),
            history: Vec::new(),
        }
    }

    pub fn actor(sys: &impl ActorRefFactory, rocks_db: Arc<DB>, network_channel: NetworkChannelRef) -> Result<NetworkChannelPlayerRef, CreateError> {
        sys.actor_of(
            Props::new_args(Self::new, (network_channel, rocks_db)),
            Self::name(),
        )
    }

    // TODO: Replace Option with Result
    fn load_string_payload(&mut self, index: u64) -> Option<String> {
        if let Ok(payload) = self.payloads.get_record(index) {
            if let Some(payload) = payload {
                String::from_utf8(payload).ok()
            } else {
                None
            }
        } else {
            None
        }
    }

    fn load_message_payload(&mut self, index: u64) -> Option<PeerMessageResponse> {
        if let Some(payload) = self.load_raw_payload(index) {
            PeerMessageResponse::from_bytes(payload).ok()
        } else {
            None
        }
    }

    fn load_raw_payload(&mut self, index: u64) -> Option<Vec<u8>> {
        if let Some(ret) = self.payloads.get_record(index).ok() {
            ret
        } else {
            None
        }
    }

    fn process_message(&mut self, _: PlayerSignal) {
        // Replay message:
        // 1. Load the message data
        // 2. Deserialize it
        // 3. Create Testing actors to check the consistency
        // 4. Send it by the NetworkChannel
    }
}

impl Actor for NetworkChannelPlayer {
    type Msg = PlayerSignal;

    fn pre_start(&mut self, ctx: &Context<Self::Msg>) {
        match self.events.load_events() {
            Ok(history) => {
                self.history = history;
                ctx.myself.tell(PlayerSignal::Start, None);
            }
            Err(_err) => {} // Log failed reading
        }
    }

    fn recv(&mut self, _ctx: &Context<Self::Msg>, msg: Self::Msg, _sender: Option<BasicActorRef>) {
        if let PlayerSignal::Start = msg {
            self.start = Instant::now();
        }
        self.process_message(msg);
    }
}