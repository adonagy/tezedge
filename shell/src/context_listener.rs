// Copyright (c) SimpleStaking and Tezedge Contributors
// SPDX-License-Identifier: MIT

use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::thread::JoinHandle;

use failure::Error;
use riker::actors::*;
use slog::{debug, Logger, warn};

use storage::{ContextStorage, ContextRecordKey, ContextRecordValue};
use structure::SkipList;
use tezos_context::channel::ContextAction;
use tezos_wrapper::service::IpcEvtServer;
use std::collections::HashMap;
use tezos_context::ContextValue;

type SharedJoinHandle = Arc<Mutex<Option<JoinHandle<Result<(), Error>>>>>;

#[actor]
pub struct ContextListener {
    /// Thread where blocks are applied will run until this is set to `false`
    listener_run: Arc<AtomicBool>,
    /// Context event listener thread
    listener_thread: SharedJoinHandle,
}

pub type ContextListenerRef = ActorRef<ContextListenerMsg>;

impl ContextListener {
    pub fn actor(sys: &impl ActorRefFactory, rocks_db: Arc<rocksdb::DB>, mut event_server: IpcEvtServer, log: Logger) -> Result<ContextListenerRef, CreateError> {
        let listener_run = Arc::new(AtomicBool::new(true));
        let block_applier_thread = {
            let listener_run = listener_run.clone();

            thread::spawn(move || {
                let mut context_storage = ContextStorage::new(rocks_db.clone());
                while listener_run.load(Ordering::Acquire) {
                    match listen_protocol_events(
                        &listener_run,
                        &mut event_server,
                        &mut context_storage,
                        rocks_db.clone(),
                        &log,
                    ) {
                        Ok(()) => debug!(log, "Context listener finished"),
                        Err(err) => {
                            if listener_run.load(Ordering::Acquire) {
                                warn!(log, "Timeout while waiting for context event connection"; "reason" => format!("{:?}", err))
                            }
                        }
                    }
                }

                Ok(())
            })
        };

        let myself = sys.actor_of(
            Props::new_args(ContextListener::new, (listener_run, Arc::new(Mutex::new(Some(block_applier_thread))))),
            ContextListener::name())?;

        Ok(myself)
    }

    /// The `ContextListener` is intended to serve as a singleton actor so that's why
    /// we won't support multiple names per instance.
    fn name() -> &'static str {
        "context-listener"
    }

    fn new((block_applier_run, block_applier_thread): (Arc<AtomicBool>, SharedJoinHandle)) -> Self {
        ContextListener {
            listener_run: block_applier_run,
            listener_thread: block_applier_thread,
        }
    }
}

impl Actor for ContextListener {
    type Msg = ContextListenerMsg;

    fn post_stop(&mut self) {
        self.listener_run.store(false, Ordering::Release);

        let _ = self.listener_thread.lock().unwrap()
            .take().expect("Thread join handle is missing")
            .join().expect("Failed to join context listener thread");
    }

    fn recv(&mut self, ctx: &Context<Self::Msg>, msg: Self::Msg, sender: Sender) {
        self.receive(ctx, msg, sender);
    }
}

fn listen_protocol_events(
    apply_block_run: &AtomicBool,
    event_server: &mut IpcEvtServer,
    context_storage: &mut ContextStorage,
    db: Arc<rocksdb::DB>,
    log: &Logger,
) -> Result<(), Error> {
    debug!(log, "Waiting for connection from protocol runner");
    let mut rx = event_server.accept()?;
    debug!(log, "Received connection from protocol runner. Starting to process context events.");

    let mut event_count = 0;
    let mut list = SkipList::new(db);
    while apply_block_run.load(Ordering::Acquire) {
        let mut state: HashMap<ContextRecordKey, ContextValue> = Default::default();
        match rx.receive() {
            Ok(ContextAction::Shutdown) => break,
            Ok(msg) => {
                if event_count % 100 == 0 {
                    debug!(log, "Received protocol event"; "count" => event_count);
                }
                event_count += 1;

                match &msg {
                    ContextAction::Set { block_hash: Some(block_hash), operation_hash, key, value, .. } => {
                        let record_key = ContextRecordKey::new(block_hash, operation_hash, key);
                        let value = value.clone();
                        let record_value = ContextRecordValue::new(msg);
                        context_storage.put(&record_key, &record_value)?;
                        state.insert(record_key, value.clone());
                    }
                    ContextAction::Copy { block_hash: Some(block_hash), operation_hash, to_key: key, from_key, .. } => {
                        let record_key = ContextRecordKey::new(block_hash, operation_hash, key);
                        let record_from_key = ContextRecordKey::new(block_hash, operation_hash, from_key);
                        let record_value = ContextRecordValue::new(msg);
                        context_storage.put(&record_key, &record_value)?;
                        if state.contains_key(&record_from_key) {
                            state.insert(record_key, state.get(&record_from_key).unwrap().clone());
                        }
                    }
                    ContextAction::Delete { block_hash: Some(block_hash), operation_hash, key, .. }
                    | ContextAction::RemoveRecord { block_hash: Some(block_hash), operation_hash, key, .. } => {
                        let record_key = ContextRecordKey::new(block_hash, operation_hash, key);
                        let record_value = ContextRecordValue::new(msg);
                        context_storage.put(&record_key, &record_value)?;
                        state.remove(&record_key);
                    }
                    ContextAction::Commit { .. } => {
                        list.push(std::mem::replace(&mut state, Default::default()));
                    }
                    _ => (),
                };
            }
            Err(err) => {
                warn!(log, "Failed to receive event from protocol runner"; "reason" => format!("{:?}", err));
                break;
            }
        }
    }

    Ok(())
}

