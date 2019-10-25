use networking::p2p::network_channel::{NetworkChannelMsg, NetworkChannelTopic, NetworkChannelRef};
use shell::shell_channel::{ShellChannelRef, ShellChannelMsg, ShellChannelTopic};
use riker::{
    actors::*,
};
use crate::{
    helpers::*,
    server::{spawn_server, control_msg::*},
};
use slog::warn;
use std::net::SocketAddr;
use tokio::runtime::Runtime;
use monitoring::{MonitorRef, commands::GetPeerStats};

pub type RpcServerRef = ActorRef<RpcServerMsg>;

#[actor(NetworkChannelMsg, ShellChannelMsg, GetCurrentHead, GetPublicKey, GetPeerStats)]
pub struct RpcServer {
    network_channel: NetworkChannelRef,
    shell_channel: ShellChannelRef,

    // Stats
    current_head: Option<CurrentHead>,
    // Network
    public_key: String,
}

impl RpcServer {
    pub fn name() -> &'static str { "rpc-server" }

    fn new((network_channel, shell_channel, public_key): (NetworkChannelRef, ShellChannelRef, String)) -> Self {
        Self {
            network_channel,
            shell_channel,
            current_head: None,
            public_key,
        }
    }

    pub fn actor(sys: &ActorSystem, network_channel: NetworkChannelRef, shell_channel: ShellChannelRef, addr: SocketAddr, runtime: &Runtime, public_key: String) -> Result<RpcServerRef, CreateError> {
        let ret = sys.actor_of(
            Props::new_args(Self::new, (network_channel, shell_channel, public_key)),
            Self::name(),
        )?;

        let server = spawn_server(&addr, sys.clone(), ret.clone());
        let inner_log = sys.log();
        runtime.spawn(async move {
            if let Err(e) = server.await {
                warn!(inner_log, "HTTP Server encountered failure"; "error" => format!("{}", e));
            }
        });
        Ok(ret)
    }
}

impl Actor for RpcServer {
    type Msg = RpcServerMsg;

    fn pre_start(&mut self, ctx: &Context<Self::Msg>) {
        self.network_channel.tell(Subscribe {
            actor: Box::new(ctx.myself()),
            topic: NetworkChannelTopic::NetworkEvents.into(),
        }, ctx.myself().into());

        self.shell_channel.tell(Subscribe {
            actor: Box::new(ctx.myself()),
            topic: ShellChannelTopic::ShellEvents.into(),
        }, ctx.myself().into());
    }

    fn recv(&mut self, ctx: &Context<Self::Msg>, msg: Self::Msg, sender: Option<BasicActorRef>) {
        self.receive(ctx, msg, sender);
    }
}

impl Receive<NetworkChannelMsg> for RpcServer {
    type Msg = RpcServerMsg;

    fn receive(&mut self, _ctx: &Context<Self::Msg>, _msg: NetworkChannelMsg, _sender: Sender) {
        /* Not yet implemented, do nothing */
    }
}

impl Receive<ShellChannelMsg> for RpcServer {
    type Msg = RpcServerMsg;

    fn receive(&mut self, _ctx: &Context<Self::Msg>, msg: ShellChannelMsg, _sender: Sender) {
        match msg {
            ShellChannelMsg::BlockApplied(data) => {
                if let Some(ref current_head) = self.current_head {
                    if current_head.level() < data.level {
                        self.current_head = Some(CurrentHead::new(data.level, data.hash.clone()));
                    }
                } else {
                    self.current_head = Some(CurrentHead::new(data.level, data.hash.clone()));
                }
            }
            _ => (/* Not yet implemented, do nothing */),
        }
    }
}

impl Receive<GetCurrentHead> for RpcServer {
    type Msg = RpcServerMsg;

    fn receive(&mut self, ctx: &Context<Self::Msg>, msg: GetCurrentHead, sender: Sender) {
        if let GetCurrentHead::Request = msg {
            if let Some(sender) = sender {
                let me: Option<BasicActorRef> = ctx.myself().into();
                if sender.try_tell(GetCurrentHead::Response(self.current_head.clone()), me).is_err() {
                    warn!(ctx.system.log(), "Failed to send response for GetCurrentHead");
                }
            }
        }
    }
}

impl Receive<GetPublicKey> for RpcServer {
    type Msg = RpcServerMsg;

    fn receive(&mut self, ctx: &Context<Self::Msg>, msg: GetPublicKey, sender: Sender) {
        if let GetPublicKey::Request = msg {
            if let Some(sender) = sender {
                let me: Option<BasicActorRef> = ctx.myself().into();
                if sender.try_tell(GetPublicKey::Response(self.public_key.clone()), me).is_err() {
                    warn!(ctx.system.log(), "Failed to send response for GetPublicKey");
                }
            }
        }
    }
}

impl Receive<GetPeerStats> for RpcServer {
    type Msg = RpcServerMsg;

    fn receive(&mut self, ctx: &Context<Self::Msg>, msg: GetPublicKey, sender: Sender) {
        let actor = ctx.select("monitor-manager").expect("failed to get the monitor");
    }
}