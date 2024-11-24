use crate::kafka::{KafkaBroker, KafkaTopic};
use color_eyre::{
    eyre::{eyre, Context},
    Result,
};
use crossterm::event;
use ratatui::{
    crossterm::event::{Event, KeyEventKind},
    widgets::ListState,
    DefaultTerminal, Frame,
};
use rdkafka::{
    config::ClientConfig,
    consumer::{BaseConsumer, Consumer},
    producer::{BaseProducer, BaseRecord},
};
use std::time::Duration;

const TIMEOUT: Duration = Duration::from_secs(10);

pub struct App {
    pub mode: Mode,

    consumer: BaseConsumer,
    producer: BaseProducer,
    pub brokers: Vec<KafkaBroker>,
    pub topic_list: TopicList,

    pub input: String,
    pub character_index: usize,
}

pub struct TopicList {
    pub items: Vec<KafkaTopic>,
    pub state: ListState,
}

impl TopicList {
    fn new() -> Self {
        let items = Vec::new();
        let state = ListState::default();
        Self { items, state }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    #[default]
    Normal,
    Input,
    Quit,
}

impl App {
    pub fn new(brokers: Vec<String>) -> Result<Self> {
        let mut config = ClientConfig::new();
        let config = config.set("bootstrap.servers", &brokers.join(","));
        let consumer = config.create().wrap_err("Consumer creation failed")?;
        let producer = config.create().wrap_err("Producer creation failed")?;

        Ok(Self {
            mode: Mode::default(),
            consumer,
            producer,
            brokers: Vec::new(),
            topic_list: TopicList::new(),

            input: String::new(),
            character_index: 0,
        })
    }

    pub fn submit_input(&mut self) -> Result<()> {
        let record = BaseRecord::to(self.topic_list.items[0].name.as_str())
            .payload(self.input.as_bytes())
            .key("");

        self.producer
            .send(record)
            .map_err(|(e, _)| eyre!(e))
            .wrap_err("Failed to submit input")?;
        Ok(())
    }

    pub fn refresh_metadata(&mut self) -> Result<()> {
        let metadata = self
            .consumer
            .fetch_metadata(None, TIMEOUT)
            .wrap_err("Failed to fetch metadata")?;

        self.brokers.clear();
        for broker in metadata.brokers() {
            self.brokers.push(KafkaBroker::from(broker));
        }

        self.topic_list.items.clear();
        for topic in metadata.topics() {
            self.topic_list.items.push(KafkaTopic::from(topic));
        }

        Ok(())
    }

    pub fn run(mut self, mut terminal: DefaultTerminal) -> Result<()> {
        self.refresh_metadata()?;
        while self.is_running() {
            terminal
                .draw(|frame| self.draw(frame))
                .wrap_err("terminal.draw")?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn is_running(&self) -> bool {
        self.mode != Mode::Quit
    }

    fn draw(&mut self, frame: &mut Frame) {
        let area = frame.area();
        let buf = frame.buffer_mut();
        self.render_normal_page(area, buf);
        if self.mode == Mode::Input {
            self.render_input_modal(frame);
        }
    }

    fn handle_events(&mut self) -> Result<()> {
        let timeout = Duration::from_secs_f64(1.0 / 50.0);
        if !event::poll(timeout)? {
            return Ok(());
        }
        match event::read()? {
            Event::Key(key) if key.kind == KeyEventKind::Press => {
                if self.mode == Mode::Input {
                    self.handle_key_press_input(key)?;
                } else {
                    self.handle_key_press_normal(key)?;
                }
            }
            _ => {}
        }
        Ok(())
    }
}
