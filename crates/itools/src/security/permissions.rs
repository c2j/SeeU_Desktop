use std::collections::HashMap;
use uuid::Uuid;
use anyhow::Result;
use chrono::{DateTime, Utc};

use crate::roles::{UserRole, Permission};
use crate::state::PermissionLevel;

/// Permission manager for handling access control
#[derive(Debug)]
pub struct PermissionManager {
    /// Cached permission decisions
    permission_cache: HashMap<String, CachedPermission>,
    
    /// Active permission grants
    active_grants: HashMap<Uuid, PermissionGrant>,
    
    /// Permission policies
    policies: Vec<PermissionPolicy>,
}

/// Cached permission decision
#[derive(Debug, Clone)]
pub struct CachedPermission {
    pub result: PermissionResult,
    pub expires_at: DateTime<Utc>,
    pub context_hash: u64,
}

/// Permission grant for a specific session
#[derive(Debug, Clone)]
pub struct PermissionGrant {
    pub grant_id: Uuid,
    pub user_role: UserRole,
    pub plugin_id: Uuid,
    pub permissions: Vec<Permission>,
    pub granted_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub conditions: Vec<GrantCondition>,
}

/// Conditions for permission grants
#[derive(Debug, Clone)]
pub enum GrantCondition {
    TimeWindow { start: String, end: String },
    IpRange(String),
    ResourcePattern(String),
    MaxUsageCount(u32),
    RequireConfirmation,
}

/// Permission check request
#[derive(Debug, Clone)]
pub struct PermissionCheck {
    pub user_role: UserRole,
    pub plugin_id: Uuid,
    pub action: String,
    pub resource: String,
    pub context: PermissionContext,
}

/// Context for permission evaluation
#[derive(Debug, Clone)]
pub struct PermissionContext {
    pub session_id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub additional_data: HashMap<String, String>,
}

/// Result of permission check
#[derive(Debug, Clone, PartialEq)]
pub enum PermissionResult {
    Granted,
    Denied { reason: String },
    RequiresConfirmation { message: String },
    RequiresElevation { required_role: UserRole },
}

/// Permission policy
#[derive(Debug, Clone)]
pub struct PermissionPolicy {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub rules: Vec<PolicyRule>,
    pub priority: i32,
    pub enabled: bool,
}

/// Policy rule
#[derive(Debug, Clone)]
pub struct PolicyRule {
    pub condition: PolicyCondition,
    pub action: PolicyAction,
}

/// Policy condition
#[derive(Debug, Clone)]
pub enum PolicyCondition {
    RoleEquals(UserRole),
    ActionMatches(String),
    ResourceMatches(String),
    PermissionLevelAbove(PermissionLevel),
    TimeWindow { start: String, end: String },
    And(Vec<PolicyCondition>),
    Or(Vec<PolicyCondition>),
    Not(Box<PolicyCondition>),
}

/// Policy action
#[derive(Debug, Clone)]
pub enum PolicyAction {
    Allow,
    Deny(String),
    RequireConfirmation(String),
    RequireElevation(UserRole),
    Log(String),
}

impl PermissionManager {
    /// Create a new permission manager
    pub fn new() -> Self {
        Self {
            permission_cache: HashMap::new(),
            active_grants: HashMap::new(),
            policies: Vec::new(),
        }
    }
    
    /// Initialize with default policies
    pub fn initialize(&mut self) {
        log::info!("Initializing permission manager");
        
        // Load default security policies
        self.load_default_policies();
        
        // Clean up expired entries
        self.cleanup_expired_entries();
    }
    
    /// Check if a permission is granted
    pub fn check_permission(&mut self, check: &PermissionCheck) -> Result<PermissionResult> {
        // Generate cache key
        let cache_key = self.generate_cache_key(check);
        
        // Check cache first
        if let Some(cached) = self.permission_cache.get(&cache_key) {
            if cached.expires_at > Utc::now() {
                log::debug!("Permission check cache hit for {}", cache_key);
                return Ok(cached.result.clone());
            }
        }
        
        // Evaluate permission
        let result = self.evaluate_permission(check)?;
        
        // Cache the result
        self.cache_permission_result(&cache_key, &result, &check.context);
        
        Ok(result)
    }
    
    /// Grant permissions for a plugin session
    pub fn grant_permissions(
        &mut self,
        user_role: UserRole,
        plugin_id: Uuid,
        permissions: Vec<Permission>,
        conditions: Vec<GrantCondition>,
    ) -> Result<Uuid> {
        let grant_id = Uuid::new_v4();
        
        let grant = PermissionGrant {
            grant_id,
            user_role,
            plugin_id,
            permissions,
            granted_at: Utc::now(),
            expires_at: Some(Utc::now() + chrono::Duration::hours(24)), // Default 24h expiry
            conditions,
        };
        
        self.active_grants.insert(grant_id, grant);
        
        log::info!("Granted permissions for plugin {} with grant ID {}", plugin_id, grant_id);
        
        Ok(grant_id)
    }
    
    /// Revoke a permission grant
    pub fn revoke_grant(&mut self, grant_id: Uuid) -> Result<()> {
        if self.active_grants.remove(&grant_id).is_some() {
            log::info!("Revoked permission grant {}", grant_id);
            Ok(())
        } else {
            Err(anyhow::anyhow!("Grant not found"))
        }
    }
    
    /// Get active grants for a plugin
    pub fn get_plugin_grants(&self, plugin_id: Uuid) -> Vec<&PermissionGrant> {
        self.active_grants
            .values()
            .filter(|grant| grant.plugin_id == plugin_id)
            .collect()
    }
    
    /// Add a security policy
    pub fn add_policy(&mut self, policy: PermissionPolicy) {
        log::info!("Adding security policy: {}", policy.name);
        self.policies.push(policy);
        
        // Sort by priority (higher priority first)
        self.policies.sort_by(|a, b| b.priority.cmp(&a.priority));
    }
    
    /// Remove a security policy
    pub fn remove_policy(&mut self, policy_id: Uuid) -> Result<()> {
        let initial_len = self.policies.len();
        self.policies.retain(|p| p.id != policy_id);
        
        if self.policies.len() < initial_len {
            log::info!("Removed security policy {}", policy_id);
            Ok(())
        } else {
            Err(anyhow::anyhow!("Policy not found"))
        }
    }
    
    /// Evaluate permission against policies
    fn evaluate_permission(&self, check: &PermissionCheck) -> Result<PermissionResult> {
        // First check role-based permissions
        if !check.user_role.has_permission(&check.action, &check.resource) {
            return Ok(PermissionResult::Denied {
                reason: "Role does not have required permission".to_string(),
            });
        }
        
        // Then check policies
        for policy in &self.policies {
            if !policy.enabled {
                continue;
            }
            
            for rule in &policy.rules {
                if self.evaluate_condition(&rule.condition, check) {
                    match &rule.action {
                        PolicyAction::Allow => continue,
                        PolicyAction::Deny(reason) => {
                            return Ok(PermissionResult::Denied {
                                reason: reason.clone(),
                            });
                        }
                        PolicyAction::RequireConfirmation(message) => {
                            return Ok(PermissionResult::RequiresConfirmation {
                                message: message.clone(),
                            });
                        }
                        PolicyAction::RequireElevation(role) => {
                            return Ok(PermissionResult::RequiresElevation {
                                required_role: role.clone(),
                            });
                        }
                        PolicyAction::Log(message) => {
                            log::info!("Policy log: {}", message);
                        }
                    }
                }
            }
        }
        
        // Default allow if no policies deny
        Ok(PermissionResult::Granted)
    }
    
    /// Evaluate a policy condition
    fn evaluate_condition(&self, condition: &PolicyCondition, check: &PermissionCheck) -> bool {
        match condition {
            PolicyCondition::RoleEquals(role) => check.user_role == *role,
            PolicyCondition::ActionMatches(pattern) => {
                // Simple pattern matching (could be enhanced with regex)
                check.action.contains(pattern) || pattern == "*"
            }
            PolicyCondition::ResourceMatches(pattern) => {
                check.resource.starts_with(pattern) || pattern == "*"
            }
            PolicyCondition::PermissionLevelAbove(_level) => {
                // TODO: Implement permission level checking
                false
            }
            PolicyCondition::TimeWindow { start: _, end: _ } => {
                // TODO: Implement time window checking
                true
            }
            PolicyCondition::And(conditions) => {
                conditions.iter().all(|c| self.evaluate_condition(c, check))
            }
            PolicyCondition::Or(conditions) => {
                conditions.iter().any(|c| self.evaluate_condition(c, check))
            }
            PolicyCondition::Not(condition) => {
                !self.evaluate_condition(condition, check)
            }
        }
    }
    
    /// Generate cache key for permission check
    fn generate_cache_key(&self, check: &PermissionCheck) -> String {
        format!(
            "{}:{}:{}:{}",
            check.user_role.display_name(),
            check.plugin_id,
            check.action,
            check.resource
        )
    }
    
    /// Cache permission result
    fn cache_permission_result(&mut self, key: &str, result: &PermissionResult, context: &PermissionContext) {
        let cached = CachedPermission {
            result: result.clone(),
            expires_at: Utc::now() + chrono::Duration::minutes(5), // 5 minute cache
            context_hash: self.hash_context(context),
        };
        
        self.permission_cache.insert(key.to_string(), cached);
    }
    
    /// Hash permission context for cache validation
    fn hash_context(&self, context: &PermissionContext) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        context.session_id.hash(&mut hasher);
        context.ip_address.hash(&mut hasher);
        hasher.finish()
    }
    
    /// Load default security policies
    fn load_default_policies(&mut self) {
        // Add default deny-all policy for critical operations
        let critical_policy = PermissionPolicy {
            id: Uuid::new_v4(),
            name: "Critical Operations".to_string(),
            description: "Require confirmation for critical operations".to_string(),
            rules: vec![
                PolicyRule {
                    condition: PolicyCondition::PermissionLevelAbove(PermissionLevel::High),
                    action: PolicyAction::RequireConfirmation(
                        "This operation requires elevated permissions".to_string()
                    ),
                },
            ],
            priority: 100,
            enabled: true,
        };
        
        self.policies.push(critical_policy);
    }
    
    /// Clean up expired cache entries and grants
    fn cleanup_expired_entries(&mut self) {
        let now = Utc::now();
        
        // Clean up cache
        self.permission_cache.retain(|_, cached| cached.expires_at > now);
        
        // Clean up grants
        self.active_grants.retain(|_, grant| {
            grant.expires_at.map_or(true, |expires| expires > now)
        });
    }
}

impl Default for PermissionManager {
    fn default() -> Self {
        Self::new()
    }
}
