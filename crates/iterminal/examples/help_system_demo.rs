use iterminal::help_content::{TerminalHelpContent};
use iterminal::help_ui::TerminalHelpUI;

fn main() {
    println!("iTerminal Help System Demo");
    println!("==========================");
    
    // Initialize help content
    let help_content = TerminalHelpContent::new();
    println!("✅ Help content initialized successfully");
    
    // Display overview
    println!("\n📊 Help System Overview:");
    let sections = help_content.get_sections();
    println!("  Total sections: {}", sections.len());
    
    let total_subsections: usize = sections.iter()
        .map(|s| s.subsections.len())
        .sum();
    println!("  Total subsections: {}", total_subsections);
    
    let total_examples: usize = sections.iter()
        .flat_map(|s| &s.subsections)
        .map(|sub| sub.examples.len())
        .sum();
    println!("  Total examples: {}", total_examples);
    
    // Display section list
    println!("\n📖 Available Help Sections:");
    for (i, section_key) in help_content.get_section_keys().iter().enumerate() {
        if let Some(section) = help_content.get_section(section_key) {
            println!("  {}. {}", i + 1, section.title);
            println!("     Subsections: {}", section.subsections.len());
            let examples_count: usize = section.subsections.iter()
                .map(|sub| sub.examples.len())
                .sum();
            if examples_count > 0 {
                println!("     Examples: {}", examples_count);
            }
        }
    }
    
    // Demonstrate specific sections
    println!("\n🔍 Section Details:");
    
    // Overview section
    if let Some(overview) = help_content.get_section("overview") {
        println!("\n📋 {}", overview.title);
        println!("   {}", overview.content);
        for subsection in &overview.subsections {
            println!("   ▶ {}", subsection.title);
            // Show first line of content
            if let Some(first_line) = subsection.content.lines().next() {
                println!("     {}", first_line);
            }
        }
    }
    
    // Alacritty features section
    if let Some(alacritty) = help_content.get_section("alacritty") {
        println!("\n⚡ {}", alacritty.title);
        println!("   {}", alacritty.content);
        
        for subsection in &alacritty.subsections {
            println!("   ▶ {}", subsection.title);
            if !subsection.examples.is_empty() {
                println!("     Examples:");
                for (i, example) in subsection.examples.iter().take(2).enumerate() {
                    println!("       {}. {}", i + 1, example);
                }
                if subsection.examples.len() > 2 {
                    println!("       ... and {} more", subsection.examples.len() - 2);
                }
            }
        }
    }
    
    // Session management section
    if let Some(sessions) = help_content.get_section("sessions") {
        println!("\n📚 {}", sessions.title);
        println!("   {}", sessions.content);
        
        for subsection in &sessions.subsections {
            println!("   ▶ {}", subsection.title);
            // Show bullet points
            for line in subsection.content.lines().take(3) {
                if line.trim().starts_with('•') {
                    println!("     {}", line.trim());
                }
            }
        }
    }
    
    // Export features section
    if let Some(export) = help_content.get_section("export") {
        println!("\n📤 {}", export.title);
        println!("   {}", export.content);
        
        for subsection in &export.subsections {
            println!("   ▶ {}", subsection.title);
            // Count features mentioned
            let feature_count = subsection.content.matches('•').count();
            if feature_count > 0 {
                println!("     Features: {}", feature_count);
            }
        }
    }
    
    // Keyboard shortcuts section
    if let Some(shortcuts) = help_content.get_section("shortcuts") {
        println!("\n⌨️ {}", shortcuts.title);
        
        for subsection in &shortcuts.subsections {
            println!("   ▶ {}", subsection.title);
            // Show shortcuts
            for line in subsection.content.lines() {
                if line.contains("Ctrl+") {
                    println!("     {}", line.trim());
                }
            }
        }
    }
    
    // Tips section
    if let Some(tips) = help_content.get_section("tips") {
        println!("\n💡 {}", tips.title);
        
        for subsection in &tips.subsections {
            println!("   ▶ {}", subsection.title);
            if !subsection.examples.is_empty() {
                println!("     Sample tip: {}", subsection.examples[0]);
            }
        }
    }
    
    // Test UI state management
    println!("\n🖥️ UI State Management Demo:");
    let mut help_ui = TerminalHelpUI::default();
    
    println!("  Initial state - Open: {}", help_ui.is_open);
    println!("  Selected section: {:?}", help_ui.selected_section);
    
    help_ui.open();
    println!("  After open() - Open: {}", help_ui.is_open);
    println!("  Selected section: {:?}", help_ui.selected_section);
    
    help_ui.close();
    println!("  After close() - Open: {}", help_ui.is_open);
    println!("  Search query cleared: {}", help_ui.search_query.is_empty());
    
    // Feature coverage analysis
    println!("\n📈 Feature Coverage Analysis:");
    
    // Alacritty features coverage
    if let Some(alacritty) = help_content.get_section("alacritty") {
        let content_text = format!("{} {}", 
            alacritty.content,
            alacritty.subsections.iter()
                .map(|sub| format!("{} {}", sub.title, sub.content))
                .collect::<Vec<_>>()
                .join(" ")
        ).to_lowercase();
        
        let alacritty_features = vec![
            ("GPU acceleration", content_text.contains("gpu")),
            ("Unicode support", content_text.contains("unicode")),
            ("True color", content_text.contains("color") || content_text.contains("颜色")),
            ("Font rendering", content_text.contains("font") || content_text.contains("字体")),
        ];
        
        println!("  Alacritty Features:");
        for (feature, covered) in alacritty_features {
            let status = if covered { "✅" } else { "❌" };
            println!("    {} {}", status, feature);
        }
    }
    
    // Session management features
    if let Some(sessions) = help_content.get_section("sessions") {
        let content_text = format!("{} {}", 
            sessions.content,
            sessions.subsections.iter()
                .map(|sub| format!("{} {}", sub.title, sub.content))
                .collect::<Vec<_>>()
                .join(" ")
        ).to_lowercase();
        
        let session_features = vec![
            ("Save sessions", content_text.contains("save") || content_text.contains("保存")),
            ("Restore sessions", content_text.contains("restore") || content_text.contains("恢复")),
            ("Session history", content_text.contains("history") || content_text.contains("历史")),
            ("Search functionality", content_text.contains("search") || content_text.contains("搜索")),
        ];
        
        println!("  Session Management Features:");
        for (feature, covered) in session_features {
            let status = if covered { "✅" } else { "❌" };
            println!("    {} {}", status, feature);
        }
    }
    
    // Export features
    if let Some(export) = help_content.get_section("export") {
        let content_text = format!("{} {}", 
            export.content,
            export.subsections.iter()
                .map(|sub| format!("{} {}", sub.title, sub.content))
                .collect::<Vec<_>>()
                .join(" ")
        ).to_lowercase();
        
        let export_features = vec![
            ("Markdown export", content_text.contains("markdown")),
            ("HTML export", content_text.contains("html")),
            ("Text export", content_text.contains("text") || content_text.contains("文本")),
            ("Clipboard support", content_text.contains("clipboard") || content_text.contains("剪贴板")),
        ];
        
        println!("  Export Features:");
        for (feature, covered) in export_features {
            let status = if covered { "✅" } else { "❌" };
            println!("    {} {}", status, feature);
        }
    }
    
    println!("\n🎉 Help System Demo Completed!");
    println!("In the actual application:");
    println!("  - Click '❓ 帮助' button to open help dialog");
    println!("  - Browse sections in the left sidebar");
    println!("  - Use search functionality to find specific topics");
    println!("  - View detailed examples and usage instructions");
    println!("  - Learn about Alacritty's powerful features");
}
