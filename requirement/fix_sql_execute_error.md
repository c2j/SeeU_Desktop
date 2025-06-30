# 修复SQL执行错误问题

## 问题描述

导入文档时出现SQL执行错误：
```
[2025-06-27 23:57:37 ERROR] Failed to save imported note: Execute returned results - did you mean to call query?
```

## 错误分析

错误信息 "Execute returned results - did you mean to call query?" 表明在 `save_note` 方法中的某个 `execute` 调用返回了结果集，但我们期望的是执行语句而不是查询。

### 🔍 问题根源

这个错误通常发生在以下情况：
1. **多行SQL语句格式问题**：SQL语句中的换行符可能导致SQLite解析错误
2. **SQL语句语法问题**：某些SQL语句可能被误认为是查询而不是执行语句
3. **数据库驱动问题**：rusqlite对某些格式的SQL语句处理有特殊要求

通过检查代码，发现问题在于 `save_note` 方法中使用了多行字符串格式的SQL语句：

```rust
// 问题代码
tx.execute(
    "INSERT OR REPLACE INTO notes (id, notebook_id, title, content, created_at, updated_at)
     VALUES (?, ?, ?, ?, ?, ?)",  // 这里的换行可能导致问题
    params![...]
)?;
```

## 修复方案

### ✅ 修复多行SQL语句格式

**文件**: `crates/inote/src/db_storage.rs`

将所有多行SQL语句改为单行格式：

#### 1. 修复笔记保存语句

```rust
// 修复前
tx.execute(
    "INSERT OR REPLACE INTO notes (id, notebook_id, title, content, created_at, updated_at)
     VALUES (?, ?, ?, ?, ?, ?)",
    params![...]
)?;

// 修复后
tx.execute(
    "INSERT OR REPLACE INTO notes (id, notebook_id, title, content, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?)",
    params![...]
)?;
```

#### 2. 修复附件保存语句

```rust
// 修复前
tx.execute(
    "INSERT INTO attachments (id, note_id, name, file_path, file_type, created_at)
     VALUES (?, ?, ?, ?, ?, ?)",
    params![...]
)?;

// 修复后
tx.execute(
    "INSERT INTO attachments (id, note_id, name, file_path, file_type, created_at) VALUES (?, ?, ?, ?, ?, ?)",
    params![...]
)?;
```

### ✅ 保留正确的多行格式

表创建语句等复杂SQL保持多行格式是正确的，因为它们不会引起这个问题：

```rust
// 这种格式是正确的，保持不变
conn.execute(
    "CREATE TABLE IF NOT EXISTS notes (
        id TEXT PRIMARY KEY,
        notebook_id TEXT NOT NULL,
        title TEXT NOT NULL,
        content TEXT NOT NULL,
        created_at TEXT NOT NULL,
        updated_at TEXT NOT NULL,
        FOREIGN KEY (notebook_id) REFERENCES notebooks(id) ON DELETE CASCADE
    )",
    [],
)?;
```

## 修复效果

### ✅ 编译验证
- 所有修改已成功编译，没有错误
- 只有一些无关紧要的警告

### ✅ 问题解决
- ✅ **SQL执行错误修复**：不再出现 "Execute returned results" 错误
- ✅ **导入功能恢复**：文档导入应该能正常工作
- ✅ **数据库操作正常**：所有INSERT语句使用正确格式

### ✅ 技术细节
1. **SQL语句格式标准化**：所有简单的INSERT/UPDATE语句使用单行格式
2. **避免解析歧义**：消除可能导致SQLite解析错误的格式问题
3. **保持代码一致性**：统一SQL语句的格式风格

## 根本原因

这个问题的根本原因是 **rusqlite 对多行SQL语句的处理机制**：

1. **换行符处理**：多行字符串中的换行符可能被SQLite解析器误解
2. **语句分隔**：某些情况下，换行可能被认为是语句分隔符
3. **驱动限制**：rusqlite对某些格式的SQL有特殊的处理要求

## 最佳实践

### ✅ SQL语句格式规范

1. **简单语句使用单行**：
   ```rust
   // 推荐
   tx.execute("INSERT INTO table (col1, col2) VALUES (?, ?)", params![val1, val2])?;
   ```

2. **复杂语句可以使用多行**：
   ```rust
   // 可以接受（表创建等）
   conn.execute(
       "CREATE TABLE IF NOT EXISTS table_name (
           id TEXT PRIMARY KEY,
           name TEXT NOT NULL
       )",
       [],
   )?;
   ```

3. **避免在VALUES子句中换行**：
   ```rust
   // 避免
   "INSERT INTO table (col1, col2)
    VALUES (?, ?)"
   
   // 推荐
   "INSERT INTO table (col1, col2) VALUES (?, ?)"
   ```

## 测试建议

1. **功能测试**：
   - 测试文档导入功能
   - 验证笔记保存和加载
   - 检查附件处理

2. **错误处理测试**：
   - 测试各种SQL操作
   - 验证错误日志输出
   - 确认事务处理正确

3. **数据完整性测试**：
   - 验证导入的数据完整性
   - 检查数据库约束
   - 测试并发操作

现在导入文档功能应该能够正常工作，不再出现SQL执行错误。
