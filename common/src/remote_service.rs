//! Manages remote services.

use crate::core::{BaseMessage, GetServiceResponse};
use crate::remote_services_registrar::RemoteServicesRegistrar;
#[cfg(target_os = "android")]
use crate::selinux::SeLinux;
use crate::socket_pair::{PairedStream, SocketPair};
use crate::traits::*;
use bincode::Options;
use log::{debug, error, info};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::convert::TryInto;
use std::fs::read_dir;
use std::io::{self, BufReader, BufWriter};
use std::os::unix::io::RawFd;
use std::os::unix::process::{CommandExt, ExitStatusExt};
use std::path::PathBuf;
use std::process::{Child, Command};
use std::sync::mpsc::{channel, Sender};
use std::thread;
#[cfg(target_os = "android")]
extern crate android_utils;

// The size of the buffer used to improve IPC performance.
pub static IPC_BUFFER_SIZE: usize = 128 * 1024;

// Tries to close a file descriptor if it's not whitelisted or a standard input/output one.
#[inline(always)]
fn try_close_fd(fd: libc::c_int, ipc_fd: RawFd) {
    if fd == libc::STDIN_FILENO
        || fd == libc::STDOUT_FILENO
        || fd == libc::STDERR_FILENO
        || fd == ipc_fd
    {
        info!("Not closing fd #{}", fd);
        return;
    }

    // Since we're just trying to close any fd we can find, ignore any error return values of close().
    unsafe {
        libc::close(fd);
    }
}

// Close file descriptors that we don't want to leak from the parent to the spawned process.
// Based on Gecko's similar needs for IPC at
// https://hg.mozilla.org/releases/mozilla-release/file/aca7f47aa8dbf8e5858f5d1318317b463b7a94b7/ipc/chromium/src/base/process_util_posix.cc#l116
fn close_superflous_fds(ipc_fd: RawFd) -> io::Result<()> {
    #[cfg(not(target_os = "android"))]
    const SYSTEM_DEFAULT_MAX_FD: libc::rlim_t = 8192;
    #[cfg(target_os = "android")]
    const SYSTEM_DEFAULT_MAX_FD: libc::rlim_t = 1024;

    // Get the highest value of a file descriptor in this process.
    let mut fd_max = unsafe {
        let mut result = libc::rlimit {
            rlim_cur: 0,
            rlim_max: 0,
        };
        if libc::getrlimit(libc::RLIMIT_NOFILE, &mut result) != 0 {
            error!("getrlimit failed");
            SYSTEM_DEFAULT_MAX_FD
        } else {
            result.rlim_cur
        }
    };

    // We actually know we won't unwrap because libc::INT_MAX is > 0.
    let int_max: libc::rlim_t = libc::INT_MAX.try_into().unwrap_or(0);
    if fd_max > int_max {
        fd_max = int_max;
    }

    info!("fd_max is {}", fd_max);

    let pid = unsafe { libc::getpid() };
    let pid_path_buf = PathBuf::from(format!("/proc/{}/fd", pid));

    // TODO: adjust this path for OS X
    match read_dir("/proc/self/fd") {
        Ok(dir) => {
            for item in dir {
                let entry = item?;
                let path = entry.path();
                // In child fds, there's a "fd" links to /proc/pid/fd folder
                // We should not close this "fd" to avoid panic like:
                // panicked at 'assertion failed: output.write(&bytes).is_ok()',
                // src/libstd/sys/unix/process/process_unix.rs:64:21
                if let Ok(link) = path.read_link() {
                    if link == pid_path_buf {
                        debug!("{:?}, Found, skip", link);
                        continue;
                    }
                }
                let fd: libc::c_int = entry
                    .file_name()
                    .into_string()
                    .unwrap_or_else(|_| "0".into())
                    .parse()
                    .unwrap_or(0);
                try_close_fd(fd, ipc_fd);
            }
        }
        Err(err) => {
            error!("Failed to open /proc/self/fd : {}", err);
            // Fallback to iterating on all possible fds.
            for fd in 0..fd_max {
                try_close_fd(fd as libc::c_int, ipc_fd);
            }
        }
    }

    Ok(())
}

// If the FD_CLOEXEC flag is set on the fd we need in the child, remove it.
fn unset_fd_cloexec(fd: RawFd) -> bool {
    use nix::fcntl;
    let res = fcntl::fcntl(fd, fcntl::FcntlArg::F_GETFD).unwrap();
    let mut flags = fcntl::FdFlag::from_bits(res).unwrap();

    if flags.contains(fcntl::FdFlag::FD_CLOEXEC) {
        // remove the FD_CLOEXEC flag.
        flags.remove(fcntl::FdFlag::FD_CLOEXEC);
        if let Ok(res) = fcntl::fcntl(fd, fcntl::FcntlArg::F_SETFD(flags)) {
            return res == 0;
        } else {
            return false;
        }
    }

    true
}

// Spawns a new daemon process for a given service.
// The process executable is expected to be named root_path/${service name}/daemon
fn spawn_child(
    service: &str,
    sockets: &SocketPair,
    root_path: &str,
    registrar: &RemoteServicesRegistrar,
) -> Option<Child> {
    if !unset_fd_cloexec(sockets.1.raw_fd()) {
        error!("Failed to unset the FD_CLOEXEC flag on the child fd, aborting.");
        return None;
    }

    let uid = registrar.id_for(service);
    if uid.is_none() {
        error!("Failed to get uid/gid for {} service, aborting.", service);
        return None;
    }

    let exec = format!("{}/{}/daemon", root_path, service);
    info!("Launching child daemon at {}", exec);

    let child_ipc_fd = sockets.1.raw_fd();

    #[cfg(target_os = "android")]
    let command = unsafe {
        let uid: u32 = uid.unwrap().into();
        let service = service.to_owned();
        let root_path = root_path.to_owned();

        Command::new(&exec)
            .env("IPC_FD", format!("{}", child_ipc_fd))
            .env("LD_LIBRARY_PATH", format!("{}/{}", &root_path, &service))
            .pre_exec(move || {
                use nix::unistd::{setgid, setgroups, setuid, Gid, Uid};

                // - change to context to use one specific to this daemon.
                info!("SeLinux mode: {:?}", SeLinux::getenforce());
                let context_set = SeLinux::setcon(&format!("u:r:child-daemon:s0"));
                info!(
                    "setting new (u:r:child-daemon:s0) context - {}",
                    if context_set { "success" } else { "failure" }
                );

                // Close file descriptors that we don't need.
                let _ = close_superflous_fds(child_ipc_fd);

                // set supplementary groups to access network and media
                let audio = Gid::from_raw(1005);
                let inet = Gid::from_raw(3003);
                let system = Gid::from_raw(1000);
                let _ = setgroups(&[audio, inet, system]);

                // set uid gid for the process
                let _ = setgid(Gid::from_raw(uid));
                let _ = setuid(Uid::from_raw(uid));

                Ok(())
            })
            .spawn()
    };

    #[cfg(not(target_os = "android"))]
    let command = unsafe {
        Command::new(&exec)
            .env("IPC_FD", format!("{}", sockets.1.raw_fd()))
            .env("LD_LIBRARY_PATH", format!("{}/{}", root_path, service))
            .pre_exec(move || close_superflous_fds(child_ipc_fd))
            .spawn()
    };

    match command {
        Ok(child) => {
            info!("Done with spawning child for {}", service);
            Some(child)
        }
        Err(err) => {
            error!(
                "Failed to spawn process {} for service {}: {}",
                exec, service, err
            );
            None
        }
    }
}

// The remote service manager keeps track of where to route data
// target at remote services.
// Each service name is bound to a socket pair matching a process.
// The manager spwans threads to get incoming messages and route them
// to the appropriate RemoteService.
pub struct RemoteServiceManager {
    services: Shared<HashMap<String, LockedIpcWriter>>, // Maps a service name to a transport handle.
    root_path: String,
    upstream_senders: Shared<HashMap<u32, MessageSender>>,
    rpc_sender: Shared<Option<Sender<ChildToParentMessage>>>,
    pub registrar: RemoteServicesRegistrar,
}

pub type SharedRemoteServiceManager = Shared<RemoteServiceManager>;

// Returns the value if we can unwrap, or call `continue` otherwise.
#[macro_export]
macro_rules! try_continue {
    ($what:expr) => {
        match $what {
            Ok(res) => res,
            Err(err) => {
                error!("Error: {}", err);
                continue;
            }
        }
    };
}

macro_rules! try_continue_opt {
    ($what:expr) => {
        if let Some(res) = $what {
            res
        } else {
            continue;
        }
    };
}

// Wrapper around the socket pair used to send messages to the parent.
// It ensures that multiple threads can't write to the socket at the same time
// by owning the stream and managing the locking itself.
#[derive(Clone)]
pub struct LockedIpcWriter {
    stream: Shared<PairedStream>,
}

impl LockedIpcWriter {
    pub fn new(stream: PairedStream) -> Self {
        Self {
            stream: Shared::adopt(stream),
        }
    }

    pub fn serialize<T: ?Sized>(&self, value: &T) -> Result<(), bincode::Error>
    where
        T: serde::Serialize,
    {
        let buffer = BufWriter::new(self.stream.lock().clone());
        bincode::serialize_into(buffer, &value)
    }
}

impl RemoteServiceManager {
    // Start to relay messages from the child to the session client.
    pub fn start(&self, mut child: Child, pair: &SocketPair, service_name: &str) {
        debug!("Start parent listener for remote:{}", service_name);
        let handle = pair.0.clone();
        let remote = pair.1.clone();

        let upstream_senders = self.upstream_senders.clone();
        let upstream_senders2 = self.upstream_senders.clone();

        let service_name = service_name.to_string();
        // TODO: something not that ridiculous.
        let service_name2 = service_name.clone();
        let service_name2_2 = service_name.clone();
        let service_name3 = service_name.clone();
        let service_name4 = service_name.clone();

        let rpc_sender = self.rpc_sender.clone();
        let services = self.services.clone();
        // We can't clone child to use child.kill() in a different thread,
        // so we use the raw pid instead.
        let child_pid = child.id();

        // Watchdog thread that monitors the livelyness of the child daemon.
        // Not calling wait() on the child causes it to go zombie if it dies.
        thread::Builder::new()
            .name(format!(
                "RemoteServiceManager-child-watchdog-{}",
                service_name
            ))
            .spawn(move || {
                let exit_code = match child.wait() {
                    Ok(status) => {
                        info!(
                            "child daemon for `{}` service exited with status {}",
                            &service_name4, status
                        );
                        let mut code = status.code().unwrap_or(-1);
                        if code == -1 {
                            // try to get the signal number if it's available.
                            // If so, return -1XX for XX
                            if let Some(signal) = status.signal() {
                                code = -100 - signal;
                            }
                        }
                        code
                    }
                    Err(err) => {
                        error!(
                            "child daemon for `{}` service died with error: {}",
                            &service_name4, err
                        );
                        -2
                    }
                };

                // The child exited. We can't really recover from that since we don't
                // know if it's holding state, so we send an event to the service
                // to notify that it's not usable anymore. The client is responsible for
                // trying to spin it again.

                // Notify all the ws sessions that this child has died so all RemoteServices
                // need to be cleaned up.
                services.lock().remove(&service_name4);
                let upstream_sessions = upstream_senders2.lock();
                for session in &(*upstream_sessions) {
                    let sender = session.1;
                    sender.send_raw_message(MessageKind::ChildDaemonCrash(
                        service_name4.clone(),
                        exit_code,
                        child_pid,
                    ));
                }

                info!("Exiting watchdog thread for `{}`", &service_name4);
                // Notify the ipc thread to stop itself.
                let buffer = BufWriter::new(remote);
                let _ = bincode::serialize_into(buffer, &ChildToParentMessage::Stop);
            })
            .unwrap_or_else(|_| {
                panic!(
                    "Failed to create RemoteServiceManager-child-watchdog-{} thread",
                    service_name3
                );
            });

        thread::Builder::new()
            .name(format!("RemoteServiceManager-ipc-{}", service_name))
            .spawn(move || {
                loop {
                    debug!(
                        "RemoteServiceManager Waiting for ChildToParentMessage for {}",
                        service_name
                    );
                    let reader = BufReader::with_capacity(IPC_BUFFER_SIZE, handle.clone());
                    let response: ChildToParentMessage = match bincode::deserialize_from(reader) {
                        Ok(res) => res,
                        Err(err) => {
                            use nix::sys::signal::{kill, Signal};

                            error!("Error decoding child message: {}", err);
                            // If we fail to decode, there is no good way to recover so instead we just
                            // kill the child. Our watchdog thread will then do the cleanup.
                            let _ =
                                kill(nix::unistd::Pid::from_raw(child_pid as _), Signal::SIGKILL);
                            // Exit this thread.
                            break;
                        }
                    };
                    match response {
                        ChildToParentMessage::Packet(tracker_id, buffer) => {
                            debug!("RemoteService got packet from child, len={}", buffer.len());
                            let lock = upstream_senders.lock();
                            debug!("Tracking {} WS sessions", lock.len());

                            let sender = try_continue_opt!(lock.get(&tracker_id.session()));
                            sender.send_raw_message(MessageKind::Data(tracker_id, buffer));
                        }
                        ChildToParentMessage::Stop => {
                            info!(
                                "Stopping relaying thread for RemoteService-{}",
                                service_name
                            );
                            // Notify that we are stopping, so for instance we don't
                            // deadlock when crashing during service creation.
                            let mut lock = rpc_sender.lock();
                            if let Some(ref rpc_sender) = *lock {
                                try_continue!(rpc_sender.send(ChildToParentMessage::Stop));
                                // We can use the rpc_sender only once.
                                *lock = None;
                            }
                            break;
                        }
                        ChildToParentMessage::Created(tracker_id, result) => {
                            let mut lock = rpc_sender.lock();
                            if let Some(ref rpc_sender) = *lock {
                                try_continue!(rpc_sender
                                    .send(ChildToParentMessage::Created(tracker_id, result)));
                                // We can use the rpc_sender only once.
                                *lock = None;
                            } else {
                                error!("No rpc sender available to process Created message");
                            }
                        }
                        ChildToParentMessage::ObjectReleased(tracker_id, result) => {
                            let mut lock = rpc_sender.lock();
                            if let Some(ref rpc_sender) = *lock {
                                try_continue!(rpc_sender.send(
                                    ChildToParentMessage::ObjectReleased(tracker_id, result)
                                ));
                                // We can use the rpc_sender only once.
                                *lock = None;
                            } else {
                                error!("No rpc sender available to process ObjectReleased message");
                            }
                        }
                    }
                }
                info!("Exiting ipc message thread for `{}`", &service_name2_2);
            })
            .unwrap_or_else(|_| {
                panic!(
                    "Failed to create RemoteServiceManager-ipc-{} thread",
                    service_name2
                );
            });
    }

    pub fn new(root_path: &str, registrar: RemoteServicesRegistrar) -> Self {
        RemoteServiceManager {
            services: Shared::<_>::default(),
            root_path: root_path.into(),
            upstream_senders: Shared::<_>::default(),
            rpc_sender: Shared::<_>::default(),
            registrar,
        }
    }

    pub fn set_rpc_sender(&mut self, sender: Sender<ChildToParentMessage>) {
        let mut data = self.rpc_sender.lock();
        *data = Some(sender);
    }

    // Returns the existing handle if possible, or spawns and out of
    // process daemon if needed.
    pub fn ensure(&mut self, service_name: &str) -> Option<LockedIpcWriter> {
        info!(
            "RemoteChildManager ensure child daemon for `{}` is available.",
            service_name
        );
        let mut lock = self.services.lock();
        match lock.get(service_name) {
            Some(writer) => {
                info!("Already running, will use existing daemon");
                Some(writer.clone())
            }
            None => {
                // Create a new socket pair for this remote service.
                if let Ok(pair) = SocketPair::new() {
                    // Spawn a new process.
                    if let Some(child) =
                        spawn_child(service_name, &pair, &self.root_path, &self.registrar)
                    {
                        info!("Using a remote daemon for {}", service_name);
                        if let Err(err) = android_utils::AndroidProperties::set(
                            "kaios.media-daemon.pid",
                            &child.id().to_string(),
                        ) {
                            error!("Failed to set kaios.media-daemon.pid, error: {:?}", err);
                        }
                        let writer = LockedIpcWriter::new(pair.0.clone());
                        lock.insert(service_name.into(), writer.clone());
                        self.start(child, &pair, service_name);
                        return Some(writer);
                    }
                }

                None
            }
        }
    }

    pub fn enable_event(&self, session_id: u32, object_id: u32, event_id: u32) {
        debug!(
            "RemoteServiceManager enable event session_id #{} object_id #{} event_id #{}",
            session_id, object_id, event_id
        );
        for service in &(*self.services.lock()) {
            debug!(
                "-> service {} enable event session_id #{} object_id #{} event_id #{}",
                service.0, session_id, object_id, event_id
            );
            let msg = ParentToChildMessage::EnableEvent(session_id, object_id, event_id);
            if let Err(err) = service.1.serialize(&msg) {
                error!("Failed to serialize enable_event: {}", err);
            }
        }
    }

    pub fn disable_event(&self, session_id: u32, object_id: u32, event_id: u32) {
        for service in &(*self.services.lock()) {
            let msg = ParentToChildMessage::DisableEvent(session_id, object_id, event_id);
            if let Err(err) = service.1.serialize(&msg) {
                error!("Failed to serialize disable_event: {}", err);
            }
        }
    }

    // Track an upstream (WS or Unix Socket) session.
    pub fn add_upstream_session(&mut self, session_id: u32, sender: MessageSender) {
        let mut senders = self.upstream_senders.lock();
        senders.insert(session_id, sender);
        debug!(
            "add_upstream_session: Now tracking {} sessions",
            senders.len()
        );
    }

    // Untrack an upstream (WS or Unix Socket) session.
    pub fn remove_upstream_session(&mut self, session_id: u32) {
        let mut senders = self.upstream_senders.lock();
        senders.remove(&session_id);
        debug!(
            "remove_upstream_session: Now tracking {} sessions",
            senders.len()
        );
    }
}

// The shim used in the main process to relay messages
// to the out of process services.
pub struct RemoteService {
    id: SessionTrackerId,
    service_name: String, // The name of the "real" service
    handle: LockedIpcWriter,
    manager: SharedRemoteServiceManager,
}

crate::impl_shared_state!(RemoteService, EmptyState, EmptyConfig);

impl Service<RemoteService> for RemoteService {
    fn on_request(&mut self, transport: &SessionSupport, message: &BaseMessage) {
        // Relay the request to the child.
        match crate::get_bincode().serialize(message) {
            Ok(buffer) => {
                let msg = ParentToChildMessage::Request(transport.session_id(), buffer);
                if let Err(err) = self.handle.serialize(&msg) {
                    error!("Failed to serialize request: {}", err);
                }
            }
            Err(err) => {
                error!("Failed to serialize message: {:?}", err);
            }
        }
    }

    /// Called when we need a human readable representation of the request.
    fn format_request(&mut self, _transport: &SessionSupport, _message: &BaseMessage) -> String {
        // TODO: Relay to the child
        "RemoteService request".into()
    }

    fn release_object(&mut self, object_id: u32) -> bool {
        let msg = ParentToChildMessage::ReleaseObject(self.id, object_id);
        // Use the manager to wait for the child response.
        let (rpc_sender, rpc_receiver) = channel();
        self.manager.lock().set_rpc_sender(rpc_sender);
        if let Err(err) = self.handle.serialize(&msg) {
            error!("Failed to serialize release_object: {}", err);
        }
        let response: ChildToParentMessage = rpc_receiver.recv().unwrap();
        match response {
            ChildToParentMessage::ObjectReleased(_tracker_id, result) => result,
            _ => {
                error!("Unexpected message releasing object {}", object_id);
                false
            }
        }
    }

    fn create_remote(
        id: SessionTrackerId,
        origin_attributes: &OriginAttributes,
        _context: SharedSessionContext,
        manager: SharedRemoteServiceManager,
        service_name: &str,
        service_fingerprint: &str,
    ) -> Result<Self, String> {
        let manager2 = manager.clone();
        let mut manager = manager.lock();
        match manager.ensure(service_name) {
            Some(writer) => {
                // Send the parameters to the child to create the service and
                // returns based on the status of this call.
                // We need to block here because making this async would force us
                // to make create_remote() itself async.
                let msg = ParentToChildMessage::CreateService(
                    service_name.into(),
                    service_fingerprint.into(),
                    id,
                    origin_attributes.clone(),
                );

                // Use the manager to wait for the child response.
                let (rpc_sender, rpc_receiver) = channel();
                manager.set_rpc_sender(rpc_sender);
                if let Err(err) = writer.serialize(&msg) {
                    error!("Failed to serialize CreateService: ${}", err);
                }

                let response: ChildToParentMessage = rpc_receiver.recv().unwrap();
                match response {
                    ChildToParentMessage::Created(_tracker_id, response) => {
                        info!("Creating remote service result: {:?}", response);
                        match response {
                            GetServiceResponse::Success(_) => {
                                let service = RemoteService {
                                    id,
                                    handle: writer,
                                    service_name: service_name.into(),
                                    manager: manager2,
                                };
                                Ok(service)
                            }
                            GetServiceResponse::InternalError(msg) => Err(msg),
                            other => Err(format!("Remote service creation failure: {:?}", other)),
                        }
                    }
                    _ => {
                        error!("Unexpected message creating remote {}", service_name);
                        Err(format!(
                            "Unexpected message creating remote {}",
                            service_name
                        ))
                    }
                }
            }
            None => Err(format!("Failed to spawn remote service {}", service_name)),
        }
    }
}

impl Drop for RemoteService {
    fn drop(&mut self) {
        info!(
            "Dropping RemoteService #{:?} : {}",
            self.id, self.service_name
        );
        let msg = ParentToChildMessage::ReleaseService(self.service_name.clone(), self.id);
        if let Err(err) = self.handle.serialize(&msg) {
            error!("Failed to serialize ReleaseService: {}", err);
        }
    }
}

// The set of messages that are exchanged between the parent and child daemons.

#[derive(Serialize, Deserialize)]
pub enum ParentToChildMessage {
    CreateService(String, String, SessionTrackerId, OriginAttributes), // TODO: add SessionContext
    ReleaseService(String, SessionTrackerId),                          // service name, service id
    Request(u32, Vec<u8>), // session id, serialized protobuf -> vec<u8> of BaseMessage
    EnableEvent(u32, u32, u32), // session_id, object_id, event_id
    DisableEvent(u32, u32, u32), // session_id, object_id, event_id
    ReleaseObject(SessionTrackerId, u32), // service_id, object_id
}

#[derive(Serialize, Deserialize)]
pub enum ChildToParentMessage {
    Created(SessionTrackerId, GetServiceResponse), // Whether we successfully instatiated a service.
    Packet(SessionTrackerId, Vec<u8>),             // A generic encoded protobuf.
    ObjectReleased(SessionTrackerId, bool),        // Whether we successfully released an object.
    Stop,                                          // Forces the connection to close.
}
