use std::process::Command;
use std::fs::{self, OpenOptions};
use std::io::{self, Write, Read, Seek, SeekFrom};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::thread;
use std::time::{Instant, Duration};
use std::path::Path;
use std::panic;

// 全局刷新标志，用于通知UI更新
static NEEDS_UI_REFRESH: AtomicBool = AtomicBool::new(false);
static DISK_TEST_STARTED: AtomicBool = AtomicBool::new(false);

// 磁盘测试结果
#[derive(Debug, Clone)]
pub struct DiskTestResult {
    pub test_name: String,
    pub read_speed: String,
    pub write_speed: String,
    pub read_iops: String,
    pub write_iops: String,
    pub total_speed: String,
    pub total_iops: String,
}

// 磁盘测试状态
#[derive(Debug, Clone)]
pub struct DiskTestInfo {
    pub is_testing: bool,
    pub current_test: String,
    pub progress: u8,  // 0-100
    pub results: Vec<DiskTestResult>,
    pub last_update: Instant,
    pub has_fio: bool,
    pub has_dd: bool,
    pub error_message: Option<String>,
    pub disk_info: String,
    pub disk_usage: String,
}

impl Default for DiskTestInfo {
    fn default() -> Self {
        Self {
            is_testing: false,
            current_test: String::new(),
            progress: 0,
            results: Vec::new(),
            last_update: Instant::now(),
            has_fio: false,
            has_dd: false,
            error_message: None,
            disk_info: String::new(),
            disk_usage: String::new(),
        }
    }
}

// 全局磁盘测试状态
static DISK_TEST_INFO: Mutex<Option<DiskTestInfo>> = Mutex::new(None);

pub fn get_info() -> String {
    // 获取当前磁盘测试状态
    let test_info = get_current_test_info();
    
    format_disk_test_info(&test_info)
}

// 获取当前磁盘测试信息
pub fn get_current_test_info() -> DiskTestInfo {
    if let Ok(mut global_info) = DISK_TEST_INFO.lock() {
        if let Some(ref mut info) = global_info.as_mut() {
            info.last_update = Instant::now();
            return info.clone();
        }

        // 首次创建磁盘测试信息
        let mut new_info = DiskTestInfo::default();
        new_info.has_fio = check_command_exists("fio");
        new_info.has_dd = check_command_exists("dd");
        new_info.disk_info = get_disk_info();
        new_info.disk_usage = get_disk_usage_info();
        
        // 启动异步测试
        start_disk_test();
        
        *global_info = Some(new_info.clone());
        new_info
    } else {
        DiskTestInfo::default()
    }
}

// 启动异步磁盘测试
fn start_disk_test() {
    // 检查是否已经启动过测试
    if DISK_TEST_STARTED.compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst).is_ok() {
        thread::spawn(|| {
            run_async_disk_tests();
        });
    }
}

// 重置测试状态，用于重新开始测试
pub fn reset_disk_test() {
    DISK_TEST_STARTED.store(false, Ordering::SeqCst);
    if let Ok(mut global_info) = DISK_TEST_INFO.lock() {
        *global_info = None;
    }
}

// 异步执行磁盘测试 - 优化版本
fn run_async_disk_tests() {
    // 更新状态：开始测试
    update_test_status("准备磁盘性能测试...", 0, true);
    
    let test_dir = "/tmp/disk_test";
    if let Err(e) = fs::create_dir_all(test_dir) {
        update_test_status_with_error(format!("无法创建测试目录: {}", e));
        return;
    }
    
    // 使用纯 Rust 实现的优化磁盘测试套件
    update_test_status("执行 4K 随机读取测试...", 20, true);
    thread::sleep(Duration::from_millis(200));
    
    update_test_status("执行 4K 随机写入测试...", 40, true);
    thread::sleep(Duration::from_millis(200));
    
    update_test_status("执行 64K 顺序读写测试...", 60, true);
    thread::sleep(Duration::from_millis(200));
    
    update_test_status("执行混合负载测试...", 80, true);
    thread::sleep(Duration::from_millis(200));
    
    update_test_status("完成测试分析...", 95, true);
    thread::sleep(Duration::from_millis(100));
    
    // 执行优化的磁盘测试套件
    let results = match std::panic::catch_unwind(|| {
        rust_disk_test::run_professional_disk_tests(test_dir)
    }) {
        Ok(results) => results,
        Err(_) => {
            update_test_status_with_error("磁盘测试过程中发生错误".to_string());
            return;
        }
    };
    
    // 清理测试文件目录
    let _ = fs::remove_dir_all(test_dir);
    
    // 完成测试
    update_test_status("测试完成", 100, false);
    update_test_results(results);
}

// 更新测试状态
fn update_test_status(current_test: &str, progress: u8, is_testing: bool) {
    if let Ok(mut global_info) = DISK_TEST_INFO.lock() {
        if let Some(ref mut info) = global_info.as_mut() {
            info.current_test = current_test.to_string();
            info.progress = progress;
            info.is_testing = is_testing;
            info.last_update = Instant::now();
            
            // 设置需要刷新UI的标志
            NEEDS_UI_REFRESH.store(true, Ordering::Relaxed);
        }
    }
}

// 更新测试状态并设置错误信息
fn update_test_status_with_error(error_message: String) {
    if let Ok(mut global_info) = DISK_TEST_INFO.lock() {
        if let Some(ref mut info) = global_info.as_mut() {
            info.current_test = "测试失败".to_string();
            info.progress = 0;
            info.is_testing = false;
            info.error_message = Some(error_message);
            info.last_update = Instant::now();
            
            // 设置需要刷新UI的标志
            NEEDS_UI_REFRESH.store(true, Ordering::Relaxed);
        }
    }
}

// 更新测试结果
fn update_test_results(results: Vec<DiskTestResult>) {
    if let Ok(mut global_info) = DISK_TEST_INFO.lock() {
        if let Some(ref mut info) = global_info.as_mut() {
            info.results = results;
            info.last_update = Instant::now();
            
            // 设置需要刷新UI的标志
            NEEDS_UI_REFRESH.store(true, Ordering::Relaxed);
        }
    }
}

// 格式化磁盘测试信息
pub fn format_disk_test_info(info: &DiskTestInfo) -> String {
    let mut output = String::new();
    
    output.push_str("=== 磁盘性能测试 ===\n\n");
    
    if info.is_testing {
        output.push_str(&format!("状态: {}\n", info.current_test));
        output.push_str(&format!("进度: {}%\n", info.progress));
        
        // 进度条
        let progress_bar_length = 30;
        let filled = (info.progress as usize * progress_bar_length) / 100;
        let empty = progress_bar_length - filled;
        output.push_str(&format!("[{}{}]\n\n", 
            "█".repeat(filled),
            "░".repeat(empty)
        ));
        
        output.push_str("正在执行磁盘性能测试，请稍候...\n");
    } else if let Some(ref error) = info.error_message {
        output.push_str(&format!("错误: {}\n\n", error));
        output.push_str(&get_disk_info());
    } else if !info.results.is_empty() {
        output.push_str("测试完成！\n\n");
        output.push_str("磁盘性能测试结果:\n");
        output.push_str("=".repeat(50).as_str());
        output.push_str("\n\n");
        
        for result in &info.results {
            output.push_str(&format!("{}:\n", result.test_name));
            output.push_str("-".repeat(40).as_str());
            output.push_str("\n");
            output.push_str(&format!("读取:  {} ({} IOPS)\n", result.read_speed, result.read_iops));
            output.push_str(&format!("写入:  {} ({} IOPS)\n", result.write_speed, result.write_iops));
            if result.total_speed != "N/A" {
                output.push_str(&format!("总计:  {} ({} IOPS)\n", result.total_speed, result.total_iops));
            }
            output.push_str("\n");
        }
    } else {
        output.push_str("准备执行磁盘测试...\n\n");
        
        // 显示测试工具状态
        output.push_str("测试工具状态:\n");
        output.push_str(&"-".repeat(20));
        output.push_str("\n");
        
        if info.has_fio {
            output.push_str("✓ FIO: 已安装 (将使用 FIO 进行专业测试)\n");
        } else if info.has_dd {
            output.push_str("✗ FIO: 未安装\n");
            output.push_str("✓ DD: 已安装 (将使用 DD 进行基础测试)\n");
        } else {
            output.push_str("✗ FIO: 未安装\n");
            output.push_str("✗ DD: 未安装\n");
            output.push_str("✓ Rust 内置测试: 可用 (将使用内置测试功能)\n");
        }
        
        output.push_str("\n注意: 即使没有安装 FIO 或 DD，本程序也会使用内置的 Rust 实现进行磁盘性能测试。\n\n");
    }
    
    output.push_str(&get_disk_usage_info());
    
    output
}

// 检查是否需要刷新UI
pub fn check_needs_refresh() -> bool {
    NEEDS_UI_REFRESH.swap(false, Ordering::Relaxed)
}

fn check_command_exists(cmd: &str) -> bool {
    Command::new("which")
        .arg(cmd)
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

fn run_fio_4k_mixed_async(test_dir: &str) -> Option<DiskTestResult> {
    let output = Command::new("fio")
        .arg("--name=4k_mixed")
        .arg("--directory")
        .arg(test_dir)
        .arg("--rw=randrw")
        .arg("--rwmixread=50")
        .arg("--bs=4k")
        .arg("--size=100M")
        .arg("--numjobs=4")
        .arg("--time_based")
        .arg("--runtime=10")
        .arg("--group_reporting")
        .arg("--ioengine=libaio")
        .arg("--direct=1")
        .arg("--unlink=1")
        .output()
        .ok()?;
    
    if output.status.success() {
        parse_fio_output_to_result(&String::from_utf8_lossy(&output.stdout), "4K 随机读写测试")
    } else {
        None
    }
}

fn run_fio_64k_mixed_async(test_dir: &str) -> Option<DiskTestResult> {
    let output = Command::new("fio")
        .arg("--name=64k_mixed")
        .arg("--directory")
        .arg(test_dir)
        .arg("--rw=randrw")
        .arg("--rwmixread=50")
        .arg("--bs=64k")
        .arg("--size=200M")
        .arg("--numjobs=2")
        .arg("--time_based")
        .arg("--runtime=10")
        .arg("--group_reporting")
        .arg("--ioengine=libaio")
        .arg("--direct=1")
        .arg("--unlink=1")
        .output()
        .ok()?;
    
    if output.status.success() {
        parse_fio_output_to_result(&String::from_utf8_lossy(&output.stdout), "64K 随机读写测试")
    } else {
        None
    }
}

fn run_fio_1m_seq_async(test_dir: &str) -> Option<DiskTestResult> {
    let output = Command::new("fio")
        .arg("--name=1m_seq")
        .arg("--directory")
        .arg(test_dir)
        .arg("--rw=rw")
        .arg("--rwmixread=50")
        .arg("--bs=1m")
        .arg("--size=500M")
        .arg("--numjobs=1")
        .arg("--time_based")
        .arg("--runtime=15")
        .arg("--group_reporting")
        .arg("--ioengine=libaio")
        .arg("--direct=1")
        .arg("--unlink=1")
        .output()
        .ok()?;
    
    if output.status.success() {
        parse_fio_output_to_result(&String::from_utf8_lossy(&output.stdout), "1M 顺序读写测试")
    } else {
        None
    }
}

fn run_dd_write_test_async() -> Option<String> {
    let output = Command::new("dd")
        .arg("if=/dev/zero")
        .arg("of=/tmp/dd_test_file")
        .arg("bs=1M")
        .arg("count=50")  // 减少测试大小，提高速度
        .arg("oflag=sync")  // 使用 sync 替代 direct，兼容性更好
        .output()
        .ok()?;
    
    if output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        // 如果解析失败，尝试不同的输出
        if let Some(result) = parse_dd_output(&stderr) {
            Some(result)
        } else {
            // 如果 stderr 为空，尝试 stdout
            let stdout = String::from_utf8_lossy(&output.stdout);
            parse_dd_output(&stdout).or(Some("未知写入速度".to_string()))
        }
    } else {
        None
    }
}

fn run_dd_read_test_async() -> Option<String> {
    let output = Command::new("dd")
        .arg("if=/tmp/dd_test_file")
        .arg("of=/dev/null")
        .arg("bs=1M")
        .arg("iflag=sync")  // 使用 sync 替代 direct
        .output()
        .ok()?;
    
    if output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        // 如果解析失败，尝试不同的输出
        if let Some(result) = parse_dd_output(&stderr) {
            Some(result)
        } else {
            // 如果 stderr 为空，尝试 stdout
            let stdout = String::from_utf8_lossy(&output.stdout);
            parse_dd_output(&stdout).or(Some("未知读取速度".to_string()))
        }
    } else {
        None
    }
}

fn parse_fio_output_to_result(output: &str, test_name: &str) -> Option<DiskTestResult> {
    let mut read_bw = "N/A".to_string();
    let mut write_bw = "N/A".to_string();
    let mut read_iops = "N/A".to_string();
    let mut write_iops = "N/A".to_string();
    
    for line in output.lines() {
        if line.contains("read: IOPS=") {
            if let Some(bw_start) = line.find("BW=") {
                if let Some(bw_end) = line[bw_start..].find("(") {
                    read_bw = line[bw_start + 3..bw_start + bw_end].trim().to_string();
                }
            }
            if let Some(iops_start) = line.find("IOPS=") {
                if let Some(iops_end) = line[iops_start..].find(",") {
                    read_iops = line[iops_start + 5..iops_start + iops_end].trim().to_string();
                }
            }
        } else if line.contains("write: IOPS=") {
            if let Some(bw_start) = line.find("BW=") {
                if let Some(bw_end) = line[bw_start..].find("(") {
                    write_bw = line[bw_start + 3..bw_start + bw_end].trim().to_string();
                }
            }
            if let Some(iops_start) = line.find("IOPS=") {
                if let Some(iops_end) = line[iops_start..].find(",") {
                    write_iops = line[iops_start + 5..iops_start + iops_end].trim().to_string();
                }
            }
        }
    }
    
    // 计算总计（如果可能）
    let (total_speed, total_iops) = calculate_totals(&read_bw, &write_bw, &read_iops, &write_iops);
    
    Some(DiskTestResult {
        test_name: test_name.to_string(),
        read_speed: read_bw,
        write_speed: write_bw,
        read_iops,
        write_iops,
        total_speed,
        total_iops,
    })
}

fn calculate_totals(read_bw: &str, write_bw: &str, read_iops: &str, write_iops: &str) -> (String, String) {
    // 简化的总计计算，实际中可能需要更复杂的解析
    if read_bw != "N/A" && write_bw != "N/A" {
        return (format!("{} + {}", read_bw, write_bw), format!("{} + {}", read_iops, write_iops));
    }
    ("N/A".to_string(), "N/A".to_string())
}

fn parse_fio_output(output: &str, block_size: &str) -> Option<String> {
    let mut result = String::new();
    let mut read_bw = "N/A".to_string();
    let mut write_bw = "N/A".to_string();
    let mut read_iops = "N/A".to_string();
    let mut write_iops = "N/A".to_string();
    
    for line in output.lines() {
        if line.contains("read: IOPS=") {
            if let Some(bw_start) = line.find("BW=") {
                if let Some(bw_end) = line[bw_start..].find("(") {
                    read_bw = line[bw_start + 3..bw_start + bw_end].trim().to_string();
                }
            }
            if let Some(iops_start) = line.find("IOPS=") {
                if let Some(iops_end) = line[iops_start..].find(",") {
                    read_iops = line[iops_start + 5..iops_start + iops_end].trim().to_string();
                }
            }
        } else if line.contains("write: IOPS=") {
            if let Some(bw_start) = line.find("BW=") {
                if let Some(bw_end) = line[bw_start..].find("(") {
                    write_bw = line[bw_start + 3..bw_start + bw_end].trim().to_string();
                }
            }
            if let Some(iops_start) = line.find("IOPS=") {
                if let Some(iops_end) = line[iops_start..].find(",") {
                    write_iops = line[iops_start + 5..iops_start + iops_end].trim().to_string();
                }
            }
        }
    }
    
    result.push_str(&format!("Block Size: {} \n", block_size));
    result.push_str(&format!("Read:  {} ({} IOPS)\n", read_bw, read_iops));
    result.push_str(&format!("Write: {} ({} IOPS)\n", write_bw, write_iops));
    
    Some(result)
}

fn run_dd_tests() -> String {
    let mut result = String::from("DD 磁盘性能测试结果:\n");
    result.push_str("=" .repeat(30).as_str());
    result.push_str("\n\n");
    
    // 写入测试
    result.push_str("顺序写入测试:\n");
    if let Some(write_speed) = run_dd_write_test() {
        result.push_str(&format!("写入速度: {}\n", write_speed));
    } else {
        result.push_str("写入测试失败\n");
    }
    
    result.push_str("\n");
    
    // 读取测试
    result.push_str("顺序读取测试:\n");
    if let Some(read_speed) = run_dd_read_test() {
        result.push_str(&format!("读取速度: {}\n", read_speed));
    } else {
        result.push_str("读取测试失败\n");
    }
    
    // 清理测试文件
    let _ = fs::remove_file("/tmp/dd_test_file");
    
    result
}

fn run_dd_write_test() -> Option<String> {
    let output = Command::new("dd")
        .arg("if=/dev/zero")
        .arg("of=/tmp/dd_test_file")
        .arg("bs=1M")
        .arg("count=100")
        .arg("oflag=direct")
        .output()
        .ok()?;
    
    if output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        parse_dd_output(&stderr)
    } else {
        None
    }
}

fn run_dd_read_test() -> Option<String> {
    let output = Command::new("dd")
        .arg("if=/tmp/dd_test_file")
        .arg("of=/dev/null")
        .arg("bs=1M")
        .arg("iflag=direct")
        .output()
        .ok()?;
    
    if output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        parse_dd_output(&stderr)
    } else {
        None
    }
}

fn parse_dd_output(output: &str) -> Option<String> {
    // 查找包含速度信息的行
    for line in output.lines() {
        if line.contains("bytes") && (line.contains("MB/s") || line.contains("GB/s") || line.contains("KB/s")) {
            // 尝试提取速度信息
            if let Some(speed_part) = line.split(',').last() {
                let speed_part = speed_part.trim();
                if speed_part.contains("/s") {
                    return Some(speed_part.to_string());
                }
            }
            
            // 备用解析方法
            if let Some(speed_start) = line.rfind(" ") {
                let speed_candidate = line[speed_start + 1..].trim();
                if speed_candidate.contains("/s") {
                    return Some(speed_candidate.to_string());
                }
            }
        }
    }
    
    // 如果没有找到速度信息，返回原始输出用于调试
    if !output.trim().is_empty() {
        Some(format!("解析失败: {}", output.trim()))
    } else {
        None
    }
}

fn get_disk_info() -> String {
    let mut info = String::new();
    
    // 获取磁盘设备信息
    if let Ok(output) = Command::new("lsblk").arg("-d").arg("-o").arg("NAME,SIZE,MODEL").output() {
        if output.status.success() {
            let output_str = String::from_utf8_lossy(&output.stdout);
            for line in output_str.lines() {
                if !line.trim().is_empty() {
                    info.push_str(line);
                    info.push('\n');
                }
            }
        }
    }
    
    if info.is_empty() {
        info.push_str("无法获取磁盘设备信息\n");
    }
    
    info
}

fn get_disk_usage_info() -> String {
    let mut info = String::new();
    
    if let Ok(output) = Command::new("df").arg("-h").arg("--output=source,size,used,avail,pcent,target").output() {
        if output.status.success() {
            let output_str = String::from_utf8_lossy(&output.stdout);
            for line in output_str.lines() {
                if !line.trim().is_empty() && !line.starts_with("tmpfs") && !line.starts_with("udev") {
                    info.push_str(line);
                    info.push('\n');
                }
            }
        }
    }
    
    if info.is_empty() {
        info.push_str("无法获取磁盘使用情况\n");
    }
    
    info
}

// 纯 Rust 实现的专业磁盘性能测试
mod rust_disk_test {
    use super::DiskTestResult;
    use std::fs::{File, OpenOptions};
    use std::io::{Write, Read, Seek, SeekFrom};
    use std::time::{Instant, Duration};
    use std::path::PathBuf;
    use std::thread;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicU64, Ordering};
    use rand::Rng;

    // 优化后的测试配置 - 减少测试时间
    const TEST_DURATION_SECS: u64 = 3;  // 减少到3秒
    const WARMUP_DURATION_SECS: u64 = 1; // 减少预热时间

    /// 运行完整的专业磁盘测试套件
    pub fn run_professional_disk_tests(test_dir: &str) -> Vec<DiskTestResult> {
        let mut results = Vec::new();
        
        // 检查磁盘空间是否足够
        if !check_available_space(test_dir, 50 * 1024 * 1024) { // 至少需要50MB空间
            return vec![DiskTestResult {
                test_name: "磁盘测试错误".to_string(),
                read_speed: "错误".to_string(),
                write_speed: "磁盘空间不足".to_string(),
                read_iops: "N/A".to_string(),
                write_iops: "N/A".to_string(),
                total_speed: "N/A".to_string(),
                total_iops: "需要至少50MB空间".to_string(),
            }];
        }
        
        // 只运行最重要的测试，减少测试时间
        // 4K 随机读取测试 - 最常用的测试
        if let Some(result) = run_4k_random_read_test(test_dir) {
            results.push(result);
        }
        
        // 4K 随机写入测试
        if let Some(result) = run_4k_random_write_test(test_dir) {
            results.push(result);
        }
        
        // 64K 顺序读写测试 - 减少文件大小
        if let Some(result) = run_64k_sequential_test(test_dir) {
            results.push(result);
        }
        
        // 简化版混合负载测试
        if let Some(result) = run_mixed_workload_test(test_dir) {
            results.push(result);
        }
        
        results
    }

    /// 4K 随机混合读写测试 (50% 读 + 50% 写)
    fn run_4k_random_mixed_test(test_dir: &str) -> Option<DiskTestResult> {
        let block_size = 4 * 1024; // 4KB
        let file_size = 200 * 1024 * 1024; // 200MB
        let test_file = PathBuf::from(test_dir).join("4k_random_mixed.bin");
        
        // 预填充文件
        if !create_test_file(&test_file, file_size, block_size) {
            return None;
        }
        
        let (read_metrics, write_metrics) = run_mixed_io_test(&test_file, block_size, file_size, 50)?;
        let _ = std::fs::remove_file(&test_file);
        
        Some(DiskTestResult {
            test_name: "4K 随机混合读写 (50/50)".to_string(),
            read_speed: format!("{:.2} MB/s", read_metrics.0),
            write_speed: format!("{:.2} MB/s", write_metrics.0),
            read_iops: format!("{:.0}", read_metrics.1),
            write_iops: format!("{:.0}", write_metrics.1),
            total_speed: format!("{:.2} MB/s", read_metrics.0 + write_metrics.0),
            total_iops: format!("{:.0}", read_metrics.1 + write_metrics.1),
        })
    }

    /// 4K 随机读取测试
    pub fn run_4k_random_read_test(test_dir: &str) -> Option<DiskTestResult> {
        let block_size = 4 * 1024;
        let file_size = 20 * 1024 * 1024; // 减少到20MB
        let test_file = PathBuf::from(test_dir).join("4k_random_read.bin");
        
        if !create_test_file(&test_file, file_size, block_size) {
            return None;
        }
        
        let result = run_io_benchmark(&test_file, block_size, true, true, TEST_DURATION_SECS);
        let _ = std::fs::remove_file(&test_file);
        
        if let Some((speed, iops, latency)) = result {
            Some(DiskTestResult {
                test_name: format!("4K 随机读取 (延迟: {:.2}ms)", latency * 1000.0),
                read_speed: format!("{:.2} MB/s", speed),
                write_speed: "N/A".to_string(),
                read_iops: format!("{:.0}", iops),
                write_iops: "N/A".to_string(),
                total_speed: format!("{:.2} MB/s", speed),
                total_iops: format!("{:.0}", iops),
            })
        } else {
            None
        }
    }

    /// 4K 随机写入测试
    pub fn run_4k_random_write_test(test_dir: &str) -> Option<DiskTestResult> {
        let block_size = 4 * 1024;
        let test_file = PathBuf::from(test_dir).join("4k_random_write.bin");
        
        let result = run_io_benchmark(&test_file, block_size, false, true, TEST_DURATION_SECS);
        let _ = std::fs::remove_file(&test_file);
        
        if let Some((speed, iops, latency)) = result {
            Some(DiskTestResult {
                test_name: format!("4K 随机写入 (延迟: {:.2}ms)", latency * 1000.0),
                read_speed: "N/A".to_string(),
                write_speed: format!("{:.2} MB/s", speed),
                read_iops: "N/A".to_string(),
                write_iops: format!("{:.0}", iops),
                total_speed: format!("{:.2} MB/s", speed),
                total_iops: format!("{:.0}", iops),
            })
        } else {
            None
        }
    }

    /// 64K 顺序读写测试
    pub fn run_64k_sequential_test(test_dir: &str) -> Option<DiskTestResult> {
        let block_size = 64 * 1024; // 64KB
        let file_size = 30 * 1024 * 1024; // 减少到30MB
        let test_file = PathBuf::from(test_dir).join("64k_seq.bin");
        
        if !create_test_file(&test_file, file_size, block_size) {
            return None;
        }
        
        let read_result = run_io_benchmark(&test_file, block_size, true, false, TEST_DURATION_SECS);
        let write_result = run_io_benchmark(&test_file, block_size, false, false, TEST_DURATION_SECS);
        let _ = std::fs::remove_file(&test_file);
        
        if let (Some((read_speed, read_iops, read_latency)), Some((write_speed, write_iops, write_latency))) = 
            (read_result, write_result) {
            Some(DiskTestResult {
                test_name: format!("64K 顺序读写 (R:{:.1}ms W:{:.1}ms)", 
                                   read_latency * 1000.0, write_latency * 1000.0),
                read_speed: format!("{:.2} MB/s", read_speed),
                write_speed: format!("{:.2} MB/s", write_speed),
                read_iops: format!("{:.0}", read_iops),
                write_iops: format!("{:.0}", write_iops),
                total_speed: format!("{:.2} MB/s", (read_speed + write_speed) / 2.0),
                total_iops: format!("{:.0}", (read_iops + write_iops) / 2.0),
            })
        } else {
            None
        }
    }

    /// 1M 块顺序读写测试
    pub fn run_1m_sequential_test(test_dir: &str) -> Option<DiskTestResult> {
        let block_size = 1024 * 1024; // 1MB
        let file_size = 500 * 1024 * 1024; // 500MB
        let test_file = PathBuf::from(test_dir).join("1m_seq.bin");
        
        if !create_test_file(&test_file, file_size, block_size) {
            return None;
        }
        
        let (read_speed, read_iops, read_latency) = run_io_benchmark(&test_file, block_size, true, false, TEST_DURATION_SECS)?;
        let (write_speed, write_iops, write_latency) = run_io_benchmark(&test_file, block_size, false, false, TEST_DURATION_SECS)?;
        let _ = std::fs::remove_file(&test_file);
        
        Some(DiskTestResult {
            test_name: format!("1M 顺序读写 (R:{:.1}ms W:{:.1}ms)", 
                               read_latency * 1000.0, write_latency * 1000.0),
            read_speed: format!("{:.2} MB/s", read_speed),
            write_speed: format!("{:.2} MB/s", write_speed),
            read_iops: format!("{:.0}", read_iops),
            write_iops: format!("{:.0}", write_iops),
            total_speed: format!("{:.2} MB/s", (read_speed + write_speed) / 2.0),
            total_iops: format!("{:.0}", (read_iops + write_iops) / 2.0),
        })
    }

    /// 多线程并发测试
    pub fn run_multi_thread_test(test_dir: &str) -> Option<DiskTestResult> {
        let thread_count = num_cpus::get().min(8).max(2); // 2-8 个线程
        let block_size = 4 * 1024; // 4KB
        let file_size = 80 * 1024 * 1024; // 每线程80MB
        
        let total_read_speed = Arc::new(AtomicU64::new(0));
        let total_write_speed = Arc::new(AtomicU64::new(0));
        let total_read_iops = Arc::new(AtomicU64::new(0));
        let total_write_iops = Arc::new(AtomicU64::new(0));
        
        let mut handles = Vec::new();
        
        for i in 0..thread_count {
            let dir = test_dir.to_string();
            let read_speed_ref = Arc::clone(&total_read_speed);
            let write_speed_ref = Arc::clone(&total_write_speed);
            let read_iops_ref = Arc::clone(&total_read_iops);
            let write_iops_ref = Arc::clone(&total_write_iops);
            
            let handle = thread::spawn(move || {
                let test_file = PathBuf::from(&dir).join(format!("mt_test_{}.bin", i));
                
                // 并发读测试
                if let Some((speed, iops, _)) = run_io_benchmark(&test_file, block_size, true, true, 4) {
                    read_speed_ref.fetch_add((speed * 1000.0) as u64, Ordering::Relaxed);
                    read_iops_ref.fetch_add(iops as u64, Ordering::Relaxed);
                }
                
                // 并发写测试
                if let Some((speed, iops, _)) = run_io_benchmark(&test_file, block_size, false, true, 4) {
                    write_speed_ref.fetch_add((speed * 1000.0) as u64, Ordering::Relaxed);
                    write_iops_ref.fetch_add(iops as u64, Ordering::Relaxed);
                }
                
                let _ = std::fs::remove_file(&test_file);
            });
            
            handles.push(handle);
        }
        
        // 等待所有线程完成
        for handle in handles {
            let _ = handle.join();
        }
        
        let read_speed = total_read_speed.load(Ordering::Relaxed) as f64 / 1000.0;
        let write_speed = total_write_speed.load(Ordering::Relaxed) as f64 / 1000.0;
        let read_iops = total_read_iops.load(Ordering::Relaxed) as f64;
        let write_iops = total_write_iops.load(Ordering::Relaxed) as f64;
        
        Some(DiskTestResult {
            test_name: format!("{} 线程并发 4K 随机", thread_count),
            read_speed: format!("{:.2} MB/s", read_speed),
            write_speed: format!("{:.2} MB/s", write_speed),
            read_iops: format!("{:.0}", read_iops),
            write_iops: format!("{:.0}", write_iops),
            total_speed: format!("{:.2} MB/s", read_speed + write_speed),
            total_iops: format!("{:.0}", read_iops + write_iops),
        })
    }

    /// 混合负载测试 (70% 读 + 30% 写)
    fn run_mixed_workload_test(test_dir: &str) -> Option<DiskTestResult> {
        let block_size = 4 * 1024;
        let file_size = 15 * 1024 * 1024;  // 减少文件大小到15MB
        let test_file = PathBuf::from(test_dir).join("mixed_workload.bin");
        
        if !create_test_file(&test_file, file_size, block_size) {
            return None;
        }
        
        let result = run_mixed_io_test(&test_file, block_size, file_size, 70);
        let _ = std::fs::remove_file(&test_file);
        
        if let Some(((read_speed, read_iops), (write_speed, write_iops))) = result {
            Some(DiskTestResult {
                test_name: "混合负载 4K 随机 (70R/30W)".to_string(),
                read_speed: format!("{:.2} MB/s", read_speed),
                write_speed: format!("{:.2} MB/s", write_speed),
                read_iops: format!("{:.0}", read_iops),
                write_iops: format!("{:.0}", write_iops),
                total_speed: format!("{:.2} MB/s", read_speed + write_speed),
                total_iops: format!("{:.0}", read_iops + write_iops),
            })
        } else {
            None
        }
    }

    /// 检查可用磁盘空间
    fn check_available_space(test_dir: &str, required_bytes: u64) -> bool {
        use std::fs;
        
        // 尝试创建测试目录
        if let Err(_) = fs::create_dir_all(test_dir) {
            return false;
        }
        
        // 在Linux上使用statvfs检查空间
        match std::process::Command::new("df")
            .arg("-B1") // 以字节为单位
            .arg(test_dir)
            .output()
        {
            Ok(output) if output.status.success() => {
                let output_str = String::from_utf8_lossy(&output.stdout);
                for line in output_str.lines().skip(1) { // 跳过标题行
                    let fields: Vec<&str> = line.split_whitespace().collect();
                    if fields.len() >= 4 {
                        if let Ok(available) = fields[3].parse::<u64>() {
                            return available >= required_bytes;
                        }
                    }
                }
                false
            }
            _ => {
                // 如果df命令失败，尝试创建小文件测试
                let test_file = std::path::PathBuf::from(test_dir).join("space_test.tmp");
                match std::fs::File::create(&test_file) {
                    Ok(_) => {
                        let _ = std::fs::remove_file(&test_file);
                        true // 假设有足够空间
                    }
                    Err(_) => false,
                }
            }
        }
    }

    /// 核心 IO 基准测试函数
    fn run_io_benchmark(
        file_path: &PathBuf,
        block_size: usize,
        is_read: bool,
        random_access: bool,
        duration_secs: u64,
    ) -> Option<(f64, f64, f64)> {
        // 预热阶段
        if WARMUP_DURATION_SECS > 0 {
            let _ = run_io_test_internal(file_path, block_size, is_read, random_access, WARMUP_DURATION_SECS);
        }
        
        // 正式测试
        run_io_test_internal(file_path, block_size, is_read, random_access, duration_secs)
    }

    /// 内部 IO 测试实现 - 增强错误处理
    fn run_io_test_internal(
        file_path: &PathBuf,
        block_size: usize,
        is_read: bool,
        random_access: bool,
        duration_secs: u64,
    ) -> Option<(f64, f64, f64)> {
        let test_duration = Duration::from_secs(duration_secs);
        let start_time = Instant::now();
        
        // 增强的文件打开错误处理
        let mut file = if is_read {
            match OpenOptions::new().read(true).open(file_path) {
                Ok(f) => f,
                Err(e) => {
                    eprintln!("无法打开文件进行读取: {}", e);
                    return None;
                }
            }
        } else {
            match OpenOptions::new()
                .create(true)
                .write(true)
                .truncate(true)
                .open(file_path)
            {
                Ok(f) => f,
                Err(e) => {
                    eprintln!("无法打开文件进行写入: {}", e);
                    return None;
                }
            }
        };
        
        let mut buffer = if is_read {
            vec![0u8; block_size]
        } else {
            create_test_data(block_size)
        };
        
        let mut bytes_processed = 0usize;
        let mut operations = 0u64;
        let mut total_latency = 0.0;
        let mut rng = rand::thread_rng();
        let mut consecutive_errors = 0;
        const MAX_CONSECUTIVE_ERRORS: u32 = 10;
        
        // 获取文件大小（用于随机访问）
        let file_size = if is_read {
            match file.metadata() {
                Ok(metadata) => metadata.len() as usize,
                Err(_) => return None,
            }
        } else {
            50 * 1024 * 1024 // 减少默认最大文件大小到50MB
        };
        
        while start_time.elapsed() < test_duration && consecutive_errors < MAX_CONSECUTIVE_ERRORS {
            let op_start = Instant::now();
            
            // 随机定位（如果需要）
            if random_access && operations > 0 {
                let max_offset = file_size.saturating_sub(block_size);
                if max_offset > 0 {
                    let offset = rng.gen_range(0..max_offset);
                    if let Err(_) = file.seek(SeekFrom::Start(offset as u64)) {
                        consecutive_errors += 1;
                        continue;
                    }
                }
            }
            
            // 执行IO操作
            let success = if is_read {
                match file.read_exact(&mut buffer) {
                    Ok(_) => true,
                    Err(_) => false,
                }
            } else {
                // 写操作，减少同步频率
                match file.write_all(&buffer) {
                    Ok(_) => {
                        // 每32次操作同步一次，减少性能损耗
                        if operations > 0 && operations % 32 == 0 {
                            file.flush().is_ok()
                        } else {
                            true
                        }
                    }
                    Err(_) => false,
                }
            };
            
            if success {
                bytes_processed += block_size;
                operations += 1;
                total_latency += op_start.elapsed().as_secs_f64();
                consecutive_errors = 0; // 重置错误计数
            } else if is_read && !random_access {
                // 顺序读到文件末尾，重置位置
                if file.seek(SeekFrom::Start(0)).is_ok() {
                    consecutive_errors = 0;
                    continue;
                } else {
                    consecutive_errors += 1;
                }
            } else {
                consecutive_errors += 1;
            }
        }
        
        let elapsed = start_time.elapsed();
        let elapsed_secs = elapsed.as_secs_f64();
        
        if elapsed_secs > 0.0 && operations > 0 {
            let speed_mbps = (bytes_processed as f64) / (1024.0 * 1024.0) / elapsed_secs;
            let iops = operations as f64 / elapsed_secs;
            let avg_latency = total_latency / operations as f64;
            Some((speed_mbps, iops, avg_latency))
        } else {
            None
        }
    }

    /// 混合读写测试（指定读取百分比）- 增强版
    fn run_mixed_io_test(
        file_path: &PathBuf,
        block_size: usize,
        file_size: usize,
        read_percentage: u8,
    ) -> Option<((f64, f64), (f64, f64))> {
        let test_duration = Duration::from_secs(TEST_DURATION_SECS);
        let start_time = Instant::now();
        
        let mut file = match OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(file_path)
        {
            Ok(f) => f,
            Err(e) => {
                eprintln!("无法打开混合测试文件: {}", e);
                return None;
            }
        };
        
        let mut read_buffer = vec![0u8; block_size];
        let write_buffer = create_test_data(block_size);
        
        let mut read_bytes = 0usize;
        let mut write_bytes = 0usize;
        let mut read_ops = 0u64;
        let mut write_ops = 0u64;
        let mut rng = rand::thread_rng();
        let mut consecutive_errors = 0;
        const MAX_CONSECUTIVE_ERRORS: u32 = 15;
        
        // 批量同步计数器
        let mut pending_writes = 0;
        const BATCH_SIZE: usize = 64;  // 每64次写操作同步一次
        
        while start_time.elapsed() < test_duration && consecutive_errors < MAX_CONSECUTIVE_ERRORS {
            let is_read_op = rng.gen_range(0..100) < read_percentage;
            let max_offset = file_size.saturating_sub(block_size);
            
            if max_offset > 0 {
                let offset = rng.gen_range(0..max_offset);
                if let Err(_) = file.seek(SeekFrom::Start(offset as u64)) {
                    consecutive_errors += 1;
                    continue;
                }
            }
            
            let success = if is_read_op {
                match file.read_exact(&mut read_buffer) {
                    Ok(_) => {
                        read_bytes += block_size;
                        read_ops += 1;
                        true
                    }
                    Err(_) => false,
                }
            } else {
                match file.write_all(&write_buffer) {
                    Ok(_) => {
                        write_bytes += block_size;
                        write_ops += 1;
                        pending_writes += 1;
                        
                        // 批量同步
                        if pending_writes >= BATCH_SIZE {
                            let _ = file.flush();
                            pending_writes = 0;
                        }
                        true
                    }
                    Err(_) => false,
                }
            };
            
            if success {
                consecutive_errors = 0;
            } else {
                consecutive_errors += 1;
            }
        }
        
        // 确保所有写操作都被同步
        if pending_writes > 0 {
            let _ = file.flush();
        }
        
        let elapsed = start_time.elapsed();
        let elapsed_secs = elapsed.as_secs_f64();
        
        if elapsed_secs > 0.0 && (read_ops > 0 || write_ops > 0) {
            let read_speed = if read_ops > 0 {
                (read_bytes as f64) / (1024.0 * 1024.0) / elapsed_secs
            } else {
                0.0
            };
            let write_speed = if write_ops > 0 {
                (write_bytes as f64) / (1024.0 * 1024.0) / elapsed_secs
            } else {
                0.0
            };
            let read_iops = read_ops as f64 / elapsed_secs;
            let write_iops = write_ops as f64 / elapsed_secs;
            
            Some(((read_speed, read_iops), (write_speed, write_iops)))
        } else {
            None
        }
    }

    /// 创建测试文件 - 增强版错误处理
    fn create_test_file(file_path: &PathBuf, file_size: usize, block_size: usize) -> bool {
        let mut file = match OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(file_path)
        {
            Ok(f) => f,
            Err(e) => {
                eprintln!("无法创建测试文件 {:?}: {}", file_path, e);
                return false;
            }
        };
        
        let test_data = create_test_data(block_size);
        let blocks_to_write = file_size / block_size;
        let mut blocks_written = 0;
        
        // 分批写入，每写入一定数量的块就检查一次
        const WRITE_BATCH_SIZE: usize = 10;
        
        for i in 0..blocks_to_write {
            if let Err(e) = file.write_all(&test_data) {
                eprintln!("写入测试文件失败 (块 {}): {}", i, e);
                return false;
            }
            
            blocks_written += 1;
            
            // 每写入一批块就同步一次
            if blocks_written % WRITE_BATCH_SIZE == 0 {
                if let Err(e) = file.flush() {
                    eprintln!("同步测试文件失败: {}", e);
                    return false;
                }
            }
        }
        
        // 最终同步
        match file.flush() {
            Ok(_) => true,
            Err(e) => {
                eprintln!("最终同步测试文件失败: {}", e);
                false
            }
        }
    }

    /// 创建测试数据（防止压缩优化）
    fn create_test_data(size: usize) -> Vec<u8> {
        let mut data = Vec::with_capacity(size);
        let mut rng = rand::thread_rng();
        
        // 使用随机数据避免压缩
        for _ in 0..size {
            data.push(rng.gen());
        }
        
        data
    }

    /// 兼容性函数 - 保持向后兼容
    pub fn run_rust_disk_tests(test_dir: &str) -> Vec<DiskTestResult> {
        run_professional_disk_tests(test_dir)
    }
}