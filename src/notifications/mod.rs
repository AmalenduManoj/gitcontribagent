use async_trait::async_trait;
use crate::models::notification::Notification;

#[async_trait]
pub trait NotificationProvider: Send + Sync {
    async fn send(&self, notification: &Notification) -> anyhow::Result<()>;
}

pub mod email;
