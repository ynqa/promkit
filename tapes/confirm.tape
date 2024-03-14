Output tapes/confirm.gif

Require cargo

Set Shell "bash"
Set Theme "Dracula"
Set FontSize 32
Set Width 1200
Set Height 600

Type@50ms "cargo run -q --example confirm" Enter Sleep 1s
Type "invalid" Sleep 1s
Enter Sleep 1s
Ctrl+U Type "yes" Sleep 1s
Enter Sleep 2s
