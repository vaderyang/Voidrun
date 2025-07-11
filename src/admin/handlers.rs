use super::*;
use axum::{
    extract::{Path, State, Query},
    response::{Html, Json},
    http::StatusCode,
};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;
use serde_json::json;
use std::collections::HashMap;

use crate::sandbox::manager::SandboxManager;
use crate::admin::ui::ADMIN_UI_HTML;
use crate::sandbox::SandboxMode;

pub async fn admin_ui() -> Html<&'static str> {
    Html(ADMIN_UI_HTML)
}

pub async fn get_system_status(
    State(app_state): State<Arc<RwLock<SandboxManager>>>,
) -> Result<Json<SystemStatus>, StatusCode> {
    let manager = app_state.read().await;
    
    let uptime = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| {
            error!("Failed to get system time: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .as_secs();
    
    let active_sandboxes = manager.list_sandboxes().await.len() as u32;
    
    // Try to get real system resource usage, fallback to unavailable if fails
    let memory_usage = get_system_memory_usage().await
        .unwrap_or_else(|e| {
            error!("Failed to get memory usage: {}", e);
            ResourceUsage {
                used: 0.0,
                total: 0.0,
                percentage: 0.0,
            }
        });
    
    let cpu_usage = get_system_cpu_usage().await
        .unwrap_or_else(|e| {
            error!("Failed to get CPU usage: {}", e);
            ResourceUsage {
                used: 0.0,
                total: 0.0,
                percentage: 0.0,
            }
        });
    
    let status = SystemStatus {
        uptime,
        active_sandboxes,
        total_sandboxes_created: active_sandboxes, // TODO: Implement persistent counter
        backend_type: format!("{:?}", manager.get_backend_type()),
        version: env!("CARGO_PKG_VERSION").to_string(),
        memory_usage,
        cpu_usage,
    };
    
    Ok(Json(status))
}

// Helper function to extract numbers from lines like "Pages free: 12345."
fn extract_number_from_line(line: &str) -> u64 {
    line.split_whitespace()
        .find(|part| part.chars().all(|c| c.is_ascii_digit() || c == '.'))
        .and_then(|s| s.trim_end_matches('.').parse::<u64>().ok())
        .unwrap_or(0)
}

async fn get_system_memory_usage() -> Result<ResourceUsage, String> {
    #[cfg(target_os = "linux")]
    {
        use std::fs;
        
        // Read /proc/meminfo on Linux systems
        let meminfo = fs::read_to_string("/proc/meminfo")
            .map_err(|e| format!("Failed to read /proc/meminfo: {}", e))?;
        
        let mut total = 0;
        let mut available = 0;
        
        for line in meminfo.lines() {
            if line.starts_with("MemTotal:") {
                total = line.split_whitespace().nth(1)
                    .and_then(|s| s.parse::<u64>().ok())
                    .unwrap_or(0);
            } else if line.starts_with("MemAvailable:") {
                available = line.split_whitespace().nth(1)
                    .and_then(|s| s.parse::<u64>().ok())
                    .unwrap_or(0);
            }
        }
        
        if total == 0 {
            return Err("Could not parse memory information".to_string());
        }
        
        let used = total - available;
        let percentage = (used as f64 / total as f64) * 100.0;
        
        Ok(ResourceUsage {
            used: used as f64 / 1024.0, // Convert KB to MB
            total: total as f64 / 1024.0, // Convert KB to MB
            percentage,
        })
    }
    
    #[cfg(target_os = "macos")]
    {
        use std::process::Command;
        
        // Use vm_stat command on macOS
        let output = Command::new("vm_stat")
            .output()
            .map_err(|e| format!("Failed to run vm_stat: {}", e))?;
        
        if !output.status.success() {
            return Err("vm_stat command failed".to_string());
        }
        
        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut page_size = 4096; // Default page size
        let mut free_pages = 0;
        let mut active_pages = 0;
        let mut inactive_pages = 0;
        let mut wired_pages = 0;
        let mut compressed_pages = 0;
        
        for line in stdout.lines() {
            if line.starts_with("Mach Virtual Memory Statistics:") {
                // Extract page size if mentioned
                if let Some(size_str) = line.split("page size of ").nth(1) {
                    if let Some(size_part) = size_str.split(" bytes").next() {
                        page_size = size_part.parse().unwrap_or(4096);
                    }
                }
            } else if line.starts_with("Pages free:") {
                free_pages = extract_number_from_line(line);
            } else if line.starts_with("Pages active:") {
                active_pages = extract_number_from_line(line);
            } else if line.starts_with("Pages inactive:") {
                inactive_pages = extract_number_from_line(line);
            } else if line.starts_with("Pages wired down:") {
                wired_pages = extract_number_from_line(line);
            } else if line.starts_with("Pages occupied by compressor:") {
                compressed_pages = extract_number_from_line(line);
            }
        }
        
        let total_pages = free_pages + active_pages + inactive_pages + wired_pages + compressed_pages;
        let used_pages = total_pages - free_pages;
        
        if total_pages == 0 {
            return Err("Could not parse memory information from vm_stat".to_string());
        }
        
        let total_bytes = total_pages * page_size;
        let used_bytes = used_pages * page_size;
        let percentage = (used_bytes as f64 / total_bytes as f64) * 100.0;
        
        Ok(ResourceUsage {
            used: used_bytes as f64 / 1024.0 / 1024.0, // Convert to MB
            total: total_bytes as f64 / 1024.0 / 1024.0, // Convert to MB
            percentage,
        })
    }
    
    #[cfg(not(any(target_os = "linux", target_os = "macos")))]
    {
        Err("System memory monitoring not supported on this platform".to_string())
    }
}


async fn get_system_cpu_usage() -> Result<ResourceUsage, String> {
    #[cfg(target_os = "linux")]
    {
        use std::fs;
        use std::time::Duration;
        use tokio::time::sleep;
        
        // Read /proc/stat twice with a small delay to calculate CPU usage
        let stat1 = fs::read_to_string("/proc/stat")
            .map_err(|e| format!("Failed to read /proc/stat: {}", e))?;
        
        let cpu1 = parse_cpu_line(&stat1)
            .ok_or("Failed to parse first CPU stats")?;
        
        sleep(Duration::from_millis(100)).await;
        
        let stat2 = fs::read_to_string("/proc/stat")
            .map_err(|e| format!("Failed to read /proc/stat: {}", e))?;
        
        let cpu2 = parse_cpu_line(&stat2)
            .ok_or("Failed to parse second CPU stats")?;
        
        let total_diff = cpu2.total - cpu1.total;
        let idle_diff = cpu2.idle - cpu1.idle;
        
        if total_diff == 0 {
            return Err("No CPU time elapsed".to_string());
        }
        
        let cpu_usage = ((total_diff - idle_diff) as f64 / total_diff as f64) * 100.0;
        
        Ok(ResourceUsage {
            used: cpu_usage,
            total: 100.0,
            percentage: cpu_usage,
        })
    }
    
    #[cfg(target_os = "macos")]
    {
        use std::process::Command;
        use std::time::Duration;
        use tokio::time::sleep;
        
        // Use iostat command on macOS to get CPU usage
        let output1 = Command::new("iostat")
            .args(["-c", "1"])
            .output()
            .map_err(|e| format!("Failed to run iostat: {}", e))?;
        
        if !output1.status.success() {
            return Err("iostat command failed".to_string());
        }
        
        sleep(Duration::from_millis(100)).await;
        
        let output2 = Command::new("iostat")
            .args(["-c", "1"])
            .output()
            .map_err(|e| format!("Failed to run iostat: {}", e))?;
        
        if !output2.status.success() {
            return Err("iostat command failed".to_string());
        }
        
        let stdout = String::from_utf8_lossy(&output2.stdout);
        let cpu_usage = parse_iostat_cpu(&stdout)
            .ok_or("Failed to parse CPU usage from iostat")?;
        
        Ok(ResourceUsage {
            used: cpu_usage,
            total: 100.0,
            percentage: cpu_usage,
        })
    }
    
    #[cfg(not(any(target_os = "linux", target_os = "macos")))]
    {
        Err("System CPU monitoring not supported on this platform".to_string())
    }
}

#[cfg(target_os = "linux")]
struct CpuStats {
    total: u64,
    idle: u64,
}

#[cfg(target_os = "linux")]
fn parse_cpu_line(stat_content: &str) -> Option<CpuStats> {
    let first_line = stat_content.lines().next()?;
    if !first_line.starts_with("cpu ") {
        return None;
    }
    
    let parts: Vec<&str> = first_line.split_whitespace().collect();
    if parts.len() < 5 {
        return None;
    }
    
    let user: u64 = parts[1].parse().ok()?;
    let nice: u64 = parts[2].parse().ok()?;
    let system: u64 = parts[3].parse().ok()?;
    let idle: u64 = parts[4].parse().ok()?;
    let iowait: u64 = parts.get(5).and_then(|s| s.parse().ok()).unwrap_or(0);
    let irq: u64 = parts.get(6).and_then(|s| s.parse().ok()).unwrap_or(0);
    let softirq: u64 = parts.get(7).and_then(|s| s.parse().ok()).unwrap_or(0);
    let steal: u64 = parts.get(8).and_then(|s| s.parse().ok()).unwrap_or(0);
    
    let total = user + nice + system + idle + iowait + irq + softirq + steal;
    
    Some(CpuStats { total, idle })
}

#[cfg(target_os = "macos")]
fn parse_iostat_cpu(iostat_output: &str) -> Option<f64> {
    // iostat output format:
    //           cpu     load average
    //     us    sy    id   1m   5m   15m
    //      5.2   3.1  91.7  2.1  2.3  2.4
    
    let lines: Vec<&str> = iostat_output.lines().collect();
    if lines.len() < 3 {
        return None;
    }
    
    // Find the line with CPU percentages (usually the last line)
    for line in lines.iter().rev() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 3 {
            // Try to parse the idle percentage (third column)
            if let Ok(idle) = parts[2].parse::<f64>() {
                return Some(100.0 - idle); // CPU usage = 100% - idle%
            }
        }
    }
    
    None
}

async fn get_sandbox_cpu_usage(sandbox_id: &str) -> Result<f64, String> {
    #[cfg(feature = "docker")]
    {
        use bollard::Docker;
        use bollard::container::StatsOptions;
        use futures_util::StreamExt;
        
        let docker = Docker::connect_with_local_defaults()
            .map_err(|e| format!("Failed to connect to Docker: {}", e))?;
        
        let options = StatsOptions {
            stream: false,
            one_shot: true,
        };
        
        let mut stream = docker.stats(sandbox_id, Some(options));
        
        if let Some(result) = stream.next().await {
            let stats = result
                .map_err(|e| format!("Failed to get container stats: {}", e))?;
            
            // Calculate CPU usage percentage
            let cpu_usage = {
                let cpu_delta = stats.cpu_stats.cpu_usage.total_usage - stats.precpu_stats.cpu_usage.total_usage;
                let system_delta = stats.cpu_stats.system_cpu_usage.unwrap_or(0) - stats.precpu_stats.system_cpu_usage.unwrap_or(0);
                
                if system_delta > 0 {
                    let cpu_count = stats.cpu_stats.cpu_usage.percpu_usage.as_ref().map(|v| v.len()).unwrap_or(1);
                    (cpu_delta as f64 / system_delta as f64) * cpu_count as f64 * 100.0
                } else {
                    0.0
                }
            };
            
            Ok(cpu_usage)
        } else {
            Err("No stats available for container".to_string())
        }
    }
    
    #[cfg(not(feature = "docker"))]
    {
        Err(format!("Docker feature not enabled for CPU monitoring of sandbox {}", sandbox_id))
    }
}

async fn get_container_stats(sandbox_id: &str) -> Result<serde_json::Value, String> {
    #[cfg(feature = "docker")]
    {
        use bollard::Docker;
        use bollard::container::StatsOptions;
        use futures_util::StreamExt;
        
        let docker = Docker::connect_with_local_defaults()
            .map_err(|e| format!("Failed to connect to Docker: {}", e))?;
        
        let options = StatsOptions {
            stream: false,
            one_shot: true,
        };
        
        let mut stream = docker.stats(sandbox_id, Some(options));
        
        if let Some(result) = stream.next().await {
            let stats = result
                .map_err(|e| format!("Failed to get container stats: {}", e))?;
            
            // Calculate memory usage
            let (memory_used, memory_limit, memory_percentage) = {
                let used = stats.memory_stats.usage.unwrap_or(0) as f64;
                let limit = stats.memory_stats.limit.unwrap_or(0) as f64;
                let percentage = if limit > 0.0 { (used / limit) * 100.0 } else { 0.0 };
                (used / 1024.0 / 1024.0, limit / 1024.0 / 1024.0, percentage) // Convert to MB
            };
            
            // Calculate CPU usage
            let cpu_percentage = {
                let cpu_delta = stats.cpu_stats.cpu_usage.total_usage - stats.precpu_stats.cpu_usage.total_usage;
                let system_delta = stats.cpu_stats.system_cpu_usage.unwrap_or(0) - stats.precpu_stats.system_cpu_usage.unwrap_or(0);
                
                if system_delta > 0 {
                    let cpu_count = stats.cpu_stats.cpu_usage.percpu_usage.as_ref().map(|v| v.len()).unwrap_or(1);
                    (cpu_delta as f64 / system_delta as f64) * cpu_count as f64 * 100.0
                } else {
                    0.0
                }
            };
            
            // Calculate network usage
            let (bytes_in, bytes_out) = if let Some(networks) = &stats.networks {
                let mut total_rx = 0;
                let mut total_tx = 0;
                for (_, network) in networks {
                    total_rx += network.rx_bytes;
                    total_tx += network.tx_bytes;
                }
                (total_rx, total_tx)
            } else {
                (0, 0)
            };
            
            // Calculate disk usage (block I/O)
            let (disk_read, disk_write) = if let Some(io_service_bytes_recursive) = &stats.blkio_stats.io_service_bytes_recursive {
                let mut total_read = 0;
                let mut total_write = 0;
                
                for entry in io_service_bytes_recursive {
                    if entry.op == "read" {
                        total_read += entry.value;
                    } else if entry.op == "write" {
                        total_write += entry.value;
                    }
                }
                (total_read, total_write)
            } else {
                (0, 0)
            };
            
            let resources = json!({
                "memory": {
                    "used": memory_used,
                    "limit": memory_limit,
                    "percentage": memory_percentage
                },
                "cpu": {
                    "percentage": cpu_percentage,
                    "cores": cpu_percentage / 100.0
                },
                "disk": {
                    "read_bytes": disk_read,
                    "write_bytes": disk_write,
                    "used": (disk_read + disk_write) as f64 / 1024.0 / 1024.0, // Convert to MB
                    "limit": 1024.0, // Default limit
                    "percentage": ((disk_read + disk_write) as f64 / 1024.0 / 1024.0 / 1024.0) * 100.0
                },
                "network": {
                    "bytes_in": bytes_in,
                    "bytes_out": bytes_out
                }
            });
            
            Ok(resources)
        } else {
            Err("No stats available for container".to_string())
        }
    }
    
    #[cfg(not(feature = "docker"))]
    {
        Err(format!("Docker feature not enabled for stats of sandbox {}", sandbox_id))
    }
}

async fn get_container_logs(sandbox_id: &str, lines: u32) -> Result<Vec<LogEntry>, String> {
    #[cfg(feature = "docker")]
    {
        use bollard::Docker;
        use bollard::container::LogsOptions;
        use futures_util::StreamExt;
        use chrono::{DateTime, Utc};
        
        let docker = Docker::connect_with_local_defaults()
            .map_err(|e| format!("Failed to connect to Docker: {}", e))?;
        
        let options = LogsOptions::<String> {
            follow: false,
            stdout: true,
            stderr: true,
            since: 0,
            until: 0,
            timestamps: true,
            tail: lines.to_string(),
        };
        
        let mut stream = docker.logs(sandbox_id, Some(options));
        let mut logs = Vec::new();
        
        while let Some(result) = stream.next().await {
            match result {
                Ok(log_output) => {
                    let (level, message) = match log_output {
                        bollard::container::LogOutput::StdOut { message } => {
                            ("INFO", String::from_utf8_lossy(&message).to_string())
                        }
                        bollard::container::LogOutput::StdErr { message } => {
                            ("ERROR", String::from_utf8_lossy(&message).to_string())
                        }
                        bollard::container::LogOutput::StdIn { message } => {
                            ("INPUT", String::from_utf8_lossy(&message).to_string())
                        }
                        bollard::container::LogOutput::Console { message } => {
                            ("CONSOLE", String::from_utf8_lossy(&message).to_string())
                        }
                    };
                    
                    // Parse timestamp if present
                    let (timestamp, clean_message) = if let Some(space_pos) = message.find(' ') {
                        let timestamp_str = &message[..space_pos];
                        let msg = &message[space_pos + 1..];
                        
                        // Try to parse the timestamp
                        if let Ok(parsed_time) = DateTime::parse_from_rfc3339(timestamp_str) {
                            (parsed_time.to_rfc3339(), msg.to_string())
                        } else {
                            (Utc::now().to_rfc3339(), message)
                        }
                    } else {
                        (Utc::now().to_rfc3339(), message)
                    };
                    
                    logs.push(LogEntry {
                        timestamp,
                        level: level.to_string(),
                        message: clean_message.trim().to_string(),
                        sandbox_id: Some(sandbox_id.to_string()),
                    });
                }
                Err(e) => {
                    error!("Error reading container logs: {}", e);
                    break;
                }
            }
        }
        
        // Sort logs by timestamp (newest first)
        logs.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        
        Ok(logs)
    }
    
    #[cfg(not(feature = "docker"))]
    {
        Err(format!("Docker feature not enabled for logs of sandbox {} (requested {} lines)", sandbox_id, lines))
    }
}

async fn get_system_logs_impl(lines: u32) -> Result<Vec<LogEntry>, String> {
    use std::fs;
    use std::process::Command;
    use chrono::{DateTime, Utc};
    
    // Try different approaches based on the platform
    #[cfg(target_os = "linux")]
    let journalctl_result = Command::new("journalctl")
        .args(["-u", "sandbox-service", "-n", &lines.to_string(), "--no-pager", "--output=json"])
        .output();
    
    #[cfg(target_os = "macos")]
    let journalctl_result = Command::new("log")
        .args(["show", "--last", &format!("{}h", std::cmp::max(1, lines / 10)), "--style", "syslog"])
        .output();
    
    #[cfg(not(any(target_os = "linux", target_os = "macos")))]
    let journalctl_result: Result<std::process::Output, std::io::Error> = Err(std::io::Error::new(std::io::ErrorKind::Unsupported, "Platform not supported"));
    
    if let Ok(output) = journalctl_result {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let mut logs = Vec::new();
            
            for line in stdout.lines() {
                if line.trim().is_empty() {
                    continue;
                }
                
                if let Ok(entry) = serde_json::from_str::<serde_json::Value>(line) {
                    let timestamp = entry.get("__REALTIME_TIMESTAMP")
                        .and_then(|v| v.as_str())
                        .and_then(|s| s.parse::<u64>().ok())
                        .map(|microseconds| {
                            let seconds = microseconds / 1_000_000;
                            let nanos = (microseconds % 1_000_000) * 1_000;
                            DateTime::from_timestamp(seconds as i64, nanos as u32)
                                .unwrap_or(Utc::now())
                                .to_rfc3339()
                        })
                        .unwrap_or_else(|| Utc::now().to_rfc3339());
                    
                    let level = entry.get("PRIORITY")
                        .and_then(|v| v.as_str())
                        .map(|p| match p {
                            "0" | "1" | "2" | "3" => "ERROR",
                            "4" => "WARN",
                            "5" | "6" => "INFO",
                            "7" => "DEBUG",
                            _ => "INFO",
                        })
                        .unwrap_or("INFO");
                    
                    let message = entry.get("MESSAGE")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();
                    
                    logs.push(LogEntry {
                        timestamp,
                        level: level.to_string(),
                        message,
                        sandbox_id: None,
                    });
                }
            }
            
            if !logs.is_empty() {
                return Ok(logs);
            }
        }
    }
    
    // Fallback to reading log files directly
    #[cfg(target_os = "linux")]
    let log_paths = [
        "/var/log/syslog",
        "/var/log/messages",
        "/var/log/sandbox-service.log",
    ];
    
    #[cfg(target_os = "macos")]
    let log_paths = [
        "/var/log/system.log",
        "/usr/local/var/log/sandbox-service.log",
        "/tmp/sandbox-service.log",
    ];
    
    #[cfg(not(any(target_os = "linux", target_os = "macos")))]
    let log_paths: [&str; 0] = [];
    
    for log_path in &log_paths {
        if let Ok(content) = fs::read_to_string(log_path) {
            let mut logs = Vec::new();
            let lines_vec: Vec<&str> = content.lines().collect();
            let start_index = lines_vec.len().saturating_sub(lines as usize);
            
            for line in &lines_vec[start_index..] {
                if line.trim().is_empty() {
                    continue;
                }
                
                // Try to parse syslog format
                let (timestamp, level, message) = parse_syslog_line(line);
                
                logs.push(LogEntry {
                    timestamp,
                    level,
                    message,
                    sandbox_id: None,
                });
            }
            
            if !logs.is_empty() {
                return Ok(logs);
            }
        }
    }
    
    Err(format!("No system logs found (requested {} lines)", lines))
}

fn parse_syslog_line(line: &str) -> (String, String, String) {
    use chrono::{DateTime, Utc};
    
    // Try to parse different syslog formats
    // Format: Jan 1 12:34:56 hostname program[pid]: message
    let parts: Vec<&str> = line.splitn(4, ' ').collect();
    
    if parts.len() >= 4 {
        let timestamp_str = format!("{} {} {}", parts[0], parts[1], parts[2]);
        
        // Try to parse timestamp - if it fails, use current time
        let timestamp = if let Ok(parsed) = chrono::NaiveDateTime::parse_from_str(
            &format!("{} {}", chrono::Utc::now().format("%Y"), timestamp_str),
            "%Y %b %d %H:%M:%S"
        ) {
DateTime::<Utc>::from_naive_utc_and_offset(parsed, Utc).to_rfc3339()
        } else {
            Utc::now().to_rfc3339()
        };
        
        let rest = parts[3];
        let level = if rest.contains("ERROR") || rest.contains("error") {
            "ERROR"
        } else if rest.contains("WARN") || rest.contains("warn") {
            "WARN"
        } else if rest.contains("DEBUG") || rest.contains("debug") {
            "DEBUG"
        } else {
            "INFO"
        };
        
        (timestamp, level.to_string(), rest.to_string())
    } else {
        // Fallback for lines that don't match expected format
        (Utc::now().to_rfc3339(), "INFO".to_string(), line.to_string())
    }
}

#[derive(Debug)]
struct ApiResponse {
    status: u16,
    headers: HashMap<String, String>,
    body: String,
}

async fn make_api_request(request: ApiTestRequest) -> Result<ApiResponse, String> {
    use reqwest::Client;
    use std::time::Duration;
    
    let client = Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;
    
    // Build the full URL - assume we're testing our own API
    let base_url = "http://127.0.0.1:8070"; // Default server address
    let url = if request.path.starts_with('/') {
        format!("{}{}", base_url, request.path)
    } else {
        format!("{}/{}", base_url, request.path)
    };
    
    // Create the request builder
    let mut req_builder = match request.method.to_uppercase().as_str() {
        "GET" => client.get(&url),
        "POST" => client.post(&url),
        "PUT" => client.put(&url),
        "DELETE" => client.delete(&url),
        "PATCH" => client.patch(&url),
        "HEAD" => client.head(&url),
        _ => return Err(format!("Unsupported HTTP method: {}", request.method)),
    };
    
    // Add headers if provided
    if let Some(headers) = &request.headers {
        for (key, value) in headers {
            req_builder = req_builder.header(key, value);
        }
    }
    
    // Add body if provided and method supports it
    if let Some(body) = &request.body {
        if !matches!(request.method.to_uppercase().as_str(), "GET" | "HEAD") {
            // Try to determine content type
            let content_type = request.headers
                .as_ref()
                .and_then(|h| h.get("content-type").or_else(|| h.get("Content-Type")))
                .map(|s| s.as_str())
                .unwrap_or("application/json");
            
            req_builder = req_builder
                .header("Content-Type", content_type)
                .body(body.clone());
        }
    }
    
    // Send the request
    let response = req_builder.send().await
        .map_err(|e| format!("Failed to send request: {}", e))?;
    
    // Extract response data
    let status = response.status().as_u16();
    
    // Convert headers to HashMap
    let mut headers = HashMap::new();
    for (key, value) in response.headers() {
        if let Ok(value_str) = value.to_str() {
            headers.insert(key.to_string(), value_str.to_string());
        }
    }
    
    // Get response body
    let body = response.text().await
        .map_err(|e| format!("Failed to read response body: {}", e))?;
    
    Ok(ApiResponse {
        status,
        headers,
        body,
    })
}

pub async fn list_sandboxes(
    State(app_state): State<Arc<RwLock<SandboxManager>>>,
) -> Result<Json<Vec<SandboxInfo>>, StatusCode> {
    let manager = app_state.read().await;
    let sandboxes = manager.get_all_sandboxes().await;
    
    // Only log when there are sandboxes to avoid spamming logs
    if sandboxes.len() > 0 {
        debug!("Admin: Found {} sandboxes", sandboxes.len());
        for sandbox in &sandboxes {
            debug!("Admin: Sandbox ID: {}, Status: {:?}", sandbox.id, sandbox.status);
        }
    } else {
        debug!("Admin: No active sandboxes found");
    }
    
    let mut sandbox_infos = Vec::new();
    
    for sandbox in sandboxes {
        let info = SandboxInfo {
            id: sandbox.id.clone(),
            status: format!("{:?}", sandbox.status),
            runtime: sandbox.request.runtime.clone(),
            created_at: sandbox.created_at.to_rfc3339(),
            uptime: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_secs() - sandbox.created_at.timestamp() as u64)
                .unwrap_or(0),
            memory_mb: sandbox.request.memory_limit_mb,
            cpu_percentage: get_sandbox_cpu_usage(&sandbox.id).await.unwrap_or(0.0),
            dev_server_url: if sandbox.request.dev_server.unwrap_or(false) && matches!(sandbox.request.mode, Some(SandboxMode::Persistent)) {
                Some(format!("http://127.0.0.1:8070/proxy/{}/", sandbox.id))
            } else {
                None
            },
            allocated_port: sandbox.dev_server_port,
            is_persistent: matches!(sandbox.request.mode, Some(SandboxMode::Persistent)),
            container_id: sandbox.container_id.clone(),
        };
        sandbox_infos.push(info);
    }
    
    Ok(Json(sandbox_infos))
}

pub async fn get_sandbox_info(
    Path(sandbox_id): Path<String>,
    State(app_state): State<Arc<RwLock<SandboxManager>>>,
) -> Result<Json<SandboxInfo>, StatusCode> {
    let manager = app_state.read().await;
    let sandboxes = manager.get_all_sandboxes().await;
    
    let sandbox = sandboxes
        .into_iter()
        .find(|s| s.id == sandbox_id)
        .ok_or(StatusCode::NOT_FOUND)?;
    
    let info = SandboxInfo {
        id: sandbox.id.clone(),
        status: format!("{:?}", sandbox.status),
        runtime: sandbox.request.runtime.clone(),
        created_at: sandbox.created_at.to_rfc3339(),
        uptime: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs() - sandbox.created_at.timestamp() as u64)
            .unwrap_or(0),
        memory_mb: sandbox.request.memory_limit_mb,
        cpu_percentage: get_sandbox_cpu_usage(&sandbox.id).await.unwrap_or(0.0),
        dev_server_url: if sandbox.request.dev_server.unwrap_or(false) && matches!(sandbox.request.mode, Some(SandboxMode::Persistent)) {
            Some(format!("http://127.0.0.1:8070/proxy/{}/", sandbox.id))
        } else {
            None
        },
        allocated_port: sandbox.dev_server_port,
        is_persistent: matches!(sandbox.request.mode, Some(SandboxMode::Persistent)),
        container_id: sandbox.container_id.clone(),
    };
    
    Ok(Json(info))
}

pub async fn get_sandbox_logs(
    Path(sandbox_id): Path<String>,
    Query(query): Query<LogQuery>,
    State(app_state): State<Arc<RwLock<SandboxManager>>>,
) -> Result<Json<Vec<LogEntry>>, StatusCode> {
    let manager = app_state.read().await;
    let sandboxes = manager.get_all_sandboxes().await;
    
    let _sandbox = sandboxes
        .into_iter()
        .find(|s| s.id == sandbox_id)
        .ok_or(StatusCode::NOT_FOUND)?;
    
    // Get actual container logs
    let logs = match get_container_logs(&sandbox_id, query.lines.unwrap_or(100)).await {
        Ok(logs) => logs,
        Err(e) => {
            error!("Failed to get logs for sandbox {}: {}", sandbox_id, e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };
    
    Ok(Json(logs))
}

pub async fn force_stop_sandbox(
    Path(sandbox_id): Path<String>,
    State(app_state): State<Arc<RwLock<SandboxManager>>>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let mut manager = app_state.write().await;
    
    match manager.delete_sandbox(&sandbox_id).await {
        Ok(_) => {
            info!("Force stopped sandbox: {}", sandbox_id);
            Ok(Json(json!({
                "success": true,
                "message": format!("Sandbox {} stopped successfully", sandbox_id)
            })))
        }
        Err(e) => {
            error!("Failed to force stop sandbox {}: {}", sandbox_id, e);
            Ok(Json(json!({
                "success": false,
                "message": format!("Failed to stop sandbox: {}", e)
            })))
        }
    }
}

pub async fn get_sandbox_resources(
    Path(sandbox_id): Path<String>,
    State(app_state): State<Arc<RwLock<SandboxManager>>>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let manager = app_state.read().await;
    let sandboxes = manager.get_all_sandboxes().await;
    
    let _sandbox = sandboxes
        .into_iter()
        .find(|s| s.id == sandbox_id)
        .ok_or(StatusCode::NOT_FOUND)?;
    
    // Get actual container stats
    let resources = match get_container_stats(&sandbox_id).await {
        Ok(stats) => stats,
        Err(e) => {
            error!("Failed to get container stats for {}: {}", sandbox_id, e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };
    
    Ok(Json(resources))
}

pub async fn get_system_logs(
    Query(query): Query<LogQuery>,
) -> Result<Json<Vec<LogEntry>>, StatusCode> {
    // Get actual system logs
    let logs = match get_system_logs_impl(query.lines.unwrap_or(100)).await {
        Ok(logs) => logs,
        Err(e) => {
            error!("Failed to get system logs: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };
    
    Ok(Json(logs))
}

pub async fn get_api_docs() -> Result<Json<Vec<ApiEndpoint>>, StatusCode> {
    let endpoints = vec![
        ApiEndpoint {
            method: "POST".to_string(),
            path: "/sandbox".to_string(),
            description: "Create a new sandbox".to_string(),
            parameters: vec![
                ApiParameter {
                    name: "runtime".to_string(),
                    param_type: "string".to_string(),
                    required: true,
                    description: "Runtime environment (bun, node, typescript)".to_string(),
                },
                ApiParameter {
                    name: "mode".to_string(),
                    param_type: "string".to_string(),
                    required: false,
                    description: "Execution mode (oneshot, persistent)".to_string(),
                },
                ApiParameter {
                    name: "dev_server".to_string(),
                    param_type: "boolean".to_string(),
                    required: false,
                    description: "Enable development server".to_string(),
                },
            ],
            example_request: Some(r#"{
  "runtime": "bun",
  "code": "console.log('Hello World');",
  "mode": "persistent",
  "dev_server": true,
  "timeout_ms": 30000,
  "memory_limit_mb": 256
}"#.to_string()),
            example_response: Some(r#"{
  "id": "abc123",
  "status": "Created",
  "runtime": "bun",
  "created_at": "2025-07-09T11:45:00Z"
}"#.to_string()),
        },
        ApiEndpoint {
            method: "GET".to_string(),
            path: "/sandbox/{id}".to_string(),
            description: "Get sandbox information".to_string(),
            parameters: vec![
                ApiParameter {
                    name: "id".to_string(),
                    param_type: "string".to_string(),
                    required: true,
                    description: "Sandbox ID".to_string(),
                },
            ],
            example_request: None,
            example_response: Some(r#"{
  "id": "abc123",
  "status": "Running",
  "runtime": "bun",
  "created_at": "2025-07-09T11:45:00Z",
  "timeout_ms": 30000,
  "memory_limit_mb": 256
}"#.to_string()),
        },
        ApiEndpoint {
            method: "POST".to_string(),
            path: "/sandbox/{id}/execute".to_string(),
            description: "Execute code in sandbox".to_string(),
            parameters: vec![
                ApiParameter {
                    name: "id".to_string(),
                    param_type: "string".to_string(),
                    required: true,
                    description: "Sandbox ID".to_string(),
                },
            ],
            example_request: Some(r#"{
  "runtime": "bun",
  "code": "console.log('Hello from sandbox');",
  "timeout_ms": 30000
}"#.to_string()),
            example_response: Some(r#"{
  "sandbox_id": "abc123",
  "success": true,
  "stdout": "Hello from sandbox\n",
  "stderr": "",
  "exit_code": 0,
  "execution_time_ms": 125
}"#.to_string()),
        },
        ApiEndpoint {
            method: "DELETE".to_string(),
            path: "/sandbox/{id}".to_string(),
            description: "Delete sandbox".to_string(),
            parameters: vec![
                ApiParameter {
                    name: "id".to_string(),
                    param_type: "string".to_string(),
                    required: true,
                    description: "Sandbox ID".to_string(),
                },
            ],
            example_request: None,
            example_response: Some(r#"{
  "success": true,
  "message": "Sandbox deleted successfully"
}"#.to_string()),
        },
    ];
    
    Ok(Json(endpoints))
}

pub async fn test_api_endpoint(
    Json(request): Json<ApiTestRequest>,
) -> Result<Json<ApiTestResponse>, StatusCode> {
    let start_time = std::time::Instant::now();
    
    // Make actual HTTP request to the API
    let response = match make_api_request(request).await {
        Ok(response) => response,
        Err(e) => {
            error!("Failed to make API request: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };
    
    let response = ApiTestResponse {
        status: response.status,
        headers: response.headers,
        body: response.body,
        duration_ms: start_time.elapsed().as_millis() as u64,
    };
    
    Ok(Json(response))
}