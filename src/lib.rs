use std::collections::HashMap;

use cushy::animation::ZeroToOne;
use cushy::context::{EventContext, LayoutContext};
use cushy::figures::units::Px;
use cushy::figures::{FloatConversion, Round, Size};
use cushy::kludgine::app::winit::keyboard::{Key, NamedKey};
use cushy::kludgine::LazyTexture;
use cushy::styles::components::{
    BaseLineHeight, BaseTextSize, IntrinsicPadding, PrimaryColor, TextColor,
};
use cushy::styles::{Color, Dimension, Styles, Theme, ThemePair};
use cushy::value::{Destination, Dynamic, Source, Switchable};
use cushy::widget::{
    EventHandling, MakeWidget, WidgetInstance, WidgetList, WidgetRef, WrappedLayout, WrapperWidget,
    HANDLED, IGNORED,
};
use cushy::widgets::grid::Orientation;
use cushy::widgets::{Delimiter, Image, Label, Stack};
use cushy::window::{DeviceId, KeyEvent};
use cushy::{ConstraintLimit, Run};
use syntect::highlighting::ThemeSet;
use syntect::parsing::SyntaxSet;

pub struct ShowSettings {}

#[derive(Default)]
pub struct Show {
    first_slide: String,
    slides: HashMap<String, Slide>,
}

impl Show {
    pub fn with(mut self, slide: impl Into<Slide>) -> Self {
        let slide = slide.into();
        if self.first_slide.is_empty() {
            self.first_slide = slide.meta.path.clone();
        }
        self.push(slide);
        self
    }

    pub fn push(&mut self, mut slide: Slide) {
        slide.meta.index = self.slides.len();
        self.slides.insert(slide.meta.path.clone(), slide);
    }

    pub fn present(self) {
        let theme = ThemePair::default().dark;
        self.present_themed(theme)
    }

    pub fn present_themed(self, theme: Theme) {
        let current_slide = Dynamic::new(self.first_slide);
        let next_slide = Dynamic::default();
        let slide_history = Dynamic::default();
        let default_text_color = theme.surface.on_color;
        SlideSurface {
            next_slide: next_slide.clone(),
            current_slide: current_slide.clone(),
            slide_history: slide_history.clone(),
            contents: WidgetRef::new(current_slide.switcher(move |slide, _dynamic| {
                self.slides
                    .get(slide)
                    .map(|slide| {
                        slide
                            .present(&Context {
                                next_slide: &next_slide,
                                align: HAlign::Center,
                                theme: &theme,
                                color: default_text_color.into(),
                                slide_index: slide.meta.index,
                                slide_count: self.slides.len(),
                            })
                            .make_widget()
                    })
                    .unwrap_or_else(|| format!("unknown slide: {slide}").centered().make_widget())
            })),
            styles: Styles::default(),
            base_font_size: Px::ZERO,
        }
        .run()
        .expect("error launching application")
    }
}

pub struct Slide {
    meta: SlideMeta,
    contents: Element,
}

impl Slide {
    pub fn new(meta: impl Into<SlideMeta>, elements: impl Into<Element>) -> Self {
        Self {
            meta: meta.into(),
            contents: elements.into(),
        }
    }

    fn present(&self, context: &Context) -> impl MakeWidget {
        context.next_slide.set(self.meta.next_slide.clone());
        self.contents.make_widget(context)
    }
}

#[derive(Debug)]
struct SlideSurface {
    next_slide: Dynamic<String>,
    contents: WidgetRef,
    current_slide: Dynamic<String>,
    slide_history: Dynamic<Vec<String>>,
    styles: Styles,
    base_font_size: Px,
}

impl WrapperWidget for SlideSurface {
    fn child_mut(&mut self) -> &mut WidgetRef {
        &mut self.contents
    }

    fn hit_test(
        &mut self,
        _location: cushy::figures::Point<Px>,
        _context: &mut EventContext<'_>,
    ) -> bool {
        true
    }

    fn accept_focus(&mut self, _context: &mut EventContext<'_>) -> bool {
        true
    }

    fn mouse_down(
        &mut self,
        _location: cushy::figures::Point<Px>,
        _device_id: DeviceId,
        _button: cushy::kludgine::app::winit::event::MouseButton,
        context: &mut EventContext<'_>,
    ) -> EventHandling {
        context.focus();
        HANDLED
    }

    fn keyboard_input(
        &mut self,
        _device_id: DeviceId,
        input: KeyEvent,
        _is_synthetic: bool,
        context: &mut EventContext<'_>,
    ) -> EventHandling {
        enum Action {
            Next,
            Previous,
        }
        let action = match input.logical_key {
            Key::Named(NamedKey::ArrowRight | NamedKey::Space | NamedKey::Enter)
                if context.modifiers().state().is_empty() =>
            {
                Some(Action::Next)
            }
            Key::Character(ch)
                if matches!(&*ch, "l" | "n") && context.modifiers().state().is_empty() =>
            {
                Some(Action::Next)
            }
            Key::Named(NamedKey::ArrowLeft | NamedKey::Backspace)
                if context.modifiers().state().is_empty() =>
            {
                Some(Action::Previous)
            }
            Key::Character(ch)
                if matches!(&*ch, "h" | "p") && context.modifiers().state().is_empty() =>
            {
                Some(Action::Previous)
            }
            _ => None,
        };

        match action {
            Some(action) => {
                if input.state.is_pressed() {
                    match action {
                        Action::Next => {
                            let next_slide = self.next_slide.get();
                            if !next_slide.is_empty() {
                                if let Some(previous_slide) = self.current_slide.replace(next_slide)
                                {
                                    self.slide_history
                                        .map_mut(|mut history| history.push(previous_slide));
                                }
                            }
                        }
                        Action::Previous => self.slide_history.map_mut(|mut history| {
                            if let Some(previous_slide) = history.pop() {
                                self.current_slide.set(previous_slide);
                            }
                        }),
                    }
                }
                HANDLED
            }
            None => IGNORED,
        }
    }

    fn adjust_child_constraints(
        &mut self,
        available_space: Size<ConstraintLimit>,
        context: &mut LayoutContext<'_, '_, '_, '_>,
    ) -> Size<ConstraintLimit> {
        let size = context.gfx.size().into_float();
        let width_radio = size.width / 16.;
        let height_ratio = size.height / 9.;
        let min_ratio = width_radio.min(height_ratio);
        let expected_ratio = 120.;
        let base_font_size = Px::from(28. * min_ratio / expected_ratio).ceil();

        if base_font_size != self.base_font_size {
            let base_line_height = Px::from(34. * min_ratio / expected_ratio).ceil();
            let padding = Px::from(10. * min_ratio / expected_ratio).ceil();
            self.styles
                .insert(&BaseTextSize, Dimension::Px(base_font_size));
            self.styles
                .insert(&BaseLineHeight, Dimension::Px(base_line_height));
            self.styles
                .insert(&IntrinsicPadding, Dimension::Px(padding));

            context.attach_styles(self.styles.clone());
            self.base_font_size = base_font_size;
        }

        // let base_font_size =
        available_space.map(|d| ConstraintLimit::Fill(d.max()))
    }

    fn position_child(
        &mut self,
        _size: Size<Px>,
        available_space: Size<ConstraintLimit>,
        _context: &mut LayoutContext<'_, '_, '_, '_>,
    ) -> WrappedLayout {
        available_space
            .map(ConstraintLimit::max)
            // Size::new(
            //     available_space
            //         .width
            //         .fit_measured(size.width, context.gfx.scale()),
            //     available_space
            //         .height
            //         .fit_measured(size.height, context.gfx.scale()),
            // )
            .into()
    }
}

#[derive(Clone, Copy, Debug)]
pub enum ElementColor {
    Color(Color),
    Primary,
}

impl ElementColor {
    pub fn resolve(&self, context: &Context) -> Color {
        match self {
            ElementColor::Color(color) => *color,
            ElementColor::Primary => context.theme.primary.color,
        }
    }
}

impl From<Color> for ElementColor {
    fn from(value: Color) -> Self {
        Self::Color(value)
    }
}

impl From<PrimaryColor> for ElementColor {
    fn from(_value: PrimaryColor) -> Self {
        ElementColor::Primary
    }
}

#[derive(Clone)]
pub struct Context<'a> {
    next_slide: &'a Dynamic<String>,
    align: HAlign,
    theme: &'a Theme,
    color: ElementColor,
    slide_index: usize,
    slide_count: usize,
}

pub struct SlideMeta {
    path: String,
    index: usize,
    next_slide: String,
}

impl SlideMeta {
    pub fn new(path: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            next_slide: String::new(),
            index: usize::MAX,
        }
    }

    pub fn with_next_slide(mut self, next_slide: impl Into<String>) -> Self {
        self.next_slide = next_slide.into();
        self
    }

    pub fn path(&self) -> &str {
        &self.path
    }
}

impl From<String> for SlideMeta {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

impl<'a> From<&'a String> for SlideMeta {
    fn from(value: &'a String) -> Self {
        Self::new(value)
    }
}

impl<'a> From<&'a str> for SlideMeta {
    fn from(value: &'a str) -> Self {
        Self::new(value)
    }
}

pub struct Element {
    pub kind: Box<dyn SlideElement>,
    pub align: Option<HAlign>,
    pub color: Option<ElementColor>,
    pub attrs: HashMap<String, String>,
}

pub trait SlideElement: Send + 'static {
    fn make_widget(&self, context: &Context) -> WidgetInstance;
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum HAlign {
    Left,
    Center,
    Right,
    Fill,
}

impl Element {
    pub fn new(kind: impl SlideElement) -> Self {
        Self {
            kind: Box::new(kind),
            align: None,
            color: None,
            attrs: HashMap::new(),
        }
    }

    pub fn centered(mut self) -> Self {
        self.align = Some(HAlign::Center);
        self
    }

    pub fn left_aligned(mut self) -> Self {
        self.align = Some(HAlign::Left);
        self
    }

    pub fn right_aligned(mut self) -> Self {
        self.align = Some(HAlign::Right);
        self
    }

    pub fn fill(mut self) -> Self {
        self.align = Some(HAlign::Fill);
        self
    }

    pub fn text_color(mut self, color: impl Into<ElementColor>) -> Self {
        self.color = Some(color.into());
        self
    }

    fn make_widget(&self, context: &Context) -> WidgetInstance {
        let mut context = context.clone();
        context.align = self.align.unwrap_or(context.align);
        if let Some(color) = self.color {
            context.color = color;
        }

        let widget = self.kind.make_widget(&context);

        match context.align {
            HAlign::Left => widget.align_left().make_widget(),
            HAlign::Center => widget.centered().make_widget(),
            HAlign::Right => widget.align_right().make_widget(),
            HAlign::Fill => widget,
        }
    }
}

impl<T> From<T> for Element
where
    T: SlideElement,
{
    fn from(value: T) -> Self {
        Self::new(value)
    }
}

impl SlideElement for LazyTexture {
    fn make_widget(&self, _context: &Context) -> WidgetInstance {
        Image::new(self.clone())
            .aspect_fit_around(Size::squared(ZeroToOne::new(0.5)))
            .make_widget()
    }
}

pub struct Text(pub String);

impl SlideElement for Text {
    fn make_widget(&self, context: &Context) -> WidgetInstance {
        self.0
            .clone()
            .with(&TextColor, context.color.resolve(context))
            .make_widget()
    }
}

macro_rules! impl_heading {
    ($type:ident, $fn:ident) => {
        pub struct $type(Element);

        impl SlideElement for $type {
            fn make_widget(&self, context: &Context) -> WidgetInstance {
                self.0.make_widget(context).$fn().make_widget()
            }
        }

        pub fn $fn(contents: impl Into<Element>) -> Element {
            $type(contents.into()).into()
        }
    };
}

impl_heading!(H1, h1);
impl_heading!(H2, h2);
impl_heading!(H3, h3);
impl_heading!(H4, h4);
impl_heading!(H5, h5);
impl_heading!(H6, h6);

pub struct Split {
    elements: Vec<SplitElement>,
    orientation: Orientation,
}

impl SlideElement for Split {
    fn make_widget(&self, context: &Context) -> WidgetInstance {
        let mut expand = false;
        let widgets = self
            .elements
            .iter()
            .map(|e| {
                expand |= matches!(e.measurement, SplitMeasurement::Expand { .. });

                let widget = e.element.make_widget(context);
                match &e.measurement {
                    SplitMeasurement::Fit => widget,
                    SplitMeasurement::Expand { weight } => {
                        expand = true;
                        if *weight == 1 {
                            match self.orientation {
                                Orientation::Row => widget.expand_vertically(),
                                Orientation::Column => widget.expand_horizontally(),
                            }
                        } else {
                            widget.expand_weighted(*weight)
                        }
                        .make_widget()
                    }
                }
            })
            .collect::<WidgetList>();
        let stack = match self.orientation {
            Orientation::Row => Stack::rows(widgets),
            Orientation::Column => Stack::columns(widgets),
        };
        // if expand {
        //     match self.orientation {
        //         Orientation::Row => stack.expand_vertically(),
        //         Orientation::Column => stack.expand_horizontally(),
        //     }
        //     .make_widget()
        // } else {
        stack.make_widget()
        // }
    }
}

pub struct SplitElement {
    pub element: Element,
    pub measurement: SplitMeasurement,
}

pub enum SplitMeasurement {
    Fit,
    Expand { weight: u8 },
}

pub fn hsplit(elements: impl SplitElements) -> Element {
    Split {
        orientation: Orientation::Column,
        elements: elements.into_elements(),
    }
    .into()
}

pub fn vsplit(elements: impl SplitElements) -> Element {
    Split {
        orientation: Orientation::Row,
        elements: elements.into_elements(),
    }
    .into()
}

pub trait SplitElements {
    fn into_elements(self) -> Vec<SplitElement>;
}

// impl<T> SplitElements for T
// where
//     T: IntoSplitElement,
// {
//     fn into_elements(self) -> Vec<SplitElement> {
//         vec![self.into_split_element()]
//     }
// }

impl SplitElements for Vec<SplitElement> {
    fn into_elements(self) -> Vec<SplitElement> {
        self
    }
}

macro_rules! impl_all_tuples {
    ($macro_name:ident) => {
        impl_all_tuples!($macro_name, 1);
    };
    ($macro_name:ident, 1) => {
        $macro_name!(T0 0 t0);
        $macro_name!(T0 0 t0, T1 1 t1);
        $macro_name!(T0 0 t0, T1 1 t1, T2 2 t2);
        $macro_name!(T0 0 t0, T1 1 t1, T2 2 t2, T3 3 t3);
        $macro_name!(T0 0 t0, T1 1 t1, T2 2 t2, T3 3 t3, T4 4 t4);
        $macro_name!(T0 0 t0, T1 1 t1, T2 2 t2, T3 3 t3, T4 4 t4, T5 5 t5);
    };
}

macro_rules! impl_split_elements_for_tuples {
    ($($type:ident $field:tt $var:ident),+) => {
        impl<$($type),+> SplitElements for ($($type,)+)
        where
            $($type: IntoSplitElement),+
        {
            fn into_elements(self) -> Vec<SplitElement> {
                vec![$(self.$field.into_split_element()),+]
            }
        }
    };
}

impl_all_tuples!(impl_split_elements_for_tuples);

pub trait IntoSplitElement {
    fn into_split_element(self) -> SplitElement;
}

impl<T> IntoSplitElement for T
where
    T: Into<Element>,
{
    fn into_split_element(self) -> SplitElement {
        SplitElement {
            element: self.into(),
            measurement: SplitMeasurement::Expand { weight: 1 },
        }
    }
}

impl IntoSplitElement for SplitElement {
    fn into_split_element(self) -> SplitElement {
        self
    }
}

pub fn fit(element: impl Into<Element>) -> SplitElement {
    SplitElement {
        element: element.into(),
        measurement: SplitMeasurement::Fit,
    }
}

pub fn expand(element: impl Into<Element>) -> SplitElement {
    expand_weighted(1, element)
}

pub fn expand_weighted(weight: u8, element: impl Into<Element>) -> SplitElement {
    SplitElement {
        element: element.into(),
        measurement: SplitMeasurement::Expand { weight },
    }
}

pub fn stack(elements: impl Elements) -> Element {
    vsplit(
        elements
            .into_elements()
            .into_iter()
            .map(fit)
            .collect::<Vec<_>>(),
    )
}

pub fn hstack(elements: impl Elements) -> Element {
    hsplit(
        elements
            .into_elements()
            .into_iter()
            .map(fit)
            .collect::<Vec<_>>(),
    )
}

pub trait Elements {
    fn into_elements(self) -> Vec<Element>;
}

// impl<T> Elements for T
// where
//     T: Into<Element>,
// {
//     fn into_elements(self) -> Vec<Element> {
//         vec![self.into()]
//     }
// }

impl Elements for Vec<Element> {
    fn into_elements(self) -> Vec<Element> {
        self
    }
}

macro_rules! impl_elements_for_tuples {
    ($($type:ident $field:tt $var:ident),+) => {
        impl<$($type),+> Elements for ($($type,)+)
        where
            $($type: Into<Element>),+
        {
            fn into_elements(self) -> Vec<Element> {
                vec![$(self.$field.into()),+]
            }
        }
    };
}

impl_all_tuples!(impl_elements_for_tuples);

mod code;

struct Code {
    lang: String,
    source: String,
}

impl SlideElement for Code {
    fn make_widget(&self, _context: &Context) -> WidgetInstance {
        let ts = ThemeSet::load_defaults();

        code::CodeView::new(
            self.lang.clone(),
            SyntaxSet::load_defaults_newlines(),
            ts.themes
                .get("base16-mocha.dark")
                .expect("missing theme")
                .clone(),
            self.source.clone(),
        )
        .contain()
        .make_widget()
    }
}

pub fn code(lang: impl Into<String>, source: impl Into<String>) -> Element {
    Code {
        lang: lang.into(),
        source: source.into(),
    }
    .into()
}

struct Group(Element);

impl SlideElement for Group {
    fn make_widget(&self, context: &Context) -> WidgetInstance {
        self.0.make_widget(context).contain().make_widget()
    }
}

pub fn group(elements: impl Elements) -> Element {
    Group(stack(elements)).into()
}

pub struct SlideIndex;

impl SlideElement for SlideIndex {
    fn make_widget(&self, context: &Context) -> WidgetInstance {
        Label::new(context.slide_index + 1).make_widget()
    }
}

pub struct SlideCount;

impl SlideElement for SlideCount {
    fn make_widget(&self, context: &Context) -> WidgetInstance {
        Label::new(context.slide_count).make_widget()
    }
}

struct Hr;

impl SlideElement for Hr {
    fn make_widget(&self, _context: &Context) -> WidgetInstance {
        Delimiter::horizontal().make_widget()
    }
}

pub fn hr() -> Element {
    Hr.into()
}

struct Vr;

impl SlideElement for Vr {
    fn make_widget(&self, _context: &Context) -> WidgetInstance {
        Delimiter::horizontal().make_widget()
    }
}

pub fn vr() -> Element {
    Hr.into()
}

pub struct List {
    elements: Vec<Element>,
}

impl SlideElement for List {
    fn make_widget(&self, context: &Context) -> WidgetInstance {
        self.elements
            .iter()
            .map(|e| e.make_widget(context))
            .collect::<WidgetList>()
            .into_list()
            .make_widget()
    }
}

pub fn list(elements: impl Elements) -> List {
    List {
        elements: elements.into_elements(),
    }
}

impl From<String> for Element {
    fn from(value: String) -> Self {
        Text(value).into()
    }
}

impl<'a> From<&'a String> for Element {
    fn from(value: &'a String) -> Self {
        Self::from(value.clone())
    }
}

impl<'a> From<&'a str> for Element {
    fn from(value: &'a str) -> Self {
        Self::from(value.to_string())
    }
}

#[derive(Clone)]
pub struct LazyWidget<W>(W);

impl<W> LazyWidget<W> {
    pub fn new(function: W) -> Self {
        Self(function)
    }
}

impl<W, MW> SlideElement for LazyWidget<MW>
where
    MW: Fn() -> W + Send + 'static,
    W: MakeWidget,
{
    fn make_widget(&self, _context: &Context) -> WidgetInstance {
        (self.0)().make_widget()
    }
}
