//! ASCII art banner for interactive mode.

use std::io::IsTerminal;

/// ANSI true-color escape sequences for the mole banner palette.
struct Colors {
    dirt_dark: &'static str,
    dirt_light: &'static str,
    fur: &'static str,
    nose: &'static str,
    paws: &'static str,
    title: &'static str,
    subtitle: &'static str,
    reset: &'static str,
}

const COLOR: Colors = Colors {
    dirt_dark: "\x1b[38;2;101;67;33m",  // Dark brown
    dirt_light: "\x1b[38;2;139;90;43m", // Lighter brown
    fur: "\x1b[38;2;88;88;88m",         // Dark gray
    nose: "\x1b[38;2;255;182;193m",     // Pink nose
    paws: "\x1b[38;2;139;115;85m",      // Tan paws
    title: "\x1b[1;38;2;205;133;63m",   // Bold peru
    subtitle: "\x1b[38;2;120;120;140m", // Gray-blue
    reset: "\x1b[0m",
};

const PLAIN: Colors = Colors {
    dirt_dark: "",
    dirt_light: "",
    fur: "",
    nose: "",
    paws: "",
    title: "",
    subtitle: "",
    reset: "",
};

/// Prints the Burrow mole banner to stdout.
///
/// Renders ANSI true-color when stdout is a terminal,
/// falls back to plain text otherwise.
pub fn print_banner() {
    let c = if std::io::stdout().is_terminal() {
        &COLOR
    } else {
        &PLAIN
    };

    let dd = c.dirt_dark;
    let dl = c.dirt_light;
    let fr = c.fur;
    let ns = c.nose;
    let pw = c.paws;
    let tt = c.title;
    let st = c.subtitle;
    let r = c.reset;

    println!(
        r#"
{dd}    ~~~~~~~~~~~~~~~~{r}
{dl}  ~~~~~~~~~~~~~~~~~~{r}      {tt}    ____  __  ______  ____  ____  _       __{r}
{dd} ~~~~{fr}▄▄▄▄▄▄▄▄▄▄{dd}~~~~~{r}      {tt}   / __ )/ / / / __ \/ __ \/ __ \| |     / /{r}
{dl}~~~{fr}▄█▀▀▀▀▀▀▀▀▀▀█▄{dl}~~~~{r}      {tt}  / __  / / / / /_/ / /_/ / / / / | /| / / {r}
{dd}~~{fr}█▀░░{ns}●{fr}░░░░{ns}●{fr}░░▀█{dd}~~~{r}      {tt} / /_/ / /_/ / _, _/ _, _/ /_/ /| |/ |/ /  {r}
{dl}~~{fr}█░░░░{ns}▀{fr}░░░░░░░█{dl}~~~{r}      {tt}/_____/\____/_/ |_/_/ |_|\____/ |__/|__/   {r}
{dd}~~{fr}█▄░░░░░░░░░░▄█{dd}~~~{r}
{dl}~~~{fr}▀█▄▄{pw}▀▀{fr}░{pw}▀▀{fr}▄▄█▀{dl}~~~~{r}      {st}"Digging deep into your secrets..."{r}
{dd}~~~~{pw}▀██{fr}░{pw}██▀{dd}~~~~~~~~{r}
{dl}  ~~~~~~~~~~~~~~~~~~{r}
{dd}    ~~~~~~~~~~~~~~~~{r}
"#
    );
}
