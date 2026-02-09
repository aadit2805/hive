# Hive

Real-time AI Agent Visualization - Watch agents work together like players on a field.

```
    ◈ HIVE

         ╭─────────────────────────────────────────────╮
         │                 Authentication              │
         │                                             │
         │        ◉ explorer-1                        │
         │          ·  ·                              │
         │              ·  ·  ·                       │
         │  Database          ·  ·  ◐ builder-1      │
         │                         ·                  │
         │     ◍ planner-1    ─────────────          │
         │                              ·             │
         │                          ◌ tester-1       │
         │                                            │
         │                   Testing                  │
         ╰─────────────────────────────────────────────╯

    Agents: 4/4  Speed: 1.0x                    ? help
```

## Features

- **Semantic Positioning**: Agents are positioned in 2D space based on what they're working on. Similar concepts cluster together naturally.
- **Heat Maps**: Background color gradient shows cumulative work intensity - see where the action is happening.
- **Trails**: Fading paths show each agent's movement history, creating beautiful trace patterns.
- **Connections**: Lines appear between agents when they communicate, with animated fade in/out.
- **Time Travel**: Record and replay sessions, scrub through history at variable speeds.
- **Demo Mode**: Built-in simulation to try it instantly without setup.

## Installation

### From Source

```bash
git clone https://github.com/yourusername/hive
cd hive
cargo build --release
```

The binary will be at `target/release/hive`.

### From Cargo

```bash
cargo install hive-viz
```

## Usage

### Demo Mode

Try it instantly with simulated agents:

```bash
hive --demo
```

### Watch Events File

Monitor a JSON lines file for real agent events:

```bash
hive --file events.jsonl
```

### Options

```
Options:
  -f, --file <FILE>  Path to the events file to watch (JSON lines format)
      --demo         Run in demo mode with simulated agents
      --no-heatmap   Disable heat map display
      --no-trails    Disable trail display
      --no-landmarks Disable landmark display
  -h, --help         Print help
  -V, --version      Print version
```

## Controls

| Key | Action |
|-----|--------|
| `q` / `Esc` | Quit |
| `Space` | Pause/Resume |
| `+` / `-` | Speed up/down |
| `r` | Toggle replay mode |
| `←` / `→` | Seek backward/forward (in replay) |
| `h` | Toggle heat map |
| `t` | Toggle trails |
| `l` | Toggle landmarks |
| `c` | Clear heat map |
| `?` | Show help |

## Event Format

Agents write events as JSON lines to a file. Hive watches this file for changes.

### Agent Update

```json
{
  "type": "agent_update",
  "agent_id": "explorer-1",
  "status": "active",
  "focus": ["authentication", "jwt", "middleware"],
  "intensity": 0.8,
  "message": "Analyzing auth flow",
  "timestamp": 1706812345
}
```

**Fields:**
- `agent_id`: Unique identifier for the agent
- `status`: One of `active`, `thinking`, `waiting`, `idle`, `error`
- `focus`: Array of keywords describing current work area
- `intensity`: 0.0-1.0 representing work intensity (affects brightness/size)
- `message`: Current status message
- `timestamp`: Unix timestamp

### Connection

```json
{
  "type": "connection",
  "from": "explorer-1",
  "to": "planner-1",
  "label": "found relevant file",
  "timestamp": 1706812347
}
```

### Landmark

Define semantic regions on the field:

```json
{
  "type": "landmark",
  "id": "auth-cluster",
  "label": "Authentication",
  "keywords": ["jwt", "oauth", "session", "login"],
  "timestamp": 1706812340
}
```

## Integrating with Your Agents

To visualize your own AI agents:

1. Create a shared events file:
   ```bash
   touch /tmp/hive-events.jsonl
   ```

2. Have agents append events:
   ```python
   import json
   import time

   def emit_event(event):
       with open('/tmp/hive-events.jsonl', 'a') as f:
           f.write(json.dumps(event) + '\n')

   emit_event({
       "type": "agent_update",
       "agent_id": "my-agent",
       "status": "active",
       "focus": ["task", "subtask"],
       "intensity": 0.7,
       "message": "Working on something",
       "timestamp": int(time.time())
   })
   ```

3. Run hive:
   ```bash
   hive --file /tmp/hive-events.jsonl
   ```

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                        HIVE                                  │
│                                                              │
│  ┌──────────────┐    ┌──────────────┐    ┌──────────────┐  │
│  │ File Watcher │───▶│ Event Queue  │───▶│ State Engine │  │
│  └──────────────┘    └──────────────┘    └──────────────┘  │
│                                                 │            │
│                                                 ▼            │
│  ┌──────────────┐    ┌──────────────┐    ┌──────────────┐  │
│  │   Renderer   │◀───│  Animation   │◀───│   Semantic   │  │
│  │  (ratatui)   │    │    Loop      │    │  Positioning │  │
│  └──────────────┘    └──────────────┘    └──────────────┘  │
│                                                              │
│  ┌──────────────┐    ┌──────────────┐                       │
│  │  Heat Map    │    │   History    │                       │
│  │  Calculator  │    │   (Replay)   │                       │
│  └──────────────┘    └──────────────┘                       │
└─────────────────────────────────────────────────────────────┘
```

## License

MIT
