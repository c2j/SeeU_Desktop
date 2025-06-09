# SeeU-Desktop (Rust + egui + eframe)

欢迎来到 SeeU-Desktop 项目！这是一个完全使用 Rust 原生技术栈构建的现代化桌面应用程序，基于 egui 和 eframe 框架实现跨平台图形界面，旨在提供极致性能和原生体验。

## ✨ 项目概述

SeeU-Desktop 采用纯 Rust 实现，摒弃了混合架构的复杂性，通过 egui 和 eframe 框架提供高性能、低延迟的用户界面。项目特点包括：

- **纯 Rust 实现**：从核心逻辑到界面渲染，全部使用 Rust 语言
- **高性能渲染**：基于 egui 的即时模式 GUI，渲染效率高，资源占用低
- **模块化设计**：核心功能以独立模块形式实现，便于维护和扩展
- **跨平台原生体验**：同时支持 Windows、macOS 和 Linux，提供一致的原生体验
- **低资源占用**：相比混合架构，内存占用更少，启动更快

## 🚀 主要功能

### 核心功能

- **跨平台支持**：Windows、macOS 和 Linux 平台一致体验
- **多工作区**：终端、文件管理、数据分析等多功能工作区
- **AI 助手**：内置智能助手，支持自然语言交互
- **系统资源监控**：实时监控 CPU、内存使用情况

### 专业模块

- **iNote**：高性能笔记应用
  - 支持 Markdown 和富文本编辑
  - 树形笔记本结构
  - 标签云管理
  - 附件支持

- **iSearch**：文件搜索功能
  - 高效文件索引
  - 复合查询支持
  - 实时搜索结果
  - 多格式文件内容搜索

## 🏁 开始使用

### 先决条件

确保您的系统已安装以下软件：

- [Rust](https://www.rust-lang.org/tools/install) (建议使用最新稳定版)
- 根据您的操作系统，可能需要安装额外的构建工具：
  - **Windows**：安装 Visual Studio 构建工具
  - **macOS**：安装 Xcode 命令行工具 (`xcode-select --install`)
  - **Linux**：安装 `build-essential`、`libclang-dev` 和 `pkg-config`
- **vcpkg**（推荐）：用于跨平台依赖管理和静态链接

### 安装

1. 克隆或下载此仓库到您的本地计算机
2. 打开终端，导航到项目根目录：
   ```bash
   cd SeeU-Desktop
   ```
3. 构建项目：
   ```bash
   cargo build --release
   ```

### 运行应用程序

要启动应用程序，请运行：

```bash
cargo run --release
```

### 构建可执行文件

要为当前平台构建可执行文件，请运行：

```bash
cargo build --release
```

编译后的可执行文件将位于 `target/release/` 目录中。

## 📂 项目结构

```
SeeU-Desktop/
├── README-N.md            # 本项目说明文件
├── Cargo.toml             # Rust 项目配置
├── Cargo.lock             # 依赖锁定文件
├── assets/                # 静态资源
│   ├── fonts/             # 字体文件
│   ├── icons/             # 图标资源
│   └── themes/            # 主题配置
├── src/                   # 主程序源代码
│   ├── main.rs            # 程序入口点
│   ├── app.rs             # 应用程序主体
│   ├── ui/                # UI 组件
│   │   ├── mod.rs         # 模块定义
│   │   ├── navigation.rs  # 导航栏组件
│   │   ├── workspace.rs   # 工作区组件
│   │   ├── ai_chat.rs     # AI 助手组件
│   │   ├── status_bar.rs  # 状态栏组件
│   │   └── theme.rs       # 主题管理
│   ├── modules/           # 功能模块
│   │   ├── mod.rs         # 模块定义
│   │   ├── terminal.rs    # 终端模块
│   │   ├── file_manager.rs # 文件管理模块
│   │   └── data_analysis.rs # 数据分析模块
│   ├── services/          # 服务层
│   │   ├── mod.rs         # 模块定义
│   │   ├── ai_service.rs  # AI 服务
│   │   ├── file_service.rs # 文件服务
│   │   └── system_service.rs # 系统服务
│   └── utils/             # 工具函数
│       ├── mod.rs         # 模块定义
│       ├── config.rs      # 配置管理
│       └── logger.rs      # 日志系统
├── crates/                # 子模块 crates
│   ├── inote/             # iNote 模块
│   │   ├── src/           # 源代码
│   │   │   ├── lib.rs     # 库入口点
│   │   │   ├── notebook.rs # 笔记本管理
│   │   │   ├── note.rs    # 笔记管理
│   │   │   ├── tag.rs     # 标签管理
│   │   │   ├── ui.rs      # 用户界面
│   │   │   └── storage.rs # 数据存储
│   │   └── Cargo.toml     # 模块配置
│   └── isearch/           # iSearch 模块
│       ├── src/           # 源代码
│       │   ├── lib.rs     # 库入口点
│       │   ├── indexer.rs # 索引服务
│       │   ├── schema.rs  # 索引模式
│       │   ├── ui.rs      # 用户界面
│       │   └── watcher.rs # 文件监视
│       └── Cargo.toml     # 模块配置
└── tests/                 # 测试代码
    ├── integration_tests.rs # 集成测试
    └── ui_tests.rs        # UI 测试
```

## 🏗️ 系统架构

SeeU-Desktop 采用纯 Rust 分层架构设计，主要包括以下几个层次：

```
┌───────────────────────────────┐
│         egui Native UI        │
│  ┌───────────┬─────────────┐  │
│  │  AI Chat  │  System     │  │
│  │  Panel    │  Monitor    │  │
│  └─────┬─────┴──────┬──────┘  │
├────────┼────────────┼─────────┤
│ eframe │  Core       │ Platform│
│ Context│  Services   │ Adapter │
│        │┌─┬─┬─┬─┬─┐ │ Interface
│        ││P│F│D│S│M│ │  ┌─────┐
│        ││r│i│a│e│c│ │  │Win  │
│        ││o│l│t│c│p│ │  │API  │
│        ││c│e│a│u│ │ │  └──┬──┘
└────────┴┴─┴─┴─┴─┴─┘ └───┼───┘
                      ┌────┴────┐
                      │ Linux   │
                      │ Syscall │
                      └─────────┘
```

### 核心模块

- **egui Native UI**：基于 egui 的原生用户界面
- **eframe Context**：跨平台窗口和渲染上下文
- **Core Services**：
  - Process Manager（进程管理）
  - File Operator（文件操作）
  - Data Pipeline（数据工作流）
  - Security（安全控制）
  - MCP Protocol（扩展协议）
- **Platform Adapter**：平台适配层

## 📱 界面设计与实现

### 主界面框架

```
+---------------------------+
| 全局搜索栏（支持自然语言） |
+----+------+---------------+----+
|导航| 侧边栏| 主工作区      |右侧|
|栏  | 快速入| 多标签页      |边栏|
|    | 口    | ①终端         |(AI |
|    | 设备状| ②文件管理     |助手|
|    | 态    | ③数据分析     |可折|
|    | 插件市|               |叠) |
|    | 场    |               |    |
+----+------+---------------+----+
| 状态栏（系统资源监控/AI助手图标）|
+------------------------------------+
```

#### 实现方式

主界面使用 egui 的 `SidePanel`、`TopPanel` 和 `CentralPanel` 布局：

```rust
impl eframe::App for SeeUApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // 顶部面板 - 搜索栏
        egui::TopPanel::top("top_panel").show(ctx, |ui| {
            self.render_search_bar(ui);
        });

        // 左侧导航栏
        egui::SidePanel::left("nav_panel")
            .resizable(false)
            .default_width(48.0)
            .show(ctx, |ui| {
                self.render_navigation(ui);
            });

        // 左侧边栏
        egui::SidePanel::left("side_panel")
            .resizable(true)
            .default_width(200.0)
            .show(ctx, |ui| {
                self.render_sidebar(ui);
            });

        // 右侧边栏 (AI助手)
        if self.show_right_sidebar {
            egui::SidePanel::right("right_panel")
                .resizable(true)
                .default_width(300.0)
                .show(ctx, |ui| {
                    self.render_ai_sidebar(ui);
                });
        }

        // 主工作区
        egui::CentralPanel::default().show(ctx, |ui| {
            self.render_workspace(ui);
        });

        // 状态栏
        egui::TopPanel::bottom("status_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                // 左侧 - 系统资源监控
                ui.horizontal(|ui| {
                    let cpu = self.system_service.get_cpu_usage();
                    let memory = self.system_service.get_memory_usage();

                    ui.label(format!("CPU: {:.1}%", cpu));
                    ui.separator();
                    ui.label(format!("内存: {:.1}%", memory));
                });

                // 右侧 - 工具图标
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    // AI助手图标
                    if ui.add(egui::Button::new("🤖 AI助手")
                        .selected(self.show_right_sidebar))
                        .clicked() {
                        self.show_right_sidebar = !self.show_right_sidebar;
                    }

                    // 版本信息
                    ui.label("SeeU Desktop v0.1.0");
                });
            });
        });

        // 右侧边栏按钮点击时切换显示状态
        // 在状态栏中的AI助手按钮已经处理了这个逻辑
    }
}
```

### 导航栏实现

导航栏使用 egui 的垂直布局和图标按钮：

```rust
fn render_navigation(&mut self, ui: &mut egui::Ui) {
    ui.vertical_centered(|ui| {
        ui.add_space(8.0);

        // 搜索图标
        if ui.add(egui::Button::image(self.icons.search.clone())
            .selected(self.active_module == Module::Search))
            .clicked() {
            self.active_module = Module::Search;
        }

        // 笔记图标
        if ui.add(egui::Button::image(self.icons.note.clone())
            .selected(self.active_module == Module::Note))
            .clicked() {
            self.active_module = Module::Note;
        }

        // 终端图标
        if ui.add(egui::Button::image(self.icons.terminal.clone())
            .selected(self.active_module == Module::Terminal))
            .clicked() {
            self.active_module = Module::Terminal;
        }

        // 文件图标
        if ui.add(egui::Button::image(self.icons.files.clone())
            .selected(self.active_module == Module::Files))
            .clicked() {
            self.active_module = Module::Files;
        }

        // 数据分析图标
        if ui.add(egui::Button::image(self.icons.data.clone())
            .selected(self.active_module == Module::DataAnalysis))
            .clicked() {
            self.active_module = Module::DataAnalysis;
        }
    });
}
```

### iNote 界面

```
+---------------------------+
| 标题栏 + 搜索框           |
+-----------+---------------+
| 侧边栏    | 编辑区域       |
| 笔记本树  | Markdown/富文本|
| 标签云    | 分屏预览       |
|           |               |
|           |               |
+-----------+---------------+
| 附件托盘                  |
+---------------------------+
```

#### 实现方式

iNote 模块使用 egui 的分割面板和自定义编辑器：

```rust
fn render_inote(&mut self, ui: &mut egui::Ui) {
    // 顶部搜索和标题
    ui.horizontal(|ui| {
        ui.label("🔍");
        ui.text_edit_singleline(&mut self.note_search);
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.button("➕ 新建").clicked() {
                self.create_new_note();
            }
        });
    });

    // 主体分割面板
    egui::containers::SplitLeft::new("note_split", &mut self.note_split_ratio)
        .show(ui, |left, right| {
            // 左侧笔记本树和标签
            left.vertical(|ui| {
                self.render_notebook_tree(ui);
                ui.separator();
                self.render_tag_cloud(ui);
            });

            // 右侧编辑器
            right.vertical(|ui| {
                // 标题编辑
                ui.horizontal(|ui| {
                    ui.heading("标题: ");
                    ui.text_edit_singleline(&mut self.current_note.title);
                });

                ui.separator();

                // 编辑器类型选择
                ui.horizontal(|ui| {
                    ui.selectable_value(&mut self.editor_mode, EditorMode::Markdown, "Markdown");
                    ui.selectable_value(&mut self.editor_mode, EditorMode::RichText, "富文本");
                });

                // 编辑器内容
                match self.editor_mode {
                    EditorMode::Markdown => self.render_markdown_editor(ui),
                    EditorMode::RichText => self.render_rich_text_editor(ui),
                }
            });
        });

    // 底部附件托盘
    ui.separator();
    ui.horizontal_wrapped(|ui| {
        ui.label("附件: ");
        for attachment in &self.current_note.attachments {
            if ui.button(&attachment.name).clicked() {
                self.open_attachment(attachment);
            }
        }
        if ui.button("➕ 添加附件").clicked() {
            self.add_attachment();
        }
    });
}
```


### iSearch 界面

iSearch 是一个高性能的文件搜索模块，提供强大的索引和搜索功能，支持多种文件格式的内容搜索。

```
+----------------------------------+
| 搜索框 | 过滤器▼ | 设置 | 索引    |
+----------------------------------+
| 目录树         | 搜索结果列表     |
| (可索引目录)   | 文件名           |
|                | 路径             |
|                | 匹配内容预览     |
|                | 最后修改时间     |
|                |                  |
+----------------+------------------+
| 状态: 已索引 10,245 文件 (2.3GB)  |
+----------------------------------+
```

#### 功能特性

- **高效索引**：增量更新指定目录的文件系统快照
- **混合搜索**：支持 `name:report AND content:Q4` 等复合查询语法
- **实时响应**：输入时即时显示结果（Debounce 300ms）
- **多格式支持**：索引和搜索 TXT、PDF、DOC、DOCX、XLS、XLSX、PPT、PPTX、MD 等多种文件格式
- **内容预览**：搜索结果中直接显示匹配内容的上下文
- **高级过滤**：按文件类型、大小、修改日期等条件过滤
- **安全控制**：显式文件访问授权机制

#### 实现方式

iSearch 模块使用 egui 的分割面板和自定义搜索结果渲染：

```rust
fn render_isearch(&mut self, ui: &mut egui::Ui) {
    // 顶部搜索栏和工具按钮
    ui.horizontal(|ui| {
        // 搜索图标和输入框
        ui.label("🔍");
        let response = ui.add(
            egui::TextEdit::singleline(&mut self.search_query)
                .hint_text("输入搜索关键词...")
                .desired_width(ui.available_width() - 250.0)
        );

        // 自动聚焦搜索框
        if self.should_focus_search {
            response.request_focus();
            self.should_focus_search = false;
        }

        // 过滤器下拉菜单
        ui.horizontal(|ui| {
            ui.label("过滤器");
            if ui.button("▼").clicked() {
                self.show_filter_dropdown = true;
            }
        });

        // 设置按钮
        if ui.button("⚙ 设置").clicked() {
            self.show_search_settings = true;
        }

        // 索引按钮
        if ui.button("📂 索引").clicked() {
            self.show_index_dialog = true;
        }
    });

    // 过滤器下拉菜单
    if self.show_filter_dropdown {
        egui::Window::new("搜索过滤器")
            .fixed_size([300.0, 400.0])
            .anchor(egui::Align2::RIGHT_TOP, [0.0, 30.0])
            .show(ui.ctx(), |ui| {
                ui.heading("文件类型");
                ui.checkbox(&mut self.filters.include_docs, "文档 (DOC, DOCX, PDF, TXT)");
                ui.checkbox(&mut self.filters.include_spreadsheets, "表格 (XLS, XLSX, CSV)");
                ui.checkbox(&mut self.filters.include_presentations, "演示文稿 (PPT, PPTX)");
                ui.checkbox(&mut self.filters.include_code, "代码 (RS, JS, PY, CPP, ...)");
                ui.checkbox(&mut self.filters.include_images, "图片 (JPG, PNG, GIF, ...)");

                ui.separator();
                ui.heading("时间范围");
                ui.horizontal(|ui| {
                    ui.label("从:");
                    ui.add(egui::widgets::DatePickerButton::new(&mut self.filters.date_from));
                });
                ui.horizontal(|ui| {
                    ui.label("至:");
                    ui.add(egui::widgets::DatePickerButton::new(&mut self.filters.date_to));
                });

                ui.separator();
                ui.heading("文件大小");
                ui.horizontal(|ui| {
                    ui.label("最小:");
                    ui.add(egui::DragValue::new(&mut self.filters.min_size_kb).suffix(" KB"));
                });
                ui.horizontal(|ui| {
                    ui.label("最大:");
                    ui.add(egui::DragValue::new(&mut self.filters.max_size_kb).suffix(" KB"));
                });

                ui.separator();
                ui.horizontal(|ui| {
                    if ui.button("应用").clicked() {
                        self.apply_filters();
                        self.show_filter_dropdown = false;
                    }
                    if ui.button("重置").clicked() {
                        self.reset_filters();
                    }
                    if ui.button("取消").clicked() {
                        self.show_filter_dropdown = false;
                    }
                });
            });
    }

    // 主体分割面板
    egui::containers::SplitLeft::new("search_split", &mut self.search_split_ratio)
        .show(ui, |left, right| {
            // 左侧索引目录树
            left.vertical(|ui| {
                ui.heading("索引目录");
                ui.separator();

                egui::ScrollArea::vertical().show(ui, |ui| {
                    self.render_directory_tree(ui);
                });

                ui.separator();
                ui.horizontal(|ui| {
                    if ui.button("添加目录").clicked() {
                        self.add_directory();
                    }
                    if ui.button("移除目录").clicked() && self.selected_directory.is_some() {
                        self.remove_directory();
                    }
                });
            });

            // 右侧搜索结果
            right.vertical(|ui| {
                ui.heading("搜索结果");
                ui.separator();

                if self.search_query.is_empty() {
                    ui.centered_and_justified(|ui| {
                        ui.label("请输入搜索关键词...");
                    });
                } else if self.search_results.is_empty() && !self.is_searching {
                    ui.centered_and_justified(|ui| {
                        ui.label("未找到匹配结果");
                    });
                } else if self.is_searching {
                    ui.centered_and_justified(|ui| {
                        ui.spinner();
                        ui.label("正在搜索...");
                    });
                } else {
                    self.render_search_results(ui);
                }
            });
        });

    // 底部状态栏
    ui.separator();
    ui.horizontal(|ui| {
        ui.label(format!(
            "状态: 已索引 {} 文件 ({:.1} GB)",
            self.index_stats.total_files,
            self.index_stats.total_size_bytes as f64 / 1_073_741_824.0
        ));

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if self.is_indexing {
                ui.spinner();
                ui.label("正在索引...");
            } else if self.index_stats.last_updated.is_some() {
                ui.label(format!(
                    "上次更新: {}",
                    self.index_stats.last_updated.unwrap().format("%Y-%m-%d %H:%M:%S")
                ));
            }
        });
    });

    // 索引对话框
    if self.show_index_dialog {
        self.render_index_dialog(ui.ctx());
    }

    // 设置对话框
    if self.show_search_settings {
        self.render_search_settings(ui.ctx());
    }
}

// 渲染搜索结果
fn render_search_results(&self, ui: &mut egui::Ui) {
    egui::ScrollArea::vertical().show(ui, |ui| {
        for result in &self.search_results {
            ui.push_id(result.id.clone(), |ui| {
                ui.add_space(4.0);

                // 文件名和图标
                ui.horizontal(|ui| {
                    // 文件类型图标
                    let icon = match result.file_type.as_str() {
                        "pdf" => "📄",
                        "doc" | "docx" => "📝",
                        "xls" | "xlsx" => "📊",
                        "ppt" | "pptx" => "📽",
                        "txt" | "md" => "📃",
                        "rs" | "js" | "py" | "cpp" => "💻",
                        "jpg" | "png" | "gif" => "🖼",
                        _ => "📁",
                    };
                    ui.label(icon);

                    // 文件名
                    ui.heading(&result.filename);

                    // 文件大小和日期
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(format!("{}", result.modified.format("%Y-%m-%d %H:%M")));
                        ui.label(format!("{:.1} KB", result.size_bytes as f64 / 1024.0));
                    });
                });

                // 文件路径
                ui.horizontal(|ui| {
                    ui.label("📂");
                    ui.label(&result.path);
                });

                // 匹配内容预览
                if !result.content_preview.is_empty() {
                    ui.add_space(4.0);
                    let mut text = egui::RichText::new(&result.content_preview);

                    // 高亮匹配的关键词
                    for keyword in self.search_query.split_whitespace() {
                        if result.content_preview.to_lowercase().contains(&keyword.to_lowercase()) {
                            text = text.highlight();
                            break;
                        }
                    }

                    ui.add(egui::Label::new(text).wrap(true));
                }

                // 点击打开文件
                if ui.button("打开文件").clicked() {
                    self.open_file(&result.path);
                }

                ui.add_space(4.0);
                ui.separator();
            });
        }
    });
}

// 索引对话框
fn render_index_dialog(&mut self, ctx: &egui::Context) {
    let mut open = true;
    egui::Window::new("索引设置")
        .open(&mut open)
        .resizable(true)
        .default_size([500.0, 400.0])
        .show(ctx, |ui| {
            ui.heading("索引配置");

            ui.collapsing("索引选项", |ui| {
                ui.checkbox(&mut self.index_settings.index_file_content, "索引文件内容");
                ui.checkbox(&mut self.index_settings.watch_for_changes, "监视文件变化");
                ui.checkbox(&mut self.index_settings.follow_symlinks, "跟随符号链接");

                ui.horizontal(|ui| {
                    ui.label("最大文件大小:");
                    ui.add(egui::DragValue::new(&mut self.index_settings.max_file_size_mb).suffix(" MB"));
                });

                ui.horizontal(|ui| {
                    ui.label("排除模式:");
                    ui.text_edit_singleline(&mut self.index_settings.exclude_pattern);
                });
            });

            ui.separator();
            ui.heading("索引目录");

            egui::ScrollArea::vertical().show(ui, |ui| {
                for (idx, dir) in self.indexed_directories.iter().enumerate() {
                    ui.horizontal(|ui| {
                        ui.label(format!("{}. {}", idx + 1, dir.path));
                        if ui.button("移除").clicked() {
                            self.directory_to_remove = Some(idx);
                        }
                    });
                }
            });

            if let Some(idx) = self.directory_to_remove.take() {
                self.indexed_directories.remove(idx);
            }

            ui.separator();
            ui.horizontal(|ui| {
                if ui.button("添加目录").clicked() {
                    if let Some(path) = rfd::FileDialog::new().pick_folder() {
                        self.indexed_directories.push(IndexedDirectory {
                            path: path.display().to_string(),
                            last_indexed: None,
                        });
                    }
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("开始索引").clicked() {
                        self.start_indexing();
                        self.show_index_dialog = false;
                    }

                    if ui.button("取消").clicked() {
                        self.show_index_dialog = false;
                    }
                });
            });
        });

    if !open {
        self.show_index_dialog = false;
    }
}
```


### AI 助手界面

AI 助手采用右侧边栏方式实现，提供历史会话选择、聊天记录显示和丰富的输入选项。

```
+----------------------------------+
| SeeU AI助手          | 设置       |
+----------------------------------+
|     历史消息▼       | + 新消息     |
+----------------------------------+
|                                  |
| AI: 你好！我是SeeU智能助手...    |
|                                  |
| 用户: 请帮我...                  |
|                                  |
| AI: 好的，我可以...              |
|                                  |
+----------------------------------+
|  输入消息...                    |
|                                |
+----------------------------------+
| 模型▼     | 📎     | @    | ✈发送  |
+----------------------------------+
```

#### 实现方式

```rust
fn render_ai_sidebar(&mut self, ui: &mut egui::Ui) {
    ui.vertical(|ui| {
        // 标题栏
        ui.horizontal(|ui| {
            ui.heading("SeeU AI助手");

            // 历史会话选择下拉框
            ui.horizontal(|ui| {
                ui.label("历史");
                if ui.button("▼").clicked() {
                    self.show_history_dropdown = true;
                }

                // 新建会话按钮
                if ui.button("+").clicked() {
                    self.create_new_chat_session();
                }
            });

            // 设置按钮
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("⚙").clicked() {
                    self.show_ai_settings = true;
                }
            });
        });

        // 历史会话下拉菜单
        if self.show_history_dropdown {
            egui::Window::new("历史会话")
                .fixed_size([250.0, 300.0])
                .anchor(egui::Align2::RIGHT_TOP, [0.0, 30.0])
                .show(ui.ctx(), |ui| {
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        for (idx, session) in self.chat_sessions.iter().enumerate() {
                            let selected = idx == self.active_session_idx;
                            if ui.selectable_label(selected, &session.name).clicked() {
                                self.active_session_idx = idx;
                                self.load_chat_session(idx);
                                self.show_history_dropdown = false;
                            }
                        }
                    });

                    ui.separator();
                    if ui.button("新建会话").clicked() {
                        self.create_new_chat_session();
                        self.show_history_dropdown = false;
                    }
                });
        }

        ui.separator();

        // 聊天历史区域
        egui::ScrollArea::vertical()
            .stick_to_bottom(true)
            .auto_shrink([false; 2])
            .show(ui, |ui| {
                for message in &self.chat_messages {
                    ui.horizontal_wrapped(|ui| {
                        match message.sender {
                            Sender::AI => {
                                ui.label(egui::RichText::new("AI: ")
                                    .color(egui::Color32::from_rgb(0, 128, 255))
                                    .strong());
                            },
                            Sender::User => {
                                ui.label(egui::RichText::new("用户: ")
                                    .color(egui::Color32::from_rgb(0, 180, 0))
                                    .strong());
                            }
                        }
                        ui.label(&message.content);
                    });
                    ui.add_space(8.0);
                }
            });

        // 输入区域
        ui.separator();

        // 底部工具栏
        ui.horizontal(|ui| {
            // 模型选择下拉框
            ui.horizontal(|ui| {
                egui::ComboBox::from_id_source("model_selector")
                    .selected_text(&self.ai_settings.model)
                    .width(100.0)
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.ai_settings.model, "qwen3:4b".to_string(), "Qwen 3 (4B)");
                        ui.selectable_value(&mut self.ai_settings.model, "llama3:8b".to_string(), "Llama 3 (8B)");
                        ui.selectable_value(&mut self.ai_settings.model, "gpt-3.5-turbo".to_string(), "GPT-3.5");
                    });
            });

            // 附件按钮
            if ui.button("📎").clicked() {
                self.show_attachment_dialog();
            }

            // @命令按钮
            if ui.button("@").clicked() {
                self.show_command_menu();
            }

            // 输入框
            let response = ui.add(
                egui::TextEdit::multiline(&mut self.chat_input)
                    .desired_width(ui.available_width() - 40.0)
                    .hint_text("输入消息...")
                    .desired_rows(1)
            );

            // 发送按钮 - 纸飞机图标，旋转15度
            if ui.add(egui::Button::new(egui::RichText::new("✈").size(24.0).rotation(0.26)) // 约15度
                .min_size(egui::vec2(32.0, 32.0)))
                .clicked() ||
               (ui.input(|i| i.key_pressed(egui::Key::Enter) && !i.modifiers.shift)) {
                self.send_chat_message();
            }

            // 自动聚焦输入框
            if self.should_focus_chat {
                response.request_focus();
                self.should_focus_chat = false;
            }
        });
    });

    // 附件对话框
    if self.show_attachment_dialog {
        self.render_attachment_dialog(ui.ctx());
    }

    // @命令菜单
    if self.show_command_menu {
        self.render_command_menu(ui.ctx());
    }

    // 设置弹窗
    if self.show_ai_settings {
        self.show_ai_settings_window(ui.ctx());
    }
}

// 附件对话框
fn render_attachment_dialog(&mut self, ctx: &egui::Context) {
    let mut open = true;
    egui::Window::new("添加附件")
        .open(&mut open)
        .resizable(false)
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.button("图片").clicked() {
                    if let Some(path) = rfd::FileDialog::new()
                        .add_filter("图片", &["png", "jpg", "jpeg", "gif"])
                        .pick_file() {
                        self.add_attachment(path, AttachmentType::Image);
                        self.show_attachment_dialog = false;
                    }
                }

                if ui.button("文件").clicked() {
                    if let Some(path) = rfd::FileDialog::new()
                        .pick_file() {
                        self.add_attachment(path, AttachmentType::File);
                        self.show_attachment_dialog = false;
                    }
                }

                if ui.button("代码片段").clicked() {
                    self.show_code_snippet_dialog = true;
                    self.show_attachment_dialog = false;
                }
            });
        });

    if !open {
        self.show_attachment_dialog = false;
    }
}

// @命令菜单
fn render_command_menu(&mut self, ctx: &egui::Context) {
    let commands = [
        "@文件", "@代码", "@终端", "@数据", "@图表", "@搜索"
    ];

    egui::Window::new("命令菜单")
        .fixed_size([200.0, 250.0])
        .anchor(egui::Align2::CENTER_BOTTOM, [0.0, -50.0])
        .show(ctx, |ui| {
            for cmd in &commands {
                if ui.selectable_label(false, *cmd).clicked() {
                    self.chat_input.push_str(&format!("{} ", cmd));
                    self.show_command_menu = false;
                }
            }
        });
}

// AI 设置窗口
fn show_ai_settings_window(&mut self, ctx: &egui::Context) {
    let mut open = true;
    egui::Window::new("AI 助手设置")
        .open(&mut open)
        .resizable(false)
        .show(ctx, |ui| {
            ui.heading("模型设置");

            ui.horizontal(|ui| {
                ui.label("API URL:");
                ui.text_edit_singleline(&mut self.ai_settings.api_url);
            });

            ui.horizontal(|ui| {
                ui.label("API Key:");
                ui.text_edit_singleline(&mut self.ai_settings.api_key);
            });

            ui.horizontal(|ui| {
                ui.label("模型:");
                egui::ComboBox::from_id_source("model_select")
                    .selected_text(&self.ai_settings.model)
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.ai_settings.model, "qwen3:4b".to_string(), "Qwen 3 (4B)");
                        ui.selectable_value(&mut self.ai_settings.model, "llama3:8b".to_string(), "Llama 3 (8B)");
                        ui.selectable_value(&mut self.ai_settings.model, "gpt-3.5-turbo".to_string(), "GPT-3.5 Turbo");
                    });
            });

            ui.separator();

            ui.horizontal(|ui| {
                if ui.button("保存").clicked() {
                    self.save_ai_settings();
                    self.show_ai_settings = false;
                }

                if ui.button("取消").clicked() {
                    self.show_ai_settings = false;
                }
            });
        });

    if !open {
        self.show_ai_settings = false;
    }
}
```

## 🔧 技术实现细节

### 状态管理

使用 Rust 的所有权系统和 egui 的状态管理机制：

```rust
pub struct SeeUApp {
    // 全局状态
    active_module: Module,
    show_right_sidebar: bool,

    // 模块状态
    inote: inote::INoteState,
    isearch: isearch::ISearchState,
    terminal: terminal::TerminalState,
    file_manager: file_manager::FileManagerState,
    data_analysis: data_analysis::DataAnalysisState,

    // AI 助手状态
    chat_messages: Vec<ChatMessage>,
    chat_input: String,
    should_focus_chat: bool,
    show_ai_settings: bool,
    show_history_dropdown: bool,
    show_attachment_dialog: bool,
    show_code_snippet_dialog: bool,
    show_command_menu: bool,
    chat_sessions: Vec<ChatSession>,
    active_session_idx: usize,
    ai_settings: AISettings,

    // 资源
    icons: IconSet,
    fonts: FontSet,

    // 服务
    ai_service: AiService,
    system_service: SystemService,
}

// 聊天会话结构体
pub struct ChatSession {
    pub id: String,
    pub name: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub messages: Vec<ChatMessage>,
}

impl ChatSession {
    pub fn new(name: String) -> Self {
        let now = chrono::Utc::now();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            created_at: now,
            updated_at: now,
            messages: vec![ChatMessage {
                id: uuid::Uuid::new_v4().to_string(),
                sender: Sender::AI,
                content: "你好！我是SeeU智能助手，有什么我可以帮助你的？".to_string(),
                timestamp: now,
            }],
        }
    }
}

// 聊天消息结构体
pub struct ChatMessage {
    pub id: String,
    pub sender: Sender,
    pub content: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

// 发送者枚举
pub enum Sender {
    User,
    AI,
}

// 附件类型枚举
pub enum AttachmentType {
    Image,
    File,
    CodeSnippet,
}

// AI 设置结构体
pub struct AISettings {
    pub api_url: String,
    pub api_key: String,
    pub model: String,
    pub temperature: f32,
    pub max_tokens: u32,
    pub streaming: bool,
}

impl Default for AISettings {
    fn default() -> Self {
        Self {
            api_url: "http://localhost:11434/v1".to_string(),
            api_key: "EMPTY".to_string(),
            model: "qwen3:4b".to_string(),
            temperature: 0.7,
            max_tokens: 2000,
            streaming: true,
        }
    }
}
```

### 主题支持

使用 egui 的主题系统实现亮色/暗色主题：

```rust
fn configure_visuals(ctx: &egui::Context, theme: Theme) {
    match theme {
        Theme::Dark => {
            ctx.set_visuals(egui::Visuals {
                dark_mode: true,
                panel_fill: egui::Color32::from_rgb(30, 30, 30),
                window_fill: egui::Color32::from_rgb(40, 40, 40),
                // 其他暗色主题设置...
                ..Default::default()
            });
        },
        Theme::Light => {
            ctx.set_visuals(egui::Visuals {
                dark_mode: false,
                panel_fill: egui::Color32::from_rgb(240, 240, 240),
                window_fill: egui::Color32::from_rgb(250, 250, 250),
                // 其他亮色主题设置...
                ..Default::default()
            });
        }
    }
}
```

### 性能优化

利用 egui 的即时模式 GUI 特性进行性能优化：

```rust
// 使用 egui 的 Id 系统缓存计算密集型操作
fn render_data_visualization(&mut self, ui: &mut egui::Ui) {
    let data_id = ui.id().with("data_visualization");
    let plot_data = ui.memory_mut(|mem| {
        mem.data.get_temp::<PlotData>(data_id)
            .unwrap_or_else(|| {
                // 只在需要时计算
                let data = self.compute_expensive_plot_data();
                mem.data.insert_temp(data_id, data.clone());
                data
            })
    });

    // 使用缓存的数据绘制图表
    self.draw_plot(ui, &plot_data);
}
```


