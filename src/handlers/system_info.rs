use serde::{Deserialize, Serialize};
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::thread;
use std::time::Instant;
use sysinfo::System;

// 全局刷新标志，用于通知UI更新
static NEEDS_UI_REFRESH: AtomicBool = AtomicBool::new(false);

// 基础系统信息 - 同步获取
#[derive(Debug, Clone)]
pub struct BasicSystemInfo {
    pub uptime: String,
    pub hostname: String,
    pub cpu_model: String,
    pub cpu_arch: String,
    pub cpu_cores: usize,
    pub cpu_frequency: String,
    pub cpu_usage: f32,
    pub load_avg: String,
    pub memory_total: u64,
    pub memory_used: u64,
    pub swap_total: u64,
    pub swap_used: u64,
    pub disk_info: Vec<DiskInfo>,
    pub network_stats: NetworkStats,
    pub network_algorithm: String,
    pub dns_servers: Vec<String>,
    pub system_time: String,
    pub kernel: String,
    pub distro: String,
    pub vm_type: String,
}

// 网络流量统计
#[derive(Debug, Clone)]
pub struct NetworkStats {
    pub rx_bytes: u64,
    pub tx_bytes: u64,
    pub rx_packets: u64,
    pub tx_packets: u64,
    pub interface_name: String,
}

impl Default for NetworkStats {
    fn default() -> Self {
        Self {
            rx_bytes: 0,
            tx_bytes: 0,
            rx_packets: 0,
            tx_packets: 0,
            interface_name: "Unknown".to_string(),
        }
    }
}

// 磁盘信息
#[derive(Debug, Clone)]
pub struct DiskInfo {
    pub name: String,
    pub mount_point: String,
    pub total_space: u64,
    pub available_space: u64,
    pub file_system: String,
}

// 网络信息 - 异步获取
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkInfo {
    pub ipv4: Option<String>,
    pub ipv6: Option<String>,
    pub isp: Option<String>,
    pub asn: Option<String>,
    pub location: Option<String>,
    pub country: Option<String>,
    pub hostname: Option<String>,
}

impl Default for NetworkInfo {
    fn default() -> Self {
        Self {
            ipv4: None,
            ipv6: None,
            isp: None,
            asn: None,
            location: None,
            country: None,
            hostname: None,
        }
    }
}

// IP 信息 API 响应
#[derive(Debug, Deserialize)]
struct IpInfo {
    ip: String,
    city: Option<String>,
    region: Option<String>,
    country: Option<String>,
    org: Option<String>,
    hostname: Option<String>,
}

// 全局系统信息状态
static SYSTEM_INFO: Mutex<Option<SystemInfo>> = Mutex::new(None);
static NETWORK_FETCH_STARTED: AtomicBool = AtomicBool::new(false);

// 完整系统信息
#[derive(Debug, Clone)]
pub struct SystemInfo {
    pub basic: BasicSystemInfo,
    pub network: NetworkInfo,
    pub last_update: Instant,
    pub network_loading: bool,
}

impl SystemInfo {
    // 创建新的系统信息实例，启动异步网络信息获取
    pub fn new() -> Self {
        let basic = Self::get_basic_info();
        let info = SystemInfo {
            basic,
            network: NetworkInfo::default(),
            last_update: Instant::now(),
            network_loading: true,
        };

        // 启动异步线程获取网络信息
        Self::start_network_info_fetch();
        info
    }

    // 获取基础系统信息（同步）
    fn get_basic_info() -> BasicSystemInfo {
        let mut sys = System::new_all();
        sys.refresh_all();

        // 主机名
        let hostname = System::host_name().unwrap_or_else(|| "Unknown".to_string());

        // CPU 信息
        let cpu_model = sys.cpus().first()
            .map(|cpu| cpu.brand().to_string())
            .unwrap_or_else(|| "Unknown".to_string());
        let cpu_cores = sys.cpus().len();
        let cpu_usage = sys.global_cpu_usage();
        let cpu_arch = Self::get_cpu_arch();
        let cpu_frequency = Self::get_cpu_frequency();

        // 负载信息
        let load_avg = Self::get_load_average();

        // 内存信息
        let memory_total = sys.total_memory();
        let memory_used = sys.used_memory();
        let swap_total = sys.total_swap();
        let swap_used = sys.used_swap();

        // 磁盘信息
        let disk_info = Self::get_disk_info();

        // 网络统计
        let network_stats = Self::get_network_stats();
        let network_algorithm = Self::get_network_algorithm();

        // DNS 服务器
        let dns_servers = Self::get_dns_servers();

        // 系统时间
        let system_time = Self::get_system_time();

        // 系统信息
        let kernel = System::kernel_version().unwrap_or_else(|| "Unknown".to_string());
        let distro = Self::get_distro();
        let vm_type = Self::get_vm_type();
        let uptime = Self::format_uptime(System::uptime());

        BasicSystemInfo {
            uptime,
            hostname,
            cpu_model,
            cpu_arch,
            cpu_cores,
            cpu_frequency,
            cpu_usage,
            load_avg,
            memory_total,
            memory_used,
            swap_total,
            swap_used,
            disk_info,
            network_stats,
            network_algorithm,
            dns_servers,
            system_time,
            kernel,
            distro,
            vm_type,
        }
    }

    // 获取发行版信息
    fn get_distro() -> String {
        if let Ok(output) = Command::new("lsb_release").arg("-d").arg("-s").output() {
            if output.status.success() {
                return String::from_utf8_lossy(&output.stdout)
                    .trim()
                    .trim_matches('"')
                    .to_string();
            }
        }

        // 备用方法：读取 /etc/os-release
        if let Ok(content) = std::fs::read_to_string("/etc/os-release") {
            for line in content.lines() {
                if line.starts_with("PRETTY_NAME=") {
                    return line.split('=').nth(1)
                        .unwrap_or("Unknown")
                        .trim_matches('"')
                        .to_string();
                }
            }
        }

        "Unknown".to_string()
    }

    // 检测虚拟化类型
    fn get_vm_type() -> String {
        // 检查常见的虚拟化指标
        let checks = vec![
            ("systemd-detect-virt", vec![]),
            ("dmidecode", vec!["-s", "system-product-name"]),
            ("lscpu", vec![]),
        ];

        for (cmd, args) in checks {
            if let Ok(output) = Command::new(cmd).args(&args).output() {
                if output.status.success() {
                    let result = String::from_utf8_lossy(&output.stdout).to_lowercase();
                    
                    if cmd == "systemd-detect-virt" && !result.trim().is_empty() && result.trim() != "none" {
                        return result.trim().to_string();
                    }
                    
                    if cmd == "dmidecode" {
                        if result.contains("kvm") { return "KVM".to_string(); }
                        if result.contains("vmware") { return "VMware".to_string(); }
                        if result.contains("virtualbox") { return "VirtualBox".to_string(); }
                        if result.contains("xen") { return "Xen".to_string(); }
                    }
                    
                    if cmd == "lscpu" {
                        if result.contains("hypervisor") { return "Virtual Machine".to_string(); }
                    }
                }
            }
        }

        // 检查 /proc/cpuinfo
        if let Ok(content) = std::fs::read_to_string("/proc/cpuinfo") {
            let content = content.to_lowercase();
            if content.contains("hypervisor") {
                return "Virtual Machine".to_string();
            }
        }

        "Physical".to_string()
    }

    // 格式化运行时间
    fn format_uptime(uptime_seconds: u64) -> String {
        let days = uptime_seconds / 86400;
        let hours = (uptime_seconds % 86400) / 3600;
        let minutes = (uptime_seconds % 3600) / 60;
        
        if days > 0 {
            format!("{}d {}h {}m", days, hours, minutes)
        } else if hours > 0 {
            format!("{}h {}m", hours, minutes)
        } else {
            format!("{}m", minutes)
        }
    }

    // 启动异步网络信息获取
    fn start_network_info_fetch() {
        // 检查是否已经启动过网络获取
        if NETWORK_FETCH_STARTED.compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst).is_ok() {
            thread::spawn(|| {
                let network_info = Self::fetch_network_info();
                
                // 更新全局状态
                if let Ok(mut global_info) = SYSTEM_INFO.lock() {
                    if let Some(ref mut info) = global_info.as_mut() {
                        info.network = network_info;
                        info.network_loading = false;
                        info.last_update = Instant::now();
                        
                        // 设置需要刷新UI的标志
                        NEEDS_UI_REFRESH.store(true, Ordering::Relaxed);
                    }
                }
            });
        }
    }

    // 获取网络信息
    fn fetch_network_info() -> NetworkInfo {
        let mut network_info = NetworkInfo::default();

        // 获取 IPv4 信息
        if let Some(ipv4_info) = Self::get_ip_info("https://ipinfo.io/json", false) {
            network_info.ipv4 = Some(ipv4_info.ip.clone());
            network_info.isp = ipv4_info.org.clone();
            network_info.hostname = ipv4_info.hostname.clone();
            
            if let (Some(city), Some(region), Some(country)) = 
                (&ipv4_info.city, &ipv4_info.region, &ipv4_info.country) {
                network_info.location = Some(format!("{}, {}", city, region));
                network_info.country = Some(country.clone());
            }
        }

        // 获取 IPv6 信息
        if let Some(ipv6_info) = Self::get_ip_info("https://ipv6.ipinfo.io/json", true) {
            network_info.ipv6 = Some(ipv6_info.ip);
        }

        // 如果没有获取到 ISP 信息，尝试从 ASN 获取
        if network_info.isp.is_none() {
            if let Some(asn_info) = Self::get_asn_info() {
                network_info.asn = Some(asn_info);
            }
        }

        network_info
    }

    // 获取 IP 信息
    fn get_ip_info(url: &str, is_ipv6: bool) -> Option<IpInfo> {
        let mut cmd = Command::new("curl");
        cmd.arg("-s")
           .arg("--max-time").arg("10")
           .arg("--connect-timeout").arg("5");
        
        if is_ipv6 {
            cmd.arg("-6");
        } else {
            cmd.arg("-4");
        }
        
        cmd.arg(url);

        if let Ok(output) = cmd.output() {
            if output.status.success() {
                let json_str = String::from_utf8_lossy(&output.stdout);
                if let Ok(ip_info) = serde_json::from_str::<IpInfo>(&json_str) {
                    return Some(ip_info);
                }
            }
        }
        None
    }

    // 获取 ASN 信息
    fn get_asn_info() -> Option<String> {
        if let Ok(output) = Command::new("curl")
            .arg("-s")
            .arg("--max-time").arg("5")
            .arg("https://ipinfo.io/org")
            .output() {
            if output.status.success() {
                let org = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if !org.is_empty() {
                    return Some(org);
                }
            }
        }
        None
    }

    // 获取当前系统信息（带缓存和异步更新）
    pub fn get_current() -> SystemInfo {
        if let Ok(mut global_info) = SYSTEM_INFO.lock() {
            if let Some(ref mut info) = global_info.as_mut() {
                // 刷新基础系统信息（CPU、内存等）
                info.basic = Self::get_basic_info();
                info.last_update = Instant::now();
                return info.clone();
            }

            // 首次创建系统信息
            let new_info = SystemInfo::new();
            *global_info = Some(new_info.clone());
            new_info
        } else {
            // 如果锁获取失败，直接创建新实例
            SystemInfo::new()
        }
    }

    // 获取 CPU 架构
    fn get_cpu_arch() -> String {
        if let Ok(output) = Command::new("uname").arg("-m").output() {
            if output.status.success() {
                return String::from_utf8_lossy(&output.stdout).trim().to_string();
            }
        }
        "Unknown".to_string()
    }

    // 获取 CPU 频率
    fn get_cpu_frequency() -> String {
        // 尝试从 /proc/cpuinfo 获取
        if let Ok(content) = std::fs::read_to_string("/proc/cpuinfo") {
            for line in content.lines() {
                if line.starts_with("cpu MHz") {
                    if let Some(freq_str) = line.split(':').nth(1) {
                        if let Ok(freq) = freq_str.trim().parse::<f64>() {
                            return format!("{:.0} MHz", freq);
                        }
                    }
                }
            }
        }

        // 尝试从 lscpu 获取
        if let Ok(output) = Command::new("lscpu").output() {
            if output.status.success() {
                let result = String::from_utf8_lossy(&output.stdout);
                for line in result.lines() {
                    if line.contains("CPU max MHz") || line.contains("CPU MHz") {
                        if let Some(freq_str) = line.split(':').nth(1) {
                            if let Ok(freq) = freq_str.trim().parse::<f64>() {
                                return format!("{:.0} MHz", freq);
                            }
                        }
                    }
                }
            }
        }

        "Unknown".to_string()
    }

    // 获取负载平均值
    fn get_load_average() -> String {
        if let Ok(content) = std::fs::read_to_string("/proc/loadavg") {
            let parts: Vec<&str> = content.trim().split_whitespace().collect();
            if parts.len() >= 3 {
                return format!("{} {} {}", parts[0], parts[1], parts[2]);
            }
        }
        "Unknown".to_string()
    }

    // 获取磁盘信息
    fn get_disk_info() -> Vec<DiskInfo> {
        let mut disk_info = Vec::new();

        // 使用 df 命令获取磁盘信息
        if let Ok(output) = Command::new("df").arg("-h").arg("-T").output() {
            if output.status.success() {
                let result = String::from_utf8_lossy(&output.stdout);
                for line in result.lines().skip(1) {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 7 && !parts[0].starts_with("tmpfs") && !parts[0].starts_with("devtmpfs") {
                        let name = parts[0].to_string();
                        let file_system = parts[1].to_string();
                        let mount_point = parts[6].to_string();
                        
                        // 跳过虚拟文件系统
                        if mount_point.starts_with("/proc") || mount_point.starts_with("/sys") || 
                           mount_point.starts_with("/dev") || mount_point.starts_with("/run") {
                            continue;
                        }

                        // 解析大小信息（转换为字节）
                        let total_space = Self::parse_size_to_bytes(parts[2]);
                        let used_space = Self::parse_size_to_bytes(parts[3]);
                        let available_space = total_space - used_space;

                        disk_info.push(DiskInfo {
                            name,
                            mount_point,
                            total_space,
                            available_space,
                            file_system,
                        });
                    }
                }
            }
        }

        disk_info
    }

    // 解析大小字符串为字节数
    fn parse_size_to_bytes(size_str: &str) -> u64 {
        if let Some(last_char) = size_str.chars().last() {
            let number_part = &size_str[..size_str.len()-1];
            if let Ok(number) = number_part.parse::<f64>() {
                return match last_char.to_ascii_uppercase() {
                    'K' => (number * 1024.0) as u64,
                    'M' => (number * 1024.0 * 1024.0) as u64,
                    'G' => (number * 1024.0 * 1024.0 * 1024.0) as u64,
                    'T' => (number * 1024.0 * 1024.0 * 1024.0 * 1024.0) as u64,
                    _ => number as u64,
                };
            }
        }
        0
    }

    // 获取网络统计信息
    fn get_network_stats() -> NetworkStats {
        // 尝试获取默认网络接口
        let interface_name = Self::get_default_interface();
        
        // 读取 /proc/net/dev 获取网络统计
        if let Ok(content) = std::fs::read_to_string("/proc/net/dev") {
            for line in content.lines().skip(2) {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 17 {
                    let iface = parts[0].trim_end_matches(':');
                    if iface == interface_name || (interface_name == "Unknown" && !iface.starts_with("lo")) {
                        return NetworkStats {
                            interface_name: iface.to_string(),
                            rx_bytes: parts[1].parse().unwrap_or(0),
                            rx_packets: parts[2].parse().unwrap_or(0),
                            tx_bytes: parts[9].parse().unwrap_or(0),
                            tx_packets: parts[10].parse().unwrap_or(0),
                        };
                    }
                }
            }
        }

        NetworkStats::default()
    }

    // 获取默认网络接口
    fn get_default_interface() -> String {
        if let Ok(output) = Command::new("ip").arg("route").arg("show").arg("default").output() {
            if output.status.success() {
                let result = String::from_utf8_lossy(&output.stdout);
                for line in result.lines() {
                    if line.contains("default via") {
                        let parts: Vec<&str> = line.split_whitespace().collect();
                        for (i, part) in parts.iter().enumerate() {
                            if part == &"dev" && i + 1 < parts.len() {
                                return parts[i + 1].to_string();
                            }
                        }
                    }
                }
            }
        }
        "Unknown".to_string()
    }

    // 获取网络算法（TCP 拥塞控制）
    fn get_network_algorithm() -> String {
        if let Ok(content) = std::fs::read_to_string("/proc/sys/net/ipv4/tcp_congestion_control") {
            return content.trim().to_string();
        }
        "Unknown".to_string()
    }

    // 获取 DNS 服务器
    fn get_dns_servers() -> Vec<String> {
        let mut dns_servers = Vec::new();

        // 读取 /etc/resolv.conf
        if let Ok(content) = std::fs::read_to_string("/etc/resolv.conf") {
            for line in content.lines() {
                if line.starts_with("nameserver") {
                    if let Some(ip) = line.split_whitespace().nth(1) {
                        dns_servers.push(ip.to_string());
                    }
                }
            }
        }

        if dns_servers.is_empty() {
            dns_servers.push("Unknown".to_string());
        }

        dns_servers
    }

    // 获取系统时间
    fn get_system_time() -> String {
        use std::time::{SystemTime, UNIX_EPOCH};
        
        if let Ok(duration) = SystemTime::now().duration_since(UNIX_EPOCH) {
            let secs = duration.as_secs();
            let datetime = chrono::DateTime::from_timestamp(secs as i64, 0);
            if let Some(dt) = datetime {
                return dt.format("%Y-%m-%d %H:%M:%S UTC").to_string();
            }
        }
        
        "Unknown".to_string()
    }
}

// 格式化字节大小为 GiB
fn format_bytes_gib(bytes: u64) -> String {
    const GIB: u64 = 1024 * 1024 * 1024;
    format!("{:.1} GiB", bytes as f64 / GIB as f64)
}

// 格式化字节大小为 GB
fn format_bytes_gb(bytes: u64) -> String {
    const GB: u64 = 1000 * 1000 * 1000;
    format!("{:.1} GB", bytes as f64 / GB as f64)
}

// 格式化系统信息为可读字符串
pub fn format_system_info(info: &SystemInfo) -> String {
    let mut output = String::new();
    
    // 系统基础信息
    output.push_str(&format!("uptime: {}\n", info.basic.uptime));
    output.push_str(&format!("CPU: {} ({} cores)\n", info.basic.cpu_model, info.basic.cpu_cores));
    output.push_str(&format!("CPU Usage: {:.1}%\n", info.basic.cpu_usage));
    output.push_str(&format!("Memory: {} / {} ({:.1}%)\n", 
        format_bytes_gib(info.basic.memory_used),
        format_bytes_gib(info.basic.memory_total),
        (info.basic.memory_used as f64 / info.basic.memory_total as f64) * 100.0
    ));
    
    // 磁盘信息
    if !info.basic.disk_info.is_empty() {
        let total_disk: u64 = info.basic.disk_info.iter().map(|d| d.total_space).sum();
        let used_disk: u64 = info.basic.disk_info.iter().map(|d| d.total_space - d.available_space).sum();
        
        output.push_str(&format!("Disk: {} / {} ({:.1}%)\n",
            format_bytes_gib(used_disk),
            format_bytes_gib(total_disk),
            if total_disk > 0 { (used_disk as f64 / total_disk as f64) * 100.0 } else { 0.0 }
        ));
    }
    
    output.push_str(&format!("Distro: {}\n", info.basic.distro));
    output.push_str(&format!("Kernel: {}\n", info.basic.kernel));
    output.push_str(&format!("VM Type: {}\n", info.basic.vm_type));
    
    // 网络信息
    output.push_str("\n--- 网络信息 ---\n");
    
    if info.network_loading {
        output.push_str("IPv4: Loading...\n");
        output.push_str("IPv6: Loading...\n");
        output.push_str("ISP: Loading...\n");
        output.push_str("ASN: Loading...\n");
        output.push_str("Host: Loading...\n");
        output.push_str("Location: Loading...\n");
        output.push_str("Country: Loading...\n");
    } else {
        if let Some(ref ipv4) = info.network.ipv4 {
            output.push_str(&format!("IPv4: {}\n", ipv4));
        }
        
        if let Some(ref ipv6) = info.network.ipv6 {
            output.push_str(&format!("IPv6: {}\n", ipv6));
        }
        
        if let Some(ref isp) = info.network.isp {
            output.push_str(&format!("ISP: {}\n", isp));
        }
        
        if let Some(ref asn) = info.network.asn {
            output.push_str(&format!("ASN: {}\n", asn));
        }
        
        if let Some(ref hostname) = info.network.hostname {
            output.push_str(&format!("Host: {}\n", hostname));
        }
        
        if let Some(ref location) = info.network.location {
            output.push_str(&format!("Location: {}\n", location));
        }
        
        if let Some(ref country) = info.network.country {
            output.push_str(&format!("Country: {}\n", country));
        }
    }
    
    output
}

// 主要接口：获取系统信息字符串
pub fn get_info() -> String {
    let system_info = SystemInfo::get_current();
    format_system_info(&system_info)
}

// 检查是否需要刷新UI
pub fn check_needs_refresh() -> bool {
    NEEDS_UI_REFRESH.swap(false, std::sync::atomic::Ordering::Relaxed)
}
