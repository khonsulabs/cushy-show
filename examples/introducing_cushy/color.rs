use cushy::figures;
use cushy::styles::{Color, Hsla};
use cushy::value::{Dynamic, Source};
use cushy::widget::MakeWidget;
use cushy::widgets::color::{HslaPicker, RgbaPicker};
use cushy::widgets::Space;
use figures::units::Lp;
use figures::Size;

// ANCHOR_START
pub fn color_pickers() -> impl MakeWidget {
    let color = Dynamic::new(Color::RED);
    let color_as_string = color.map_each(|color| format!("{color:?}"));

    let hsl = color.linked(|color| Hsla::from(*color), |hsl| Color::from(*hsl));

    "HSLa Picker"
        .and(HslaPicker::new(hsl).expand())
        .and("RGBa Picker")
        .and(RgbaPicker::new(color.clone()))
        .into_rows()
        .expand()
        .and(
            "Picked Color"
                .and(Space::colored(color).size(Size::squared(Lp::inches(1))))
                .and(color_as_string)
                .into_rows()
                .fit_horizontally(),
        )
        .into_columns()
}
// ANCHOR_END
