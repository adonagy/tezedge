#![allow(dead_code)]

use std::sync::{Arc, Mutex};

use futures::channel::oneshot::{channel, Sender as ChannelSender};
use futures::future::RemoteHandle;
use futures::FutureExt;

use riker::actors::*;

/// Fork of ask pattern from the `riker-patterns` repo.
/// Send specific actor an message, and await the response
pub fn ask<Msg, Ctx, R, T>(ctx: &Ctx, receiver: &T, msg: Msg) -> RemoteHandle<R>
    where
        Msg: Message,
        R: Message,
        Ctx: TmpActorRefFactory + Run,
        T: Tell<Msg>,
{
    let (tx, rx) = channel::<R>();
    let tx = Arc::new(Mutex::new(Some(tx)));

    let props = Props::new_args(Box::new(AskActor::new), tx);
    let actor = ctx.tmp_actor_of(props).unwrap();
    receiver.tell(msg, Some(actor.into()));

    ctx.run(rx.map(|r| r.unwrap())).unwrap()
}

struct AskActor<Msg> {
    tx: Arc<Mutex<Option<ChannelSender<Msg>>>>,
}

impl<Msg: Message> AskActor<Msg> {
    fn new(tx: Arc<Mutex<Option<ChannelSender<Msg>>>>) -> BoxActor<Msg> {
        let ask = AskActor { tx };
        Box::new(ask)
    }
}

impl<Msg: Message> Actor for AskActor<Msg> {
    type Msg = Msg;

    fn recv(&mut self, ctx: &Context<Msg>, msg: Msg, _: Sender) {
        if let Ok(mut tx) = self.tx.lock() {
            tx.take().unwrap().send(msg).unwrap();
        }
        ctx.stop(&ctx.myself);
    }
}