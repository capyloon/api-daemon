// Fallback implementation of GenerationWorker

use crate::cgroups::{CGroupError, GenerationWorker};

pub struct CGroupsWorker();

impl CGroupsWorker {
    pub fn new() -> Self {
        CGroupsWorker()
    }
}

// Dummy implementation fo GenerationWorker
//
// Just do nothing as a fallback for the platforms that doesn't
// support CGroup.
impl GenerationWorker for CGroupsWorker {
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
        _removings: &mut [i32],
        _movings: &mut [(i32, String)],
    ) -> Result<(), CGroupError> {
        // Not supported!
        Ok(())
    }
}
