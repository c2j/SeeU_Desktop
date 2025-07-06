/// Terminal help content and feature descriptions
use std::collections::HashMap;

/// Represents a help section with title and content
#[derive(Debug, Clone)]
pub struct HelpSection {
    pub title: String,
    pub content: String,
    pub subsections: Vec<HelpSubsection>,
}

/// Represents a subsection within a help section
#[derive(Debug, Clone)]
pub struct HelpSubsection {
    pub title: String,
    pub content: String,
    pub examples: Vec<String>,
}

/// Terminal help content manager
#[derive(Debug)]
pub struct TerminalHelpContent {
    sections: HashMap<String, HelpSection>,
}

impl TerminalHelpContent {
    /// Create a new help content manager with default content
    pub fn new() -> Self {
        let mut content = Self {
            sections: HashMap::new(),
        };
        content.initialize_default_content();
        content
    }

    /// Get all help sections
    pub fn get_sections(&self) -> Vec<&HelpSection> {
        self.sections.values().collect()
    }

    /// Get a specific help section by key
    pub fn get_section(&self, key: &str) -> Option<&HelpSection> {
        self.sections.get(key)
    }

    /// Initialize default help content
    fn initialize_default_content(&mut self) {
        // Terminal Features Overview
        self.add_section("overview", HelpSection {
            title: "🖥️ iTerminal 功能概览".to_string(),
            content: "iTerminal 是基于 Alacritty 终端引擎的现代化终端模拟器，提供高性能、GPU 加速的终端体验。".to_string(),
            subsections: vec![
                HelpSubsection {
                    title: "核心特性".to_string(),
                    content: "• 基于 Alacritty 的高性能 GPU 渲染\n• 会话管理和历史记录\n• 多标签页支持\n• 导出功能（Markdown、HTML、纯文本）\n• 可配置的外观和行为".to_string(),
                    examples: vec![],
                },
                HelpSubsection {
                    title: "性能优势".to_string(),
                    content: "• GPU 加速渲染，流畅的滚动和响应\n• 低延迟输入处理\n• 高效的内存使用\n• 跨平台兼容性".to_string(),
                    examples: vec![],
                },
            ],
        });

        // Alacritty Features
        self.add_section("alacritty", HelpSection {
            title: "⚡ Alacritty 特色功能".to_string(),
            content: "iTerminal 继承了 Alacritty 的所有强大功能，提供最佳的终端性能体验。".to_string(),
            subsections: vec![
                HelpSubsection {
                    title: "GPU 加速渲染".to_string(),
                    content: "使用 OpenGL 进行硬件加速渲染，确保即使在大量文本输出时也能保持流畅。".to_string(),
                    examples: vec![
                        "大文件查看: cat large_file.txt".to_string(),
                        "日志实时监控: tail -f /var/log/system.log".to_string(),
                        "编译输出: cargo build --verbose".to_string(),
                    ],
                },
                HelpSubsection {
                    title: "Unicode 和 Emoji 支持".to_string(),
                    content: "完整支持 Unicode 字符集，包括 Emoji、中文、日文等多语言字符。".to_string(),
                    examples: vec![
                        "echo '🚀 Hello 世界 こんにちは'".to_string(),
                        "ls -la 📁文件夹".to_string(),
                        "git log --oneline --graph".to_string(),
                    ],
                },
                HelpSubsection {
                    title: "真彩色支持".to_string(),
                    content: "支持 24-bit 真彩色显示，提供丰富的颜色表现。".to_string(),
                    examples: vec![
                        "curl -s https://raw.githubusercontent.com/JohnMorales/dotfiles/master/colors/24-bit-color.sh | bash".to_string(),
                        "ls --color=always".to_string(),
                        "vim with colorscheme".to_string(),
                    ],
                },
                HelpSubsection {
                    title: "字体渲染优化".to_string(),
                    content: "高质量的字体渲染，支持连字（ligatures）和字体回退。".to_string(),
                    examples: vec![
                        "支持编程字体连字: != >= <= => ->".to_string(),
                        "中英文混排显示优化".to_string(),
                        "等宽字体精确对齐".to_string(),
                    ],
                },
            ],
        });

        // Session Management
        self.add_section("sessions", HelpSection {
            title: "📚 会话管理".to_string(),
            content: "强大的会话管理功能，让您可以保存、恢复和组织终端工作环境。".to_string(),
            subsections: vec![
                HelpSubsection {
                    title: "会话操作".to_string(),
                    content: "• 创建新会话: 点击 '+ New Session'\n• 关闭会话: 点击 'Close Session'\n• 保存会话: 点击 '💾 Save Session'\n• 查看历史: 点击 '📚 Session History'".to_string(),
                    examples: vec![
                        "开发会话: 保存包含项目目录和运行状态的会话".to_string(),
                        "系统监控: 保存包含 htop、tail 等监控命令的会话".to_string(),
                        "数据库操作: 保存数据库连接和查询历史".to_string(),
                    ],
                },
                HelpSubsection {
                    title: "会话历史".to_string(),
                    content: "• 自动保存工作目录和环境变量\n• 支持标签和备注组织\n• 全文搜索功能\n• 安全的内容存储".to_string(),
                    examples: vec![
                        "按项目标签组织会话".to_string(),
                        "搜索特定命令或输出".to_string(),
                        "恢复上次工作环境".to_string(),
                    ],
                },
            ],
        });

        // Export Features
        self.add_section("export", HelpSection {
            title: "📤 导出功能".to_string(),
            content: "将终端内容导出为多种格式，便于文档编写和分享。".to_string(),
            subsections: vec![
                HelpSubsection {
                    title: "支持格式".to_string(),
                    content: "• Markdown: 适合技术文档和笔记\n• HTML: 保留颜色和格式的网页格式\n• 纯文本: 简单的文本格式\n• 剪贴板: 快速复制到其他应用".to_string(),
                    examples: vec![
                        "导出命令执行过程到文档".to_string(),
                        "分享彩色的日志输出".to_string(),
                        "保存重要的系统信息".to_string(),
                    ],
                },
                HelpSubsection {
                    title: "导出选项".to_string(),
                    content: "• 选择导出范围（全部/选定内容）\n• 包含/排除颜色信息\n• 自定义文件名和路径\n• 批量导出多个会话".to_string(),
                    examples: vec![
                        "只导出错误信息部分".to_string(),
                        "保留 ANSI 颜色代码".to_string(),
                        "按日期组织导出文件".to_string(),
                    ],
                },
            ],
        });

        // Keyboard Shortcuts
        self.add_section("shortcuts", HelpSection {
            title: "⌨️ 键盘快捷键".to_string(),
            content: "提高效率的键盘快捷键和终端操作技巧。".to_string(),
            subsections: vec![
                HelpSubsection {
                    title: "基本快捷键".to_string(),
                    content: "• Ctrl+C: 中断当前命令\n• Ctrl+D: 退出当前会话\n• Ctrl+L: 清屏\n• Ctrl+R: 搜索命令历史\n• Ctrl+A: 光标移到行首\n• Ctrl+E: 光标移到行尾".to_string(),
                    examples: vec![
                        "Ctrl+C 停止长时间运行的命令".to_string(),
                        "Ctrl+R 快速查找之前执行的命令".to_string(),
                        "Ctrl+L 清理屏幕内容".to_string(),
                    ],
                },
                HelpSubsection {
                    title: "高级操作".to_string(),
                    content: "• Ctrl+Z: 暂停进程到后台\n• fg: 恢复后台进程\n• bg: 后台运行进程\n• jobs: 查看后台任务\n• history: 查看命令历史".to_string(),
                    examples: vec![
                        "Ctrl+Z 暂停 vim 编辑器".to_string(),
                        "fg 恢复暂停的编辑器".to_string(),
                        "nohup command & 后台运行命令".to_string(),
                    ],
                },
                HelpSubsection {
                    title: "文本选择和复制".to_string(),
                    content: "• 鼠标拖拽: 选择文本\n• 双击: 选择单词\n• 三击: 选择整行\n• Ctrl+Shift+C: 复制选中文本\n• Ctrl+Shift+V: 粘贴文本".to_string(),
                    examples: vec![
                        "选择命令输出进行复制".to_string(),
                        "快速选择文件路径".to_string(),
                        "复制错误信息到剪贴板".to_string(),
                    ],
                },
            ],
        });

        // Configuration
        self.add_section("config", HelpSection {
            title: "⚙️ 配置选项".to_string(),
            content: "自定义终端外观和行为，打造个性化的使用体验。".to_string(),
            subsections: vec![
                HelpSubsection {
                    title: "外观设置".to_string(),
                    content: "• 字体选择和大小调整\n• 颜色主题配置\n• 透明度和模糊效果\n• 窗口大小和位置".to_string(),
                    examples: vec![
                        "设置编程字体如 Fira Code".to_string(),
                        "选择深色或浅色主题".to_string(),
                        "调整终端透明度".to_string(),
                    ],
                },
                HelpSubsection {
                    title: "行为设置".to_string(),
                    content: "• 滚动缓冲区大小\n• 光标样式和闪烁\n• 鼠标操作行为\n• 启动时的默认设置".to_string(),
                    examples: vec![
                        "增加滚动历史行数".to_string(),
                        "设置光标为块状或线状".to_string(),
                        "配置鼠标滚轮行为".to_string(),
                    ],
                },
            ],
        });

        // Tips and Tricks
        self.add_section("tips", HelpSection {
            title: "💡 使用技巧".to_string(),
            content: "提高终端使用效率的实用技巧和最佳实践。".to_string(),
            subsections: vec![
                HelpSubsection {
                    title: "命令行技巧".to_string(),
                    content: "• 使用 Tab 键自动补全\n• 使用 !! 重复上一个命令\n• 使用 !string 执行以 string 开头的最近命令\n• 使用 alias 创建命令别名".to_string(),
                    examples: vec![
                        "alias ll='ls -la'".to_string(),
                        "cd /very/long/path + Tab 补全".to_string(),
                        "sudo !! 以 sudo 权限重复上一命令".to_string(),
                    ],
                },
                HelpSubsection {
                    title: "工作流优化".to_string(),
                    content: "• 使用会话保存工作环境\n• 利用导出功能记录操作过程\n• 合理组织会话标签和备注\n• 定期清理不需要的会话".to_string(),
                    examples: vec![
                        "为不同项目创建专门的会话".to_string(),
                        "导出重要的配置过程".to_string(),
                        "使用标签分类开发和运维会话".to_string(),
                    ],
                },
                HelpSubsection {
                    title: "性能优化".to_string(),
                    content: "• 避免在终端中显示过大的文件\n• 使用 less 或 more 分页查看长输出\n• 定期清理命令历史\n• 合理设置滚动缓冲区大小".to_string(),
                    examples: vec![
                        "less large_file.txt 而不是 cat".to_string(),
                        "command | head -100 限制输出行数".to_string(),
                        "history -c 清理命令历史".to_string(),
                    ],
                },
            ],
        });
    }

    /// Add a new help section
    fn add_section(&mut self, key: &str, section: HelpSection) {
        self.sections.insert(key.to_string(), section);
    }

    /// Get section keys in a logical order
    pub fn get_section_keys(&self) -> Vec<&str> {
        vec![
            "overview",
            "alacritty", 
            "sessions",
            "export",
            "shortcuts",
            "config",
            "tips",
        ]
    }
}

impl Default for TerminalHelpContent {
    fn default() -> Self {
        Self::new()
    }
}
