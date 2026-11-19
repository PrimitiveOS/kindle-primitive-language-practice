use std::{
    fs,
    path::{Path, PathBuf},
};

use language_practice_core::word_bank;
use language_practice_render::View;
use primitive_app::{font_paths, frame, write_png, App, Change, HEIGHT, WIDTH};
use primitive_ui::{Canvas, Input};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let output = PathBuf::from(
        std::env::args_os()
            .nth(1)
            .unwrap_or_else(|| "build/previews".into()),
    );
    fs::create_dir_all(&output)?;
    let fonts =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../doannt-kindle-launcher/assets/fonts");
    let mut canvas = Canvas::new(WIDTH, HEIGHT, &font_paths(&fonts))?;
    let save = std::env::temp_dir().join(format!("language-practice-{}.json", std::process::id()));
    let mut view = View::open(save.clone());
    let mut capture = |view: &View, name: &str, dark: bool| {
        write_png(
            &output.join(name),
            WIDTH,
            HEIGHT,
            &frame(view, &mut canvas, dark),
        )
    };
    capture(&view, "lessons.png", false)?;
    assert_eq!(view.input(Input::Tap(550.0, 564.0)), Change::Full);
    capture(&view, "example.png", false)?;
    assert_eq!(view.input(Input::Tap(350.0, 1490.0)), Change::Full);
    assert_eq!(view.input(Input::Tap(560.0, 660.0)), Change::Partial);
    capture(&view, "feedback-dark.png", true)?;
    assert_eq!(view.input(Input::Tap(350.0, 1490.0)), Change::Full);
    assert_eq!(view.input(Input::Tap(560.0, 660.0)), Change::Partial);
    assert_eq!(view.input(Input::Tap(350.0, 1490.0)), Change::Full);
    let bank = word_bank(0, 4);
    for required in 0..4 {
        let slot = bank.iter().position(|&value| value == required).unwrap();
        assert_eq!(
            view.input(Input::Tap(
                200.0 + (slot % 3) as f32 * 404.0,
                800.0 + (slot / 3) as f32 * 135.0
            )),
            Change::Partial
        );
    }
    capture(&view, "word-order.png", false)?;
    let mut resumed = View::open(save.clone());
    assert_eq!(resumed.input(Input::Tap(340.0, 258.0)), Change::Full);
    assert_eq!(resumed.study.current().unwrap().draft, [0, 1, 2, 3]);
    assert_eq!(resumed.input(Input::Tap(350.0, 1490.0)), Change::Partial);
    assert_eq!(resumed.input(Input::Tap(350.0, 1490.0)), Change::Full);

    assert_eq!(resumed.input(Input::Tap(1110.0, 85.0)), Change::Full);
    assert_eq!(resumed.input(Input::Tap(953.0, 255.0)), Change::Full);
    assert_eq!(resumed.input(Input::Tap(550.0, 564.0)), Change::Full);
    assert_eq!(resumed.study.current().unwrap().position(), (1, 1));
    assert_eq!(resumed.input(Input::Tap(560.0, 802.0)), Change::Partial);
    assert_eq!(resumed.study.total_mistakes(), 0);
    fs::remove_file(save)?;
    Ok(())
}
