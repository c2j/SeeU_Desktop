use std::collections::HashMap;
use chrono::{DateTime, Utc, NaiveDate};
use regex::Regex;

/// Advanced search query structure
#[derive(Debug, Clone)]
pub struct AdvancedQuery {
    pub terms: Vec<SearchTerm>,
    pub operators: Vec<LogicalOperator>,
    pub filters: HashMap<String, FilterValue>,
}

/// Search term with modifiers
#[derive(Debug, Clone)]
pub struct SearchTerm {
    pub text: String,
    pub modifiers: Vec<SearchModifier>,
    pub is_negated: bool,
    pub is_required: bool,
    pub is_exact: bool,
    pub is_regex: bool,
}

/// Logical operators between terms
#[derive(Debug, Clone, PartialEq)]
pub enum LogicalOperator {
    And,
    Or,
    Not,
}

/// Search modifiers
#[derive(Debug, Clone, PartialEq)]
pub enum SearchModifier {
    CaseSensitive,
    CaseInsensitive,
    ExactMatch,
    FileOnly,
    FolderOnly,
    PathMatch,
    NoPathMatch,
    RegexEnabled,
    RegexDisabled,
}

/// Filter values for advanced search
#[derive(Debug, Clone)]
pub enum FilterValue {
    Text(String),
    Number(NumberFilter),
    Date(DateFilter),
    Size(SizeFilter),
    List(Vec<String>),
}

/// Number filter with comparison operators
#[derive(Debug, Clone)]
pub struct NumberFilter {
    pub operator: ComparisonOperator,
    pub value: u64,
}

/// Date filter with comparison operators
#[derive(Debug, Clone)]
pub struct DateFilter {
    pub operator: ComparisonOperator,
    pub value: DateTime<Utc>,
}

/// Size filter with comparison operators and units
#[derive(Debug, Clone)]
pub struct SizeFilter {
    pub operator: ComparisonOperator,
    pub value: u64, // Size in bytes
}

/// Comparison operators for filters
#[derive(Debug, Clone, PartialEq)]
pub enum ComparisonOperator {
    Equal,
    NotEqual,
    GreaterThan,
    GreaterThanOrEqual,
    LessThan,
    LessThanOrEqual,
    Between(u64, u64), // For ranges
}

/// Advanced search parser
pub struct AdvancedSearchParser {
    regex_cache: HashMap<String, Regex>,
}

impl AdvancedSearchParser {
    pub fn new() -> Self {
        Self {
            regex_cache: HashMap::new(),
        }
    }

    /// Parse an advanced search query string
    pub fn parse(&mut self, query: &str) -> Result<AdvancedQuery, String> {
        let mut terms = Vec::new();
        let mut operators = Vec::new();
        let mut filters = HashMap::new();

        // Tokenize the query
        let tokens = self.tokenize(query)?;
        
        let mut i = 0;
        while i < tokens.len() {
            let token = &tokens[i];

            // Check for filters (function-like syntax)
            if let Some(filter_result) = self.parse_filter(token)? {
                filters.insert(filter_result.0, filter_result.1);
                i += 1;
                continue;
            }

            // Check for logical operators
            if let Some(op) = self.parse_operator(token) {
                operators.push(op);
                i += 1;
                continue;
            }

            // Parse search term with modifiers
            let (term, consumed) = self.parse_term(&tokens[i..])?;
            terms.push(term);
            i += consumed;
        }

        Ok(AdvancedQuery {
            terms,
            operators,
            filters,
        })
    }

    /// Tokenize the query string, respecting quotes and escapes
    fn tokenize(&self, query: &str) -> Result<Vec<String>, String> {
        let mut tokens = Vec::new();
        let mut current_token = String::new();
        let mut in_quotes = false;
        let mut quote_char = '"';
        let mut escaped = false;
        let mut chars = query.chars().peekable();

        while let Some(ch) = chars.next() {
            if escaped {
                current_token.push(ch);
                escaped = false;
                continue;
            }

            match ch {
                '\\' => {
                    escaped = true;
                }
                '"' | '\'' => {
                    if !in_quotes {
                        in_quotes = true;
                        quote_char = ch;
                    } else if ch == quote_char {
                        in_quotes = false;
                        // Add the quoted content as a token
                        if !current_token.is_empty() {
                            tokens.push(format!("\"{}\"", current_token));
                            current_token.clear();
                        }
                    } else {
                        current_token.push(ch);
                    }
                }
                ' ' | '\t' | '\n' => {
                    if in_quotes {
                        current_token.push(ch);
                    } else if !current_token.is_empty() {
                        tokens.push(current_token.clone());
                        current_token.clear();
                    }
                }
                '(' | ')' => {
                    if in_quotes {
                        current_token.push(ch);
                    } else {
                        if !current_token.is_empty() {
                            tokens.push(current_token.clone());
                            current_token.clear();
                        }
                        tokens.push(ch.to_string());
                    }
                }
                _ => {
                    current_token.push(ch);
                }
            }
        }

        if in_quotes {
            return Err("Unclosed quote in search query".to_string());
        }

        if !current_token.is_empty() {
            tokens.push(current_token);
        }

        Ok(tokens)
    }

    /// Parse a filter (e.g., "size:>1MB", "date:today", "ext:pdf;doc")
    fn parse_filter(&self, token: &str) -> Result<Option<(String, FilterValue)>, String> {
        if !token.contains(':') {
            return Ok(None);
        }

        let parts: Vec<&str> = token.splitn(2, ':').collect();
        if parts.len() != 2 {
            return Ok(None);
        }

        let filter_name = parts[0].to_lowercase();
        let filter_value = parts[1];

        match filter_name.as_str() {
            "size" => {
                let size_filter = self.parse_size_filter(filter_value)?;
                Ok(Some((filter_name, FilterValue::Size(size_filter))))
            }
            "date" | "datemodified" | "dm" => {
                let date_filter = self.parse_date_filter(filter_value)?;
                Ok(Some((filter_name, FilterValue::Date(date_filter))))
            }
            "ext" | "extension" => {
                let extensions: Vec<String> = filter_value
                    .split(';')
                    .map(|s| s.trim().to_lowercase())
                    .collect();
                Ok(Some((filter_name, FilterValue::List(extensions))))
            }
            "filetype" | "type" => {
                Ok(Some((filter_name, FilterValue::Text(filter_value.to_lowercase()))))
            }
            "filename" | "name" => {
                Ok(Some((filter_name, FilterValue::Text(filter_value.to_string()))))
            }
            "path" => {
                Ok(Some((filter_name, FilterValue::Text(filter_value.to_string()))))
            }
            "parent" => {
                Ok(Some((filter_name, FilterValue::Text(filter_value.to_string()))))
            }
            _ => Ok(None), // Unknown filter, treat as regular term
        }
    }

    /// Parse size filter (e.g., ">1MB", "<=500KB", "1GB..2GB")
    fn parse_size_filter(&self, value: &str) -> Result<SizeFilter, String> {
        // Handle range syntax (e.g., "1MB..2GB")
        if value.contains("..") {
            let parts: Vec<&str> = value.split("..").collect();
            if parts.len() == 2 {
                let start_size = self.parse_size_value(parts[0])?;
                let end_size = self.parse_size_value(parts[1])?;
                return Ok(SizeFilter {
                    operator: ComparisonOperator::Between(start_size, end_size),
                    value: start_size,
                });
            }
        }

        // Handle comparison operators
        let (operator, size_str) = if value.starts_with(">=") {
            (ComparisonOperator::GreaterThanOrEqual, &value[2..])
        } else if value.starts_with("<=") {
            (ComparisonOperator::LessThanOrEqual, &value[2..])
        } else if value.starts_with('>') {
            (ComparisonOperator::GreaterThan, &value[1..])
        } else if value.starts_with('<') {
            (ComparisonOperator::LessThan, &value[1..])
        } else if value.starts_with('=') {
            (ComparisonOperator::Equal, &value[1..])
        } else {
            (ComparisonOperator::Equal, value)
        };

        let size_bytes = self.parse_size_value(size_str)?;
        Ok(SizeFilter {
            operator,
            value: size_bytes,
        })
    }

    /// Parse size value with units (e.g., "1MB", "500KB", "2.5GB")
    fn parse_size_value(&self, value: &str) -> Result<u64, String> {
        let value = value.trim();
        
        // Extract number and unit
        let (number_str, unit) = if value.ends_with("TB") || value.ends_with("tb") {
            (&value[..value.len()-2], "TB")
        } else if value.ends_with("GB") || value.ends_with("gb") {
            (&value[..value.len()-2], "GB")
        } else if value.ends_with("MB") || value.ends_with("mb") {
            (&value[..value.len()-2], "MB")
        } else if value.ends_with("KB") || value.ends_with("kb") {
            (&value[..value.len()-2], "KB")
        } else if value.ends_with('B') || value.ends_with('b') {
            (&value[..value.len()-1], "B")
        } else {
            // No unit, assume bytes
            (value, "B")
        };

        let number: f64 = number_str.parse()
            .map_err(|_| format!("Invalid size number: {}", number_str))?;

        let multiplier = match unit.to_uppercase().as_str() {
            "B" => 1,
            "KB" => 1024,
            "MB" => 1024 * 1024,
            "GB" => 1024 * 1024 * 1024,
            "TB" => 1024_u64.pow(4),
            _ => return Err(format!("Unknown size unit: {}", unit)),
        };

        Ok((number * multiplier as f64) as u64)
    }

    /// Parse date filter (e.g., "today", "2023-01-01", ">yesterday")
    fn parse_date_filter(&self, value: &str) -> Result<DateFilter, String> {
        // Handle comparison operators
        let (operator, date_str) = if value.starts_with(">=") {
            (ComparisonOperator::GreaterThanOrEqual, &value[2..])
        } else if value.starts_with("<=") {
            (ComparisonOperator::LessThanOrEqual, &value[2..])
        } else if value.starts_with('>') {
            (ComparisonOperator::GreaterThan, &value[1..])
        } else if value.starts_with('<') {
            (ComparisonOperator::LessThan, &value[1..])
        } else if value.starts_with('=') {
            (ComparisonOperator::Equal, &value[1..])
        } else {
            (ComparisonOperator::Equal, value)
        };

        let date = self.parse_date_value(date_str)?;
        Ok(DateFilter {
            operator,
            value: date,
        })
    }

    /// Parse date value (e.g., "today", "yesterday", "2023-01-01")
    fn parse_date_value(&self, value: &str) -> Result<DateTime<Utc>, String> {
        let value = value.trim().to_lowercase();
        let now = Utc::now();

        match value.as_str() {
            "today" => Ok(now.date_naive().and_hms_opt(0, 0, 0).unwrap().and_utc()),
            "yesterday" => {
                let yesterday = now.date_naive().pred_opt()
                    .ok_or("Invalid yesterday date")?;
                Ok(yesterday.and_hms_opt(0, 0, 0).unwrap().and_utc())
            }
            _ => {
                // Try to parse as ISO date (YYYY-MM-DD)
                if let Ok(date) = NaiveDate::parse_from_str(&value, "%Y-%m-%d") {
                    return Ok(date.and_hms_opt(0, 0, 0).unwrap().and_utc());
                }

                // Try to parse as year only
                if let Ok(year) = value.parse::<i32>() {
                    if let Some(date) = NaiveDate::from_ymd_opt(year, 1, 1) {
                        return Ok(date.and_hms_opt(0, 0, 0).unwrap().and_utc());
                    }
                }

                Err(format!("Invalid date format: {}", value))
            }
        }
    }

    /// Parse logical operator
    fn parse_operator(&self, token: &str) -> Option<LogicalOperator> {
        match token.to_uppercase().as_str() {
            "AND" | "&&" => Some(LogicalOperator::And),
            "OR" | "||" => Some(LogicalOperator::Or),
            "NOT" | "!" => Some(LogicalOperator::Not),
            _ => None,
        }
    }

    /// Parse a search term with modifiers
    fn parse_term(&self, tokens: &[String]) -> Result<(SearchTerm, usize), String> {
        if tokens.is_empty() {
            return Err("No tokens to parse".to_string());
        }

        let mut modifiers = Vec::new();
        let mut is_negated = false;
        let mut is_required = false;
        let mut is_exact = false;
        let mut is_regex = false;
        let mut text = String::new();
        let mut consumed = 0;

        for (i, token) in tokens.iter().enumerate() {
            consumed = i + 1;

            // Check for modifiers
            if token.ends_with(':') && i + 1 < tokens.len() {
                let modifier = token[..token.len()-1].to_lowercase();
                match modifier.as_str() {
                    "case" => modifiers.push(SearchModifier::CaseSensitive),
                    "nocase" => modifiers.push(SearchModifier::CaseInsensitive),
                    "exact" => modifiers.push(SearchModifier::ExactMatch),
                    "file" | "files" => modifiers.push(SearchModifier::FileOnly),
                    "folder" | "folders" => modifiers.push(SearchModifier::FolderOnly),
                    "path" => modifiers.push(SearchModifier::PathMatch),
                    "nopath" => modifiers.push(SearchModifier::NoPathMatch),
                    "regex" => modifiers.push(SearchModifier::RegexEnabled),
                    "noregex" => modifiers.push(SearchModifier::RegexDisabled),
                    _ => {
                        // Not a modifier, treat as regular text
                        text = token.clone();
                        break;
                    }
                }
                continue;
            }

            // Check for special prefixes
            if token.starts_with('+') {
                is_required = true;
                text = token[1..].to_string();
                break;
            } else if token.starts_with('-') || token.starts_with('!') {
                is_negated = true;
                text = token[1..].to_string();
                break;
            } else if (token.starts_with('"') && token.ends_with('"')) ||
                      (token.starts_with('\'') && token.ends_with('\'')) {
                is_exact = true;
                text = token[1..token.len()-1].to_string();
                break;
            } else {
                text = token.clone();
                break;
            }
        }

        // Apply modifiers to flags
        for modifier in &modifiers {
            match modifier {
                SearchModifier::ExactMatch => is_exact = true,
                SearchModifier::RegexEnabled => is_regex = true,
                SearchModifier::RegexDisabled => is_regex = false,
                _ => {}
            }
        }

        Ok((SearchTerm {
            text,
            modifiers,
            is_negated,
            is_required,
            is_exact,
            is_regex,
        }, consumed))
    }
}

impl Default for AdvancedSearchParser {
    fn default() -> Self {
        Self::new()
    }
}
