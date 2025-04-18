use dialoguer::{Input, theme::ColorfulTheme};

fn main() {
    let color_theme = ColorfulTheme::default();
    let name: String = Input::with_theme(&color_theme)
        .with_prompt("Your name?")
        .interact_text()
        .unwrap();

    println!("Your name is: {}", name);
}
