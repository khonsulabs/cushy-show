use std::borrow::Cow;

use cushy::figures::units::{Px, UPx};
use cushy::figures::{IntoUnsigned, Point, ScreenScale, Size, Zero};
use cushy::kludgine::text::{MeasuredText, Text, TextOrigin};
use cushy::kludgine::DrawableExt;
use cushy::styles::components::TextSize;
use cushy::styles::{Color, FamilyOwned, Style, Weight};
use cushy::widget::Widget;
use syntect::easy::HighlightLines;
use syntect::highlighting::{FontStyle, Theme};
use syntect::parsing::SyntaxSet;
use syntect::util::LinesWithEndings;

#[derive(Debug)]
pub struct CodeView {
    extension: Cow<'static, str>,
    syntax_set: SyntaxSet,
    theme: Theme,
    source: Cow<'static, str>,
    measured_lines: Vec<Vec<MeasuredText<Px>>>,
    size: Size<UPx>,
    line_height: Px,
    cached_text_size: Px,
}

impl CodeView {
    pub fn new(
        extension: impl Into<Cow<'static, str>>,
        syntax_set: SyntaxSet,
        theme: Theme,
        source: impl Into<Cow<'static, str>>,
    ) -> Self {
        Self {
            extension: extension.into(),
            syntax_set,
            theme,
            source: source.into(),
            measured_lines: Vec::new(),
            size: Size::ZERO,
            line_height: Px::ZERO,
            cached_text_size: Px::ZERO,
        }
    }

    fn highlight(&mut self, context: &mut cushy::context::LayoutContext<'_, '_, '_, '_>) {
        let text_size = context.get(&TextSize).into_px(context.gfx.scale());
        if (text_size != self.cached_text_size || self.measured_lines.is_empty())
            && !self.source.is_empty()
        {
            self.measured_lines.clear();
            self.cached_text_size = text_size;
            let mut max_x = Px::ZERO;
            let mut y = Px::ZERO;
            let syntax = self
                .syntax_set
                .find_syntax_by_extension(&self.extension)
                .expect("missing syntax definition");
            let mut highlighter = HighlightLines::new(syntax, &self.theme);
            context.gfx.set_font_family(FamilyOwned::Monospace);
            self.line_height = context.gfx.line_height().into_px(context.gfx.scale());
            for line in LinesWithEndings::from(self.source.as_ref()) {
                y += self.line_height;
                let mut spans = Vec::new();
                let mut x = Px::ZERO;
                for (style, text) in highlighter
                    .highlight_line(line, &self.syntax_set)
                    .expect("invalid syntax")
                {
                    if style.font_style.contains(FontStyle::BOLD) {
                        context.gfx.set_font_weight(Weight::BOLD);
                    } else {
                        context.gfx.set_font_weight(Weight::NORMAL);
                    }

                    if style.font_style.contains(FontStyle::ITALIC) {
                        context.gfx.set_font_style(Style::Italic);
                    } else {
                        context.gfx.set_font_style(Style::Normal);
                    }

                    let span = context
                        .gfx
                        .measure_text(Text::new(text, color(style.foreground)));
                    x += span.size.width;
                    spans.push(span);
                }
                self.measured_lines.push(spans);
                max_x = max_x.max(x);
            }

            self.size = Size::new(max_x, y).into_unsigned();
        }
    }
}

fn color(color: syntect::highlighting::Color) -> Color {
    Color::new(color.r, color.g, color.b, color.a)
}

impl Widget for CodeView {
    fn redraw(&mut self, context: &mut cushy::context::GraphicsContext<'_, '_, '_, '_>) {
        let mut y = Px::ZERO;
        for line in &self.measured_lines {
            let mut x = Px::ZERO;
            for span in line {
                context
                    .gfx
                    .draw_measured_text(span.translate_by(Point::new(x, y)), TextOrigin::TopLeft);
                x += span.size.width;
            }
            y += self.line_height;
        }
    }

    fn layout(
        &mut self,
        _available_space: cushy::figures::Size<cushy::ConstraintLimit>,
        context: &mut cushy::context::LayoutContext<'_, '_, '_, '_>,
    ) -> cushy::figures::Size<cushy::figures::units::UPx> {
        self.highlight(context);
        self.size
    }
}
