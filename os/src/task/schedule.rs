use alloc::collections::VecDeque;
use alloc::sync::Arc;

use super::structs::Task;

pub trait Scheduler {
    fn add_ready_task(&mut self, t: &Arc<Task>);
    fn pick_next_task(&mut self) -> Option<Arc<Task>>;
    fn timer_tick(&mut self);
}

struct SchedulerState {
    task: Arc<Task>,
}

impl SchedulerState {
    fn new(task: Arc<Task>) -> Self {
        Self { task }
    }
}

pub struct SimpleScheduler {
    ready_queue: VecDeque<SchedulerState>,
}

impl SimpleScheduler {
    pub fn new() -> Self {
        Self {
            ready_queue: VecDeque::new(),
        }
    }
}

impl Scheduler for SimpleScheduler {
    fn add_ready_task(&mut self, t: &Arc<Task>) {
        self.ready_queue.push_back(SchedulerState::new(t.clone()));
    }

    fn pick_next_task(&mut self) -> Option<Arc<Task>> {
        self.ready_queue.pop_front().map(|s| s.task)
    }

    fn timer_tick(&mut self) {}
}
