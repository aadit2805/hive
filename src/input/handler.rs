use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
use std::time::Duration;

/// Processed input events for the application
#[derive(Debug, Clone)]
pub enum InputEvent {
    /// Quit the application
    Quit,
    /// Toggle pause
    TogglePause,
    /// Speed up playback
    SpeedUp,
    /// Speed down playback
    SpeedDown,
    /// Toggle replay mode
    ToggleReplay,
    /// Seek backward in replay
    SeekBackward,
    /// Seek forward in replay
    SeekForward,
    /// Toggle heat map display
    ToggleHeatMap,
    /// Toggle trails display
    ToggleTrails,
    /// Toggle landmarks display
    ToggleLandmarks,
    /// Clear heat map
    ClearHeatMap,
    /// Toggle help overlay
    ToggleHelp,
    /// Cycle through display modes (Minimal -> Standard -> Debug)
    CycleDisplayMode,
    /// Set display mode to Minimal
    SetModeMinimal,
    /// Set display mode to Standard
    SetModeStandard,
    /// Set display mode to Debug
    SetModeDebug,
    /// Mouse hover at position
    MouseHover { x: u16, y: u16 },
    /// Mouse click at position
    MouseClick { x: u16, y: u16 },
    /// Terminal resize
    Resize { width: u16, height: u16 },
    /// Close help (any key when help is shown)
    CloseHelp,
    /// Enter filter mode (/)
    EnterFilterMode,
    /// Character input for filter text
    CharInput(char),
    /// Apply filter (Enter when in filter mode)
    ApplyFilter,
    /// Clear filter (0 key)
    ClearFilter,
    /// Exit filter mode (Esc when in filter mode)
    ExitFilterMode,
    /// No event
    None,
}

/// Input handler for processing terminal events
pub struct InputHandler {
    help_visible: bool,
    filter_mode: bool,
}

impl InputHandler {
    pub fn new() -> Self {
        Self {
            help_visible: false,
            filter_mode: false,
        }
    }

    /// Set help visibility state
    pub fn set_help_visible(&mut self, visible: bool) {
        self.help_visible = visible;
    }

    /// Set filter mode state
    pub fn set_filter_mode(&mut self, active: bool) {
        self.filter_mode = active;
    }

    /// Check if filter mode is active
    pub fn is_filter_mode(&self) -> bool {
        self.filter_mode
    }

    /// Poll for input events with timeout
    pub fn poll(&mut self, timeout: Duration) -> Option<InputEvent> {
        if event::poll(timeout).ok()? {
            match event::read().ok()? {
                Event::Key(key_event) => Some(self.handle_key(key_event)),
                Event::Mouse(mouse_event) => Some(self.handle_mouse(mouse_event)),
                Event::Resize(width, height) => Some(InputEvent::Resize { width, height }),
                _ => None,
            }
        } else {
            None
        }
    }

    /// Handle keyboard input
    fn handle_key(&self, event: KeyEvent) -> InputEvent {
        // If help is visible, any key closes it
        if self.help_visible {
            return InputEvent::CloseHelp;
        }

        // If filter mode is active, handle filter-specific input
        if self.filter_mode {
            return self.handle_filter_key(event);
        }

        match event.code {
            // Quit
            KeyCode::Char('q') | KeyCode::Esc => InputEvent::Quit,

            // Ctrl+C to quit
            KeyCode::Char('c') if event.modifiers.contains(KeyModifiers::CONTROL) => {
                InputEvent::Quit
            }

            // Pause
            KeyCode::Char(' ') => InputEvent::TogglePause,

            // Speed controls
            KeyCode::Char('+') | KeyCode::Char('=') => InputEvent::SpeedUp,
            KeyCode::Char('-') | KeyCode::Char('_') => InputEvent::SpeedDown,

            // Replay
            KeyCode::Char('r') => InputEvent::ToggleReplay,
            KeyCode::Left => InputEvent::SeekBackward,
            KeyCode::Right => InputEvent::SeekForward,

            // Display toggles (legacy - still work for fine-grained control)
            KeyCode::Char('h') => InputEvent::ToggleHeatMap,
            KeyCode::Char('t') => InputEvent::ToggleTrails,
            KeyCode::Char('l') => InputEvent::ToggleLandmarks,
            KeyCode::Char('c') => InputEvent::ClearHeatMap,

            // Display mode controls
            KeyCode::Char('m') => InputEvent::CycleDisplayMode,
            KeyCode::Char('1') => InputEvent::SetModeMinimal,
            KeyCode::Char('2') => InputEvent::SetModeStandard,
            KeyCode::Char('3') => InputEvent::SetModeDebug,

            // Help
            KeyCode::Char('?') => InputEvent::ToggleHelp,

            // Filter mode
            KeyCode::Char('/') => InputEvent::EnterFilterMode,
            KeyCode::Char('0') => InputEvent::ClearFilter,

            _ => InputEvent::None,
        }
    }

    /// Handle keyboard input when in filter mode
    fn handle_filter_key(&self, event: KeyEvent) -> InputEvent {
        match event.code {
            // Exit filter mode
            KeyCode::Esc => InputEvent::ExitFilterMode,

            // Apply filter
            KeyCode::Enter => InputEvent::ApplyFilter,

            // Character input for filter text
            KeyCode::Char(c) => InputEvent::CharInput(c),

            // Backspace removes last character (treated as special char input)
            KeyCode::Backspace => InputEvent::CharInput('\x08'),

            _ => InputEvent::None,
        }
    }

    /// Handle mouse input
    fn handle_mouse(&self, event: MouseEvent) -> InputEvent {
        match event.kind {
            MouseEventKind::Moved => InputEvent::MouseHover {
                x: event.column,
                y: event.row,
            },
            MouseEventKind::Down(MouseButton::Left) => InputEvent::MouseClick {
                x: event.column,
                y: event.row,
            },
            _ => InputEvent::None,
        }
    }
}

impl Default for InputHandler {
    fn default() -> Self {
        Self::new()
    }
}
