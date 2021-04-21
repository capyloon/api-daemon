use log::{error, info};
use std::cmp::{Ord, Ordering};
use std::collections::{HashMap, HashSet};
use thiserror::Error;

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
//   committed.  Therefore, at most one of generations that were
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
    pub fn groups(&self) -> &Groups {
        &self.groups
    }

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
        self.commit_apply(generation, &mut super::WorkerType::new())
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
        match self.retrieve_groups(generation) {
            Ok(groups) => groups.get_group_path(group),
            Err(err) => Err(err),
        }
    }

    pub fn log(&self) {
        info!(
            "  Uncommitted Groups count: {}",
            self.uncommitted_groups.len()
        );
        self.groups.log();
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

type AttrChange = (Vec<String>, Vec<(String, String)>);

impl Groups {
    pub fn log(&self) {
        info!("  CGroup Names   : {}", self.names2groups.len());
        info!("  CGroup proc ids: {}", self.proc_ids2groups.len());
    }

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
                self.move_process_out(*proc_id);
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

    fn move_process_out(&mut self, proc_id: i32) {
        if let Some(group) = self.get_group_of_process(proc_id) {
            for (i, id) in group.proc_ids.iter().enumerate() {
                if *id == proc_id {
                    group.proc_ids.remove(i);
                    break;
                }
            }
            self.proc_ids2groups.remove(&proc_id);
        }
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
        self.move_process_out(proc_id);
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
            self.move_process_out(*proc_id);
        }
        for (proc_id, group_name) in movings.iter() {
            self.move_process(*proc_id, group_name.clone())?;
        }
        Ok(())
    }

    fn make_sure_attr_chgs<F>(attr_chgs: &mut HashMap<String, AttrChange>, group: &str, mut f: F)
    where
        F: FnMut(&mut (Vec<String>, Vec<(String, String)>)),
    {
        let value = attr_chgs
            .entry(group.into())
            .or_insert((Vec::<String>::new(), Vec::<(String, String)>::new()));
        f(&mut *value);
    }

    pub fn apply_diff(&self, target: &Groups, activities: &mut dyn GenerationWorker) {
        let mut added_procs = Vec::<(i32, String)>::new();
        let mut removed_procs = Vec::<i32>::new();
        let mut collect_proc = |old: &Group, new: &Group| {
            let old_procs: HashSet<i32> = old.proc_ids.iter().cloned().collect();
            let new_procs: HashSet<i32> = new.proc_ids.iter().cloned().collect();
            for id in new_procs.difference(&old_procs) {
                if let Ok(path) = target.get_group_path(&new.name) {
                    added_procs.push((*id, path));
                }
            }
            for id in old_procs.difference(&new_procs) {
                removed_procs.push(*id);
            }
        };

        let mut attr_chgs = HashMap::<String, AttrChange>::new();

        let mut collect_attrs = |old: &Group, new: &Group| {
            let old_attrs: HashSet<String> = old.attributes.keys().cloned().collect();
            let new_attrs: HashSet<String> = new.attributes.keys().cloned().collect();
            let gname = &old.name;
            for attr in old_attrs.difference(&new_attrs) {
                Self::make_sure_attr_chgs(&mut attr_chgs, gname, |(removed_attrs, _)| {
                    removed_attrs.push(attr.clone());
                });
            }
            for attr in new_attrs.difference(&old_attrs) {
                Self::make_sure_attr_chgs(&mut attr_chgs, gname, |(_, set_attrs)| {
                    if let Some(value) = new.attributes.get(attr) {
                        set_attrs.push((attr.clone(), value.clone()));
                    } else {
                        error!("Failed to get new.attributes {}", attr);
                    }
                });
            }
            for attr in new_attrs.intersection(&old_attrs) {
                if let (Some(old_value), Some(new_value)) =
                    (old.attributes.get(attr), new.attributes.get(attr))
                {
                    if *old_value != *new_value {
                        Self::make_sure_attr_chgs(&mut attr_chgs, gname, |(_, set_attrs)| {
                            set_attrs.push((attr.clone(), new_value.clone()));
                        });
                    }
                } else {
                    error!("Failed to get old.attributes or new.attributes {}", attr);
                }
            }
        };

        let mut preordered = vec!["<<root>>"];
        let mut removed_groups = Vec::<&str>::new();
        let mut added_groups = Vec::<(&str, &str)>::new();
        let mut i = 0;
        while i < preordered.len() {
            let group_name = preordered[i];

            if let (Some(group), Some(tgroup)) = (
                self.names2groups.get(group_name),
                target.names2groups.get(group_name),
            ) {
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
            } else {
                error!(
                    "Failed to get {} from self.names2groups={} or target.names2groups={}",
                    group_name,
                    self.names2groups.get(group_name).is_some(),
                    target.names2groups.get(group_name).is_some(),
                );
            }
            i += 1;
        }

        activities.start_applying(self.generation, target.generation);

        // Expand the list of removed groups to their leaves.
        let mut i = 0;
        while i < removed_groups.len() {
            if let Some(group) = self.names2groups.get(removed_groups[i]) {
                for child in group.children.iter() {
                    removed_groups.push(child);
                }
            } else {
                error!("Failed to find {} in self.names2groups", removed_groups[i]);
            }
            i += 1;
        }
        // Phase 1
        // From removed leaves to removed roots.
        for group_name in removed_groups.iter().rev() {
            if let Ok(path) = self.get_group_path(group_name) {
                let _ = activities.remove_group(&path);
            }
        }

        // Phase 2
        let mut i = 0;
        while i < added_groups.len() {
            let (group_name, parent_name) = added_groups[i];
            if let Ok(path) = target.get_group_path(&parent_name) {
                let _ = activities.add_group(group_name, &path);
            }
            if let Some(group) = target.names2groups.get(group_name) {
                for proc_id in group.proc_ids.iter() {
                    if let Ok(path) = target.get_group_path(group_name) {
                        added_procs.push((*proc_id, path));
                    }
                }
                if !group.attributes.is_empty() {
                    Self::make_sure_attr_chgs(&mut attr_chgs, &group_name, |_| {});
                }
                for (attr, value) in group.attributes.iter() {
                    if let Some((_, set_attrs)) = attr_chgs.get_mut(group_name) {
                        set_attrs.push((attr.clone(), value.clone()));
                    } else {
                        error!("Failed to find {} in attr_chgs", group_name);
                    }
                }
                for child_name in group.children.iter() {
                    added_groups.push((child_name, group_name));
                }
            }
            i += 1;
        }

        // Phase 3
        for (group_name, (removed_attrs, set_attrs)) in attr_chgs.iter_mut() {
            if let Ok(path) = target.get_group_path(group_name) {
                let _ = activities.update_group_attrs(&path, set_attrs, removed_attrs);
            }
        }

        // Phase 4
        if !removed_procs.is_empty() || !added_procs.is_empty() {
            let mut removed_procs: HashSet<i32> = removed_procs.iter().cloned().collect();
            for (proc_id, _) in added_procs.iter() {
                removed_procs.remove(proc_id);
            }
            let mut removed_procs: Vec<i32> = removed_procs.iter().cloned().collect();
            let _ = activities.move_processes(&mut removed_procs, &mut added_procs);
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
                None => {
                    error!("Failed to get group path for {}", group_name);
                    return Err(CGroupError::UnknownGroup);
                }
            };
            names.push(name);
            name = &group.parent;
        }
        names.reverse();
        Ok(names.join("/"))
    }
}
