use async_nats::Client;
use eyre::Result;

pub struct MqClient {
    pub nats: Client,
    subject_name: String,
}

impl MqClient {
    pub async fn new(server_url: &str, subject_name: &str) -> Result<Self> {
        let client = async_nats::connect(server_url)
            .await
            .map_err(|e| eyre::eyre!("NATS connect failed: {}", e))?;
        Ok(Self {
            nats: client,
            subject_name: subject_name.to_string(),
        })
    }

    pub async fn produce_record(&self, record: String) -> Result<()> {
        self.nats
            .publish(self.subject_name.clone(), record.into())
            .await
            .map_err(|e| eyre::eyre!("NATS publish failed: {}", e))?;
        Ok(())
    }

    pub async fn get_kv(&self, bucket: &str) -> Result<async_nats::jetstream::kv::Store> {
        let js = async_nats::jetstream::new(self.nats.clone());
        js.get_key_value(bucket).await.map_err(Into::into)
    }
}
