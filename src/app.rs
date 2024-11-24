use crate::kafka::{KafkaBroker, KafkaTopic};
use color_eyre::{eyre::Context, Result};
use crossterm::event;
use ratatui::{
    crossterm::event::{Event, KeyEventKind},
    widgets::ListState,
    DefaultTerminal, Frame,
};
use rdkafka::config::ClientConfig;
use rdkafka::consumer::{BaseConsumer, Consumer};
use rdkafka::metadata::Metadata;
use std::time::Duration;

const TIMEOUT: Duration = Duration::from_secs(10);

pub struct App {
    pub mode: Mode,

    metadata: Metadata,

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
    pub fn new(brokers: Vec<String>) -> Self {
        let consumer: BaseConsumer = ClientConfig::new()
            .set("bootstrap.servers", &brokers.join(","))
            .create()
            .expect("Consumer creation failed");

        let metadata = consumer
            .fetch_metadata(None, TIMEOUT)
            .expect("Failed to fetch metadata");

        Self {
            mode: Mode::default(),
            metadata,
            brokers: Vec::new(),
            topic_list: TopicList::new(),

            input: String::new(),
            character_index: 0,
        }
    }

    pub fn refresh_metadata(&mut self) -> Result<()> {
        self.brokers.clear();
        for broker in self.metadata.brokers() {
            self.brokers.push(KafkaBroker::from(broker));
        }

        self.topic_list.items.clear();
        for topic in self.metadata.topics() {
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
                    self.handle_key_press_input(key);
                } else {
                    self.handle_key_press_normal(key);
                }
            }
            _ => {}
        }
        Ok(())
    }
}
