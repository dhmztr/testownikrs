# Testownik-rs

A quiz learning application written in pure Rust with a native GUI using the `iced` framework. This is a complete rewrite of the original Testownik project created by PWr students in Electron.js and Vue.js.

## Description

Testownik-rs is a native cross-platform quiz application with advanced learning features including:
- **Spaced Repetition System**: Questions repeat based on your performance
- **Session Persistence**: Save and resume quiz sessions anytime
- **Smart Randomization**: Questions and answers are randomized for better learning
- **Dark/Light Theme**: Switch between themes for comfortable viewing
- **Progress Tracking**: Monitor your learning progress in real-time

## Features

- ✅ Pure Rust GUI application using iced framework
- ✅ Parse questions from text files (single file with `##` separators)
- ✅ Support for single-choice and multiple-choice questions (Type X)
- ✅ Support for select/dropdown questions (Type Y)
- ✅ Text and image content support (via `[img]` tags)
- ✅ UTF-8 and Windows-1250 encoding support
- ✅ **Spaced Repetition Learning System**:
  - Questions decrease on correct answers
  - Questions increase on incorrect answers
  - Configurable repetition settings
  - Questions removed when mastered (0 repetitions)
- ✅ **Session Management**:
  - Save progress automatically
  - Resume from any saved session
  - Multiple quiz sessions supported
- ✅ **Smart Features**:
  - Randomized question order
  - Randomized answer order
  - Visual feedback (green for correct, red for incorrect)
  - Progress tracking and statistics
- ✅ **Theme Support**: Dark and light modes
- ✅ **Hot Streak**: after a configurable number of consecutive correct answers
  (default 3) a 🔥 hot streak badge and banner appear; the streak levels up at
  2× and 3× the threshold, the best streak is saved per session and shown in
  the session list and on the completion screen. Toggle it and change the
  threshold in the main menu settings panel ("Edytuj").
- ✅ **Burning screen**: during a hot streak animated flames rise from the
  bottom of the quiz screen, behind the cards and buttons. The longer the
  streak, the higher the fire (up to half the window) and the more it shifts
  from orange to deep red, with more embers; it fades out when the streak
  breaks. Can be turned off separately ("Płonący ekran").

## Installation

### Requirements

- Rust 1.70 or newer (latest stable version recommended)
- Cargo (installed with Rust)
- System dependencies for iced (see below)

### System Dependencies

#### Linux
```bash
# Ubuntu/Debian
sudo apt install libxkbcommon-dev libwayland-dev libxcb-shape0-dev libxcb-xfixes0-dev

# Fedora
sudo dnf install libxkbcommon-devel wayland-devel libxcb-devel
```

#### macOS
No additional dependencies required.

#### Windows
No additional dependencies required.

### Build

```bash
cargo build --release
```

The executable will be available at `target/release/testownik-rs`

## Usage

### Running the Application

```bash
./target/release/testownik-rs
```

or in development mode:

```bash
cargo run
```

A quiz file can be opened directly by passing its path:

```bash
cargo run -- path/to/quiz.txt
```

### Using the GUI

1. **Main Menu**:
   - Click "📁 Select Quiz File" to start a new quiz with the example questions
   - View and continue recent quiz sessions
   - Toggle between light and dark themes
   - View repetition settings

2. **Quiz Screen**:
   - Read the question and select your answer(s)
   - Click "Submit Answer" to check your response
   - See immediate feedback (green = correct, red = incorrect)
   - Click "Next Question" to continue (keys: `1`-`9` select, `Space`/`Enter` submit/next)
   - Click "Save & Exit" to return to the main menu

3. **Learning System**:
   - Questions you answer correctly will appear fewer times
   - Questions you answer incorrectly will repeat more
   - Complete mastery removes questions from the active set
   - Track your progress with the progress bar

## Quiz File Format

Questions in the file are separated by `##` (two hash marks on a separate line).

### Type X Questions (Single/Multiple Choice)

Structure:
```
X<correct answers>
Question content
Answer 1
Answer 2
Answer 3
Answer 4
```

- First line: `X` + binary string indicating correct answers (e.g., `X1010` = answers 1 and 3 are correct)
- Second line: question text or `[img]path/to/image[/img]`
- Following lines: answer options (can also use [img] tags)

Example:
```
X1010
Which programming languages are compiled?
Rust
Python
C++
JavaScript
```

### Type Y Questions (Select/Dropdown)

Structure:
```
Y<correct options>
Question text with {choice 1} and {choice 2}
option1;;option2;;option3
optionA;;optionB;;optionC
```

- First line: `Y` + digits indicating correct option for each select (e.g., `Y123` = select 1 has option 1, select 2 has option 2, select 3 has option 3)
- Second line: text with `{choice N}` placeholders (N is 1-based)
- Following lines: options separated by `;;` (each line is one dropdown)

Example:
```
Y231
Match {choice 1} to {choice 2} and {choice 3}
value1;;value2;;value3
optionA;;optionB;;optionC
test1;;test2;;test3;;test4
```

### Images

Images are marked with `[img]path/to/image[/img]` tags. Paths are relative to
the quiz file. If the file is not found at that exact location, it is searched
for in subfolders (up to 3 levels deep) and matched case-insensitively, so
`Z1.PNG`, `obrazki\z1.png` and `z1.png` all work. Missing images are shown as
a visible "Nie znaleziono obrazka" placeholder. Images keep their aspect ratio
and are scaled down (never up) to fit.

Image lines placed between the question text and the answers are attached to
the question when there are more lines than answers declared in the `X`
header (e.g. `X1000` = 4 answers). Tags may also appear inline in question or
answer text.

```
X0100
[img]diagram.png[/img]
Answer 1
Answer 2 (correct)
Answer 3
```

### Full Example File

See `example_questions.txt` in the repository for a complete example.

## Project Structure

```
.
├── Cargo.toml              # Project configuration and dependencies
├── src/
│   ├── main.rs            # Application entry point
│   ├── gui/               # GUI implementation with iced
│   │   ├── mod.rs
│   │   ├── app.rs         # Main application state and logic
│   │   ├── main_menu.rs   # Main menu screen
│   │   ├── quiz.rs        # Quiz screen with answer selection
│   │   └── styles.rs      # Custom styles
│   ├── models/            # Data structures
│   │   ├── mod.rs
│   │   ├── question.rs    # Question and answer models
│   │   ├── session.rs     # Quiz session with progress tracking
│   │   └── settings.rs    # Application and repetition settings
│   ├── persistence/       # Save/load functionality
│   │   ├── mod.rs
│   │   └── storage.rs     # JSON-based session storage
│   ├── utils/             # Helper functions
│   │   ├── mod.rs
│   │   └── randomizer.rs  # Randomization utilities
│   ├── types.rs           # Core question type definitions
│   └── parser.rs          # Question file parser
├── example_questions.txt  # Example quiz file
└── README.md              # This file
```

## Development

### Running Tests

```bash
cargo test
```

### Code Formatting

```bash
cargo fmt
```

### Linting

```bash
cargo clippy
```

### Building for Release

```bash
cargo build --release
```

## Repetition Settings

The spaced repetition system uses these default settings (configurable per session):

- **Initial Repetitions**: 2 (how many times a question appears initially)
- **Correct Decrease**: 1 (decrease count by this when answered correctly)
- **Incorrect Increase**: 1 (increase count by this when answered incorrectly)
- **Maximum Repetitions**: 10 (cap on how many times a question can repeat)

## Data Storage

Session data is stored in:
- **Linux**: `~/.local/share/testownik-rs/`
- **macOS**: `~/Library/Application Support/testownik-rs/`
- **Windows**: `%LOCALAPPDATA%\testownik-rs\`

Sessions are saved as JSON files and can be manually backed up or deleted.

## License

MIT - see LICENSE.md file

## Authors

- Kamil Golec <kumalgfilms@gmail.com> - Original Testownik project creator
- Rewritten in Rust with GUI - 2026

## History

This project is a complete rewrite of the Testownik application from JavaScript/Electron/Vue.js to native Rust. The new version adds:
- Native GUI using iced framework
- Spaced repetition learning system
- Session persistence and management
- Improved performance and cross-platform support
