//! 核心参数解析器 - 负责解析和填充模板中的参数

use super::parameter_types::{ParameterType, SystemParameter};
use super::providers::{ClipboardProvider, ProviderError};
use super::template_parser::TemplateParser;
use std::collections::HashMap;
use thiserror::Error;
use tracing::debug;

/// 参数解析器错误
#[derive(Error, Debug)]
pub enum ResolverError {
    #[error("参数数量不匹配: 期望 {expected} 个参数, 但提供了 {provided} 个")]
    ArgumentCountMismatch { expected: usize, provided: usize },

    #[error("参数提供者错误: {0}")]
    ProviderError(#[from] ProviderError),

    #[error("模板解析错误: {0}")]
    ParseError(String),
}

/// 系统参数的快照 - 在特定时机捕获系统状态
///
/// 捕获时机:
/// - 窗口句柄: 从 AppState 读取(已在唤醒时保存)
/// - 选中文本: 从 AppState 读取(已在唤醒时保存)
/// - 剪贴板: 在调用 capture() 时实时获取
#[derive(Debug, Clone)]
pub struct SystemParameterSnapshot {
    /// 剪贴板内容
    pub clipboard: Option<String>,
    /// 窗口句柄(唤醒前的前台窗口)
    pub window_handle: Option<String>,
    /// 选中文本(唤醒前活动窗口的选中内容)
    pub selection: Option<String>,
}

impl SystemParameterSnapshot {
    /// 创建新的快照
    ///
    /// - 剪贴板: 获取当前剪贴板内容
    /// - 窗口句柄: 从 AppState 读取已保存的值(在 ui_controller 唤醒时保存)
    /// - 选中文本: 从 AppState 读取已保存的值(在 ui_controller 唤醒时保存)
    pub fn capture() -> Self {
        use crate::utils::service_locator::ServiceLocator;

        // 获取剪贴板内容(当前时刻)
        let clipboard = ClipboardProvider::get_value().ok();

        // 从 AppState 读取已保存的窗口句柄(唤醒时保存的)
        let state = ServiceLocator::get_state();
        let window_handle = state
            .get_previous_foreground_window()
            .map(|hwnd| hwnd.to_string());

        // 从 AppState 读取已保存的选中文本(唤醒时保存的)
        let selection = state.get_previous_selection();

        debug!(
            "📸 捕获系统参数快照: clipboard={}, hwnd={}, selection={}",
            clipboard.as_deref().unwrap_or("<empty>"),
            window_handle.as_deref().unwrap_or("<null>"),
            selection
                .as_ref()
                .map(|s| format!("{}字符", s.len()))
                .unwrap_or_else(|| "<empty>".to_string())
        );

        Self {
            clipboard,
            window_handle,
            selection,
        }
    }

    /// 获取指定系统参数的值
    pub fn get(&self, param: SystemParameter) -> String {
        match param {
            SystemParameter::Clipboard => self.clipboard.clone().unwrap_or_default(),
            SystemParameter::WindowHandle => self
                .window_handle
                .clone()
                .unwrap_or_else(|| "0".to_string()),
            SystemParameter::Selection => self.selection.clone().unwrap_or_default(),
        }
    }
}

/// 核心参数解析器
#[derive(Debug)]
pub struct ParameterResolver;

impl ParameterResolver {
    /// 创建新的参数解析器
    pub fn new() -> Self {
        Self
    }

    /// 解析并填充模板
    ///
    /// # 参数
    /// - `template`: 模板字符串
    /// - `user_args`: 用户提供的参数
    /// - `snapshot`: 系统参数快照(在用户按回车时捕获)
    pub fn resolve_template(
        &self,
        template: &str,
        user_args: &[String],
        snapshot: &SystemParameterSnapshot,
    ) -> Result<String, ResolverError> {
        // 1. 解析模板获取所有参数
        let parameters = TemplateParser::parse(template);

        // 2. 验证用户参数数量
        let user_param_count = parameters
            .iter()
            .filter(|p| matches!(p.param_type, ParameterType::Positional(_)))
            .count();

        if user_args.len() != user_param_count {
            return Err(ResolverError::ArgumentCountMismatch {
                expected: user_param_count,
                provided: user_args.len(),
            });
        }

        // 3. 收集所有参数值
        let mut param_values: HashMap<usize, String> = HashMap::new();

        for param in &parameters {
            let value = match &param.param_type {
                ParameterType::Positional(index) => {
                    user_args.get(*index).cloned().unwrap_or_default()
                }
                ParameterType::System(sys_param) => {
                    // 从快照中获取系统参数值
                    snapshot.get(*sys_param)
                }
            };

            // 使用参数在模板中的起始位置作为key
            param_values.insert(param.start_pos, value);
        }

        // 4. 按照参数在模板中的位置，从后往前替换(避免位置偏移)
        let mut sorted_params = parameters.clone();
        sorted_params.sort_by_key(|b| std::cmp::Reverse(b.start_pos));

        let mut result = template.to_string();
        for param in sorted_params {
            if let Some(value) = param_values.get(&param.start_pos) {
                result.replace_range(param.start_pos..param.end_pos, value);
            }
        }

        debug!("模板填充完成: {} -> {}", template, result);
        Ok(result)
    }

    /// 统计模板中需要用户输入的参数数量
    pub fn count_user_parameters(&self, template: &str) -> usize {
        TemplateParser::count_user_parameters(template)
    }

    /// 检查模板是否包含系统参数
    pub fn has_system_parameters(&self, template: &str) -> bool {
        TemplateParser::has_system_parameters(template)
    }
}

impl Default for ParameterResolver {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_no_parameters() {
        let resolver = ParameterResolver::new();
        let snapshot = SystemParameterSnapshot {
            clipboard: Some("test".to_string()),
            window_handle: Some("12345".to_string()),
            selection: None,
        };

        let result = resolver
            .resolve_template("notepad.exe", &[], &snapshot)
            .unwrap();

        assert_eq!(result, "notepad.exe");
    }

    #[test]
    fn test_resolve_positional_parameters() {
        let resolver = ParameterResolver::new();
        let snapshot = SystemParameterSnapshot {
            clipboard: None,
            window_handle: None,
            selection: None,
        };

        let result = resolver
            .resolve_template("cmd /c echo {}", &["hello".to_string()], &snapshot)
            .unwrap();

        assert_eq!(result, "cmd /c echo hello");
    }

    #[test]
    fn test_resolve_clipboard_parameter() {
        let resolver = ParameterResolver::new();
        let snapshot = SystemParameterSnapshot {
            clipboard: Some("clipboard_content".to_string()),
            window_handle: None,
            selection: None,
        };

        let result = resolver
            .resolve_template("notepad {clip}", &[], &snapshot)
            .unwrap();

        assert_eq!(result, "notepad clipboard_content");
    }

    #[test]
    fn test_resolve_hwnd_parameter() {
        let resolver = ParameterResolver::new();
        let snapshot = SystemParameterSnapshot {
            clipboard: None,
            window_handle: Some("98765".to_string()),
            selection: None,
        };

        let result = resolver
            .resolve_template("tool --hwnd {hwnd}", &[], &snapshot)
            .unwrap();

        assert_eq!(result, "tool --hwnd 98765");
    }

    #[test]
    fn test_resolve_selection_parameter() {
        let resolver = ParameterResolver::new();
        let snapshot = SystemParameterSnapshot {
            clipboard: None,
            window_handle: None,
            selection: Some("selected text".to_string()),
        };

        let result = resolver
            .resolve_template("translate {selection}", &[], &snapshot)
            .unwrap();

        assert_eq!(result, "translate selected text");
    }

    #[test]
    fn test_resolve_mixed_parameters() {
        let resolver = ParameterResolver::new();
        let snapshot = SystemParameterSnapshot {
            clipboard: Some("clip_data".to_string()),
            window_handle: Some("54321".to_string()),
            selection: Some("selected".to_string()),
        };

        let result = resolver
            .resolve_template(
                "program {} {clip} {} {hwnd} {selection}",
                &["arg1".to_string(), "arg2".to_string()],
                &snapshot,
            )
            .unwrap();

        assert_eq!(result, "program arg1 clip_data arg2 54321 selected");
    }

    #[test]
    fn test_argument_count_mismatch() {
        let resolver = ParameterResolver::new();
        let snapshot = SystemParameterSnapshot {
            clipboard: None,
            window_handle: None,
            selection: None,
        };

        let result = resolver.resolve_template("program {} {}", &["arg1".to_string()], &snapshot);

        assert!(result.is_err());
        match result {
            Err(ResolverError::ArgumentCountMismatch { expected, provided }) => {
                assert_eq!(expected, 2);
                assert_eq!(provided, 1);
            }
            _ => panic!("Expected ArgumentCountMismatch error"),
        }
    }

    #[test]
    fn test_count_user_parameters() {
        let resolver = ParameterResolver::new();

        assert_eq!(resolver.count_user_parameters("program {}"), 1);
        assert_eq!(resolver.count_user_parameters("program {} {}"), 2);
        assert_eq!(resolver.count_user_parameters("program {clip} {hwnd}"), 0);
        assert_eq!(resolver.count_user_parameters("program {} {clip} {}"), 2);
        assert_eq!(resolver.count_user_parameters("program {selection}"), 0);
        assert_eq!(
            resolver.count_user_parameters("program {} {selection} {}"),
            2
        );
    }
}
