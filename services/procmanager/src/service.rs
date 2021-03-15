use crate::cgroups::*;
use crate::generated::common::*;
use crate::generated::service::*;
use common::core::BaseMessage;
use log::{debug, error};
use std::convert::Into;

use common::traits::{
    OriginAttributes, Service, SessionSupport, Shared, SharedSessionContext, StateLogger,
};

pub struct ProcessSharedData {
    pub cgservice: CGService,
}

impl StateLogger for ProcessSharedData {
    fn log(&self) {
        self.cgservice.log();
    }
}

pub struct ProcManagerService {
    genid: GenID,
    shared_state: Shared<<ProcManagerService as Service<ProcManagerService>>::State>,
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

impl Into<String> for GroupType {
    fn into(self) -> String {
        match self {
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
    fn reset(&mut self, responder: &ProcessServiceResetResponder) {
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

    fn begin(&mut self, responder: &ProcessServiceBeginResponder, caller: String) {
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
    fn commit(&mut self, responder: &ProcessServiceCommitResponder) {
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

    fn abort(&mut self, responder: &ProcessServiceAbortResponder) {
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

    fn add(&mut self, responder: &ProcessServiceAddResponder, pid: i64, group: GroupType) {
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

    fn remove(&mut self, responder: &ProcessServiceRemoveResponder, pid: i64) {
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

    fn pending(&mut self, responder: &ProcessServicePendingResponder) {
        responder.resolve(self.genid != 0);
    }
}

impl Service<ProcManagerService> for ProcManagerService {
    type State = ProcessSharedData;

    fn shared_state() -> Shared<Self::State> {
        let mut shared = ProcessSharedData {
            cgservice: CGService::default(),
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
        Shared::adopt(shared)
    }

    fn create(
        _attrs: &OriginAttributes,
        _context: SharedSessionContext,
        shared_obj: Shared<Self::State>,
        _helper: SessionSupport,
    ) -> Result<ProcManagerService, String> {
        debug!("ProcManagerService::create");
        let service = ProcManagerService {
            genid: 0,
            shared_state: shared_obj,
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
