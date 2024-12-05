use crate::{
    tabs::{BrokerTab, GroupTab, Tab, TopicTab},
    theme::THEME,
};
use color_eyre::{eyre::Context, Result};
use ratatui::{
    buffer::Buffer,
    crossterm::event::{Event, EventStream, KeyEventKind},
    layout::{Constraint, Layout, Rect},
    style::Color,
    text::Line,
    widgets::{Paragraph, Tabs, Widget},
    DefaultTerminal, Frame,
};

use futures::StreamExt;
use rdkafka::{
    admin::AdminClient, client::DefaultClientContext, config::ClientConfig, consumer::BaseConsumer,
    producer::BaseProducer,
};
use std::time::Duration;
use strum::IntoEnumIterator;

pub struct App {
    mode: Mode,
    pub tab: Tab,
    admin: AdminClient<DefaultClientContext>,
    consumer: BaseConsumer,
    producer: BaseProducer,

    broker_tab: BrokerTab,
    group_tab: GroupTab,
    topic_tab: TopicTab,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    #[default]
    TabChoose,
    Tab,
    Quit,
    Refresh,
}

impl App {
    const FRAMES_PER_SECOND: f32 = 60.0;

    pub fn new(brokers: Vec<String>) -> Result<Self> {
        let mut config = ClientConfig::new();
        let config = config.set("bootstrap.servers", &brokers.join(","));
        let consumer: BaseConsumer = config.create().wrap_err("Consumer creation failed")?;
        let producer: BaseProducer = config.create().wrap_err("Producer creation failed")?;
        let admin = config
            .create::<AdminClient<DefaultClientContext>>()
            .wrap_err("Admin creation failed")?;

        let topic_tab = TopicTab::new();
        let broker_tab = BrokerTab::new();
        let group_tab = GroupTab::new();
        Ok(Self {
            mode: Mode::default(),
            tab: Tab::default(),
            consumer,
            producer,
            admin,
            topic_tab,
            broker_tab,
            group_tab,
        })
    }

    pub fn refresh_matadata(&mut self) -> Result<()> {
        let tabs = if self.mode == Mode::Tab {
            vec![self.tab]
        } else {
            Tab::iter().collect()
        };

        for tab in tabs {
            match tab {
                Tab::Topic => self.topic_tab.refresh_matadata(&self.consumer),
                Tab::Group => self.group_tab.refresh_matadata(&self.consumer)?,
                Tab::Broker => self.broker_tab.refresh_matadata(&self.consumer)?,
            }
        }
        Ok(())
    }

    pub async fn run(mut self, mut terminal: DefaultTerminal) -> Result<()> {
        self.refresh_matadata()?;

        let period = Duration::from_secs_f32(1.0 / Self::FRAMES_PER_SECOND);
        let mut interval = tokio::time::interval(period);
        let mut events = EventStream::new();

        while self.is_running() {
            tokio::select! {
                _ = interval.tick() => {
                    terminal.draw(|frame| self.draw(frame))?;
                }
                Some(Ok(event)) = events.next() => self.handle_event(&event).await?,
            }
        }
        Ok(())
    }

    fn is_running(&self) -> bool {
        self.mode != Mode::Quit
    }

    fn draw(&mut self, frame: &mut Frame) {
        let area = frame.area();
        let buf: &mut Buffer = frame.buffer_mut();
        let [title_bar, main_area, bottom_bar] = Layout::vertical([
            Constraint::Length(2),
            Constraint::Fill(1),
            Constraint::Length(1),
        ])
        .areas(area);

        self.render_title_bar(title_bar, buf);
        self.render_bottom_bar(bottom_bar, buf);

        match self.tab {
            Tab::Topic => self.topic_tab.render(main_area, buf),
            Tab::Group => self.group_tab.render(main_area, buf),
            Tab::Broker => self.broker_tab.render(main_area, buf),
        }
    }

    async fn handle_event(&mut self, event: &Event) -> Result<()> {
        self.mode = match event {
            Event::Key(key) if key.kind == KeyEventKind::Press => match self.mode {
                Mode::TabChoose => self.handle_tab_select(key)?,
                Mode::Tab => match self.tab {
                    Tab::Topic => {
                        self.topic_tab
                            .handle_key_press(key, &self.producer, &self.admin)
                            .await?
                    }
                    Tab::Group => self.group_tab.handle_key_press(key)?,
                    Tab::Broker => self.broker_tab.handle_key_press(key)?,
                },
                _ => self.mode,
            },
            _ => self.mode,
        };

        Ok(())
    }

    fn render_title_bar(&mut self, area: Rect, buf: &mut Buffer) {
        let [title, tabs] =
            Layout::vertical([Constraint::Length(1), Constraint::Length(1)]).areas(area);
        Paragraph::new("Kafka TUI")
            .style(THEME.app_title)
            .centered()
            .render(title, buf);

        let tab_titles = Tab::iter().map(Tab::title);
        Tabs::new(tab_titles)
            .style(THEME.tabs)
            .highlight_style(if self.mode == Mode::TabChoose {
                THEME.tabs_selected
            } else {
                THEME.tabs
            })
            .select(self.tab as usize)
            .divider(" ")
            .padding("", "")
            .render(tabs, buf);
    }

    fn render_bottom_bar(&mut self, area: Rect, buf: &mut Buffer) {
        let spans = match self.tab {
            Tab::Topic => self.topic_tab.bottom_bar_spans(),
            _ => vec![],
        };

        Line::from(spans)
            .centered()
            .style((Color::Indexed(236), Color::Indexed(232)))
            .render(area, buf);
    }
}
