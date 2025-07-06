use crate::help_content::{TerminalHelpContent, HelpSection, HelpSubsection};

/// UI state for the terminal help dialog
#[derive(Debug)]
pub struct TerminalHelpUI {
    /// Whether the help dialog is open
    pub is_open: bool,
    /// Currently selected section
    pub selected_section: Option<String>,
    /// Help content manager
    help_content: TerminalHelpContent,
    /// Search query for filtering help content
    pub search_query: String,
    /// Whether to show the search results
    pub show_search_results: bool,
}

impl Default for TerminalHelpUI {
    fn default() -> Self {
        Self {
            is_open: false,
            selected_section: Some("overview".to_string()),
            help_content: TerminalHelpContent::new(),
            search_query: String::new(),
            show_search_results: false,
        }
    }
}

impl TerminalHelpUI {
    /// Open the help dialog
    pub fn open(&mut self) {
        self.is_open = true;
        if self.selected_section.is_none() {
            self.selected_section = Some("overview".to_string());
        }
    }

    /// Close the help dialog
    pub fn close(&mut self) {
        self.is_open = false;
        self.search_query.clear();
        self.show_search_results = false;
    }

    /// Render the help dialog
    pub fn render(&mut self, ctx: &eframe::egui::Context) {
        if !self.is_open {
            return;
        }

        eframe::egui::Window::new("❓ iTerminal 帮助")
            .default_width(900.0)
            .default_height(700.0)
            .resizable(true)
            .collapsible(false)
            .show(ctx, |ui| {
                self.render_content(ui);
            });
    }

    /// Render the main content of the help dialog
    fn render_content(&mut self, ui: &mut eframe::egui::Ui) {
        ui.horizontal(|ui| {
            // Left sidebar with sections
            ui.vertical(|ui| {
                ui.set_width(200.0);
                self.render_sidebar(ui);
            });

            ui.separator();

            // Main content area
            ui.vertical(|ui| {
                self.render_main_content(ui);
            });
        });

        ui.separator();

        // Bottom buttons
        ui.horizontal(|ui| {
            if ui.button("❌ 关闭").clicked() {
                self.close();
            }

            ui.with_layout(eframe::egui::Layout::right_to_left(eframe::egui::Align::Center), |ui| {
                if ui.button("🔍 搜索帮助").clicked() {
                    self.show_search_results = !self.show_search_results;
                }
            });
        });
    }

    /// Render the sidebar with section navigation
    fn render_sidebar(&mut self, ui: &mut eframe::egui::Ui) {
        ui.heading("📖 帮助目录");
        ui.add_space(10.0);

        // Search box
        if self.show_search_results {
            ui.horizontal(|ui| {
                ui.label("🔍");
                ui.add(eframe::egui::TextEdit::singleline(&mut self.search_query)
                    .id(eframe::egui::Id::new("help_search_query"))
                    .hint_text("搜索帮助内容..."));
            });
            ui.add_space(5.0);
        }

        eframe::egui::ScrollArea::vertical()
            .id_source("help_sidebar")
            .show(ui, |ui| {
                if self.show_search_results && !self.search_query.is_empty() {
                    self.render_search_results(ui);
                } else {
                    self.render_section_list(ui);
                }
            });
    }

    /// Render the list of help sections
    fn render_section_list(&mut self, ui: &mut eframe::egui::Ui) {
        for section_key in self.help_content.get_section_keys() {
            if let Some(section) = self.help_content.get_section(section_key) {
                let is_selected = self.selected_section.as_ref() == Some(&section_key.to_string());
                
                let response = ui.selectable_label(is_selected, &section.title);
                
                if response.clicked() {
                    self.selected_section = Some(section_key.to_string());
                    self.show_search_results = false;
                    self.search_query.clear();
                }
            }
        }
    }

    /// Render search results
    fn render_search_results(&mut self, ui: &mut eframe::egui::Ui) {
        let query = self.search_query.to_lowercase();
        let mut found_results = false;

        for section_key in self.help_content.get_section_keys() {
            if let Some(section) = self.help_content.get_section(section_key) {
                // Check if section title or content matches
                let section_matches = section.title.to_lowercase().contains(&query) ||
                                    section.content.to_lowercase().contains(&query);
                
                // Check subsections
                let subsection_matches = section.subsections.iter().any(|sub| {
                    sub.title.to_lowercase().contains(&query) ||
                    sub.content.to_lowercase().contains(&query) ||
                    sub.examples.iter().any(|ex| ex.to_lowercase().contains(&query))
                });

                if section_matches || subsection_matches {
                    found_results = true;
                    let response = ui.selectable_label(false, format!("🔍 {}", section.title));
                    
                    if response.clicked() {
                        self.selected_section = Some(section_key.to_string());
                        self.show_search_results = false;
                        self.search_query.clear();
                    }
                }
            }
        }

        if !found_results {
            ui.label("🔍 未找到匹配的帮助内容");
        }
    }

    /// Render the main content area
    fn render_main_content(&mut self, ui: &mut eframe::egui::Ui) {
        if let Some(section_key) = &self.selected_section.clone() {
            if let Some(section) = self.help_content.get_section(section_key) {
                self.render_section_content(ui, section);
            }
        } else {
            ui.vertical_centered(|ui| {
                ui.add_space(50.0);
                ui.heading("👈 请选择一个帮助主题");
                ui.label("从左侧目录中选择您想了解的功能");
            });
        }
    }

    /// Render a specific help section
    fn render_section_content(&self, ui: &mut eframe::egui::Ui, section: &HelpSection) {
        eframe::egui::ScrollArea::vertical()
            .id_source("help_content")
            .show(ui, |ui| {
                // Section title
                ui.heading(&section.title);
                ui.add_space(10.0);

                // Section description
                if !section.content.is_empty() {
                    ui.label(&section.content);
                    ui.add_space(15.0);
                }

                // Subsections
                for subsection in &section.subsections {
                    self.render_subsection(ui, subsection);
                    ui.add_space(10.0);
                }
            });
    }

    /// Render a subsection
    fn render_subsection(&self, ui: &mut eframe::egui::Ui, subsection: &HelpSubsection) {
        // Subsection title
        ui.horizontal(|ui| {
            ui.label("▶");
            ui.heading(&subsection.title);
        });
        ui.add_space(5.0);

        // Subsection content
        if !subsection.content.is_empty() {
            // Split content by newlines and render each line
            for line in subsection.content.lines() {
                if line.trim().starts_with('•') {
                    // Bullet point
                    ui.horizontal(|ui| {
                        ui.add_space(20.0);
                        ui.label(line.trim());
                    });
                } else {
                    // Regular text
                    ui.horizontal(|ui| {
                        ui.add_space(20.0);
                        ui.label(line);
                    });
                }
            }
            ui.add_space(5.0);
        }

        // Examples
        if !subsection.examples.is_empty() {
            ui.horizontal(|ui| {
                ui.add_space(20.0);
                ui.label("📝 示例:");
            });
            
            for example in &subsection.examples {
                ui.horizontal(|ui| {
                    ui.add_space(40.0);
                    ui.code(example);
                });
            }
        }
    }
}

/// Actions that can be triggered from the help UI
#[derive(Debug, Clone)]
pub enum HelpAction {
    /// Open a specific help section
    OpenSection(String),
    /// Search for help content
    Search(String),
    /// Close the help dialog
    Close,
}
