use promkit::{
    core::crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers},
    preset::text_editor::{evaluate, IndentContext, TextEditor},
    Prompt, Signal,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Completion {
    Empty,
    Incomplete,
    Complete,
    Invalid,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    loop {
        let mut editor = TextEditor::default()
            .title("Bracket REPL (blank line: submit block, \"exit\": quit)")
            .prefix("❯❯❯ ")
            .continuation_prefix("... ")
            .indenter(delimiter_indent)
            .validator(
                |text| completion(text) == Completion::Complete,
                |_| "input is empty or contains unclosed/mismatched delimiters".to_string(),
            )
            .evaluator(|event, context| Box::pin(evaluate_delimiters(event, context)))
            .lines(8);

        let text = editor.run().await?;
        if text.trim() == "exit" {
            break;
        }

        println!("result:\n{text}");
    }

    Ok(())
}

/// A deliberately small completion policy for this example.
///
/// Only `{}`, and `[]` are syntax. Strings, comments, and escapes are not
/// interpreted. Balanced bracket input is submitted by an empty continuation
/// line; a language REPL should replace this with its parser or VM.
fn completion(text: &str) -> Completion {
    if text.trim().is_empty() {
        return Completion::Empty;
    }

    match delimiter_stack(text.chars()) {
        Ok(stack) if stack.is_empty() => Completion::Complete,
        Ok(_) => Completion::Incomplete,
        Err(()) => Completion::Invalid,
    }
}

fn requires_blank_continuation_line(text: &str) -> bool {
    text.chars().any(|character| matches!(character, '{' | '['))
}

fn ends_with_blank_line(text: &str) -> bool {
    text.rsplit_once('\n')
        .is_some_and(|(_, line)| line.trim().is_empty())
}

fn delimiter_stack(characters: impl IntoIterator<Item = char>) -> Result<Vec<char>, ()> {
    let mut stack = Vec::new();

    for character in characters {
        match character {
            '{' | '[' => stack.push(character),
            '}' if stack.pop() != Some('{') => return Err(()),
            ']' if stack.pop() != Some('[') => return Err(()),
            _ => {}
        }
    }

    Ok(stack)
}

fn delimiter_indent(context: IndentContext<'_>) -> String {
    let characters = context.text.chars().take(context.cursor);
    let depth = delimiter_stack(characters)
        .map(|stack| stack.len())
        .unwrap_or_default();
    "    ".repeat(depth)
}

async fn evaluate_delimiters(event: &Event, context: &mut TextEditor) -> anyhow::Result<Signal> {
    if matches!(
        event,
        Event::Key(KeyEvent {
            code: KeyCode::Enter,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        })
    ) {
        let text = context.editor.texteditor.text_without_cursor().to_string();
        return match completion(&text) {
            Completion::Complete
                if requires_blank_continuation_line(&text) && !ends_with_blank_line(&text) =>
            {
                evaluate::default(event, context).await
            }
            Completion::Complete | Completion::Invalid => evaluate::submit(context),
            Completion::Empty | Completion::Incomplete => evaluate::default(event, context).await,
        };
    }

    evaluate::default(event, context).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use promkit::{core::Widget, widgets::text_editor::TextPosition};

    fn enter() -> Event {
        Event::Key(KeyEvent {
            code: KeyCode::Enter,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        })
    }

    #[test]
    fn completes_only_nonempty_balanced_input() {
        assert_eq!(completion(""), Completion::Empty);
        assert_eq!(completion("value {"), Completion::Incomplete);
        assert_eq!(completion("value {\n    [item]\n}"), Completion::Complete);
        assert_eq!(completion("{]"), Completion::Invalid);
    }

    #[test]
    fn indents_to_the_open_delimiter_depth_before_the_cursor() {
        let text = "{\n[";
        assert_eq!(
            delimiter_indent(IndentContext {
                text,
                cursor: text.chars().count(),
                position: TextPosition { row: 1, column: 1 },
            }),
            "        "
        );
    }

    #[tokio::test]
    async fn enter_continues_unclosed_input() {
        let mut editor = TextEditor::default().indenter(delimiter_indent);
        editor.editor.texteditor.replace("{");

        let signal = evaluate_delimiters(&enter(), &mut editor).await.unwrap();

        assert!(matches!(signal, Signal::Continue));
        assert_eq!(
            editor.editor.texteditor.text_without_cursor().to_string(),
            "{\n    "
        );
    }

    #[tokio::test]
    async fn enter_submits_complete_single_line_input() {
        let mut editor = TextEditor::default();
        editor.editor.texteditor.replace("value");

        let signal = evaluate_delimiters(&enter(), &mut editor).await.unwrap();

        assert!(matches!(signal, Signal::Quit));
    }

    async fn assert_blank_continuation_line_submits(input: &str) {
        let mut editor = TextEditor::default()
            .prefix("❯❯❯ ")
            .continuation_prefix("... ")
            .indenter(delimiter_indent);
        editor.editor.texteditor.replace(input);

        let signal = evaluate_delimiters(&enter(), &mut editor).await.unwrap();

        assert!(matches!(signal, Signal::Continue));
        assert_eq!(
            editor.editor.texteditor.text_without_cursor().to_string(),
            format!("{input}\n")
        );
        assert_eq!(
            editor.editor.create_graphemes().graphemes.to_string(),
            format!("❯❯❯ {}\n...  ", input.replace('\n', "\n... "))
        );

        let signal = evaluate_delimiters(&enter(), &mut editor).await.unwrap();

        assert!(matches!(signal, Signal::Quit));
    }

    #[tokio::test]
    async fn empty_continuation_line_submits_a_curly_bracket_block() {
        assert_blank_continuation_line_submits("{\n    value\n}").await;
    }

    #[tokio::test]
    async fn empty_continuation_line_submits_a_square_bracket_block() {
        assert_blank_continuation_line_submits("[\n    value\n]").await;
    }
}
