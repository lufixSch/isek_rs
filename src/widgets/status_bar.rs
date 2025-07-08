use ratatui::{
    text::{Line, Span},
    widgets::{Block, Padding, Paragraph, Widget},
};

/// StatusBar Widget
#[derive(Default)]
pub struct StatusBar<'a> {
    block: Block<'a>,

    keybinds: Vec<(String, String)>,
}

impl<'a> StatusBar<'a> {
    pub fn new(keybinds: Vec<(String, String)>) -> Self {
        Self {
            keybinds,
            block: Block::new().padding(Padding::symmetric(2, 0)),
        }
    }

    /// Configure the block (border and title) for this widget
    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = block;
        self
    }
}

impl Widget for StatusBar<'_> {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer) {
        let line = Line::from(
            self.keybinds
                .iter()
                .map(|(key, description)| format!("<{}> {}  ", key, description).into())
                .collect::<Vec<Span>>(),
        );

        Paragraph::new(line).block(self.block).render(area, buf);
    }
}
