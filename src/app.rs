use std::io;
use std::path::PathBuf;

use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    buffer::Buffer,
    layout::Rect,
    widgets::Widget,
    Terminal,
};

use crate::animation::AnimationLoop;
use crate::event::{create_event_queue, EventReceiver, FileWatcher, HiveEvent};
use crate::input::{InputEvent, InputHandler};
use crate::render::{
    ActivityLog, ActivityLogWidget, DisplayMode, EmptyStateType, EmptyStateWidget,
    HeatMap, LayerRenderer, LayerVisibility, RenderLayer, RenderState,
};
use crate::state::{Field, History};

/// Application configuration
#[derive(Debug, Clone)]
pub struct AppConfig {
    pub file_path: Option<PathBuf>,
    pub demo_mode: bool,
    pub show_heatmap: bool,
    pub show_trails: bool,
    pub show_landmarks: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            file_path: None,
            demo_mode: false,
            show_heatmap: true,
            show_trails: true,
            show_landmarks: true,
        }
    }
}

/// Main application state
pub struct App {
    config: AppConfig,
    field: Field,
    history: History,
    heatmap: HeatMap,
    animation_loop: AnimationLoop,
    input_handler: InputHandler,

    // Display mode (replaces individual toggles)
    display_mode: DisplayMode,

    // Layer-based rendering (derived from display_mode)
    layer_visibility: LayerVisibility,

    // Help overlay toggle
    show_help: bool,

    // Mouse state
    mouse_position: Option<(u16, u16)>,
    selected_agent: Option<String>,

    // Hovered agent (for mouse hover detection)
    hovered_agent: Option<String>,

    // Last known field area for hit detection
    last_field_area: Option<Rect>,

    // Activity log for tracking recent agent events
    activity_log: ActivityLog,

    // Filter state
    filter_text: String,
    filter_mode: bool,

    // Running state
    running: bool,
}

impl App {
    pub fn new(config: AppConfig) -> Self {
        // Start in Standard mode (default)
        let display_mode = DisplayMode::default();
        let layer_visibility = display_mode.layer_visibility();

        Self {
            config,
            field: Field::new(),
            history: History::new(),
            heatmap: HeatMap::new(80, 24),
            animation_loop: AnimationLoop::new(),
            input_handler: InputHandler::new(),
            display_mode,
            layer_visibility,
            show_help: false,
            mouse_position: None,
            selected_agent: None,
            hovered_agent: None,
            last_field_area: None,
            activity_log: ActivityLog::new(100), // Keep last 100 activity entries
            filter_text: String::new(),
            filter_mode: false,
            running: true,
        }
    }

    /// Set the display mode and update layer visibility accordingly.
    fn set_display_mode(&mut self, mode: DisplayMode) {
        self.display_mode = mode;
        self.layer_visibility = mode.layer_visibility();
    }

    /// Cycle to the next display mode.
    fn cycle_display_mode(&mut self) {
        self.set_display_mode(self.display_mode.cycle());
    }

    /// Find an agent at the given screen position.
    ///
    /// Uses a 3x2 character hit target around each agent for easier selection.
    /// Returns the agent ID if found, None otherwise.
    fn find_agent_at_position(&self, x: u16, y: u16) -> Option<String> {
        let field_area = self.last_field_area?;

        // Check if position is within field bounds
        if x < field_area.x + 1 || x >= field_area.x + field_area.width - 1 {
            return None;
        }
        if y < field_area.y + 1 || y >= field_area.y + field_area.height - 1 {
            return None;
        }

        // Calculate inner dimensions (excluding border)
        let inner_width = field_area.width.saturating_sub(2);
        let inner_height = field_area.height.saturating_sub(2);

        if inner_width == 0 || inner_height == 0 {
            return None;
        }

        // Hit target size: 3 characters wide, 2 characters tall
        const HIT_WIDTH: u16 = 3;
        const HIT_HEIGHT: u16 = 2;

        // Check each agent
        for agent in self.field.agents.values() {
            // Convert agent's normalized position to screen coordinates
            let (agent_x, agent_y) = agent.position.to_terminal(inner_width, inner_height);
            let draw_x = field_area.x + 1 + agent_x;
            let draw_y = field_area.y + 1 + agent_y;

            // Check if click is within hit target (centered on agent)
            let left = draw_x.saturating_sub(HIT_WIDTH / 2);
            let right = draw_x + HIT_WIDTH / 2;
            let top = draw_y.saturating_sub(HIT_HEIGHT / 2);
            let bottom = draw_y + HIT_HEIGHT / 2;

            if x >= left && x <= right && y >= top && y <= bottom {
                return Some(agent.id.clone());
            }
        }

        None
    }

    /// Get agents filtered by current filter text.
    fn get_filtered_agents(&self) -> Vec<&crate::state::Agent> {
        let agents = self.field.agents_sorted();

        if self.filter_text.is_empty() {
            return agents;
        }

        let filter_lower = self.filter_text.to_lowercase();
        agents
            .into_iter()
            .filter(|agent| agent.id.to_lowercase().contains(&filter_lower))
            .collect()
    }

    /// Run the application
    pub async fn run(&mut self) -> io::Result<()> {
        // Setup terminal
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        // Create event channel
        let (event_tx, mut event_rx) = create_event_queue();

        // Start file watcher or demo mode
        let _watcher = if self.config.demo_mode {
            // Start demo event generator
            let tx = event_tx.inner();
            tokio::spawn(crate::demo::generate_demo_events(tx));
            None
        } else if let Some(ref path) = self.config.file_path {
            // Load existing events
            let watcher = FileWatcher::new(path, event_tx.inner())
                .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

            let existing_events = watcher.read_all_events();
            for event in existing_events {
                self.process_event(event.clone());
                self.history.record(event);
            }

            Some(watcher)
        } else {
            None
        };

        // Main loop
        while self.running {
            // Handle input
            self.handle_input();

            // Process new events
            self.process_incoming_events(&mut event_rx);

            // Handle replay mode
            if self.history.replay_mode {
                let replay_events = self.history.get_replay_events(self.field.playback_speed);
                for event in replay_events {
                    self.field.process_event(&event);
                }
            }

            // Update animations
            if self.animation_loop.should_render() {
                let dt = self.animation_loop.delta_time();

                // Update field state
                self.field.tick(dt);

                // Update heat map (always update to maintain state, visibility controlled at render)
                if self.layer_visibility.is_visible(RenderLayer::Heatmap) {
                    for agent in self.field.agents.values() {
                        self.heatmap.add_heat(&agent.position, agent.intensity);
                    }
                    self.heatmap.decay();
                }

                // Render
                terminal.draw(|frame| {
                    let area = frame.area();
                    // Store field area for hit detection (calculate same as in render)
                    let show_activity_log = matches!(
                        self.display_mode,
                        DisplayMode::Standard | DisplayMode::Debug
                    );
                    let activity_log_width = if show_activity_log { 30u16 } else { 0u16 };
                    let field_height = if self.history.replay_mode {
                        area.height.saturating_sub(2)
                    } else {
                        area.height.saturating_sub(1)
                    };
                    let field_width = area.width.saturating_sub(activity_log_width);
                    self.last_field_area = Some(Rect::new(area.x, area.y, field_width, field_height));

                    self.render(area, frame.buffer_mut());
                })?;

                self.animation_loop.frame_rendered();
            }

            // Small sleep to prevent busy loop
            tokio::time::sleep(self.animation_loop.time_until_next_frame()).await;
        }

        // Cleanup terminal
        disable_raw_mode()?;
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        terminal.show_cursor()?;

        Ok(())
    }

    /// Process a single event
    fn process_event(&mut self, event: HiveEvent) {
        // Add to activity log for AgentUpdate events
        if let HiveEvent::AgentUpdate(ref update) = event {
            // Get the agent's color for the activity log entry
            let color = self.field.agents.get(&update.agent_id)
                .map(|a| crate::render::get_agent_color(a.color_index))
                .unwrap_or(ratatui::style::Color::Rgb(150, 150, 150));

            // Create a descriptive message for the activity log
            let message = if update.message.is_empty() {
                format!("{:?}", update.status)
            } else {
                update.message.clone()
            };

            self.activity_log.add(update.agent_id.clone(), message, color);
        }

        self.field.process_event(&event);
    }

    /// Process incoming events from the queue
    fn process_incoming_events(&mut self, rx: &mut EventReceiver) {
        // Don't process new events in replay mode
        if self.history.replay_mode {
            return;
        }

        while let Ok(event) = rx.try_recv() {
            self.history.record(event.clone());
            self.process_event(event);
        }
    }

    /// Handle user input
    fn handle_input(&mut self) {
        let timeout = std::time::Duration::from_millis(1);

        if let Some(event) = self.input_handler.poll(timeout) {
            match event {
                InputEvent::Quit => self.running = false,

                InputEvent::TogglePause => self.field.toggle_pause(),

                InputEvent::SpeedUp => self.field.adjust_speed(0.25),

                InputEvent::SpeedDown => self.field.adjust_speed(-0.25),

                InputEvent::ToggleReplay => {
                    if self.history.replay_mode {
                        self.history.stop_replay();
                    } else {
                        self.history.start_replay();
                        // Reset field state for replay
                        self.field = Field::new();
                    }
                }

                InputEvent::SeekBackward => {
                    if self.history.replay_mode {
                        let pos = (self.history.position() - 0.05).max(0.0);
                        self.history.seek(pos);
                        self.rebuild_state_to_position();
                    }
                }

                InputEvent::SeekForward => {
                    if self.history.replay_mode {
                        let pos = (self.history.position() + 0.05).min(1.0);
                        self.history.seek(pos);
                        self.rebuild_state_to_position();
                    }
                }

                // Legacy individual toggles - still work for fine-grained control
                InputEvent::ToggleHeatMap => {
                    self.layer_visibility.toggle(RenderLayer::Heatmap);
                }

                InputEvent::ToggleTrails => {
                    self.layer_visibility.toggle(RenderLayer::Trails);
                }

                InputEvent::ToggleLandmarks => {
                    self.layer_visibility.toggle(RenderLayer::Zones);
                }

                InputEvent::ClearHeatMap => self.heatmap.clear(),

                // Display mode controls
                InputEvent::CycleDisplayMode => self.cycle_display_mode(),

                InputEvent::SetModeMinimal => self.set_display_mode(DisplayMode::Minimal),

                InputEvent::SetModeStandard => self.set_display_mode(DisplayMode::Standard),

                InputEvent::SetModeDebug => self.set_display_mode(DisplayMode::Debug),

                InputEvent::ToggleHelp => {
                    self.show_help = !self.show_help;
                    self.input_handler.set_help_visible(self.show_help);
                }

                InputEvent::CloseHelp => {
                    self.show_help = false;
                    self.input_handler.set_help_visible(false);
                }

                InputEvent::MouseHover { x, y } => {
                    self.mouse_position = Some((x, y));
                    // Update hovered agent based on mouse position
                    self.hovered_agent = self.find_agent_at_position(x, y);
                }

                InputEvent::MouseClick { x, y } => {
                    // Select agent on click
                    if let Some(agent_id) = self.find_agent_at_position(x, y) {
                        self.selected_agent = Some(agent_id);
                    } else {
                        // Clear selection when clicking empty area
                        self.selected_agent = None;
                    }
                }

                InputEvent::Resize { width, height } => {
                    self.heatmap.resize(width, height);
                }

                // Filter mode controls
                InputEvent::EnterFilterMode => {
                    self.filter_mode = true;
                    self.input_handler.set_filter_mode(true);
                }

                InputEvent::ExitFilterMode => {
                    self.filter_mode = false;
                    self.input_handler.set_filter_mode(false);
                }

                InputEvent::ApplyFilter => {
                    // Apply filter and exit filter mode
                    self.filter_mode = false;
                    self.input_handler.set_filter_mode(false);
                }

                InputEvent::CharInput(c) => {
                    if self.filter_mode {
                        if c == '\x08' {
                            // Backspace
                            self.filter_text.pop();
                        } else {
                            self.filter_text.push(c);
                        }
                    }
                }

                InputEvent::ClearFilter => {
                    self.filter_text.clear();
                    self.filter_mode = false;
                    self.input_handler.set_filter_mode(false);
                }

                InputEvent::None => {}
            }
        }
    }

    /// Rebuild field state to current history position
    fn rebuild_state_to_position(&mut self) {
        self.field = Field::new();
        let events = self.history.get_events_to_position();
        for event in events {
            self.field.process_event(&event);
        }
    }

    /// Render the entire UI using layer-based rendering.
    ///
    /// Layers are rendered in strict z-order:
    /// 1. Background (field border)
    /// 2. Zones (landmarks)
    /// 3. Grid (optional, currently disabled)
    /// 4. Heatmap (activity visualization)
    /// 5. Trails (agent movement history)
    /// 6. Connections (lines between agents)
    /// 7. Flashes (event indicators, not yet implemented)
    /// 8. Agents (primary content)
    /// 9. Labels (agent names, rendered with agents)
    /// 10. StatusIndicators (status symbols, rendered with agents)
    /// 11. UI (status bar, timeline)
    /// 12. Overlays (help panel)
    /// 13. Activity log (in Standard and Debug modes)
    fn render(&self, area: Rect, buf: &mut Buffer) {
        // Determine if we should show activity log (Standard and Debug modes)
        let show_activity_log = matches!(
            self.display_mode,
            DisplayMode::Standard | DisplayMode::Debug
        );

        // Calculate activity log width (right side panel)
        let activity_log_width = if show_activity_log { 30u16 } else { 0u16 };

        // Calculate field area (leave room for status bar, optional timeline, and activity log)
        let field_height = if self.history.replay_mode {
            area.height.saturating_sub(2)
        } else {
            area.height.saturating_sub(1)
        };
        let field_width = area.width.saturating_sub(activity_log_width);
        let field_area = Rect::new(area.x, area.y, field_width, field_height);

        // Prepare filtered agent list
        let agents: Vec<_> = self.get_filtered_agents();

        // Render empty state if no agents
        if agents.is_empty() {
            if self.filter_text.is_empty() {
                EmptyStateWidget::new(EmptyStateType::NoAgents).render(field_area, buf);
            }
            // If filter is active but no matches, we still show the field
        }

        // Prepare landmarks based on layer visibility
        let empty_landmarks = std::collections::HashMap::new();
        let landmarks = if self.layer_visibility.is_visible(RenderLayer::Zones) {
            &self.field.landmarks
        } else {
            &empty_landmarks
        };

        // Prepare heatmap reference based on layer visibility
        let heatmap_ref = if self.layer_visibility.is_visible(RenderLayer::Heatmap) {
            Some(&self.heatmap)
        } else {
            None
        };

        // Create the render state with all data needed for layer rendering
        let get_agent_position = |id: &str| self.field.get_agent_position(id);
        let render_state = RenderState {
            agents: &agents,
            selected_agent: self.selected_agent.as_deref(),
            hovered_agent: self.hovered_agent.as_deref(),
            heatmap: heatmap_ref,
            connections: &self.field.connections,
            get_agent_position: &get_agent_position,
            landmarks,
            history: &self.history,
            paused: self.field.paused,
            playback_speed: self.field.playback_speed,
            show_help: self.show_help,
            fps: self.animation_loop.fps(),
            display_mode: self.display_mode,
            filter_text: if self.filter_mode || !self.filter_text.is_empty() {
                Some(self.filter_text.as_str())
            } else {
                None
            },
            filter_mode: self.filter_mode,
        };

        // Create layer renderer and render all layers in z-order
        let layer_renderer = LayerRenderer::new(area, field_area, &self.layer_visibility);
        layer_renderer.render_all(buf, &render_state);

        // Render activity log in Standard and Debug modes
        if show_activity_log && activity_log_width > 0 {
            let activity_area = Rect::new(
                area.x + field_width,
                area.y,
                activity_log_width,
                field_height,
            );
            ActivityLogWidget::new(&self.activity_log).render(activity_area, buf);
        }

        // Render agent hover panel if an agent is hovered
        if let Some(ref hovered_id) = self.hovered_agent {
            if let Some(agent) = self.field.agents.get(hovered_id) {
                // Calculate agent's screen position
                let inner_width = field_area.width.saturating_sub(2);
                let inner_height = field_area.height.saturating_sub(2);
                let (agent_x, agent_y) = agent.position.to_terminal(inner_width, inner_height);
                let draw_x = field_area.x + 1 + agent_x;
                let draw_y = field_area.y + 1 + agent_y;

                // Calculate panel position
                let (panel_x, panel_y) = crate::render::AgentPanel::calculate_position(draw_x, draw_y, field_area);
                let (panel_width, panel_height) = crate::render::AgentPanel::dimensions();

                let panel_area = Rect::new(panel_x, panel_y, panel_width, panel_height);
                crate::render::AgentPanel::new(agent).render(panel_area, buf);
            }
        }
    }
}
