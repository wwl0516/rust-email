use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph, Wrap},
};
use tokio::sync::mpsc::UnboundedSender;

use super::Component;
use crate::action::Action;

#[derive(Default)]
pub struct MailView {
    command_tx: Option<UnboundedSender<Action>>,
    pub mail: Option<mail_protocol::Email>,
    pub current_folder: String,
    show_html: bool,
    scroll: u16,
}

impl MailView {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_mail(&mut self, mail: mail_protocol::Email) {
        self.mail = Some(mail);
        self.scroll = 0;
    }

    fn scroll_down(&mut self) {
        self.scroll = self.scroll.saturating_add(1);
    }

    fn scroll_up(&mut self) {
        self.scroll = self.scroll.saturating_sub(1);
    }
}

impl Component for MailView {
    fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> color_eyre::Result<()> {
        self.command_tx = Some(tx);
        Ok(())
    }

    fn handle_key_event(&mut self, key: crossterm::event::KeyEvent) -> color_eyre::Result<Option<Action>> {
        match key.code {
            crossterm::event::KeyCode::Char('j') | crossterm::event::KeyCode::Down => {
                self.scroll_down();
            }
            crossterm::event::KeyCode::Char('k') | crossterm::event::KeyCode::Up => {
                self.scroll_up();
            }
            crossterm::event::KeyCode::Char('h') => {
                self.show_html = true;
            }
            crossterm::event::KeyCode::Char('t') => {
                self.show_html = false;
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
        let Some(ref mail) = self.mail else {
            frame.render_widget(
                Paragraph::new(Text::from("未选择邮件"))
                    .style(Style::default().fg(Color::DarkGray))
                    .alignment(ratatui::layout::Alignment::Center),
                area,
            );
            return Ok(());
        };

        // 上下分区：邮件头 + 正文
        let chunks: [Rect; 2] = Layout::vertical([
            Constraint::Length(7),  // 邮件头
            Constraint::Min(1),     // 正文
        ])
        .areas(area);
        let (header_area, body_area) = (chunks[0], chunks[1]);

        // ── 邮件头 ──
        let header_block = Block::default()
            .borders(Borders::ALL)
            .title(format!(" 📄 {} ", mail.subject))
            .style(Style::default().fg(Color::Cyan));

        let from_display = if !mail.from.is_empty() {
            mail.from.clone()
        } else {
            "(未知发件人)".to_string()
        };

        let to_display = if mail.to.is_empty() {
            "(无收件人)".to_string()
        } else {
            mail.to.join("; ")
        };

        let header_text = Text::from(vec![
            Line::from(Span::styled(
                format!(" 发件人: {from_display}"),
                Style::default().fg(Color::White),
            )),
            Line::from(Span::styled(
                format!(" 收件人: {to_display}"),
                Style::default().fg(Color::White),
            )),
            Line::from(Span::styled(
                format!(" 日  期: {}", mail.date),
                Style::default().fg(Color::DarkGray),
            )),
        ]);

        frame.render_widget(
            Paragraph::new(header_text)
                .block(header_block)
                .wrap(Wrap { trim: false }),
            header_area,
        );

        // ── 正文 ──
        let body_content = if self.show_html {
            mail.body_html.as_deref().unwrap_or("(无 HTML 正文)")
        } else {
            mail.body_text.as_deref().unwrap_or("(无纯文本正文)")
        };

        let body_block = Block::default()
            .borders(Borders::ALL)
            .title(if self.show_html { " 🌐 HTML" } else { " 📝 纯文本" });

        frame.render_widget(
            Paragraph::new(body_content)
                .block(body_block)
                .wrap(Wrap { trim: false })
                .scroll((self.scroll, 0)),
            body_area,
        );

        Ok(())
    }
}
