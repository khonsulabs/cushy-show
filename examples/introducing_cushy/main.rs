use std::borrow::Cow;

use cushy::kludgine::{include_texture, wgpu};
use cushy::styles::components::PrimaryColor;
use cushy_show::{
    code, expand_weighted, fit, h1, h3, h5, hr, hsplit, hstack, list, stack, vsplit, Element,
    LazyWidget, Show, Slide, SlideCount, SlideIndex, SlideMeta,
};

mod animation;
mod color;
mod counter;

fn main() {
    Show::default()
        .with(Slide::new(
            SlideMeta::new("title").with_next_slide("01"),
            stack((
                h1("Introducing Cushy").text_color(PrimaryColor),
                h3("A reactive GUI framework for Rust"),
                h5("Ecton (He/Him)"),
            )),
        ))
        .with(content_slide(
            "01",
            "What is Cushy?",
            Some("02"),
            list((
                "Graphical User Interface for Rust",
                "wgpu-based rendering",
                "No existing widget toolchains",
                "Reactive data model",
                "Fun",
            )),
        ))
        .with(content_slide(
            "02",
            "Basic Example",
            Some("03"),
            hsplit((
                expand_weighted(2, code("rs", include_str!("./counter.rs"))),
                LazyWidget::new(counter::counter),
            )),
        ))
        // ANCHOR_START
        .with(content_slide(
            "03",
            "What is this?",
            Some("04"),
            hsplit((
                list((
                    "cushy-show: Interactive presentations",
                    "Written in ~12 hours",
                    "Powered by Cushy",
                )),
                expand_weighted(
                    3,
                    stack((
                        include_texture!("./idea.png", wgpu::FilterMode::Linear).unwrap(),
                        code("rs", anchored_range(include_str!("./main.rs"))),
                    )),
                ),
            )),
        ))
        // ANCHOR_END
        .with(content_slide(
            "04",
            "Animations",
            Some("05"),
            hsplit((
                expand_weighted(
                    2,
                    code("rs", anchored_range(include_str!("./animation.rs"))),
                ),
                LazyWidget::new(animation::animation),
            )),
        ))
        .with(content_slide(
            "05",
            "Bidirectional Bindings",
            Some("06"),
            hsplit((
                expand_weighted(3, code("rs", anchored_range(include_str!("./color.rs")))),
                LazyWidget::new(color::color_pickers),
            )),
        ))
        .with(content_slide(
            "06",
            "What's next for Cushy?",
            Some("07"),
            list((
                "v0.1: Initial alpha (Dec 18)",
                "v0.2: Multi-window support (Dec 27)",
                "v0.3: Generalized wgpu support, VirtualRecorder (soon)",
            )),
        ))
        .with(content_slide(
            "07",
            "Learn More",
            None,
            stack((
                h1("Questions?"),
                Element::from("https://cushy.rs/").text_color(PrimaryColor),
            )),
        ))
        .present();
}

fn content_slide(
    name: &str,
    title: &str,
    next_slide: Option<&str>,
    contents: impl Into<Element>,
) -> Slide {
    let mut meta = SlideMeta::new(name);
    if let Some(next_slide) = next_slide {
        meta = meta.with_next_slide(next_slide);
    }
    Slide::new(
        meta,
        vsplit((
            fit(stack((h3(title.to_string()).left_aligned(), hr()))),
            contents,
            fit(stack((
                hr(),
                hsplit((
                    fit(h5("Introducing Cushy")),
                    "",
                    fit(hstack((SlideIndex, "/", SlideCount))),
                )),
            ))),
        )),
    )
}

fn leading_whitespace(source: &str) -> &str {
    let mut common_whitepsace = "";
    for (index, ch) in source.char_indices() {
        if ch.is_ascii_whitespace() {
            common_whitepsace = &source[..index + ch.len_utf8()];
        } else {
            break;
        }
    }
    common_whitepsace
}

fn anchored_range(source: &str) -> Cow<'_, str> {
    let (_, anchored) = source
        .split_once("// ANCHOR_START")
        .expect("missing anchor start");
    let (anchored, _) = anchored
        .split_once("// ANCHOR_END")
        .expect("missing anchor end");
    let anchored = anchored.trim_start_matches(['\r', '\n']).trim_end();
    let mut common_whitespace = leading_whitespace(anchored);
    let mut line_count = 1;
    if !common_whitespace.is_empty() {
        for line in anchored.lines().skip(1) {
            line_count += 1;
            let ws = leading_whitespace(&line[..line.len() - 1]);
            if let Some(matching_until) = common_whitespace
                .bytes()
                .zip(ws.bytes())
                .enumerate()
                .find_map(|(index, (a, b))| (a != b).then_some(index))
            {
                common_whitespace = &common_whitespace[..matching_until];
                if common_whitespace.is_empty() {
                    break;
                }
            }
        }
    }

    if common_whitespace.is_empty() {
        Cow::Borrowed(anchored)
    } else {
        let mut trimmed =
            String::with_capacity(source.len() - line_count * common_whitespace.len());
        for line in anchored.lines() {
            trimmed.push_str(&line[common_whitespace.len()..]);
            trimmed.push('\n');
        }
        Cow::Owned(trimmed)
    }
}
