use cushy::value::{Destination, Dynamic, IntoReader, Source};
use cushy::widget::MakeWidget;

pub fn counter() -> impl MakeWidget {
    // Create a dynamic usize.
    let count = Dynamic::new(0_isize);

    // Create a new label displaying `count`
    count
        .to_label()
        // Use the label as the contents of a button
        .into_button()
        // Set the `on_click` callback to a closure that increments the counter.
        .on_click(move |_| count.set(count.get() + 1))
}
