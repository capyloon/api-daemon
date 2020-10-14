use std::collections::{HashMap, HashSet};

use std::cmp::{Ord, Ordering};
use std::fs::{read_dir, File, OpenOptions};
use std::io::Write;
use std::path::Path;
use thiserror::Error;

use log::{debug, error};

const CGROUP_MEM: &str = "/dev/memcg/b2g";

pub type GenID = u64;

//
// Interactions of Classes:
//
// '''
//              uncommitted_groups/
//              groups                        <<tree>>
//   CGService ------------> Groups{genid} --------------> Group
//       \                      |
//        \                     |
//         \                    v
//          +--------> GenerationWorker
// '''
//
// CGService:
//
//   It is a public interface to clients.  Clients access a |groups|
//   with its |genid|, generation ID.  Clients known only generation
//   ID and process ID to move processes among groups.
//
//   There is always exactly one active generation, a |groups|,
//   anytime.  Other |groups|s are uncommitted.  Uncommitted ones can
//   replace the current active one by committing itself.
//
// Groups:
//
//   A |Groups| is associated with a generation.  Generations are
//   always created from an existing generation.  Only the generation
//   that was created from the current active generation can be
//   committed.  Therefore, at most one of genrations that were
//   created from the same one can be committed, and other ones will
//   fail or abort.  It is a very simple way to implement the concept
//   of transactions.
//
// GenerationWorker:
//
//   It provides the service that updates the cgroups filesystem.
//

#[derive(Error, Debug, PartialEq)]
pub enum CGroupError {
    #[error("invalid genration")]
    InvalidGen,
    #[error("unknown group")]
    UnknownGroup,
    #[error("duplicated group name")]
    DupGroup,
    #[error("confliction of generations")]
    ConflictGen,
    #[error("confliction of an attribute: {0}")]
    ConflictAttr(String),
    #[error("phase error")]
    PhaseError,
    #[error("unknown error")]
    Unknown,
}

#[derive(Clone, Eq, Copy)]
enum CGroupsPhase {
    Start = 0,
    GroupRemove,
    GroupAdd,
    Attrs,
    Processes,
}

impl CGroupsPhase {
    fn move_to(&self, target: CGroupsPhase) -> Result<CGroupsPhase, CGroupError> {
        if *self > target {
            Err(CGroupError::PhaseError)
        } else {
            Ok(target)
        }
    }
}

impl PartialOrd for CGroupsPhase {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some((&(*self as i32)).cmp(&(*other as i32)))
    }
}

impl Ord for CGroupsPhase {
    fn cmp(&self, other: &Self) -> Ordering {
        (&(*self as i32)).cmp(&(*other as i32))
    }
}

impl PartialEq for CGroupsPhase {
    fn eq(&self, other: &Self) -> bool {
        (*self as i32) == (*other as i32)
    }
}

//
// Worker called by CGService to apply changes.
//
// This trait should be implemented to apply changes of cgroups file
// system. and should be passed to apply_diff() and commit_apply() of
// CGService.
//
pub trait GenerationWorker {
    fn start_applying(&mut self, _old_gen: GenID, _new_gen: GenID) {}

    fn remove_group(&mut self, group_path: &str) -> Result<(), CGroupError>;

    fn add_group(&mut self, group_name: &str, parent: &str) -> Result<(), CGroupError>;

    fn update_group_attrs(
        &mut self,
        group_path: &str,
        to_set: &mut [(String, String)],
        to_remove: &mut [String],
    ) -> Result<(), CGroupError>;

    fn move_processes(
        &mut self,
        removings: &mut [i32],
        movings: &mut [(i32, String)],
    ) -> Result<(), CGroupError>;
}

fn get_all_pids() -> impl Iterator<Item = i32> {
    read_dir(Path::new("/proc"))
        .expect("fail to read /proc")
        .filter_map(|x| {
            let entry = x.as_ref().unwrap().path();
            if !entry.is_dir() {
                return None;
            }
            let cgroup_path = &(entry.to_str().unwrap())[6..];
            let firstchar = cgroup_path.chars().next().unwrap();
            if !firstchar.is_ascii_digit() {
                return None;
            }
            Some(cgroup_path.parse::<i32>().unwrap())
        })
}

#[cfg(target_os = "android")]
pub struct DefaultCGroupsWorker();

#[cfg(target_os = "android")]
impl DefaultCGroupsWorker {
    fn new() -> DefaultCGroupsWorker {
        DefaultCGroupsWorker()
    }
}

//
// Add & remove processes to cgroups.
//
// We don't support cgroup adding and removing yet.  The assumption is
// all cgroups are pre-created.  Attribute changing are not supported
// yet, too.
//
#[cfg(target_os = "android")]
impl GenerationWorker for DefaultCGroupsWorker {
    fn remove_group(&mut self, _group_path: &str) -> Result<(), CGroupError> {
        // Not support yet!
        Ok(())
    }

    fn add_group(&mut self, _group_name: &str, _parent: &str) -> Result<(), CGroupError> {
        // Not support yet!
        Ok(())
    }

    fn update_group_attrs(
        &mut self,
        _group_path: &str,
        _to_set: &mut [(String, String)],
        _to_remove: &mut [String],
    ) -> Result<(), CGroupError> {
        // Not support yet!
        Ok(())
    }

    fn move_processes(
        &mut self,
        removings: &mut [i32],
        movings: &mut [(i32, String)],
    ) -> Result<(), CGroupError> {
        let pids: HashSet<_> = get_all_pids().collect();

        if !removings.is_empty() {
            let path = format!("{}/cgroup.procs", CGROUP_MEM);
            let mut file = OpenOptions::new()
                .write(true)
                .open(path)
                .expect("fail to open cgroup.procs");
            for pid in removings.iter().filter(|x| pids.contains(&x)) {
                if let Err(err) = write!(file, "{}", pid) {
                    error!("Failed to write to {}/cgroup.procs : {}", CGROUP_MEM, err);
                }
            }
        }

        if movings.is_empty() {
            return Ok(());
        }

        // Sorting by group_paths, an opened file object of a
        // cgroup.procs can be reused by following pids of the same
        // group if there are.
        movings.sort_by(|(_, x_path), (_, y_path)| x_path.cmp(&y_path));

        let unknown_pids = movings.iter().filter(|x| !pids.contains(&x.0));
        for (pid, group_path) in unknown_pids {
            debug!("Fail to add pid={} to {}", pid, group_path);
        }

        let mut last_name_file: Option<(&str, File)> = None;
        let known_pids = movings.iter().filter(|x| pids.contains(&x.0));
        for (pid, group_path) in known_pids {
            debug!("Move {} to {}", pid, group_path);
            if last_name_file.is_none() || !last_name_file.as_ref().unwrap().0.eq(group_path) {
                let path = format!("{}/{}/cgroup.procs", CGROUP_MEM, group_path);
                let file = OpenOptions::new()
                    .write(true)
                    .open(path)
                    .expect("fail to open cgroup.procs");
                last_name_file = Some((group_path, file));
            }
            let file = &mut last_name_file.as_mut().unwrap().1;

            if let Err(err) = write!(file, "{}", pid) {
                error!(
                    "Fail to write pid={} to {}/cgroup.procs : {}",
                    pid, group_path, err
                );
                return self.move_processes(removings, movings);
            }
        }
        Ok(())
    }
}

#[cfg(not(target_os = "android"))]
pub struct DummyCGroupsWorker();

#[cfg(not(target_os = "android"))]
impl DummyCGroupsWorker {
    fn new() -> DummyCGroupsWorker {
        DummyCGroupsWorker()
    }
}

//
// Dummy implementation fo GenerationWorker
//
// Just do nothing as a fallback for the platforms that doesn't
// support CGroup.
//
#[cfg(not(target_os = "android"))]
impl GenerationWorker for DummyCGroupsWorker {
    fn remove_group(&mut self, _group_path: &str) -> Result<(), CGroupError> {
        // Not support yet!
        Ok(())
    }

    fn add_group(&mut self, _group_name: &str, _parent: &str) -> Result<(), CGroupError> {
        // Not support yet!
        Ok(())
    }

    fn update_group_attrs(
        &mut self,
        _group_path: &str,
        _to_set: &mut [(String, String)],
        _to_remove: &mut [String],
    ) -> Result<(), CGroupError> {
        // Not support yet!
        Ok(())
    }

    fn move_processes(
        &mut self,
        removings: &mut [i32],
        movings: &mut [(i32, String)],
    ) -> Result<(), CGroupError> {
        // Not supported!
        Ok(())
    }
}

//
// CGroup Service manages cgroups for processes
//
// All processes are organized as groups, aka struct Groups.  Each
// group, aka struct Group, has been mapped to a cgroup, the hierachy
// of groups is the hierachy of the directory of cgroups filesystem.
// So far, the hierachy is pre-defined.
//
//  - b2g/
//    - fg/
//    - bg/
//      - try_to_keep/
//
// The instances of |GenerationWorker| trait are responsible for the
// actual works of changing cgroups file system.
// |DefaultCGroupsWorker| is a simple implementation of it.  Please
// check |apply_diff()| and |commit_appky()| for details.
//
// Everytime making changes of groups, a new generation should be
// created by creating a copy of groups from an existing generation
// and making changes on the new copy.  Once finish changing, the new
// generation should be committed to activate it and purge the
// previous generation.
//
// To access groups, the user should always give a generation ID, aka
// GenID.  The ID of latest active generation is available by calling
// |get_active()|.  All changes should be made between |begin()| and
// |commit()|.  |begin()| will create a new generation and return its
// ID.  The user should make changes against the returned ID.  This
// pattern allow multiple users to access this service and make
// changes concurrently.  But, for now, there is only one generation
// will success, all others will fail to commit.  Maybe someday, the
// servcie can merge changes from different generations if possible.
//
// PHASES:
//
// All changes must be made in order of phases;
//
//  1. remove_groups(),
//  2. add_groups(),
//  3. update_group_attrs(), then
//  4. move_processes().
//
// Once you make a change of a later phase, you can not make any
// changes of ealier phases anymore.  This restriction can avoid any
// unexpected side-effects caused by the differences/inconsistency
// between/of the order of calling functions and the order of updating
// the cgroups file system.
//
pub struct CGService {
    groups: Groups,
    uncommitted_groups: HashMap<u64, Groups>,
    next_gen_id: u64,
}

impl Default for CGService {
    fn default() -> Self {
        CGService {
            groups: Groups::new(String::from("<empty>")),
            uncommitted_groups: HashMap::<u64, Groups>::new(),
            next_gen_id: 1,
        }
    }
}

impl CGService {
    pub fn get_active(&self) -> GenID {
        self.groups.generation
    }
    pub fn retrieve_groups(&self, generation: GenID) -> Result<&Groups, CGroupError> {
        if self.groups.generation == generation {
            Ok(&self.groups)
        } else {
            match self.uncommitted_groups.get(&generation) {
                Some(groups) => Ok(groups),
                _ => Err(CGroupError::InvalidGen),
            }
        }
    }
    pub fn retrieve_group(
        &self,
        generation: GenID,
        group_name: &str,
    ) -> Result<&Group, CGroupError> {
        match self.retrieve_groups(generation) {
            Ok(groups) => match groups.names2groups.get(group_name) {
                Some(group) => Ok(group),
                _ => Err(CGroupError::UnknownGroup),
            },
            _ => Err(CGroupError::InvalidGen),
        }
    }

    //
    // Create a new generation.
    //
    // A generation is a transaction of changing cgroups.
    //
    // The GenID here must be the ID of current active generation, the
    // one has been committed least recently, or it will fail.  It can
    // make sure client's states are consistent with the API daemon.
    // For example, if a client read info from this service before
    // calling this function, the service would find if the client's
    // information is out-of-date.
    //
    // A new generation is a copy of the current generation.  And, the
    // user should make changes on this copy, and commit or rollback
    // the generation later.  That means it allows two or more clients
    // to make changes on cgroups concurrently, but only one of them
    // will success eventually.
    //
    pub fn begin(&mut self, generation: GenID, caller: String) -> Result<GenID, CGroupError> {
        if self.groups.generation != generation {
            return Err(CGroupError::InvalidGen);
        }
        let mut groups = self.groups.clone();
        groups.creator = caller;
        groups.source_generation = groups.generation;
        let gen_id = self.next_gen_id;
        groups.generation = gen_id;
        self.uncommitted_groups.insert(groups.generation, groups);
        self.next_gen_id += 1;
        Ok(gen_id)
    }

    fn _commit(&mut self, generation: GenID) -> Result<(GenID, Groups), CGroupError> {
        if let Some(mut groups) = self.uncommitted_groups.remove(&generation) {
            if groups.source_generation != self.groups.generation {
                // There is another generation that is derived from
                // the same source generation having committed.
                return Err(CGroupError::ConflictGen);
            }

            groups.phase = CGroupsPhase::Start;
            std::mem::swap(&mut self.groups, &mut groups);
            // GenID and old groups
            Ok((self.groups.generation, groups))
        } else {
            Err(CGroupError::InvalidGen)
        }
    }

    //
    // Activate the generation of the given ID without update cgroups
    // file system.
    //
    pub fn commit_noop(&mut self, generation: GenID) -> Result<GenID, CGroupError> {
        self._commit(generation).map(|(genid, _)| genid)
    }

    //
    // Commit a new generation and apply the changes it made.
    //
    // The new generation will be compared with the current active
    // generation.  The differences will be applied by calling the
    // instance of |GenerationWorker| passed in.
    //
    pub fn commit_apply(
        &mut self,
        generation: GenID,
        worker: &mut dyn GenerationWorker,
    ) -> Result<GenID, CGroupError> {
        match self._commit(generation) {
            Ok((genid, old_groups)) => {
                old_groups.apply_diff(&self.groups, worker);
                Ok(genid)
            }
            Err(msg) => Err(msg),
        }
    }

    //
    // Activate the generation of the given ID.
    //
    // Make the given generation active and purge the previous one.
    // The given generation must be been created from the current
    // active generation by passing it's ID to |begin()|.  If the
    // given generation was not created from the current active
    // generation, it will fail because some other generations have
    // been committed in-between.  Hope, in future, that we can merge
    // changes from several generations.
    //
    pub fn commit(&mut self, generation: GenID) -> Result<GenID, CGroupError> {
        #[cfg(target_os = "android")]
        type WorkerType = DefaultCGroupsWorker;

        #[cfg(not(target_os = "android"))]
        type WorkerType = DummyCGroupsWorker;

        self.commit_apply(generation, &mut WorkerType::new())
    }
    //
    // Drop a generation.
    //
    pub fn rollback(&mut self, generation: GenID) -> Result<(), CGroupError> {
        if let Some(_groups) = self.uncommitted_groups.remove(&generation) {
            Ok(())
        } else {
            Err(CGroupError::InvalidGen)
        }
    }

    pub fn add_group(
        &mut self,
        generation: GenID,
        group_name: &str,
        parent: &str,
    ) -> Result<(), CGroupError> {
        if let Some(groups) = self.uncommitted_groups.get_mut(&generation) {
            groups.add_group(group_name, parent)
        } else {
            Err(CGroupError::InvalidGen)
        }
    }
    pub fn remove_group(&mut self, generation: GenID, group_name: &str) -> Result<(), CGroupError> {
        if let Some(groups) = self.uncommitted_groups.get_mut(&generation) {
            groups.remove_group(group_name)
        } else {
            Err(CGroupError::InvalidGen)
        }
    }

    pub fn update_group_attrs(
        &mut self,
        generation: GenID,
        group_name: &str,
        to_set: Vec<(String, String)>,
        to_remove: Vec<String>,
    ) -> Result<(), CGroupError> {
        if let Some(groups) = self.uncommitted_groups.get_mut(&generation) {
            groups.update_group_attrs(group_name, to_set, to_remove)
        } else {
            Err(CGroupError::InvalidGen)
        }
    }

    pub fn move_processes(
        &mut self,
        generation: GenID,
        removings: Vec<i32>,
        movings: Vec<(i32, String)>,
    ) -> Result<(), CGroupError> {
        if let Some(groups) = self.uncommitted_groups.get_mut(&generation) {
            groups.move_processes(removings, movings)
        } else {
            Err(CGroupError::InvalidGen)
        }
    }

    //
    // Compare this generation with the current generation and apply
    // the differences.
    //
    pub fn apply_diff(
        &mut self,
        generation: GenID,
        activities: &mut dyn GenerationWorker,
    ) -> Result<(), CGroupError> {
        if let Some(groups) = self.uncommitted_groups.get_mut(&generation) {
            self.groups.apply_diff(groups, activities);
            Ok(())
        } else {
            Err(CGroupError::InvalidGen)
        }
    }

    pub fn all_processes(&self, generation: GenID) -> Result<Vec<i32>, CGroupError> {
        match self.retrieve_groups(generation) {
            Ok(groups) => groups.all_processes(),
            Err(_) => Err(CGroupError::InvalidGen),
        }
    }

    pub fn get_group_path(&self, generation: GenID, group: &str) -> Result<String, CGroupError> {
        self.retrieve_groups(generation)
            .map(|groups| groups.get_group_path(group).unwrap())
    }
}

#[derive(Clone)]
pub struct Group {
    pub name: String,
    pub proc_ids: Vec<i32>,
    pub children: Vec<String>,
    pub attributes: HashMap<String, String>,
    pub parent: String,
}

impl Group {
    pub fn new(name: String, parent: String) -> Group {
        Group {
            name,
            proc_ids: Vec::<i32>::new(),
            children: Vec::<String>::new(),
            attributes: HashMap::<String, String>::new(),
            parent,
        }
    }
}

#[derive(Clone)]
pub struct Groups {
    pub generation: GenID,
    pub names2groups: HashMap<String, Group>,
    proc_ids2groups: HashMap<i32, String>,
    source_generation: GenID,
    creator: String,
    phase: CGroupsPhase,
}

impl Groups {
    pub fn new(creator: String) -> Groups {
        Groups {
            generation: 0,
            names2groups: [(
                String::from("<<root>>"),
                Group::new(String::from("<<root>>"), String::from("")),
            )]
            .iter()
            .cloned()
            .collect(),
            proc_ids2groups: HashMap::<i32, String>::new(),
            source_generation: 0,
            creator,
            phase: CGroupsPhase::Start,
        }
    }

    //
    // Remove a group.
    //
    fn remove_group(&mut self, group_name: &str) -> Result<(), CGroupError> {
        self.phase = self.phase.move_to(CGroupsPhase::GroupRemove)?;

        if let Some(group) = self.names2groups.get(group_name) {
            let children = group.children.clone();
            let proc_ids = group.proc_ids.clone();

            for child_name in children.iter() {
                self.remove_group(child_name)?;
            }
            // Stupid! Since both |group| and calling
            // |move_process_out()| need borrowed references, we need
            // to clone |proc_ids| to stop borrowing from self.
            for proc_id in proc_ids.iter() {
                self.move_process_out(*proc_id).unwrap();
            }
        }
        if let Some(group) = self.names2groups.remove(group_name) {
            if let Some(parent) = self.names2groups.get_mut(&group.parent) {
                for (i, name) in parent.children.iter().enumerate() {
                    if name == group_name {
                        parent.children.remove(i);
                        break;
                    }
                }
                Ok(())
            } else {
                Err(CGroupError::InvalidGen)
            }
        } else {
            Err(CGroupError::InvalidGen)
        }
    }
    //
    // Add a new group as a sub-group of the given parent.
    //
    // "<<root>>" is a precreated group, that is the root of all
    // groups.
    //
    fn add_group(&mut self, group_name: &str, parent: &str) -> Result<(), CGroupError> {
        self.phase = self.phase.move_to(CGroupsPhase::GroupAdd)?;

        if self.names2groups.contains_key(group_name) {
            return Err(CGroupError::DupGroup);
        }

        match self.names2groups.get_mut(parent) {
            Some(pgroup) => pgroup.children.push(String::from(group_name)),
            _ => return Err(CGroupError::UnknownGroup),
        };

        let group = Group::new(String::from(group_name), String::from(parent));
        self.names2groups.insert(String::from(group_name), group);
        Ok(())
    }

    fn update_group_attrs(
        &mut self,
        group_name: &str,
        to_set: Vec<(String, String)>,
        to_remove: Vec<String>,
    ) -> Result<(), CGroupError> {
        self.phase = self.phase.move_to(CGroupsPhase::Attrs)?;

        let setkeys: HashSet<_> = to_set.iter().map(|(k, _v)| k).cloned().collect();
        if setkeys.len() != to_set.len() {
            return Err(CGroupError::ConflictAttr("set more than once".to_string()));
        }
        let rmkeys: HashSet<_> = to_remove.iter().cloned().collect();
        if rmkeys.len() != to_remove.len() {
            return Err(CGroupError::ConflictAttr(
                "remove more than once".to_string(),
            ));
        }
        if setkeys.intersection(&rmkeys).count() != 0 {
            return Err(CGroupError::ConflictAttr(
                "set and remove the same attribute".to_string(),
            ));
        }

        if let Some(group) = self.names2groups.get_mut(group_name) {
            for key in to_remove.iter() {
                group.attributes.remove(key);
            }
            for (key, value) in to_set.iter() {
                group.attributes.insert(key.clone(), value.clone());
            }
            Ok(())
        } else {
            Err(CGroupError::InvalidGen)
        }
    }

    fn get_group_of_process(&mut self, proc_id: i32) -> Option<&mut Group> {
        match self.proc_ids2groups.remove(&proc_id) {
            Some(group_name) => match self.names2groups.get_mut(&group_name) {
                None => panic!("unknown error"),
                x => x,
            },
            None => None,
        }
    }

    fn move_process_out(&mut self, proc_id: i32) -> Result<(), CGroupError> {
        if let Some(group) = self.get_group_of_process(proc_id) {
            for (i, id) in group.proc_ids.iter().enumerate() {
                if *id == proc_id {
                    group.proc_ids.remove(i);
                    break;
                }
            }
            self.proc_ids2groups.remove(&proc_id);
        }
        Ok(())
    }

    fn move_process_in(&mut self, proc_id: i32, group_name: &str) -> Result<(), CGroupError> {
        let group_name = String::from(group_name);
        if let Some(group) = self.names2groups.get_mut(&group_name) {
            group.proc_ids.push(proc_id);
            self.proc_ids2groups.insert(proc_id, group_name);
            Ok(())
        } else {
            Err(CGroupError::UnknownGroup)
        }
    }

    fn move_process(&mut self, proc_id: i32, group_name: String) -> Result<(), CGroupError> {
        self.move_process_out(proc_id).unwrap();
        self.move_process_in(proc_id, &group_name)?;
        Ok(())
    }

    pub fn move_processes(
        &mut self,
        removings: Vec<i32>,
        movings: Vec<(i32, String)>,
    ) -> Result<(), CGroupError> {
        self.phase = self.phase.move_to(CGroupsPhase::Processes)?;

        for proc_id in removings.iter() {
            self.move_process_out(*proc_id)?;
        }
        for (proc_id, group_name) in movings.iter() {
            self.move_process(*proc_id, group_name.clone())?;
        }
        Ok(())
    }

    pub fn apply_diff(&self, target: &Groups, activities: &mut dyn GenerationWorker) {
        let mut added_procs = Vec::<(i32, String)>::new();
        let mut removed_procs = Vec::<i32>::new();
        let mut collect_proc = |old: &Group, new: &Group| {
            let old_procs: HashSet<i32> = old.proc_ids.iter().cloned().collect();
            let new_procs: HashSet<i32> = new.proc_ids.iter().cloned().collect();
            for id in new_procs.difference(&old_procs) {
                added_procs.push((*id, target.get_group_path(&new.name).unwrap()));
            }
            for id in old_procs.difference(&new_procs) {
                removed_procs.push(*id);
            }
        };

        type AttrChange = (Vec<String>, Vec<(String, String)>);

        let mut attr_chgs = HashMap::<String, AttrChange>::new();
        let make_sure_attr_chgs = |attr_chgs: &mut HashMap<String, AttrChange>, group: &str| {
            // It is very stupid that I can not do capturing
            // |attr_chgs| here for that |collect_attrs| also capture
            // it, and create two mutable references at the same time.
            if !attr_chgs.contains_key(group) {
                attr_chgs.insert(
                    String::from(group),
                    (Vec::<String>::new(), Vec::<(String, String)>::new()),
                );
            }
        };
        let mut collect_attrs = |old: &Group, new: &Group| {
            let old_attrs: HashSet<String> = old.attributes.keys().cloned().collect();
            let new_attrs: HashSet<String> = new.attributes.keys().cloned().collect();
            let gname = &old.name;
            for attr in old_attrs.difference(&new_attrs) {
                make_sure_attr_chgs(&mut attr_chgs, gname);
                let (removed_attrs, _) = attr_chgs.get_mut(gname).unwrap();
                removed_attrs.push(attr.clone());
            }
            for attr in new_attrs.difference(&old_attrs) {
                make_sure_attr_chgs(&mut attr_chgs, gname);
                let (_, set_attrs) = attr_chgs.get_mut(gname).unwrap();
                set_attrs.push((attr.clone(), new.attributes.get(attr).unwrap().clone()));
            }
            for attr in new_attrs.intersection(&old_attrs) {
                let old_value = old.attributes.get(attr).unwrap();
                let new_value = new.attributes.get(attr).unwrap();
                if *old_value != *new_value {
                    make_sure_attr_chgs(&mut attr_chgs, gname);
                    let (_, set_attrs) = attr_chgs.get_mut(gname).unwrap();
                    set_attrs.push((attr.clone(), new_value.clone()));
                }
            }
        };

        let mut preordered = vec!["<<root>>"];
        let mut removed_groups = Vec::<&str>::new();
        let mut added_groups = Vec::<(&str, &str)>::new();
        let mut i = 0;
        while i < preordered.len() {
            let group_name = preordered[i];

            let group = self.names2groups.get(group_name).unwrap();
            let tgroup = target.names2groups.get(group_name).unwrap();
            collect_proc(group, tgroup);
            collect_attrs(group, tgroup);

            let children: HashSet<&String> = group.children.iter().collect();
            let tchildren: HashSet<&String> = tgroup.children.iter().collect();
            for rm in children.difference(&tchildren) {
                removed_groups.push(rm);
            }
            for add in tchildren.difference(&children) {
                added_groups.push((add, group_name));
            }

            for child_name in children.intersection(&tchildren) {
                preordered.push(child_name);
            }
            i += 1;
        }

        activities.start_applying(self.generation, target.generation);

        // Expand the list of removed groups to their leaves.
        let mut i = 0;
        while i < removed_groups.len() {
            let group = self.names2groups.get(removed_groups[i]).unwrap();
            for child in group.children.iter() {
                removed_groups.push(child);
            }
            i += 1;
        }
        // Phase 1
        // From removed leaves to removed roots.
        for group_name in removed_groups.iter().rev() {
            activities
                .remove_group(&self.get_group_path(group_name).unwrap())
                .unwrap();
        }

        // Phase 2
        let mut i = 0;
        while i < added_groups.len() {
            let (group_name, parent_name) = added_groups[i];
            activities
                .add_group(group_name, &target.get_group_path(&parent_name).unwrap())
                .unwrap();
            let group = target.names2groups.get(group_name).unwrap();
            for proc_id in group.proc_ids.iter() {
                added_procs.push((*proc_id, target.get_group_path(group_name).unwrap()));
            }
            if !group.attributes.is_empty() {
                make_sure_attr_chgs(&mut attr_chgs, &group_name);
            }
            for (attr, value) in group.attributes.iter() {
                let (_, set_attrs) = attr_chgs.get_mut(group_name).unwrap();

                set_attrs.push((attr.clone(), value.clone()));
            }
            for child_name in group.children.iter() {
                added_groups.push((child_name, group_name));
            }
            i += 1;
        }

        // Phase 3
        for (group_name, (removed_attrs, set_attrs)) in attr_chgs.iter_mut() {
            activities
                .update_group_attrs(
                    &target.get_group_path(group_name).unwrap(),
                    set_attrs,
                    removed_attrs,
                )
                .unwrap();
        }

        // Phase 4
        if !removed_procs.is_empty() || !added_procs.is_empty() {
            let mut removed_procs: HashSet<i32> = removed_procs.iter().cloned().collect();
            for (proc_id, _) in added_procs.iter() {
                removed_procs.remove(proc_id);
            }
            let mut removed_procs: Vec<i32> = removed_procs.iter().cloned().collect();
            activities
                .move_processes(&mut removed_procs, &mut added_procs)
                .unwrap();
        }
    }

    pub fn all_processes(&self) -> Result<Vec<i32>, CGroupError> {
        let procs: Vec<i32> = self.proc_ids2groups.keys().cloned().collect();
        Ok(procs)
    }

    //
    // Get path from <<root>> to the given group.
    //
    // Pathes are separated by "/".  The path of a root is always an
    // empty string "". "a/b" is the path for the "b" group in the "a"
    // group.
    //
    pub fn get_group_path(&self, group_name: &str) -> Result<String, CGroupError> {
        let mut names = Vec::<&str>::new();
        let mut name = group_name;
        while name != "<<root>>" {
            let group = match self.names2groups.get(name) {
                Some(name) => name,
                None => return Err(CGroupError::UnknownGroup),
            };
            names.push(name);
            name = &group.parent;
        }
        names.reverse();
        Ok(names.join("/"))
    }
}

#[cfg(test)]
mod tests {
    use super::CGroupError;

    #[test]
    fn it_works() {
        let _svc = super::CGService::default();
    }
    #[test]
    fn build_groups() {
        let mut svc = super::CGService::default();
        let gid = svc.begin(0, String::from("test")).unwrap();
        svc.add_group(gid, "group1", "<<root>>").unwrap();
        svc.add_group(gid, "group2", "group1").unwrap();
        svc.add_group(gid, "group3", "group1").unwrap();
        svc.update_group_attrs(
            gid,
            "group2",
            vec![
                (String::from("key1"), String::from("value1")),
                (String::from("key2"), String::from("value2")),
                (String::from("key3"), String::from("value3")),
            ],
            vec![],
        )
        .unwrap();

        svc.commit_noop(gid).unwrap();

        let group = svc.groups.names2groups.get("<<root>>").unwrap();
        assert_eq!(String::from(""), group.parent);
        assert_eq!(String::from("<<root>>"), group.name);
        assert_eq!(group.children, vec![String::from("group1")]);

        let group = svc.groups.names2groups.get("group1").unwrap();
        assert_eq!(String::from("<<root>>"), group.parent);
        assert_eq!(String::from("group1"), group.name);
        assert_eq!(
            group.children,
            vec![String::from("group2"), String::from("group3")]
        );

        // Test over-wrote
        let gid = svc.begin(svc.get_active(), String::from("test")).unwrap();
        svc.update_group_attrs(
            gid,
            "group2",
            vec![(String::from("key1"), String::from("value1-1"))],
            vec![],
        )
        .unwrap();
        svc.commit_noop(gid).unwrap();

        let group = svc.groups.names2groups.get("group2").unwrap();
        assert_eq!(
            Some(&String::from("value1-1")),
            group.attributes.get(&String::from("key1"))
        );
        assert_eq!(
            Some(&String::from("value2")),
            group.attributes.get(&String::from("key2"))
        );
        assert_eq!(
            Some(&String::from("value3")),
            group.attributes.get(&String::from("key3"))
        );

        // Test rollback
        let gid = svc.begin(svc.get_active(), String::from("test")).unwrap();
        svc.update_group_attrs(
            gid,
            "group2",
            vec![(String::from("key1"), String::from("value1-2"))],
            vec![],
        )
        .unwrap();
        svc.rollback(gid).unwrap();
        let group = svc.retrieve_group(svc.get_active(), "group2").unwrap();
        assert_eq!(
            Some(&String::from("value1-1")),
            group.attributes.get(&String::from("key1"))
        );
    }
    #[test]
    fn set_n_remove_same_attr() {
        let mut svc = super::CGService::default();
        let gid = svc.begin(0, String::from("test")).unwrap();
        svc.add_group(gid, "group1", "<<root>>").unwrap();
        svc.add_group(gid, "group2", "group1").unwrap();
        assert_eq!(
            Err(CGroupError::ConflictAttr(
                "set and remove the same attribute".to_string()
            )),
            svc.update_group_attrs(
                gid,
                "group2",
                vec![
                    (String::from("key1"), String::from("value1")),
                    (String::from("key2"), String::from("value2")),
                    (String::from("key3"), String::from("value3"))
                ],
                vec![String::from("key2")]
            )
        );
    }
    #[test]
    fn order_of_phases_1() {
        let mut svc = super::CGService::default();
        let gid = svc.begin(0, String::from("test")).unwrap();
        svc.add_group(gid, "group1", "<<root>>").unwrap();
        svc.add_group(gid, "group2", "group1").unwrap();
        svc.update_group_attrs(
            gid,
            "group2",
            vec![
                (String::from("key1"), String::from("value1")),
                (String::from("key2"), String::from("value2")),
                (String::from("key3"), String::from("value3")),
            ],
            vec![],
        )
        .unwrap();
        // Functions should be called in the order of phases.
        assert_eq!(
            Err(CGroupError::PhaseError),
            svc.add_group(gid, "group3", "group1")
        );
    }
    #[test]
    fn order_of_phases_2() {
        let mut svc = super::CGService::default();
        let gid = svc.begin(0, String::from("test")).unwrap();
        svc.add_group(gid, "group1", "<<root>>").unwrap();
        svc.add_group(gid, "group2", "group1").unwrap();
        // Phase 4
        svc.move_processes(gid, Vec::<i32>::new(), vec![(7, String::from("group1"))])
            .unwrap();
        // Phase 2, functions should be called in the order of phases.
        assert_eq!(
            Err(CGroupError::PhaseError),
            svc.add_group(gid, "group3", "group1")
        );
    }
    #[test]
    fn remove_groups() {
        let mut svc = super::CGService::default();
        let gid = svc.begin(0, String::from("test")).unwrap();
        svc.add_group(gid, "group1", "<<root>>").unwrap();
        svc.add_group(gid, "group2", "group1").unwrap();
        svc.commit_noop(gid).unwrap();
        let gid = svc.begin(gid, String::from("test")).unwrap();
        svc.remove_group(gid, "group1").unwrap();
        match svc.retrieve_group(gid, "group1") {
            Ok(_) => panic!("should not found group1"),
            Err(e) => assert_eq!(CGroupError::UnknownGroup, e),
        }
        match svc.retrieve_group(gid, "group2") {
            Ok(_) => panic!("should not found group2"),
            Err(e) => assert_eq!(CGroupError::UnknownGroup, e),
        }
    }

    struct GenerationWorkerMock {
        log: Vec<String>,
    }
    impl GenerationWorkerMock {
        fn new() -> GenerationWorkerMock {
            GenerationWorkerMock {
                log: Vec::<String>::new(),
            }
        }
    }
    use super::GenerationWorker;
    impl GenerationWorker for GenerationWorkerMock {
        fn remove_group(&mut self, group_path: &str) -> Result<(), CGroupError> {
            self.log.push(format!("remove_group {}", group_path));
            Ok(())
        }

        fn add_group(&mut self, group_name: &str, parent: &str) -> Result<(), CGroupError> {
            self.log
                .push(format!("add_group {} {}", group_name, parent));
            Ok(())
        }

        fn update_group_attrs(
            &mut self,
            group_path: &str,
            to_set: &mut [(String, String)],
            to_remove: &mut [String],
        ) -> Result<(), CGroupError> {
            to_set.sort();
            to_remove.sort();
            self.log.push(format!(
                "update_group_attrs {} {:?} {:?}",
                group_path, to_set, to_remove
            ));
            Ok(())
        }

        fn move_processes(
            &mut self,
            removings: &mut [i32],
            movings: &mut [(i32, String)],
        ) -> Result<(), CGroupError> {
            removings.sort();
            movings.sort();
            self.log
                .push(format!("move_processes {:?} {:?}", removings, movings));
            Ok(())
        }
    }

    #[test]
    fn apply_diff_attrs() {
        let mut svc = super::CGService::default();
        let gid = svc.begin(0, String::from("test")).unwrap();
        svc.add_group(gid, "group1", "<<root>>").unwrap();
        svc.add_group(gid, "group2", "group1").unwrap();
        svc.add_group(gid, "group3", "group1").unwrap();
        svc.update_group_attrs(
            gid,
            "group2",
            vec![
                (String::from("key1"), String::from("value1")),
                (String::from("key2"), String::from("value2")),
            ],
            vec![],
        )
        .unwrap();
        svc.update_group_attrs(
            gid,
            "group3",
            vec![
                (String::from("key3"), String::from("value3")),
                (String::from("key4"), String::from("value4")),
            ],
            vec![],
        )
        .unwrap();
        svc.update_group_attrs(
            gid,
            "group1",
            vec![
                (String::from("key5"), String::from("value5")),
                (String::from("key6"), String::from("value6")),
            ],
            vec![],
        )
        .unwrap();
        svc.commit_noop(gid).unwrap();

        let gid = svc.begin(gid, String::from("test")).unwrap();
        svc.update_group_attrs(
            gid,
            "group1",
            vec![
                (String::from("key5"), String::from("value5-1")),
                (String::from("key7"), String::from("value7")),
            ],
            vec![String::from("key6")],
        )
        .unwrap();
        svc.update_group_attrs(
            gid,
            "group3",
            vec![
                (String::from("key3"), String::from("value3")),
                (String::from("key4"), String::from("value4-1")),
            ],
            Vec::<String>::new(),
        )
        .unwrap();
        svc.update_group_attrs(
            gid,
            "group1",
            vec![(String::from("key5"), String::from("value5-2"))],
            Vec::<String>::new(),
        )
        .unwrap();
        let mut log = GenerationWorkerMock::new();
        svc.apply_diff(gid, &mut log).unwrap();
        log.log.sort();
        let mut expected = GenerationWorkerMock::new();
        expected
            .update_group_attrs(
                "group1",
                &mut vec![
                    (String::from("key5"), String::from("value5-2")),
                    (String::from("key7"), String::from("value7")),
                ],
                &mut vec![String::from("key6")],
            )
            .unwrap();
        expected
            .update_group_attrs(
                "group1/group3",
                &mut vec![(String::from("key4"), String::from("value4-1"))],
                &mut vec![],
            )
            .unwrap();
        expected.log.sort();
        assert_eq!(log.log, expected.log);
        svc.commit_noop(gid).unwrap();
    }

    #[test]
    fn apply_diff_remove_groups() {
        let mut svc = super::CGService::default();
        let gid = svc.begin(0, String::from("test")).unwrap();
        svc.add_group(gid, "group1", "<<root>>").unwrap();
        svc.add_group(gid, "group2", "group1").unwrap();
        svc.add_group(gid, "group3", "group1").unwrap();
        svc.commit_noop(gid).unwrap();

        let gid = svc.begin(gid, String::from("test")).unwrap();
        svc.remove_group(gid, "group3").unwrap();
        let mut log = GenerationWorkerMock::new();
        svc.apply_diff(gid, &mut log).unwrap();
        log.log.sort();
        let mut expected = GenerationWorkerMock::new();
        expected.remove_group("group1/group3").unwrap();
        expected.log.sort();
        assert_eq!(log.log, expected.log);
        svc.commit_noop(gid).unwrap();

        let gid = svc.begin(gid, String::from("test")).unwrap();
        svc.remove_group(gid, "group1").unwrap();
        let mut log = GenerationWorkerMock::new();
        svc.apply_diff(gid, &mut log).unwrap();
        log.log.sort();
        let mut expected = GenerationWorkerMock::new();
        expected.remove_group("group1").unwrap();
        expected.remove_group("group1/group2").unwrap();
        expected.log.sort();
        assert_eq!(log.log, expected.log);
        svc.commit_noop(gid).unwrap();
    }

    #[test]
    fn apply_diff_add_groups() {
        let mut svc = super::CGService::default();
        let gid = svc.begin(0, String::from("test")).unwrap();
        svc.add_group(gid, "group1", "<<root>>").unwrap();
        svc.add_group(gid, "group2", "group1").unwrap();
        svc.add_group(gid, "group3", "group1").unwrap();
        svc.commit_noop(gid).unwrap();

        let gid = svc.begin(gid, String::from("test")).unwrap();
        svc.add_group(gid, "group4", "group3").unwrap();
        svc.add_group(gid, "group5", "group2").unwrap();
        let mut log = GenerationWorkerMock::new();
        svc.apply_diff(gid, &mut log).unwrap();
        log.log.sort();
        let mut expected = GenerationWorkerMock::new();
        expected.add_group("group4", "group1/group3").unwrap();
        expected.add_group("group5", "group1/group2").unwrap();
        expected.log.sort();
        assert_eq!(log.log, expected.log);
        svc.commit_noop(gid).unwrap();
    }

    #[test]
    fn apply_diff_move_processes() {
        let mut svc = super::CGService::default();
        let gid = svc.begin(0, String::from("test")).unwrap();
        svc.add_group(gid, "group1", "<<root>>").unwrap();
        svc.add_group(gid, "group2", "group1").unwrap();
        svc.add_group(gid, "group3", "group1").unwrap();
        svc.move_processes(
            gid,
            Vec::<i32>::new(),
            vec![(1, String::from("group3")), (2, String::from("group1"))],
        )
        .unwrap();
        svc.commit_noop(gid).unwrap();

        let gid = svc.begin(gid, String::from("test")).unwrap();
        svc.move_processes(
            gid,
            Vec::<i32>::new(),
            vec![(3, String::from("group3")), (1, String::from("group1"))],
        )
        .unwrap();
        svc.move_processes(gid, Vec::<i32>::new(), vec![(1, String::from("<<root>>"))])
            .unwrap();
        let mut log = GenerationWorkerMock::new();
        svc.apply_diff(gid, &mut log).unwrap();
        log.log.sort();
        let mut expected = GenerationWorkerMock::new();
        expected
            .move_processes(
                &mut vec![],
                &mut vec![(3, String::from("group1/group3")), (1, String::from(""))],
            )
            .unwrap();
        expected.log.sort();
        assert_eq!(log.log, expected.log);
        svc.commit_noop(gid).unwrap();

        let gid = svc.begin(gid, String::from("test")).unwrap();
        svc.remove_group(gid, "group3").unwrap();
        let mut proc_ids = svc.all_processes(gid).unwrap();
        proc_ids.sort();
        assert_eq!(vec![1, 2], proc_ids);
    }

    #[test]
    fn group_paths() {
        let mut svc = super::CGService::default();
        let gid = svc.begin(0, String::from("test")).unwrap();
        svc.add_group(gid, "group1", "<<root>>").unwrap();
        svc.add_group(gid, "group2", "group1").unwrap();
        svc.add_group(gid, "group3", "group1").unwrap();
        let path = svc.get_group_path(gid, "group3").unwrap();
        assert_eq!(path, "group1/group3");
    }
}
