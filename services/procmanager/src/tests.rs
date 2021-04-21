use crate::cgroups::CGroupError;

#[test]
fn it_works() {
    let _svc = crate::cgroups::CGService::default();
}
#[test]
fn build_groups() {
    let mut svc = crate::cgroups::CGService::default();
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

    let group = svc.groups().names2groups.get("<<root>>").unwrap();
    assert_eq!(String::from(""), group.parent);
    assert_eq!(String::from("<<root>>"), group.name);
    assert_eq!(group.children, vec![String::from("group1")]);

    let group = svc.groups().names2groups.get("group1").unwrap();
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

    let group = svc.groups().names2groups.get("group2").unwrap();
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
    let mut svc = crate::cgroups::CGService::default();
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
    let mut svc = crate::cgroups::CGService::default();
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
    let mut svc = crate::cgroups::CGService::default();
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
    let mut svc = crate::cgroups::CGService::default();
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
use crate::cgroups::GenerationWorker;
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
        to_set.sort_unstable();
        to_remove.sort_unstable();
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
        removings.sort_unstable();
        movings.sort_unstable();
        self.log
            .push(format!("move_processes {:?} {:?}", removings, movings));
        Ok(())
    }
}

#[test]
fn apply_diff_attrs() {
    let mut svc = crate::cgroups::CGService::default();
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
    log.log.sort_unstable();
    let mut expected = GenerationWorkerMock::new();
    expected
        .update_group_attrs(
            "group1",
            &mut [
                (String::from("key5"), String::from("value5-2")),
                (String::from("key7"), String::from("value7")),
            ],
            &mut [String::from("key6")],
        )
        .unwrap();
    expected
        .update_group_attrs(
            "group1/group3",
            &mut [(String::from("key4"), String::from("value4-1"))],
            &mut [],
        )
        .unwrap();
    expected.log.sort_unstable();
    assert_eq!(log.log, expected.log);
    svc.commit_noop(gid).unwrap();
}

#[test]
fn apply_diff_remove_groups() {
    let mut svc = crate::cgroups::CGService::default();
    let gid = svc.begin(0, String::from("test")).unwrap();
    svc.add_group(gid, "group1", "<<root>>").unwrap();
    svc.add_group(gid, "group2", "group1").unwrap();
    svc.add_group(gid, "group3", "group1").unwrap();
    svc.commit_noop(gid).unwrap();

    let gid = svc.begin(gid, String::from("test")).unwrap();
    svc.remove_group(gid, "group3").unwrap();
    let mut log = GenerationWorkerMock::new();
    svc.apply_diff(gid, &mut log).unwrap();
    log.log.sort_unstable();
    let mut expected = GenerationWorkerMock::new();
    expected.remove_group("group1/group3").unwrap();
    expected.log.sort_unstable();
    assert_eq!(log.log, expected.log);
    svc.commit_noop(gid).unwrap();

    let gid = svc.begin(gid, String::from("test")).unwrap();
    svc.remove_group(gid, "group1").unwrap();
    let mut log = GenerationWorkerMock::new();
    svc.apply_diff(gid, &mut log).unwrap();
    log.log.sort_unstable();
    let mut expected = GenerationWorkerMock::new();
    expected.remove_group("group1").unwrap();
    expected.remove_group("group1/group2").unwrap();
    expected.log.sort_unstable();
    assert_eq!(log.log, expected.log);
    svc.commit_noop(gid).unwrap();
}

#[test]
fn apply_diff_add_groups() {
    let mut svc = crate::cgroups::CGService::default();
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
    log.log.sort_unstable();
    let mut expected = GenerationWorkerMock::new();
    expected.add_group("group4", "group1/group3").unwrap();
    expected.add_group("group5", "group1/group2").unwrap();
    expected.log.sort_unstable();
    assert_eq!(log.log, expected.log);
    svc.commit_noop(gid).unwrap();
}

#[test]
fn apply_diff_move_processes() {
    let mut svc = crate::cgroups::CGService::default();
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
    log.log.sort_unstable();
    let mut expected = GenerationWorkerMock::new();
    expected
        .move_processes(
            &mut [],
            &mut [(3, String::from("group1/group3")), (1, String::from(""))],
        )
        .unwrap();
    expected.log.sort_unstable();
    assert_eq!(log.log, expected.log);
    svc.commit_noop(gid).unwrap();

    let gid = svc.begin(gid, String::from("test")).unwrap();
    svc.remove_group(gid, "group3").unwrap();
    let mut proc_ids = svc.all_processes(gid).unwrap();
    proc_ids.sort_unstable();
    assert_eq!(vec![1, 2], proc_ids);
}

#[test]
fn group_paths() {
    let mut svc = crate::cgroups::CGService::default();
    let gid = svc.begin(0, String::from("test")).unwrap();
    svc.add_group(gid, "group1", "<<root>>").unwrap();
    svc.add_group(gid, "group2", "group1").unwrap();
    svc.add_group(gid, "group3", "group1").unwrap();
    let path = svc.get_group_path(gid, "group3").unwrap();
    assert_eq!(path, "group1/group3");
}
