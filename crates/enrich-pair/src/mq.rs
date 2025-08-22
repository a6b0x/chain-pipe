use async_nats::{Client, Subscriber};
use eyre::Result;

pub struct MqClient {
    nats: Client,
    subject_input: String,
}

impl MqClient {
    pub async fn new(server_url: &str, subject_name: &str) -> Result<Self> {
        let client = async_nats::connect(server_url)
            .await
            .map_err(|e| eyre::eyre!("NATS connect failed: {}", e))?;
        Ok(Self {
            nats: client,
            subject_input: subject_name.to_string(),
        })
    }

    pub async fn subscribe(&self) -> Result<Subscriber> {
        Ok(self.nats.subscribe(self.subject_input.clone()).await?)
    }
}
