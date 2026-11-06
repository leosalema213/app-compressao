use serde::Serialize;
use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicU64, AtomicUsize, Ordering},
        Arc, Mutex,
    },
};
use tauri::{async_runtime::Mutex as AsyncMutex, ipc::Channel};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum JobStatus {
    Queued,
    Running,
    Cancelling,
    Finished,
    Failed,
    Cancelled,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JobUpdate {
    pub id: u64,
    pub status: JobStatus,
    pub completed: usize,
    pub total: usize,
}

type JobRegistry = Arc<Mutex<HashMap<u64, Arc<Mutex<JobStatus>>>>>;

#[derive(Default)]
pub struct JobQueue {
    pub execution: Arc<AsyncMutex<()>>,
    next_id: AtomicU64,
    jobs: JobRegistry,
}

impl JobQueue {
    pub fn register(&self, total: usize, progress: Option<Channel<JobUpdate>>) -> Job {
        let id = self.next_id.fetch_add(1, Ordering::Relaxed) + 1;
        let status = Arc::new(Mutex::new(JobStatus::Queued));
        self.jobs.lock().unwrap().insert(id, status.clone());
        let job = Job {
            id,
            status,
            jobs: self.jobs.clone(),
            total,
            completed: AtomicUsize::new(0),
            progress,
        };
        job.publish(0);
        job
    }

    fn cancel(&self, id: u64) -> bool {
        let status = self.jobs.lock().unwrap().get(&id).cloned();
        let Some(status) = status else { return false };
        let mut status = status.lock().unwrap();
        match *status {
            JobStatus::Queued | JobStatus::Running | JobStatus::Cancelling => {
                *status = JobStatus::Cancelling;
                true
            }
            _ => false,
        }
    }
}

pub struct Job {
    id: u64,
    status: Arc<Mutex<JobStatus>>,
    jobs: JobRegistry,
    total: usize,
    completed: AtomicUsize,
    progress: Option<Channel<JobUpdate>>,
}

impl Job {
    pub fn start(&self) -> bool {
        let mut status = self.status.lock().unwrap();
        if *status != JobStatus::Queued {
            return false;
        }
        *status = JobStatus::Running;
        drop(status);
        self.publish(0);
        true
    }

    pub fn cancelled(&self) -> bool {
        *self.status.lock().unwrap() == JobStatus::Cancelling
    }

    pub fn publish(&self, completed: usize) {
        let completed = completed.max(self.completed.fetch_max(completed, Ordering::Relaxed));
        if let Some(progress) = &self.progress {
            let status = *self.status.lock().unwrap();
            let _ = progress.send(JobUpdate {
                id: self.id,
                status,
                completed,
                total: self.total,
            });
        }
    }

    pub fn finish(&self, status: JobStatus, completed: usize) {
        *self.status.lock().unwrap() = status;
        self.publish(completed);
    }
}

impl Drop for Job {
    fn drop(&mut self) {
        self.jobs.lock().unwrap().remove(&self.id);
    }
}

#[tauri::command]
pub fn cancel_image_job(queue: tauri::State<'_, JobQueue>, id: u64) -> bool {
    queue.cancel(id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cancellation_is_idempotent_and_terminal_jobs_cannot_restart() {
        let queue = JobQueue::default();
        let job = queue.register(2, None);
        assert!(job.start());
        assert!(!job.start());
        assert!(queue.cancel(job.id));
        assert!(queue.cancel(job.id));
        assert!(job.cancelled());
        job.finish(JobStatus::Cancelled, 1);
        assert!(!queue.cancel(job.id));
        assert!(!job.start());
        let id = job.id;
        drop(job);
        assert!(!queue.jobs.lock().unwrap().contains_key(&id));
    }

    #[test]
    fn queued_cancelled_job_never_starts() {
        let queue = JobQueue::default();
        let job = queue.register(1, None);
        assert!(queue.cancel(job.id));
        assert!(!job.start());
        assert!(job.cancelled());
    }
}
