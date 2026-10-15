use rocket::tokio::sync::broadcast;
use serde::Serialize;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use uuid::Uuid;

pub use application::realtime::{ResumeChangePublisher, ResumeChangedAction, SectionType};

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct ResumeChangedEvent {
    #[serde(rename = "type")]
    pub kind: String,
    pub resume_id: Uuid,
    pub action: ResumeChangedAction,
}

#[derive(Clone)]
pub struct Hub {
    inner: Arc<HubInner>,
}

struct HubInner {
    topics: RwLock<HashMap<Uuid, broadcast::Sender<ResumeChangedEvent>>>,
    buffer: usize,
}

impl Hub {
    pub fn new() -> Self {
        Hub {
            inner: Arc::new(HubInner {
                topics: RwLock::new(HashMap::new()),
                buffer: 128,
            }),
        }
    }

    fn sender_for(&self, resume_id: Uuid) -> broadcast::Sender<ResumeChangedEvent> {
        {
            let guard = self
                .inner
                .topics
                .read()
                .expect("realtime hub topics read lock");

            if let Some(sender) = guard.get(&resume_id) {
                return sender.clone();
            }
        }

        let mut guard = self
            .inner
            .topics
            .write()
            .expect("realtime hub topics write lock");

        if let Some(sender) = guard.get(&resume_id) {
            return sender.clone();
        }

        let (sender, _) = broadcast::channel(self.inner.buffer);
        guard.insert(resume_id, sender.clone());
        sender
    }

    pub fn subscribe(&self, resume_id: Uuid) -> broadcast::Receiver<ResumeChangedEvent> {
        self.sender_for(resume_id).subscribe()
    }

    pub fn publish_resume_changed(&self, resume_id: Uuid, action: ResumeChangedAction) {
        let sender = self.sender_for(resume_id);
        let _ = sender.send(ResumeChangedEvent {
            kind: "resume.changed".to_string(),
            resume_id,
            action,
        });
    }
}

impl ResumeChangePublisher for Hub {
    fn resume_changed(&self, resume_id: Uuid, action: ResumeChangedAction) {
        self.publish_resume_changed(resume_id, action);
    }
}

impl Default for Hub {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn subscribe_then_publish_delivers_event() {
        let hub = Hub::new();
        let id = Uuid::nil();
        let mut rx = hub.subscribe(id);

        hub.publish_resume_changed(id, ResumeChangedAction::Updated(SectionType::PersonalInfo));

        let evt = rx.try_recv().expect("event should be delivered");
        assert_eq!(evt.resume_id, id);
        assert_eq!(
            evt.action,
            ResumeChangedAction::Updated(SectionType::PersonalInfo)
        );
        assert_eq!(evt.kind, "resume.changed");
    }
}
