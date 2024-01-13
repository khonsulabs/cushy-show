use cushy_show::*;

fn main() {
    Show::default()
        .with(Slide::new("title", h1("Hello, World")))
        .present();
}
