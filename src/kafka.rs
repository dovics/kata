use rdkafka::{
    groups::{GroupInfo, GroupMemberInfo},
    message::{BorrowedHeaders, BorrowedMessage},
    metadata::{MetadataBroker, MetadataPartition, MetadataTopic},
    Message,
};

#[derive(Debug, Clone)]
pub struct KafkaBroker {
    pub host: String,
    pub port: i32,
}

impl From<&MetadataBroker> for KafkaBroker {
    fn from(broker: &MetadataBroker) -> Self {
        Self {
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
    pub low: i64,
    pub high: i64,
}

impl From<&MetadataPartition> for KafkaPartition {
    fn from(partition: &MetadataPartition) -> Self {
        Self {
            id: partition.id(),
            leader: partition.leader(),
            replicas: partition.replicas().to_vec(),
            isr: partition.isr().to_vec(),
            low: 0,
            high: 0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct KafkaGroup {
    pub name: String,
    pub state: String,
    pub protocol: String,
    pub protocol_type: String,
    pub members: Vec<KafkaGroupMember>,
}

impl From<&GroupInfo> for KafkaGroup {
    fn from(group: &GroupInfo) -> Self {
        let members = group
            .members()
            .into_iter()
            .map(|m| KafkaGroupMember::from(m))
            .collect();

        Self {
            name: group.name().to_string(),
            state: group.state().to_string(),
            protocol: group.protocol().to_string(),
            protocol_type: group.protocol_type().to_string(),
            members,
        }
    }
}

#[derive(Debug, Clone)]
pub struct KafkaGroupMember {
    pub id: String,
    pub client_id: String,
    pub client_host: String,
}

impl From<&GroupMemberInfo> for KafkaGroupMember {
    fn from(member: &GroupMemberInfo) -> Self {
        Self {
            id: member.id().to_string(),
            client_id: member.client_id().to_string(),
            client_host: member.client_host().to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct KafkaMessage {
    // pub topic: String,
    //pub partition: i32,
    pub offset: i64,
    pub payload: String,
    pub key: String,
}

impl<'a> From<BorrowedMessage<'a>> for KafkaMessage {
    fn from(message: BorrowedMessage<'a>) -> Self {
        Self {
            // topic: message.topic().to_string(),
            // partition: message.partition(),
            offset: message.offset(),
            payload: match message.payload_view::<str>() {
                Some(Ok(payload)) => payload.to_string(),
                _ => "Unknown".to_string(),
            },
            key: match message.key_view::<str>() {
                Some(Ok(key)) => key.to_string(),
                _ => "Unknown".to_string(),
            },
        }
    }
}
