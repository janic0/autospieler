# Dauerzusagesendung

Einfaches Rust-Kommandozeilenprogramm, um die nächsten Termine in Spielerplus automatisch zuzusagen.

## Verwendung

Bisher gibt es das Programm nur als Quellcode, es muss also Rust installiert sein, siehe
https://www.rust-lang.org/tools/install.

Vor der Ausführung müssen einige Umgebungsvariablen gesetzt werden, um euch einloggen zu können:
- `DAUERZUSAGE_EMAIL` und `DAUERZUSAGE_PASSWORT`: Spielerplus Benutzername und Passwort
- `DAUERZUSAGE_ID`: Die ID, mit der euch Spielerplus intern eurem Team zuordnet. Diese findet ihr
heraus, indem ihr ganz oben links auf euren Namen/Team klickt. Ihr landet dann auf der "Team
auswählen" Seite. Der Link zu eurem Team hat das Format
`https://www.spielerplus.de/site/switch-user?id=<DAUERZUSAGE_ID>`, ihr könnt also dort die ID
auslesen.

Danach kann das Programm mit `cargo run` gebaut und ausgeführt werden. Alternativ einmal bauen mit
`cargo build --release`, das Programm findet ihr dann unter `target/release/dauerzusagesendung`
oder `target/release/dauerzusagesendung.exe`.

### Linux / macOS
```
export DAUERZUSAGE_EMAIL=lacrosse.gott@example.com
export DAUERZUSAGE_PASSWORT=Supergeheimespasswort2028
export DAUERZUSAGE_ID=1234567
cargo run
```

### Windows (ungetestet)
```
set DAUERZUSAGE_EMAIL=lacrosse.gott@example.com
set DAUERZUSAGE_PASSWORT=Supergeheimespasswort2028
set DAUERZUSAGE_ID=1234567
cargo run
```
