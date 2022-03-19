use ariadne::{Color, Fmt, Label, Report, ReportKind, Source};

fn main() {
    let src_id = "input.txt";
    let src = "2022a/03/19";

    Report::build(ReportKind::Error, src_id, 4)
        .with_message("Unexpected char")
        .with_label(
            Label::new((src_id, 4..5))
                .with_message(format!("unexpected char {}", "a".fg(Color::Red))),
        )
        .finish()
        .print((src_id, Source::from(src)))
        .unwrap();
}
