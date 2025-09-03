use async_nats::{
    jetstream::{self, consumer::DeliverPolicy},
    Client,
};
use eyre::{Result, WrapErr};
use futures_util::StreamExt;

pub struct MqClient {
    nats: Client,
    subject_name: String,
    stream_name: String,
}

impl MqClient {
    pub async fn new(server_url: &str, subject_name: &str, stream_name: &str) -> Result<Self> {
        let client = async_nats::connect(server_url)
            .await
            .map_err(|e| eyre::eyre!("NATS connect failed: {}", e))?;
        Ok(Self {
            nats: client,
            subject_name: subject_name.to_string(),
            stream_name: stream_name.to_string(),
        })
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
                durable_name: Some("price-sink".to_string()),
                deliver_policy: if from_start {
                    DeliverPolicy::All
                } else {
                    DeliverPolicy::New
                },
                filter_subject: self.subject_name.clone(),
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
}
