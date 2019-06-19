use std::collections::VecDeque;

pub mod report::Report;

// A flow is a queue of tasks to run a various points during the app cycle
struct Flow {
    pub pre_build: VecDeque<Task>,
    pub post_build: VecDeque<Task>,
    pub release: VecDeque<Task>,
}

// A task is an independent item to be ran
struct Task {
    pub name: &str,
}

trait PreBuild {
    fn pre_can_run(&self) -> Result<Report, HEMTTError>;
    fn pre_run(&self) -> Result<Report, HEMTTError>;
}

trait PostBuild {
    fn post_can_run(&self) -> Result<Report, HEMTTError>;
    fn post_run(&self) -> Result<Report, HEMTTError>;
}

trait ReleaseBuild {
    fn rel_can_run(&self) -> Result<Report, HEMTTError>;
    fn rel_run(&self) -> Result<Report, HEMTTError>;
}
