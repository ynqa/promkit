Output tapes/csv.gif

Require cargo

Set Shell "bash"
Set Theme "Dracula"
Set FontSize 32
Set Width 1200
Set Height 600

Type@50ms "cargo run -q --bin csv tapes/csv.csv" Enter Sleep 2s
Down@80ms 14 Sleep 1s
Right@50ms 36 Sleep 2s
Up@200ms 4 Sleep 1s
Left@50ms 12 Sleep 1s
Enter Sleep 2s
