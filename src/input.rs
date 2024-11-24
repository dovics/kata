use color_eyre::Result;
use ratatui::{
    buffer::Buffer,
    crossterm::event::{KeyCode, KeyEvent},
    layout::{Constraint, Flex, Layout, Position, Rect},
    style::{Modifier, Style},
    symbols::border,
    text::{Line, Span, Text},
    widgets::{Block, Clear, Paragraph, Widget},
    Frame,
};

use crate::{
    app::{App, Mode},
    theme::THEME,
};

impl App {
    pub fn render_input_modal(&mut self, frame: &mut Frame) {
        let area = frame.area();
        let popup_area = popup_area(area, 60, 40);
        frame.render_widget(Clear::default(), popup_area);
        frame.render_widget(
            Block::bordered()
                .title("Input")
                .border_set(border::ROUNDED)
                .border_style(THEME.borders),
            popup_area,
        );
        let [input_area, help_area] =
            Layout::vertical([Constraint::Fill(1), Constraint::Length(1)])
                .margin(1)
                .areas(popup_area);

        frame.set_cursor_position(Position::new(
            input_area.x + self.character_index as u16,
            input_area.y,
        ));

        self.render_input(input_area, frame.buffer_mut());
        self.render_help_message(help_area, frame.buffer_mut());
    }

    fn render_input(&mut self, area: Rect, buffer: &mut Buffer) {
        let widget = if self.topic_list.state.selected().is_none() {
            Paragraph::new("Please select a topic first")
                .style(THEME.error)
                .centered()
        } else {
            Paragraph::new(self.input.as_str()).style(THEME.content)
        };

        Widget::render(widget, area, buffer);
    }

    fn render_help_message(&mut self, area: Rect, buffer: &mut Buffer) {
        let msg = vec![
            Span::raw("Press "),
            Span::styled("Esc", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(" to stop editing, "),
            Span::styled("Enter", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(" to submit"),
        ];
        let help_message = Paragraph::new(Text::from(Line::from(msg))).style(THEME.content);
        Widget::render(help_message, area, buffer);
    }
}

impl App {
    pub fn handle_key_press_input(&mut self, key: KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Esc => self.mode = Mode::Normal,
            KeyCode::Enter => self.submit()?,
            KeyCode::Char(to_insert) => self.enter_char(to_insert),
            KeyCode::Backspace => self.delete_char(),
            KeyCode::Left => self.move_cursor_left(),
            KeyCode::Right => self.move_cursor_right(),
            _ => {}
        }
        Ok(())
    }

    fn move_cursor_left(&mut self) {
        let cursor_moved_left = self.character_index.saturating_sub(1);
        self.character_index = self.clamp_cursor(cursor_moved_left);
    }

    fn move_cursor_right(&mut self) {
        let cursor_moved_right = self.character_index.saturating_add(1);
        self.character_index = self.clamp_cursor(cursor_moved_right);
    }

    fn enter_char(&mut self, new_char: char) {
        let index = self.byte_index();
        self.input.insert(index, new_char);
        self.move_cursor_right();
    }

    fn byte_index(&self) -> usize {
        self.input
            .char_indices()
            .map(|(i, _)| i)
            .nth(self.character_index)
            .unwrap_or(self.input.len())
    }

    fn delete_char(&mut self) {
        let is_not_cursor_leftmost = self.character_index != 0;
        if is_not_cursor_leftmost {
            let current_index = self.character_index;
            let from_left_to_current_index = current_index - 1;

            let before_char_to_delete = self.input.chars().take(from_left_to_current_index);
            let after_char_to_delete = self.input.chars().skip(current_index);

            self.input = before_char_to_delete.chain(after_char_to_delete).collect();
            self.move_cursor_left();
        }
    }

    fn clamp_cursor(&self, new_cursor_pos: usize) -> usize {
        new_cursor_pos.clamp(0, self.input.chars().count())
    }

    fn reset_cursor(&mut self) {
        self.character_index = 0;
    }

    fn submit(&mut self) -> Result<()> {
        self.submit_input()?;
        self.input.clear();
        self.reset_cursor();
        self.mode = Mode::Normal;
        Ok(())
    }
}

fn popup_area(area: Rect, percent_x: u16, percent_y: u16) -> Rect {
    let vertical = Layout::vertical([Constraint::Percentage(percent_y)]).flex(Flex::Center);
    let horizontal = Layout::horizontal([Constraint::Percentage(percent_x)]).flex(Flex::Center);
    let [area] = vertical.areas(area);
    let [area] = horizontal.areas(area);
    area
}
