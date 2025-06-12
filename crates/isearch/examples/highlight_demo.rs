/// Demo showing the improved keyword highlighting functionality
use eframe::egui;
use isearch::utils::{extract_search_terms, create_highlighted_rich_text, highlight_search_terms};

fn main() -> Result<(), eframe::Error> {
    println!("=== SeeU Desktop 关键词高亮演示 ===");
    
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([800.0, 600.0]),
        ..Default::default()
    };
    
    eframe::run_native(
        "关键词高亮演示",
        options,
        Box::new(|_cc| Ok(Box::new(HighlightDemoApp::default()))),
    )
}

#[derive(Default)]
struct HighlightDemoApp {
    search_query: String,
    sample_texts: Vec<String>,
}

impl HighlightDemoApp {
    fn new() -> Self {
        Self {
            search_query: "Rust 安全 并发".to_string(),
            sample_texts: vec![
                "Rust是一种系统编程语言，专注于安全、速度和并发。".to_string(),
                "反对 **澳大利亚** 澳大利亚 反对 **玻利维亚** 南美洲 反对 **巴西** 南美洲 反对 **中非** 非洲 反对 **乍得** 非洲".to_string(),
                "这是一个包含中文字符的测试文档。Rust语言的主要特点包括内存安全和并发安全。".to_string(),
                "测试🎉emoji🚀和中文混合💻内容处理，Rust提供了安全的内存管理。".to_string(),
                "Hello world! This is a test document with Rust programming language features.".to_string(),
            ],
        }
    }
}

impl Default for HighlightDemoApp {
    fn default() -> Self {
        Self::new()
    }
}

impl eframe::App for HighlightDemoApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("🔍 SeeU Desktop 关键词高亮演示");
            ui.add_space(20.0);
            
            // Search input
            ui.horizontal(|ui| {
                ui.label("搜索查询:");
                ui.add(
                    egui::TextEdit::singleline(&mut self.search_query)
                        .hint_text("输入搜索关键词...")
                        .desired_width(300.0)
                );
            });
            
            ui.add_space(10.0);
            
            // Extract search terms
            let search_terms = extract_search_terms(&self.search_query);
            ui.horizontal(|ui| {
                ui.label("提取的搜索词:");
                ui.label(format!("{:?}", search_terms));
            });
            
            ui.add_space(20.0);
            ui.separator();
            ui.add_space(10.0);
            
            // Demo sections
            egui::ScrollArea::vertical().show(ui, |ui| {
                // Section 1: Rich Text Highlighting
                ui.group(|ui| {
                    ui.vertical(|ui| {
                        ui.label(egui::RichText::new("1. 富文本高亮效果 (推荐)").heading().strong());
                        ui.add_space(10.0);
                        
                        for (i, text) in self.sample_texts.iter().enumerate() {
                            ui.horizontal(|ui| {
                                ui.label(format!("示例 {}:", i + 1));
                            });
                            
                            if !search_terms.is_empty() {
                                let highlighted_job = create_highlighted_rich_text(text, &search_terms);
                                ui.add(egui::Label::new(highlighted_job).wrap());
                            } else {
                                ui.add(egui::Label::new(text).wrap());
                            }
                            
                            ui.add_space(8.0);
                        }
                    });
                });
                
                ui.add_space(15.0);
                
                // Section 2: Text Marker Highlighting
                ui.group(|ui| {
                    ui.vertical(|ui| {
                        ui.label(egui::RichText::new("2. 文本标记高亮效果 (兼容模式)").heading().strong());
                        ui.add_space(10.0);
                        
                        for (i, text) in self.sample_texts.iter().enumerate() {
                            ui.horizontal(|ui| {
                                ui.label(format!("示例 {}:", i + 1));
                            });
                            
                            let highlighted_text = if !search_terms.is_empty() {
                                highlight_search_terms(text, &search_terms)
                            } else {
                                text.clone()
                            };
                            
                            ui.add(egui::Label::new(highlighted_text).wrap());
                            ui.add_space(8.0);
                        }
                    });
                });
                
                ui.add_space(15.0);
                
                // Section 3: Comparison
                ui.group(|ui| {
                    ui.vertical(|ui| {
                        ui.label(egui::RichText::new("3. 效果对比").heading().strong());
                        ui.add_space(10.0);
                        
                        let sample_text = "Rust是一种系统编程语言，专注于安全、速度和并发。";
                        
                        ui.label("原始文本:");
                        ui.add(egui::Label::new(sample_text).wrap());
                        ui.add_space(5.0);
                        
                        if !search_terms.is_empty() {
                            ui.label("富文本高亮:");
                            let highlighted_job = create_highlighted_rich_text(sample_text, &search_terms);
                            ui.add(egui::Label::new(highlighted_job).wrap());
                            ui.add_space(5.0);
                            
                            ui.label("文本标记高亮:");
                            let marked_text = highlight_search_terms(sample_text, &search_terms);
                            ui.add(egui::Label::new(marked_text).wrap());
                        }
                    });
                });
                
                ui.add_space(15.0);
                
                // Section 4: Features
                ui.group(|ui| {
                    ui.vertical(|ui| {
                        ui.label(egui::RichText::new("4. 功能特性").heading().strong());
                        ui.add_space(10.0);
                        
                        ui.label("✅ Unicode安全 - 正确处理中文、emoji等多字节字符");
                        ui.label("✅ 智能匹配 - 大小写不敏感，支持部分匹配");
                        ui.label("✅ 重叠处理 - 智能处理重叠的搜索词");
                        ui.label("✅ 视觉突出 - 使用颜色高亮，提升可读性");
                        ui.label("✅ 性能优化 - 高效的字符级匹配算法");
                        ui.label("✅ 引号支持 - 支持\"完整短语\"搜索");
                        ui.label("✅ 操作符过滤 - 自动过滤filetype:等操作符");
                    });
                });
                
                ui.add_space(15.0);
                
                // Section 5: Test Cases
                ui.group(|ui| {
                    ui.vertical(|ui| {
                        ui.label(egui::RichText::new("5. 测试用例").heading().strong());
                        ui.add_space(10.0);
                        
                        let test_cases = vec![
                            ("中文测试", "安全 并发"),
                            ("英文测试", "Rust programming"),
                            ("混合测试", "Rust 安全"),
                            ("短语测试", "\"系统编程语言\""),
                            ("Emoji测试", "🎉 💻"),
                            ("操作符测试", "Rust filetype:rs +memory"),
                        ];
                        
                        for (name, query) in test_cases {
                            ui.horizontal(|ui| {
                                ui.label(format!("{}:", name));
                                if ui.small_button(query).clicked() {
                                    self.search_query = query.to_string();
                                }
                            });
                        }
                    });
                });
            });
        });
    }
}
