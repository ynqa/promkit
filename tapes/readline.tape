Output tapes/readline.gif

Require cargo

Set Shell "bash"
Set Theme "Dracula"
Set FontSize 32
Set Width 1200
Set Height 600

Type@50ms "cargo run -q --example readline" Enter Sleep 1s
Type "Hello promkit!!" Sleep 1s
Backspace 15 Type "a" Sleep 1s
Tab 1 Sleep 1s
Enter Enter Sleep 1s
Backspace 4 Sleep 1s
Tab@100ms 3 Sleep 1s
Enter Enter Sleep 2s
