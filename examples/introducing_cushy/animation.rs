use std::time::Duration;

use cushy::animation::easings::EaseInOutQuadradic;
use cushy::animation::{AnimationHandle, AnimationTarget, Spawn};
use cushy::styles::Color;
use cushy::value::{Dynamic, Source};
use cushy::widget::MakeWidget;
use cushy::widgets::Space;
use rand::{thread_rng, Rng};

fn random_color() -> Color {
    let mut rng = thread_rng();
    Color::new(rng.gen(), rng.gen(), rng.gen(), 255)
}

// ANCHOR_START
pub fn animation() -> impl MakeWidget {
    let color = Dynamic::new(random_color());
    let hex = color.map_each(|color| format!("{color:?}"));

    "Generate Color"
        .into_button()
        .on_click({
            let color = color.clone();
            let mut _animation = AnimationHandle::default();
            move |()| {
                _animation = color
                    .transition_to(random_color())
                    .over(Duration::from_secs(1))
                    .with_easing(EaseInOutQuadradic)
                    .spawn();
            }
        })
        .and(Space::colored(color).expand())
        .and(hex)
        .into_rows()
}
// ANCHOR_END
