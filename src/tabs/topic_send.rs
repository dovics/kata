use color_eyre::{eyre::eyre, Result};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier},
    symbols,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Widget},
};
use rdkafka::producer::{BaseProducer, BaseRecord};

use crate::theme::THEME;

use super::topic::TopicPage;
pub struct TopicSendForm {
    field: InputField,
    topic: String,

    partition: String,
    message: String,
    key: String,

    cursor_index: usize,
    err: Option<String>,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
enum InputField {
    #[default]
    Message,
    Key,
    Partition,
}

impl InputField {
    pub fn next(&mut self) {
        *self = match self {
            InputField::Message => InputField::Key,
            InputField::Key => InputField::Partition,
            InputField::Partition => InputField::Message,
        };
    }
}

impl TopicSendForm {
    pub fn new(topic: &str) -> Self {
        Self {
            field: InputField::default(),
            topic: topic.to_string(),
            message: String::new(),
            key: String::new(),
            partition: String::new(),

            cursor_index: 0,
            err: None,
        }
    }

    pub fn render(&self, area: Rect, buf: &mut Buffer) {
        let [key, partition, message, err] = Layout::vertical([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Fill(1),
            Constraint::Max(1),
        ])
        .areas(area);

        let mut render_paragraph = |text: String, title: &str, area: Rect, field: InputField| {
            let block = Block::new()
                .title(Line::raw(title))
                .border_set(symbols::border::ROUNDED)
                .border_style(THEME.borders)
                .borders(Borders::ALL);

            let line = if field == self.field {
                Line::from(if self.cursor_index < text.len() {
                    vec![
                        Span::raw(&text[0..self.cursor_index]).style(THEME.content),
                        Span::raw(&text[self.cursor_index..self.cursor_index + 1])
                            .style(THEME.content.bg(Color::White)),
                        Span::raw(&text[self.cursor_index + 1..]).style(THEME.content),
                    ]
                } else {
                    vec![
                        Span::raw(text.clone()).style(THEME.content),
                        Span::raw(" ").style(THEME.content.bg(Color::White)),
                    ]
                })
                .style(THEME.content.add_modifier(Modifier::UNDERLINED))
            } else {
                Line::from(vec![Span::raw(text.clone()).style(THEME.content)])
            };

            let paragraph = Paragraph::new(line).block(block);

            paragraph.render(area, buf);
        };

        render_paragraph(
            self.message.clone(),
            "Message",
            message,
            InputField::Message,
        );
        render_paragraph(self.key.clone(), "Key", key, InputField::Key);
        render_paragraph(
            self.partition.clone(),
            "Partition",
            partition,
            InputField::Partition,
        );

        if let Some(err_message) = &self.err {
            Paragraph::new(Line::from(vec![Span::raw(err_message).style(THEME.error)]))
                .render(err, buf);
        }
    }
}

impl TopicSendForm {
    pub fn handle_key_press(
        &mut self,
        key: KeyEvent,
        producer: &BaseProducer,
    ) -> Result<TopicPage> {
        match key.code {
            KeyCode::Enter => match self.field {
                InputField::Message => {
                    if let Err(_) = self.submit(producer) {
                        return Ok(TopicPage::SendEdit);
                    }
                    return Ok(TopicPage::Messages);
                }
                _ => self.change_field(),
            },
            KeyCode::Tab => self.change_field(),
            KeyCode::Esc => {
                return Ok(TopicPage::Send);
            }
            KeyCode::Left => self.move_cursor_left(),
            KeyCode::Right => self.move_cursor_right(),
            KeyCode::Char(c) => self.enter_char(c),
            KeyCode::Backspace => self.delete_char(),
            _ => {}
        }
        Ok(TopicPage::SendEdit)
    }

    fn change_field(&mut self) {
        self.field.next();
        self.cursor_index = match self.field {
            InputField::Message => self.message.len(),
            InputField::Key => self.key.len(),
            InputField::Partition => self.partition.len(),
        };
    }

    fn move_cursor_left(&mut self) {
        let cursor_moved_left = self.cursor_index.saturating_sub(1);
        self.cursor_index = cursor_moved_left;
    }

    fn move_cursor_right(&mut self) {
        let cursor_moved_right = self.cursor_index.saturating_add(1);
        self.cursor_index = cursor_moved_right;
    }

    fn enter_char(&mut self, c: char) {
        match self.field {
            InputField::Message => {
                self.message.insert(self.cursor_index, c);
            }
            InputField::Key => {
                self.key.insert(self.cursor_index, c);
            }
            InputField::Partition => {
                if !c.is_digit(10) {
                    return;
                }
                self.partition.insert(self.cursor_index, c);
            }
        };
        self.move_cursor_right();
    }

    fn delete_char(&mut self) {
        if self.cursor_index == 0 {
            return;
        }

        match self.field {
            InputField::Message => {
                self.message.remove(self.cursor_index - 1);
            }
            InputField::Key => {
                self.key.remove(self.cursor_index - 1);
            }
            InputField::Partition => {
                self.partition.remove(self.cursor_index - 1);
            }
        }
        self.move_cursor_left();
    }

    pub fn submit(&mut self, producer: &BaseProducer) -> Result<()> {
        if self.message.is_empty() {
            return Ok(());
        }

        let topic = self.topic.clone();
        let mut record = BaseRecord::to(&topic).payload(self.message.as_bytes());

        if !self.key.is_empty() {
            record = record.key(self.key.as_str());
        }

        if !self.partition.is_empty() {
            record = record.partition(self.partition.parse().unwrap());
        }

        if let Err((e, _)) = producer.send(record) {
            self.err = Some(e.to_string());
            return Err(eyre!(e));
        } else {
            self.empty();
            Ok(())
        }
    }

    pub fn empty(&mut self) {
        self.message.clear();
        self.key.clear();
        self.partition.clear();
        self.cursor_index = 0;
    }
}
