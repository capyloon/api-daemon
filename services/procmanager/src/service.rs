use crate::cgroups::*;
use crate::config::Config;
use crate::generated::common::*;
use crate::generated::service::*;
use common::core::BaseMessage;
use log::{debug, error};
use std::convert::Into;
use std::fs::{File, rename};
use std::io::prelude::*;
use std::os::unix::net::UnixDatagram;
use std::collections::HashSet;
use itertools::Itertools;

use common::traits::{
    EmptyConfig, OriginAttributes, Service, SessionSupport, Shared, SharedServiceState,
    SharedSessionContext, StateLogger,
};

#[derive(Default)]
pub struct HintManager {
    hints: HashSet<String>,
    pub config: Config,
}

impl HintManager {
    fn process_hints(&mut self, hints: &[String]) -> bool {
        for modify in hints {
            // Hints with values are not supported yet!
            match &modify[..1] {
                "+" => {
                    self.hints.insert(String::from(modify.split_at(1).1));
                }
                "-" => {
                    self.hints.remove(&String::from(modify.split_at(1).1));
                }
                _ => {
                    debug!("hints in incorrect format: {}", modify);
                    return false;
                }
            };
        }
        true
    }

    fn new() -> HintManager {
        HintManager {
            hints: HashSet::new(),
            config: Config::default(),
        }
    }

    // Update hints to the hint file.
    //
    // Afrer a crash, b2gkillerd should load hints from the hint file
    // to restore it's state.
    //
    // To make sure b2gkillered is consistent with api-daemon, two
    // rules should be followed.
    //
    //  1. api-daemon should update the hint file before sending
    //     incremental changes to b2gkillerd through the socket.
    //
    //  2. On the other hand, the b2gkillerd should read the hint file
    //     after creating and binding the socket and before handling
    //     any incremental chagnes.
    //
    // With this protocol, the consistency of b2gkillered is assured.
    fn update_hint_file(&self) {
        let hints = self.hints.iter().join(" ");
        let tmp: String = format!("{}.tmp", self.config.hints_path);
        let mut f = match File::create(&tmp) {
            Err(_) => { panic!("can not create a hint file"); },
            Ok(f) => f
        };
        if f.write_all(hints.as_bytes()).is_err() {
            panic!("can not write to the hint file.");
        }

        if rename(&tmp, &self.config.hints_path).is_err() {
            panic!("can not rename tmp hint file to a real hint file.");
        }
    }

    // Tell b2gkillerd to reset/clean hints.
    fn reset_b2gkillerd_hints(&self) {
        let socket = UnixDatagram::unbound()
            .expect("Fail to create a socket for b2gkiller_hints");
        if let Err(_err) = socket.connect(&self.config.socket_path) {
            // b2gkillerd may be not ready!
            debug!("Fail to connect to b2gkiller_hints");
            return;
        }

        if let Err(_err) = socket.send(b"reset") {
            error!("Failed to send a message to b2gkiller_hints");
        }
    }

    // Send incremental changes to b2gkillerd
    fn send_incremental_changes(&self, hints: &[String]) {
        let hints_line = hints.iter().join(" ");

        let socket = UnixDatagram::unbound()
            .expect("Fail to create a socket for b2gkiller_hints");
        if let Err(_err) = socket.connect(&self.config.socket_path) {
            // b2gkillerd may be not ready!
            debug!("Fail to connect to b2gkiller_hints");
            return;
        }

        if let Err(_err) = socket.send(format!("modify {}", hints_line).as_bytes()) {
            error!("Failed to send a message to b2gkiller_hints");
        }
    }

    // Initiialize after setting config.
    pub fn after_config(&self) {
        // Clear the hint file to make sure b2gkillerd loading correct
        // hints if b2gkillerd is not ready yet.
        self.update_hint_file();
        // Reset hints at b2gkillerd that is already there.
        self.reset_b2gkillerd_hints();
    }
}

pub struct ProcessSharedData {
    pub cgservice: CGService,
    // Maintain the list of current hints, so that we provide the list
    // to the client, b2g, without b2gkillerd.
    pub hints: HintManager,
}

impl From<&EmptyConfig> for ProcessSharedData {
    fn from(_config: &EmptyConfig) -> Self {
        let mut shared = ProcessSharedData {
            cgservice: CGService::default(),
            hints: HintManager::new(),
        };
        let genid = shared.cgservice.get_active();
        let genid = shared
            .cgservice
            .begin(genid, String::from("shared_state"))
            .unwrap();
        shared.cgservice.add_group(genid, "fg", "<<root>>").unwrap();
        shared.cgservice.add_group(genid, "bg", "<<root>>").unwrap();
        shared
            .cgservice
            .add_group(genid, "try_to_keep", "bg")
            .unwrap();
        shared.cgservice.commit_noop(genid).unwrap();

        shared
    }
}

impl StateLogger for ProcessSharedData {
    fn log(&self) {
        self.cgservice.log();
    }
}

pub struct ProcManagerService {
    genid: GenID,
    shared_state: Shared<ProcessSharedData>,
    proc_removings: Vec<i32>,
    proc_movings: Vec<(i32, String)>,
}

impl ProcManagerService {
    fn clear_generation(&mut self) {
        self.genid = 0;
        self.proc_removings.clear();
        self.proc_movings.clear();
    }
}

impl From<GroupType> for String {
    fn from(g: GroupType) -> String {
        match g {
            GroupType::Foreground => String::from("fg"),
            GroupType::Background => String::from("bg"),
            GroupType::TryToKeep => String::from("try_to_keep"),
        }
    }
}

impl ProcManager for ProcManagerService {}

impl Drop for ProcManagerService {
    fn drop(&mut self) {
        if self.genid > 0 {
            // Without this, the Groups object will leak.
            let mut state = self.shared_state.lock();
            let cgservice = &mut state.cgservice;
            cgservice.rollback(self.genid).unwrap();
        }
    }
}

impl ProcessServiceMethods for ProcManagerService {
    fn reset(&mut self, responder: ProcessServiceResetResponder) {
        if self.genid == 0 {
            debug!("No generation ID");
            responder.reject();
            return;
        }

        let mut state = self.shared_state.lock();
        let cgservice = &mut state.cgservice;
        match cgservice.all_processes(self.genid) {
            Ok(pids) => {
                if let Err(err) =
                    cgservice.move_processes(self.genid, pids, Vec::<(i32, String)>::new())
                {
                    error!("Reset error in move_processes: {}", err);
                }
            }
            Err(msg) => {
                debug!("Error: {}", msg);
                responder.reject();
            }
        }

        responder.resolve();
    }

    fn begin(&mut self, responder: ProcessServiceBeginResponder, caller: String) {
        if self.genid != 0 {
            debug!("Begin a new generation while there is one");
            responder.reject();
            return;
        }

        let mut state = self.shared_state.lock();
        let cgservice = &mut state.cgservice;
        let cur = cgservice.get_active();
        match cgservice.begin(cur, caller) {
            Ok(genid) => {
                self.genid = genid;
                responder.resolve();
            }
            Err(msg) => {
                debug!("Error: {}", msg);
                responder.reject();
            }
        }
    }
    fn commit(&mut self, responder: ProcessServiceCommitResponder) {
        if self.genid == 0 {
            debug!("No generation ID");
            responder.reject();
            return;
        }

        // Without clone(), lock() will create a mutable reference
        // that cause a confliction of multiple mutable references
        // with passing self to commit_apply().
        let shared_state = self.shared_state.clone();

        let mut state = shared_state.lock();
        let cgservice = &mut state.cgservice;
        if let Err(msg) = cgservice.move_processes(
            self.genid,
            self.proc_removings.clone(),
            self.proc_movings.clone(),
        ) {
            debug!("Error: {}", msg);
            responder.reject();
            return;
        }

        match cgservice.commit(self.genid) {
            Ok(genid) => {
                if genid != self.genid {
                    panic!("Wrong generation ID");
                }
                responder.resolve();
            }
            Err(msg) => {
                debug!("Error: {}", msg);
                responder.reject();
            }
        }
        self.clear_generation();
    }

    fn abort(&mut self, responder: ProcessServiceAbortResponder) {
        if self.genid == 0 {
            debug!("No generation ID");
            responder.reject();
            return;
        }

        {
            let mut state = self.shared_state.lock();
            let cgservice = &mut state.cgservice;
            match cgservice.rollback(self.genid) {
                Ok(()) => {
                    responder.resolve();
                }
                Err(msg) => {
                    debug!("Error: {}", msg);
                    responder.reject();
                }
            }
        }
        self.clear_generation();
    }

    fn add(&mut self, responder: ProcessServiceAddResponder, pid: i64, group: GroupType) {
        if self.genid == 0 {
            debug!("No generation ID");
            responder.reject();
            return;
        }
        if self.proc_removings.contains(&(pid as i32)) {
            debug!("Should not add & remove a process at the same generation");
            responder.reject();
            return;
        }
        let group_name = group.into();
        self.proc_movings.push((pid as i32, group_name));
        responder.resolve(true);
    }

    fn remove(&mut self, responder: ProcessServiceRemoveResponder, pid: i64) {
        if self.genid == 0 {
            debug!("No generation ID");
            responder.reject();
            return;
        }
        if !self.proc_movings.is_empty() {
            debug!("remove() should be called before calling add().");
            responder.reject();
            return;
        }
        self.proc_removings.push(pid as i32);
        responder.resolve(true);
    }

    fn pending(&mut self, responder: ProcessServicePendingResponder) {
        responder.resolve(self.genid != 0);
    }

    // Add and remove hints.
    //
    // |hints| is a string in the format of the following example.
    //
    //    +hint1 -hint2 +hint3
    //
    // This means to add "hint1" and "hint3" but remove "hint2".
    //
    // The hint file contains the list of current hints.  It will
    // always be updated to the latest version by api-daemon. When
    // b2gkillerd is loaded, it reads the list of current hints from
    // the hint file.
    //
    // Incremental changes will be sent through a unix socket to
    // b2gkillerd to notify it what are added or removed.
    fn modify_hints(&mut self, responder: ProcessServiceModifyHintsResponder, hints: Vec<String>) {
        let mut state = self.shared_state.lock();
        let hintsmgr = &mut state.hints;
        if hintsmgr.process_hints(&hints) {
            hintsmgr.update_hint_file();
            hintsmgr.send_incremental_changes(&hints);
        } else {
            responder.reject();
        }
    }

    fn reset_hints(&mut self, _responder: ProcessServiceResetHintsResponder) {
        let mut state = self.shared_state.lock();
        let hintsmgr = &mut state.hints;
        hintsmgr.hints.clear();
        hintsmgr.update_hint_file();
        hintsmgr.reset_b2gkillerd_hints();
    }

    fn hints(&mut self, responder: ProcessServiceHintsResponder) {
        let mut state = self.shared_state.lock();
        let hintsmgr = &mut state.hints;
        let hints = hintsmgr.hints.iter().join(" ");

        responder.resolve(hints);
    }
}

common::impl_shared_state!(ProcManagerService, ProcessSharedData, EmptyConfig);

impl Service<ProcManagerService> for ProcManagerService {
    fn create(
        _attrs: &OriginAttributes,
        _context: SharedSessionContext,
        _helper: SessionSupport,
    ) -> Result<ProcManagerService, String> {
        debug!("ProcManagerService::create");
        let service = ProcManagerService {
            genid: 0,
            shared_state: Self::shared_state(),
            proc_removings: vec![],
            proc_movings: vec![],
        };

        Ok(service)
    }

    fn format_request(&mut self, _transport: &SessionSupport, message: &BaseMessage) -> String {
        let req: Result<ProcManagerFromClient, common::BincodeError> =
            common::deserialize_bincode(&message.content);
        match req {
            Ok(req) => format!("ProcManagerService request: {:?}", req),
            Err(err) => format!("Unable to format ProcManagerService request: {:?}", err),
        }
    }

    // Processes a request coming from the Session.
    fn on_request(&mut self, transport: &SessionSupport, message: &BaseMessage) {
        self.dispatch_request(transport, message);
    }

    fn release_object(&mut self, object_id: u32) -> bool {
        debug!("releasing object {}", object_id);
        true
    }
}

#[cfg(test)]
mod tests {
    use std::iter::FromIterator;
    fn to_strvec(v: &String) -> Vec<String> {
        Vec::from_iter(v.split_whitespace().map(|x| String::from(x)))
    }

    #[test]
    fn process_hints() {
        let mut hints = super::HintManager::new();
        hints.process_hints(&to_strvec(&String::from("+hint1 +hint2 +hint3")));
        let mut hint123 = super::HashSet::<String>::new();
        hint123.insert(String::from("hint1"));
        hint123.insert(String::from("hint2"));
        hint123.insert(String::from("hint3"));
        assert_eq!(hints.hints, hint123);
        hints.process_hints(&to_strvec(&String::from("+hint1 -hint2 +hint4")));
        let mut hint134 = super::HashSet::<String>::new();
        hint134.insert(String::from("hint1"));
        hint134.insert(String::from("hint3"));
        hint134.insert(String::from("hint4"));
        assert_eq!(hints.hints, hint134);

        hints.process_hints(&to_strvec(&String::from("+hint5 -hint1")));
        let mut hint345 = hint134.clone();
        hint345.insert(String::from("hint5"));
        hint345.remove(&String::from("hint1"));
        assert_eq!(hints.hints, hint345);
    }
}
