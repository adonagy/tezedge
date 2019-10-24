use hyper::{Body, Response, Error, Server, Request, StatusCode, Method};
use hyper::service::{service_fn, make_service_fn};
use std::net::SocketAddr;
use futures::Future;
use crate::rpc_actor::RpcServerRef;
use riker::actors::ActorSystem;
use crate::server::ask::ask;
use serde_json;
use chrono::prelude::*;
use crate::encoding::base_types::*;
use tezos_encoding::hash::{HashEncoding, HashType};

type ServiceResult = Result<Response<Body>, Box<dyn std::error::Error + Sync + Send>>;

pub fn spawn_server(addr: &SocketAddr, sys: ActorSystem, actor: RpcServerRef) -> impl Future<Output=Result<(), Error>> {
    Server::bind(addr)
        .serve(make_service_fn(move |_| {
            let sys = sys.clone();
            let actor = actor.clone();
            async move {
                let sys = sys.clone();
                let actor = actor.clone();
                Ok::<_, Error>(service_fn(move |req| {
                    let sys = sys.clone();
                    let actor = actor.clone();
                    async move {
                        router(req, sys, actor).await
                    }
                }))
            }
        }))
}

fn timestamp() -> TimeStamp {
    TimeStamp::Integral(Utc::now().timestamp())
}

fn not_found() -> ServiceResult {
    Ok(Response::builder()
        .status(StatusCode::from_u16(404)?)
        .body(Body::from("not found"))?)
}

fn empty() -> ServiceResult {
    Ok(Response::builder()
        .status(StatusCode::from_u16(204)?)
        .body(Body::empty())?)
}

async fn bootstrapped(sys: ActorSystem, actor: RpcServerRef) -> ServiceResult {
    use crate::server::control_msg::GetCurrentHead;
    use crate::encoding::monitor::BootstrapInfo;

    let current_head = ask(&sys, &actor, GetCurrentHead::Request).await;
    if let GetCurrentHead::Response(current_head) = current_head {
        let resp = serde_json::to_string(&if let Some(current_head) = current_head {
            let hash = HashEncoding::new(HashType::BlockHash).bytes_to_string(&current_head.hash());
            BootstrapInfo::new(hash.into(), timestamp())
        } else {
            BootstrapInfo::new(String::new().into(), timestamp())
        })?;
        Ok(Response::new(Body::from(resp)))
    } else {
        empty()
    }
}

async fn commit_hash(_sys: ActorSystem, _actor: RpcServerRef) -> ServiceResult {
    let resp = serde_json::to_string(&UniString::from(env!("GIT_HASH")))?;
    Ok(Response::new(Body::from(resp)))
}

async fn public_key(sys: ActorSystem, actor: RpcServerRef) -> ServiceResult {
    use super::control_msg::GetPublicKey;

    let public_key = ask(&sys, &actor, GetPublicKey::Request).await;
    if let GetPublicKey::Response(public_key) = public_key {
        Ok(Response::new(Body::from(serde_json::to_string(&public_key)?)))
    } else {
        empty()
    }
}


async fn router(req: Request<Body>, sys: ActorSystem, actor: RpcServerRef) -> ServiceResult {
    match (req.method(), req.uri().path()) {
        (&Method::GET, "/monitor/bootstrapped") => bootstrapped(sys, actor).await,
        (&Method::GET, "/monitor/commit_hash") => commit_hash(sys, actor).await,
        (&Method::GET, "/network/self") => public_key(sys, actor).await,
        _ => not_found()
    }
}