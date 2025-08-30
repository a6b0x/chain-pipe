use async_nats::{
    jetstream::{self, consumer::DeliverPolicy, kv::Store},
    Client,
};
use eyre::{Result, WrapErr};
use futures_util::StreamExt;

pub struct MqClient {
    nats: Client,
    subject_input: String,
    subject_output: String,
    stream_name: String,
}

impl MqClient {
    pub async fn new(
        server_url: &str,
        subject_input: &str,
        subject_output: &str,
        stream_name: &str,
    ) -> Result<Self> {
        let client = async_nats::connect(server_url)
            .await
            .map_err(|e| eyre::eyre!("NATS connect failed: {}", e))?;
        Ok(Self {
            nats: client,
            subject_input: subject_input.to_string(),
            subject_output: subject_output.to_string(),
            stream_name: stream_name.to_string(),
        })
    }

    pub async fn produce_record(&self, record: String) -> Result<()> {
        self.nats
            .publish(self.subject_output.clone(), record.into())
            .await
            .map_err(|e| eyre::eyre!("NATS publish failed: {}", e))?;
        Ok(())
    }

    pub async fn jetstream_pull_from(
        &self,
        from_start: bool,
    ) -> Result<impl StreamExt<Item = Result<async_nats::jetstream::Message>>> {
        let js = jetstream::new(self.nats.clone());

        let stream = js
            .get_stream(self.stream_name.clone())
            .await
            .wrap_err("Failed to get JetStream stream")?;

        let consumer = stream
            .create_consumer(async_nats::jetstream::consumer::pull::Config {
                durable_name: Some("price-injector".to_string()),
                deliver_policy: if from_start {
                    DeliverPolicy::All
                } else {
                    DeliverPolicy::New
                },
                filter_subject: self.subject_input.clone(),
                ..Default::default()
            })
            .await
            .wrap_err("Failed to create JetStream consumer")?;

        let messages = consumer
            .messages()
            .await
            .wrap_err("Failed to get message stream from consumer")?;
        Ok(messages.map(|msg_result| {
            msg_result.map_err(|e| eyre::eyre!("JetStream message error: {}", e))
        }))
    }

    pub async fn kv_store(&self, bucket: &str) -> Result<Store> {
        let js = jetstream::new(self.nats.clone());
        js.create_key_value(async_nats::jetstream::kv::Config {
            bucket: bucket.to_string(),
            ..Default::default()
        })
        .await
        .map_err(|e| eyre::eyre!("KV bucket error: {e}"))
    }
}
