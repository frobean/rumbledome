//! Terminal User Interface for RumbleDome simulator
//! 
//! Provides real-time monitoring and control of the simulation
//! using a terminal-based interface.

use crate::engine_sim::EngineCommand;
use rumbledome_core::SystemState;
use rumbledome_hal::{SystemInputs, DriveMode};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    symbols,
    text::{Line, Span, Text},
    widgets::{
        Axis, Block, Borders, Chart, Dataset, Gauge, List, ListItem, Paragraph, Row, Table,
        Clear,
    },
    Frame, Terminal,
};
use std::io;
use anyhow::Result;

/// Terminal UI manager for the simulator
pub struct SimulatorUI {
    /// Terminal interface
    terminal: Terminal<CrosstermBackend<io::Stdout>>,
    
    /// Current system state
    system_state: SystemState,
    
    /// Current system inputs
    current_inputs: Option<SystemInputs>,
    
    /// Current duty cycle output
    current_duty: f32,
    
    /// Historical data for charts
    history: SimulationHistory,
    
    /// UI state
    ui_state: UiState,
    
    /// Error message to display
    error_message: Option<String>,
    
    /// Should quit flag
    should_quit: bool,
}

/// Historical data for charting
#[derive(Debug, Clone)]
struct SimulationHistory {
    /// Time series data
    time_points: Vec<f64>,
    
    /// Boost pressure history
    boost_pressure: Vec<(f64, f64)>,
    
    /// RPM history
    rpm: Vec<(f64, f64)>,
    
    /// Duty cycle history
    duty_cycle: Vec<(f64, f64)>,
    
    /// Torque history
    desired_torque: Vec<(f64, f64)>,
    actual_torque: Vec<(f64, f64)>,
    
    /// Maximum history length
    max_points: usize,
}

/// UI state and settings
#[derive(Debug, Clone)]
struct UiState {
    /// Currently selected tab
    active_tab: usize,
    
    /// Simulated throttle position
    throttle_position: f32,
    
    /// Selected drive mode
    drive_mode: DriveMode,
    
    /// Selected profile
    selected_profile: String,
}

impl SimulatorUI {
    /// Create new simulator UI
    pub fn new() -> Result<Self> {
        // Setup terminal
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;
        
        Ok(Self {
            terminal,
            system_state: SystemState::Initializing,
            current_inputs: None,
            current_duty: 0.0,
            history: SimulationHistory::new(),
            ui_state: UiState {
                active_tab: 0,
                throttle_position: 0.0,
                drive_mode: DriveMode::Normal,
                selected_profile: "daily".to_string(),
            },
            error_message: None,
            should_quit: false,
        })
    }
    
    /// Process UI events and return engine commands
    pub fn process_events(&mut self) -> Result<Vec<EngineCommand>> {
        let mut commands = Vec::new();
        
        // Poll for events with timeout
        if event::poll(std::time::Duration::from_millis(10))? {
            match event::read()? {
                Event::Key(key) => {
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => {
                            self.should_quit = true;
                        },
                        
                        // Throttle control
                        KeyCode::Up => {
                            self.ui_state.throttle_position = (self.ui_state.throttle_position + 5.0).min(100.0);
                            commands.push(EngineCommand::SetThrottle(self.ui_state.throttle_position));
                        },
                        KeyCode::Down => {
                            self.ui_state.throttle_position = (self.ui_state.throttle_position - 5.0).max(0.0);
                            commands.push(EngineCommand::SetThrottle(self.ui_state.throttle_position));
                        },
                        
                        // Quick throttle presets
                        KeyCode::Char('0') => {
                            self.ui_state.throttle_position = 0.0;
                            commands.push(EngineCommand::SetThrottle(0.0));
                        },
                        KeyCode::Char('5') => {
                            self.ui_state.throttle_position = 50.0;
                            commands.push(EngineCommand::SetThrottle(50.0));
                        },
                        KeyCode::Char('w') => {
                            self.ui_state.throttle_position = 100.0;
                            commands.push(EngineCommand::SetThrottle(100.0));
                        },
                        
                        // Drive mode switching
                        KeyCode::Char('n') => {
                            self.ui_state.drive_mode = DriveMode::Normal;
                            commands.push(EngineCommand::SetDriveMode(DriveMode::Normal));
                        },
                        KeyCode::Char('s') => {
                            self.ui_state.drive_mode = DriveMode::Sport;
                            commands.push(EngineCommand::SetDriveMode(DriveMode::Sport));
                        },
                        KeyCode::Char('p') => {
                            self.ui_state.drive_mode = DriveMode::SportPlus;
                            commands.push(EngineCommand::SetDriveMode(DriveMode::SportPlus));
                        },
                        KeyCode::Char('t') => {
                            self.ui_state.drive_mode = DriveMode::Track;
                            commands.push(EngineCommand::SetDriveMode(DriveMode::Track));
                        },
                        
                        // Tab switching
                        KeyCode::Tab => {
                            self.ui_state.active_tab = (self.ui_state.active_tab + 1) % 3;
                        },
                        
                        // Reset command
                        KeyCode::Char('r') => {
                            commands.push(EngineCommand::ResetToIdle);
                            self.ui_state.throttle_position = 0.0;
                        },
                        
                        // Force overboost test
                        KeyCode::Char('o') => {
                            commands.push(EngineCommand::ForceOverboost);
                        },
                        
                        _ => {}
                    }
                },
                _ => {}
            }
        }
        
        Ok(commands)
    }
    
    /// Update system status from RumbleDome core
    pub fn update_system_status(&mut self, state: &SystemState, inputs: &SystemInputs) {
        self.system_state = state.clone();
        
        // Update history
        let current_time = inputs.timestamp_ms as f64 / 1000.0;
        self.history.add_data_point(
            current_time,
            inputs.sensors.manifold_pressure_gauge as f64,
            inputs.can.rpm as f64,
            self.current_duty as f64,
            inputs.can.desired_torque as f64,
            inputs.can.actual_torque as f64,
        );
        
        self.current_inputs = Some(inputs.clone());
    }
    
    /// Set current duty cycle for display
    pub fn set_current_duty(&mut self, duty: f32) {
        self.current_duty = duty;
    }
    
    /// Get current duty cycle
    pub fn get_current_duty(&self) -> f32 {
        self.current_duty
    }
    
    /// Show error message
    pub fn show_error(&mut self, error: &str) {
        self.error_message = Some(error.to_string());
    }
    
    /// Check if should quit
    pub fn should_quit(&self) -> bool {
        self.should_quit
    }
    
    /// Render the UI
    pub fn render(&mut self) -> Result<()> {
        self.terminal.draw(|f| {
            self.draw_ui(f);
        })?;
        Ok(())
    }
    
    /// Clean up terminal
    pub fn cleanup(&mut self) -> Result<()> {
        disable_raw_mode()?;
        execute!(
            self.terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        self.terminal.show_cursor()?;
        Ok(())
    }
    
    /// Draw the main UI
    fn draw_ui<B: Backend>(&mut self, f: &mut Frame<B>) {
        let size = f.size();
        
        // Main layout
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),  // Title
                Constraint::Min(0),     // Main content
                Constraint::Length(3),  // Status bar
            ].as_ref())
            .split(size);
        
        // Title
        self.draw_title(f, chunks[0]);
        
        // Tab content
        match self.ui_state.active_tab {
            0 => self.draw_dashboard_tab(f, chunks[1]),
            1 => self.draw_charts_tab(f, chunks[1]),
            2 => self.draw_control_tab(f, chunks[1]),
            _ => self.draw_dashboard_tab(f, chunks[1]),
        }
        
        // Status bar
        self.draw_status_bar(f, chunks[2]);
        
        // Error popup if present
        if let Some(ref error) = self.error_message {
            self.draw_error_popup(f, size, error);
        }
    }
    
    /// Draw title bar
    fn draw_title<B: Backend>(&self, f: &mut Frame<B>, area: Rect) {
        let title = Paragraph::new("RumbleDome Desktop Simulator")
            .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(title, area);
    }
    
    /// Draw dashboard tab
    fn draw_dashboard_tab<B: Backend>(&self, f: &mut Frame<B>, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
            .split(area);
        
        // Left side - gauges
        self.draw_gauges(f, chunks[0]);
        
        // Right side - system info
        self.draw_system_info(f, chunks[1]);
    }
    
    /// Draw charts tab
    fn draw_charts_tab<B: Backend>(&self, f: &mut Frame<B>, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
            .split(area);
        
        // Top chart - boost and RPM
        self.draw_boost_rpm_chart(f, chunks[0]);
        
        // Bottom chart - torque
        self.draw_torque_chart(f, chunks[1]);
    }
    
    /// Draw control tab
    fn draw_control_tab<B: Backend>(&self, f: &mut Frame<B>, area: Rect) {
        let help_text = vec![
            "CONTROLS:",
            "",
            "Throttle Control:",
            "  ↑/↓     - Adjust throttle by 5%",
            "  0       - Idle (0% throttle)",
            "  5       - 50% throttle", 
            "  W       - WOT (100% throttle)",
            "",
            "Drive Modes:",
            "  N       - Normal",
            "  S       - Sport",
            "  P       - Sport+",
            "  T       - Track",
            "",
            "Other Commands:",
            "  R       - Reset to idle",
            "  O       - Force overboost test",
            "  Tab     - Switch tabs",
            "  Q/Esc   - Quit",
        ];
        
        let help_items: Vec<ListItem> = help_text
            .iter()
            .map(|item| ListItem::new(*item))
            .collect();
        
        let help_list = List::new(help_items)
            .block(Block::default().title("Help").borders(Borders::ALL))
            .style(Style::default().fg(Color::White));
        
        f.render_widget(help_list, area);
    }
    
    /// Draw gauge widgets
    fn draw_gauges<B: Backend>(&self, f: &mut Frame<B>, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(33),
                Constraint::Percentage(33), 
                Constraint::Percentage(34),
            ].as_ref())
            .split(area);
        
        // Boost gauge
        if let Some(ref inputs) = self.current_inputs {
            let boost = inputs.sensors.manifold_pressure_gauge.max(0.0);
            let boost_gauge = Gauge::default()
                .block(Block::default().title("Boost Pressure (PSI)").borders(Borders::ALL))
                .gauge_style(Style::default().fg(Color::Green))
                .percent((boost / 15.0 * 100.0) as u16)
                .label(format!("{:.1} PSI", boost));
            f.render_widget(boost_gauge, chunks[0]);
            
            // RPM gauge  
            let rpm_gauge = Gauge::default()
                .block(Block::default().title("Engine RPM").borders(Borders::ALL))
                .gauge_style(Style::default().fg(Color::Blue))
                .percent((inputs.can.rpm as f32 / 7000.0 * 100.0) as u16)
                .label(format!("{} RPM", inputs.can.rpm));
            f.render_widget(rpm_gauge, chunks[1]);
        }
        
        // Duty cycle gauge
        let duty_gauge = Gauge::default()
            .block(Block::default().title("Solenoid Duty Cycle").borders(Borders::ALL))
            .gauge_style(Style::default().fg(Color::Red))
            .percent(self.current_duty as u16)
            .label(format!("{:.1}%", self.current_duty));
        f.render_widget(duty_gauge, chunks[2]);
    }
    
    /// Draw system information
    fn draw_system_info<B: Backend>(&self, f: &mut Frame<B>, area: Rect) {
        let mut info_lines = vec![
            Line::from(vec![
                Span::styled("System State: ", Style::default().fg(Color::Yellow)),
                Span::styled(self.system_state.description(), Style::default().fg(Color::White)),
            ]),
            Line::from(vec![
                Span::styled("Drive Mode: ", Style::default().fg(Color::Yellow)),
                Span::styled(format!("{:?}", self.ui_state.drive_mode), Style::default().fg(Color::White)),
            ]),
            Line::from(vec![
                Span::styled("Throttle: ", Style::default().fg(Color::Yellow)),
                Span::styled(format!("{:.1}%", self.ui_state.throttle_position), Style::default().fg(Color::White)),
            ]),
        ];
        
        if let Some(ref inputs) = self.current_inputs {
            info_lines.extend(vec![
                Line::from(""),
                Line::from(vec![
                    Span::styled("Torque Data:", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                ]),
                Line::from(vec![
                    Span::styled("  Desired: ", Style::default().fg(Color::Yellow)),
                    Span::styled(format!("{:.1} Nm", inputs.can.desired_torque), Style::default().fg(Color::White)),
                ]),
                Line::from(vec![
                    Span::styled("  Actual:  ", Style::default().fg(Color::Yellow)),
                    Span::styled(format!("{:.1} Nm", inputs.can.actual_torque), Style::default().fg(Color::White)),
                ]),
                Line::from(vec![
                    Span::styled("  Error:   ", Style::default().fg(Color::Yellow)),
                    Span::styled(format!("{:.1} Nm", inputs.can.desired_torque - inputs.can.actual_torque), Style::default().fg(Color::White)),
                ]),
                Line::from(""),
                Line::from(vec![
                    Span::styled("Pressures:", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                ]),
                Line::from(vec![
                    Span::styled("  Manifold: ", Style::default().fg(Color::Yellow)),
                    Span::styled(format!("{:.1} PSI", inputs.sensors.manifold_pressure_gauge), Style::default().fg(Color::White)),
                ]),
                Line::from(vec![
                    Span::styled("  Dome In:  ", Style::default().fg(Color::Yellow)),
                    Span::styled(format!("{:.1} PSI", inputs.sensors.dome_input_pressure), Style::default().fg(Color::White)),
                ]),
                Line::from(vec![
                    Span::styled("  Upper:    ", Style::default().fg(Color::Yellow)),
                    Span::styled(format!("{:.1} PSI", inputs.sensors.upper_dome_pressure), Style::default().fg(Color::White)),
                ]),
            ]);
        }
        
        let info = Paragraph::new(Text::from(info_lines))
            .block(Block::default().title("System Information").borders(Borders::ALL))
            .wrap(ratatui::widgets::Wrap { trim: true });
        
        f.render_widget(info, area);
    }
    
    /// Draw boost and RPM chart
    fn draw_boost_rpm_chart<B: Backend>(&self, f: &mut Frame<B>, area: Rect) {
        let datasets = vec![
            Dataset::default()
                .name("Boost Pressure")
                .marker(symbols::Marker::Braille)
                .style(Style::default().fg(Color::Green))
                .data(&self.history.boost_pressure),
            Dataset::default()
                .name("RPM/100")
                .marker(symbols::Marker::Braille)
                .style(Style::default().fg(Color::Blue))
                .data(&self.history.rpm),
        ];
        
        let chart = Chart::new(datasets)
            .block(Block::default().title("Boost Pressure & RPM").borders(Borders::ALL))
            .x_axis(Axis::default().title("Time (s)").bounds([0.0, 60.0]))
            .y_axis(Axis::default().title("Value").bounds([0.0, 100.0]));
        
        f.render_widget(chart, area);
    }
    
    /// Draw torque chart
    fn draw_torque_chart<B: Backend>(&self, f: &mut Frame<B>, area: Rect) {
        let datasets = vec![
            Dataset::default()
                .name("Desired Torque")
                .marker(symbols::Marker::Braille)
                .style(Style::default().fg(Color::Yellow))
                .data(&self.history.desired_torque),
            Dataset::default()
                .name("Actual Torque")
                .marker(symbols::Marker::Braille)
                .style(Style::default().fg(Color::Red))
                .data(&self.history.actual_torque),
        ];
        
        let chart = Chart::new(datasets)
            .block(Block::default().title("Torque Tracking").borders(Borders::ALL))
            .x_axis(Axis::default().title("Time (s)").bounds([0.0, 60.0]))
            .y_axis(Axis::default().title("Torque (Nm)").bounds([0.0, 600.0]));
        
        f.render_widget(chart, area);
    }
    
    /// Draw status bar
    fn draw_status_bar<B: Backend>(&self, f: &mut Frame<B>, area: Rect) {
        let tabs = ["Dashboard", "Charts", "Control"];
        let tab_text = tabs.iter().enumerate().map(|(i, &tab)| {
            if i == self.ui_state.active_tab {
                Span::styled(format!("[{}]", tab), Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
            } else {
                Span::styled(format!(" {} ", tab), Style::default().fg(Color::Gray))
            }
        }).collect::<Vec<_>>();
        
        let status = Paragraph::new(Line::from(tab_text))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL));
        
        f.render_widget(status, area);
    }
    
    /// Draw error popup
    fn draw_error_popup<B: Backend>(&self, f: &mut Frame<B>, area: Rect, error: &str) {
        let popup_area = centered_rect(60, 20, area);
        f.render_widget(Clear, popup_area);
        
        let error_popup = Paragraph::new(error)
            .block(Block::default().title("Error").borders(Borders::ALL))
            .style(Style::default().fg(Color::Red))
            .alignment(Alignment::Center)
            .wrap(ratatui::widgets::Wrap { trim: true });
        
        f.render_widget(error_popup, popup_area);
    }
}

impl SimulationHistory {
    fn new() -> Self {
        Self {
            time_points: Vec::new(),
            boost_pressure: Vec::new(),
            rpm: Vec::new(),
            duty_cycle: Vec::new(),
            desired_torque: Vec::new(),
            actual_torque: Vec::new(),
            max_points: 1000,
        }
    }
    
    fn add_data_point(
        &mut self,
        time: f64,
        boost: f64,
        rpm: f64,
        duty: f64,
        desired_torque: f64,
        actual_torque: f64,
    ) {
        self.time_points.push(time);
        self.boost_pressure.push((time, boost));
        self.rpm.push((time, rpm / 100.0)); // Scale for display
        self.duty_cycle.push((time, duty));
        self.desired_torque.push((time, desired_torque));
        self.actual_torque.push((time, actual_torque));
        
        // Keep only recent data
        if self.time_points.len() > self.max_points {
            self.time_points.remove(0);
            self.boost_pressure.remove(0);
            self.rpm.remove(0);
            self.duty_cycle.remove(0);
            self.desired_torque.remove(0);
            self.actual_torque.remove(0);
        }
    }
}

/// Helper function to create a centered rectangle
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ].as_ref())
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ].as_ref())
        .split(popup_layout[1])[1]
}