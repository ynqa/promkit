Output tapes/repl.gif

Require cargo

Set Shell "bash"
Set Theme "Dracula"
Set FontSize 32
Set Width 1200
Set Height 600

Type@50ms "cargo run -q --bin repl" Enter Sleep 2s
Type "{" Enter Sleep 1s
Type "languages: [" Enter Sleep 1s
Type "Rust" Enter Sleep 1s
Backspace 4 Type "]" Enter Sleep 1s
Backspace 4 Type "}" Enter Sleep 1s
Enter Sleep 2s
Type "exit" Enter Sleep 2s
