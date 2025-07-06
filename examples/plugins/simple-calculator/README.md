# 简单计算器插件

一个为iTools设计的简单计算器插件，提供基本的数学运算和单位转换功能。

## 功能特性

### 数学计算
- 基本四则运算（+、-、*、/）
- 括号支持
- 小数运算
- 安全的表达式求值

### 单位转换
- **长度单位**: 米(m)、千米(km)、厘米(cm)、毫米(mm)、英尺(ft)、英寸(in)、英里(mile)、码(yard)
- **重量单位**: 克(g)、千克(kg)、磅(lb)、盎司(oz)、吨(ton)
- **温度单位**: 摄氏度(celsius)、华氏度(fahrenheit)、开尔文(kelvin)

## 使用方法

### 数学计算
```javascript
// 基本运算
calculate("2 + 3")          // 结果: 5
calculate("10 * (5 - 2)")   // 结果: 30
calculate("15 / 3")         // 结果: 5
calculate("2.5 * 4")        // 结果: 10

// 复杂表达式
calculate("(10 + 5) * 2 - 8 / 4")  // 结果: 28
```

### 单位转换
```javascript
// 长度转换
convert_units(1000, "m", "km")      // 1000米 = 1千米
convert_units(5, "ft", "m")         // 5英尺 = 1.524米

// 重量转换
convert_units(1, "kg", "lb")        // 1千克 = 2.20462磅
convert_units(16, "oz", "g")        // 16盎司 = 453.592克

// 温度转换
convert_units(0, "celsius", "fahrenheit")    // 0°C = 32°F
convert_units(100, "celsius", "kelvin")      // 100°C = 373.15K
```

## 配置选项

插件支持以下配置选项：

- `precision`: 计算结果的小数位数（默认：6）
- `angle_unit`: 角度单位（默认：degrees）
- `enable_advanced_functions`: 启用高级数学函数（默认：false）
- `max_expression_length`: 表达式最大长度（默认：100）
- `output_format`: 输出格式（默认：formatted）
- `enable_history`: 启用历史记录（默认：true）

## 安装方法

1. 下载插件包 `simple-calculator.itpkg`
2. 在iTools插件市场中点击"从本地安装"
3. 选择下载的插件包文件
4. 等待安装完成
5. 启用插件

## 安全性

- 表达式求值使用安全的方法，只允许数字、运算符和括号
- 输入验证确保不会执行恶意代码
- 结果验证确保返回有效的数值

## 许可证

MIT License

## 作者

示例开发者 (dev@example.com)

## 版本历史

- v1.0.0: 初始版本，支持基本计算和单位转换
