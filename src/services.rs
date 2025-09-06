use std::process::Command;

/// Result of service configuration operations
#[derive(Debug)]
pub struct ServiceResult {
    pub changed: bool,
    pub enabled_services: Vec<String>,
    pub started_services: Vec<String>,
    pub failed_services: Vec<String>,
}

/// Ensure all specified services are configured (enabled and started)
pub fn ensure_services_configured(services: &[String]) -> Result<ServiceResult, String> {
    if services.is_empty() {
        return Ok(ServiceResult {
            changed: false,
            enabled_services: Vec::new(),
            started_services: Vec::new(),
            failed_services: Vec::new(),
        });
    }

    let mut enabled_services = Vec::new();
    let mut started_services = Vec::new();
    let mut failed_services = Vec::new();
    let mut changed = false;

    for service in services {
        match ensure_service_configured(service) {
            Ok((enabled, started)) => {
                if enabled {
                    enabled_services.push(service.clone());
                    changed = true;
                }
                if started {
                    started_services.push(service.clone());
                    changed = true;
                }
            }
            Err(err) => {
                eprintln!("{}", crate::colo::red(&format!("Failed to configure service {}: {}", service, err)));
                failed_services.push(service.clone());
            }
        }
    }

    Ok(ServiceResult {
        changed,
        enabled_services,
        started_services,
        failed_services,
    })
}

/// Ensure a single service is configured (enabled and started)
fn ensure_service_configured(service_name: &str) -> Result<(bool, bool), String> {
    let mut enabled = false;
    let mut started = false;

    // Check if service is enabled
    let is_enabled = is_service_enabled(service_name)?;

    if !is_enabled {
        // Enable the service
        enable_service(service_name)?;
        enabled = true;
    }

    // Check if service is running
    let is_active = is_service_active(service_name)?;

    if !is_active {
        // Start the service
        start_service(service_name)?;
        started = true;
    }

    Ok((enabled, started))
}

/// Check if a service is enabled
fn is_service_enabled(service_name: &str) -> Result<bool, String> {
    let output = Command::new("systemctl")
        .args(&["is-enabled", service_name])
        .output()
        .map_err(|e| format!("Failed to check if service is enabled: {}", e))?;

    // systemctl is-enabled returns:
    // - "enabled" if enabled
    // - "disabled" if disabled
    // - "masked" if masked
    // - "static" if static
    // - "indirect" if indirect
    let status = String::from_utf8_lossy(&output.stdout).trim().to_string();

    match status.as_str() {
        "enabled" | "static" | "indirect" => Ok(true),
        "disabled" | "masked" => Ok(false),
        _ => {
            // If we can't determine the status, assume it's not enabled
            eprintln!("{}", crate::colo::yellow(&format!("Warning: Unknown service enable status '{}' for {}", status, service_name)));
            Ok(false)
        }
    }
}

/// Check if a service is active (running)
fn is_service_active(service_name: &str) -> Result<bool, String> {
    let output = Command::new("systemctl")
        .args(&["is-active", service_name])
        .output()
        .map_err(|e| format!("Failed to check if service is active: {}", e))?;

    let status = String::from_utf8_lossy(&output.stdout).trim().to_string();
    Ok(status == "active")
}

/// Enable a service
fn enable_service(service_name: &str) -> Result<(), String> {
    let output = Command::new("systemctl")
        .args(&["enable", service_name])
        .output()
        .map_err(|e| format!("Failed to enable service: {}", e))?;

    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!("systemctl enable failed: {}", stderr))
    }
}

/// Start a service
fn start_service(service_name: &str) -> Result<(), String> {
    let output = Command::new("systemctl")
        .args(&["start", service_name])
        .output()
        .map_err(|e| format!("Failed to start service: {}", e))?;

    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!("systemctl start failed: {}", stderr))
    }
}

/// Get all services defined in the configuration
pub fn get_configured_services(config: &crate::config::Config) -> Vec<String> {
    config.packages.values()
        .filter_map(|pkg| pkg.service.clone())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_configured_services() {
        use std::collections::HashMap;
        use crate::config::{Config, Package};

        let mut config = Config::new();

        // Add packages with services
        let pkg1 = Package {
            name: "test1".to_string(),
            config: None,
            service: Some("service1".to_string()),
            env_vars: HashMap::new(),
        };
        config.packages.insert("test1".to_string(), pkg1);

        let pkg2 = Package {
            name: "test2".to_string(),
            config: None,
            service: None, // No service
            env_vars: HashMap::new(),
        };
        config.packages.insert("test2".to_string(), pkg2);

        let pkg3 = Package {
            name: "test3".to_string(),
            config: None,
            service: Some("service3".to_string()),
            env_vars: HashMap::new(),
        };
        config.packages.insert("test3".to_string(), pkg3);

        let services = get_configured_services(&config);
        assert_eq!(services.len(), 2);
        assert!(services.contains(&"service1".to_string()));
        assert!(services.contains(&"service3".to_string()));
        assert!(!services.contains(&"service2".to_string()));
    }
}