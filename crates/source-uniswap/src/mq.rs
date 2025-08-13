use eyre::Result;
use fluvio::metadata::topic::TopicSpec;
use fluvio::{Fluvio, FluvioConfig, RecordKey};

const PARTITIONS: u32 = 1;
const REPLICAS: u32 = 1;

pub struct MqClient {
    fluvio: Fluvio,
    topic: String,
}

impl MqClient {
    pub async fn new(address: &str, topic: &str) -> Result<Self> {
        let cfg = FluvioConfig::new(address);
        let fluvio = Fluvio::connect_with_config(&cfg)
            .await
            .map_err(|e| eyre::eyre!("Failed to connect to Fluvio: {}", e))?;
        Ok(Self {
            fluvio,
            topic: topic.to_string(),
        })
    }

    pub async fn list_topics(&self) -> Result<Vec<String>> {
        let admin = self.fluvio.admin().await;
        let topics = admin
            .all::<TopicSpec>()
            .await
            .expect("Failed to list topics");
        let topic_names = topics
            .iter()
            .map(|topic| topic.name.clone())
            .collect::<Vec<String>>();
        Ok(topic_names)
    }

    pub async fn create_topic(&self, topic_name: &str) -> Result<()> {
        let admin = self.fluvio.admin().await;
        let topic_spec = TopicSpec::new_computed(PARTITIONS, REPLICAS, None);
        admin
            .create(topic_name.to_string(), false, topic_spec)
            .await
            .map_err(|e| eyre::eyre!("Failed to create topic: {}", e))?;
        Ok(())
    }

    pub async fn produce_record(&self, record: &str) -> Result<()> {
        let producer = self
            .fluvio
            .topic_producer(&self.topic)
            .await
            .map_err(|e| eyre::eyre!("Failed to create producer: {}", e))?;
        producer
            .send(RecordKey::NULL, record)
            .await
            .map_err(|e| eyre::eyre!("Failed to send record: {}", e))?;
        producer
            .flush()
            .await
            .map_err(|e| eyre::eyre!("Failed to flush producer: {}", e))?;
        Ok(())
    }
}
