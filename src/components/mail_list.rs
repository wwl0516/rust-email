use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
};
use tokio::sync::mpsc::UnboundedSender;

use super::Component;
use crate::action::Action;

#[derive(Default)]
pub struct MailList {
    command_tx: Option<UnboundedSender<Action>>,
    pub mails: Vec<mail_protocol::EmailSummary>,
    pub state: ListState,
    pub current_folder: String,
}

impl MailList {
    pub fn new() -> Self {
        Self {
            state: ListState::default().with_selected(Some(0)),
            ..Self::default()
        }
    }

    pub fn set_mails(&mut self, mails: Vec<mail_protocol::EmailSummary>) {
        self.mails = mails;
        self.state.select(if self.mails.is_empty() {
            None
        } else {
            Some(0)
        });
    }

    pub fn selected_uid(&self) -> Option<u32> {
        self.state
            .selected()
            .and_then(|i| self.mails.get(i))
            .map(|m| m.uid)
    }

    fn next(&mut self) {
        let i = self
            .state
            .selected()
            .map(|i| (i + 1).min(self.mails.len().saturating_sub(1)))
            .unwrap_or(0);
        self.state.select(Some(i));
    }

    fn prev(&mut self) {
        let i = self
            .state
            .selected()
            .map(|i| i.saturating_sub(1))
            .unwrap_or(0);
        self.state.select(Some(i));
    }
}

impl Component for MailList {
    fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> color_eyre::Result<()> {
        self.command_tx = Some(tx);
        Ok(())
    }

    fn handle_key_event(&mut self, key: crossterm::event::KeyEvent) -> color_eyre::Result<Option<Action>> {
        match key.code {
            crossterm::event::KeyCode::Char('j') | crossterm::event::KeyCode::Down => {
                self.next();
            }
            crossterm::event::KeyCode::Char('k') | crossterm::event::KeyCode::Up => {
                self.prev();
            }
            crossterm::event::KeyCode::Enter => {
                if self.selected_uid().is_some() {
                    return Ok(Some(Action::ViewMail));
                }
            }
            _ => {}
        }
        Ok(None)
    }

    fn update(&mut self, action: Action) -> color_eyre::Result<Option<Action>> {
        match action {
            Action::SelectFolder(ref name) => {
                self.current_folder = name.clone();
            }
            _ => {}
        }
        Ok(None)
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect) -> color_eyre::Result<()> {
        let block = Block::default()
            .borders(Borders::ALL)
            .title(format!(" 📨 {} ", self.current_folder))
            .style(Style::default().fg(Color::White));
        let inner = block.inner(area);

        let items: Vec<ListItem> = self
            .mails
            .iter()
            .map(|m| {
                // 已读/未读标识
                let is_seen = m.flags.contains(&mail_protocol::MailFlag::Seen);
                let has_attach = if m.has_attachments { " 📎" } else { "" };

                let flag_icon = if m.flags.contains(&mail_protocol::MailFlag::Flagged) {
                    "★ "
                } else if is_seen {
                    "  "
                } else {
                    "● "
                };

                // 截断过长的内容
                let subject = if m.subject.len() > 40 {
                    format!("{}…", &m.subject[..40])
                } else {
                    m.subject.clone()
                };

                let from = if m.from.len() > 25 {
                    format!("{}…", &m.from[..25])
                } else {
                    m.from.clone()
                };

                let date = &m.date[..m.date.len().min(10)];

                let style = if is_seen {
                    Style::default().fg(Color::DarkGray)
                } else {
                    Style::default().fg(Color::White).add_modifier(Modifier::BOLD)
                };

                ListItem::new(Line::from(vec![
                    Span::styled(flag_icon, Style::default().fg(Color::Yellow)),
                    Span::styled(
                        format!(" {:<25} ", from),
                        style,
                    ),
                    Span::styled(
                        format!(" {:<42} ", subject),
                        style,
                    ),
                    Span::styled(
                        format!(" {:<10}", date),
                        Style::default().fg(Color::DarkGray),
                    ),
                    Span::raw(has_attach),
                ]))
            })
            .collect();

        let list = List::new(items)
            .block(block)
            .highlight_style(
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol("▸ ");

        frame.render_stateful_widget(list, area, &mut self.state);

        if self.mails.is_empty() {
            frame.render_widget(
                Paragraph::new(Text::from("暂无邮件\n按 Esc 返回文件夹列表"))
                    .style(Style::default().fg(Color::DarkGray))
                    .alignment(ratatui::layout::Alignment::Center),
                inner,
            );
        }

        Ok(())
    }
}
