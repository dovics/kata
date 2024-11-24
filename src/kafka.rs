use rdkafka::metadata::{MetadataBroker, MetadataPartition, MetadataTopic};

#[derive(Debug, Clone)]
pub struct KafkaBroker {
    pub id: i32,
    pub host: String,
    pub port: i32,
}

impl From<&MetadataBroker> for KafkaBroker {
    fn from(broker: &MetadataBroker) -> Self {
        Self {
            id: broker.id(),
            host: broker.host().to_string(),
            port: broker.port(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct KafkaTopic {
    pub name: String,
    pub partitions: Vec<KafkaPartition>,
}

impl From<&MetadataTopic> for KafkaTopic {
    fn from(topic: &MetadataTopic) -> Self {
        let partitions = topic.partitions();

        Self {
            name: topic.name().to_string(),
            partitions: partitions
                .into_iter()
                .map(|p| KafkaPartition::from(p))
                .collect(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct KafkaPartition {
    pub id: i32,
    pub leader: i32,
    pub replicas: Vec<i32>,
    pub isr: Vec<i32>,
}

impl From<&MetadataPartition> for KafkaPartition {
    fn from(partition: &MetadataPartition) -> Self {
        Self {
            id: partition.id(),
            leader: partition.leader(),
            replicas: partition.replicas().to_vec(),
            isr: partition.isr().to_vec(),
        }
    }
}
