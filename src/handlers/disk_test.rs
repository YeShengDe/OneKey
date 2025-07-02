use std::process::Command;
use std::fs;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::thread;
use std::time::{Instant, Duration};

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

// 实时性能数据点
#[derive(Debug, Clone)]
pub struct PerformanceDataPoint {
    pub timestamp: f64,  // 相对时间戳（秒）
    pub read_speed: f64,  // MB/s
    pub write_speed: f64, // MB/s
    pub read_iops: f64,
    pub write_iops: f64,
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
    pub realtime_data: Vec<PerformanceDataPoint>, // 实时性能数据
    pub current_read_speed: f64,  // 当前读取速度 MB/s
    pub current_write_speed: f64, // 当前写入速度 MB/s
    pub current_read_iops: f64,   // 当前读取IOPS
    pub current_write_iops: f64,  // 当前写入IOPS
    pub test_start_time: Option<Instant>, // 测试开始时间
    pub current_test_phase: String,       // 当前测试阶段
    pub total_test_phases: usize,         // 总测试阶段数
    pub current_phase_index: usize,       // 当前阶段索引
    pub animation_frame: usize,           // 动画帧
    pub chart_data_points: Vec<(f64, f64)>, // 折线图数据点 (时间, 值)
    pub read_chart_data: Vec<(f64, f64)>,   // 读取速度图表数据
    pub write_chart_data: Vec<(f64, f64)>,  // 写入速度图表数据
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
            realtime_data: Vec::new(),
            current_read_speed: 0.0,
            current_write_speed: 0.0,
            current_read_iops: 0.0,
            current_write_iops: 0.0,
            test_start_time: None,
            current_test_phase: String::new(),
            total_test_phases: 3, // 修改为3个阶段：准备 -> 读取测试 -> 写入测试
            current_phase_index: 0,
            animation_frame: 0,
            chart_data_points: Vec::new(),
            read_chart_data: Vec::new(),
            write_chart_data: Vec::new(),
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
#[allow(dead_code)]
pub fn reset_disk_test() {
    DISK_TEST_STARTED.store(false, Ordering::SeqCst);
    if let Ok(mut global_info) = DISK_TEST_INFO.lock() {
        *global_info = None;
    }
}

// FIO风格的多块大小磁盘测试
fn run_async_disk_tests() {
    let test_dir = "/tmp/disk_test";
    if let Err(e) = fs::create_dir_all(test_dir) {
        update_test_status_with_error(format!("无法创建测试目录: {}", e));
        return;
    }
    
    let mut all_results = Vec::new();
    let block_sizes = vec![
        (4 * 1024, "4K"),
        (64 * 1024, "64K"),
        (512 * 1024, "512K"),
        (1024 * 1024, "1M"),
    ];
    
    let total_phases = block_sizes.len() * 2 + 1; // 每个块大小有读写两个阶段，加上准备阶段
    let mut current_phase = 0;
    
    // 阶段1：准备测试
    update_test_status("准备FIO风格磁盘测试", 5, true);
    update_test_phase("准备测试", current_phase);
    thread::sleep(Duration::from_millis(500));
    current_phase += 1;
    
    // 对每种块大小进行测试
    for (i, (block_size, block_name)) in block_sizes.iter().enumerate() {
        // 重新计算进度，确保不会超过100%
        let read_progress = 10 + (i * 80) / block_sizes.len() + (40 / block_sizes.len()) / 2;
        let write_progress = 10 + (i * 80) / block_sizes.len() + (40 / block_sizes.len());
        
        // 确保进度不超过95%（为完成阶段预留5%）
        let read_progress = std::cmp::min(read_progress, 95) as u8;
        let write_progress = std::cmp::min(write_progress, 95) as u8;
        
        // 读取测试
        update_test_status(&format!("正在测试 {} 读取性能", block_name), read_progress, true);
        update_test_phase(&format!("{} 读取测试", block_name), current_phase);
        
        let read_result = match std::panic::catch_unwind(|| {
            rust_disk_test::run_block_size_read_test(test_dir, *block_size, block_name)
        }) {
            Ok(result) => result,
            Err(e) => {
                let error_msg = if let Some(s) = e.downcast_ref::<String>() {
                    format!("{} 读取测试错误: {}", block_name, s)
                } else {
                    format!("{} 读取测试发生未知错误", block_name)
                };
                update_test_status_with_error(error_msg);
                return;
            }
        };
        
        if let Some(result) = read_result {
            all_results.push(result);
        }
        current_phase += 1;
        
        // 写入测试
        update_test_status(&format!("正在测试 {} 写入性能", block_name), write_progress, true);
        update_test_phase(&format!("{} 写入测试", block_name), current_phase);
        
        let write_result = match std::panic::catch_unwind(|| {
            rust_disk_test::run_block_size_write_test(test_dir, *block_size, block_name)
        }) {
            Ok(result) => result,
            Err(e) => {
                let error_msg = if let Some(s) = e.downcast_ref::<String>() {
                    format!("{} 写入测试错误: {}", block_name, s)
                } else {
                    format!("{} 写入测试发生未知错误", block_name)
                };
                update_test_status_with_error(error_msg);
                return;
            }
        };
        
        if let Some(result) = write_result {
            all_results.push(result);
        }
        current_phase += 1;
    }
    
    // 计算总计性能
    let total_results = rust_disk_test::calculate_total_performance(&all_results);
    all_results.extend(total_results);
    
    // 清理测试文件目录
    let _ = fs::remove_dir_all(test_dir);
    
    // 完成测试
    update_test_status("FIO风格测试完成", 100, false);
    update_test_phase("完成", current_phase);
    update_test_results(all_results);
}

// 更新测试阶段
fn update_test_phase(phase_name: &str, phase_index: usize) {
    if let Ok(mut global_info) = DISK_TEST_INFO.lock() {
        if let Some(ref mut info) = global_info.as_mut() {
            info.current_test_phase = phase_name.to_string();
            info.current_phase_index = phase_index;
            info.last_update = Instant::now();
            
            // 设置需要刷新UI的标志
            NEEDS_UI_REFRESH.store(true, Ordering::Relaxed);
        }
    }
}

// 更新测试状态
fn update_test_status(current_test: &str, progress: u8, is_testing: bool) {
    if let Ok(mut global_info) = DISK_TEST_INFO.lock() {
        if let Some(ref mut info) = global_info.as_mut() {
            info.current_test = current_test.to_string();
            info.progress = progress;
            info.is_testing = is_testing;
            info.last_update = Instant::now();
            
            // 更新测试阶段信息
            if is_testing {
                if info.test_start_time.is_none() {
                    info.test_start_time = Some(Instant::now());
                }
                info.current_test_phase = current_test.to_string();
                info.current_phase_index = (progress as usize * info.total_test_phases) / 100;
                
                // 更新动画帧
                info.animation_frame = (info.animation_frame + 1) % 8; // 8帧动画循环
            }
            
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

// 更新实时性能数据
fn update_realtime_data(read_speed: f64, write_speed: f64, read_iops: f64, write_iops: f64) {
    // 验证输入值，确保它们是有效的数字
    let safe_read_speed = if read_speed.is_finite() && read_speed >= 0.0 { read_speed } else { 0.0 };
    let safe_write_speed = if write_speed.is_finite() && write_speed >= 0.0 { write_speed } else { 0.0 };
    let safe_read_iops = if read_iops.is_finite() && read_iops >= 0.0 { read_iops } else { 0.0 };
    let safe_write_iops = if write_iops.is_finite() && write_iops >= 0.0 { write_iops } else { 0.0 };
    
    if let Ok(mut global_info) = DISK_TEST_INFO.lock() {
        if let Some(ref mut info) = global_info.as_mut() {
            // 更新当前值
            info.current_read_speed = safe_read_speed;
            info.current_write_speed = safe_write_speed;
            info.current_read_iops = safe_read_iops;
            info.current_write_iops = safe_write_iops;
            
            // 计算相对时间戳
            let relative_timestamp = if let Some(start_time) = info.test_start_time {
                start_time.elapsed().as_secs_f64()
            } else {
                0.0
            };
            
            // 添加数据点到历史记录
            let data_point = PerformanceDataPoint {
                timestamp: relative_timestamp,
                read_speed: safe_read_speed,
                write_speed: safe_write_speed,
                read_iops: safe_read_iops,
                write_iops: safe_write_iops,
            };
            
            info.realtime_data.push(data_point);
            
            // 更新图表数据
            if read_speed > 0.0 {
                info.read_chart_data.push((relative_timestamp, read_speed));
            }
            if write_speed > 0.0 {
                info.write_chart_data.push((relative_timestamp, write_speed));
            }
            
            // 保持最近100个数据点
            if info.realtime_data.len() > 100 {
                info.realtime_data.remove(0);
            }
            if info.read_chart_data.len() > 100 {
                info.read_chart_data.remove(0);
            }
            if info.write_chart_data.len() > 100 {
                info.write_chart_data.remove(0);
            }
            
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
        let clamped_progress = info.progress.min(100); // 确保进度不超过100%
        output.push_str(&format!("状态: {}\n", info.current_test));
        output.push_str(&format!("进度: {}%\n", clamped_progress));
        
        // 进度条
        let progress_bar_length = 30;
        let filled = (clamped_progress as usize * progress_bar_length) / 100;
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

// 更新动画帧（用于实时UI效果）
pub fn update_animation_frame() {
    if let Ok(mut global_info) = DISK_TEST_INFO.lock() {
        if let Some(ref mut info) = global_info.as_mut() {
            if info.is_testing {
                info.animation_frame = (info.animation_frame + 1) % 8;
                info.last_update = Instant::now();
                NEEDS_UI_REFRESH.store(true, Ordering::Relaxed);
            }
        }
    }
}

// 获取实时测试数据用于UI显示
pub fn get_realtime_ui_data() -> Option<(f64, f64, f64, f64, usize, String, u8)> {
    if let Ok(global_info) = DISK_TEST_INFO.lock() {
        if let Some(ref info) = global_info.as_ref() {
            Some((
                info.current_read_speed,
                info.current_write_speed,
                info.current_read_iops,
                info.current_write_iops,
                info.animation_frame,
                info.current_test_phase.clone(),
                info.progress,
            ))
        } else {
            None
        }
    } else {
        None
    }
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
    use std::fs::OpenOptions;
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

    /// 运行实时读写测试，持续更新UI数据
    pub fn run_realtime_rw_test(test_dir: &str) -> Vec<DiskTestResult> {
        let mut results = Vec::new();
        
        // 检查磁盘空间
        if !check_available_space(test_dir, 50 * 1024 * 1024) {
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
        
        // 运行实时读写测试
        if let Some(result) = run_realtime_read_write_test(test_dir) {
            results.push(result);
        }
        
        results
    }

    /// 运行指定块大小的读取测试
    pub fn run_block_size_read_test(test_dir: &str, block_size: usize, block_name: &str) -> Option<DiskTestResult> {
        // 检查磁盘空间
        let required_space = (block_size * 1000).max(50 * 1024 * 1024); // 至少50MB
        if !check_available_space(test_dir, required_space as u64) {
            return Some(DiskTestResult {
                test_name: format!("{} 读取测试错误", block_name),
                read_speed: "错误".to_string(),
                write_speed: "N/A".to_string(),
                read_iops: "N/A".to_string(),
                write_iops: "N/A".to_string(),
                total_speed: "N/A".to_string(),
                total_iops: "磁盘空间不足".to_string(),
            });
        }
        
        let file_size = calculate_optimal_file_size(block_size);
        let test_file = PathBuf::from(test_dir).join(format!("read_test_{}.bin", block_name));
        
        // 创建测试文件
        if !create_test_file(&test_file, file_size, block_size) {
            return None;
        }
        
        let test_duration = Duration::from_secs(5); // 5秒测试
        let result = run_io_test_internal(&test_file, block_size, true, true, 5);
        
        // 清理测试文件
        let _ = std::fs::remove_file(&test_file);
        
        if let Some((speed, iops, latency)) = result {
            Some(DiskTestResult {
                test_name: format!("{} 读取", block_name),
                read_speed: format_speed_with_unit(speed),
                write_speed: "N/A".to_string(),
                read_iops: format!("{:.1}k", iops / 1000.0),
                write_iops: "N/A".to_string(),
                total_speed: format_speed_with_unit(speed),
                total_iops: format!("{:.1}k", iops / 1000.0),
            })
        } else {
            None
        }
    }

    /// 运行指定块大小的写入测试
    pub fn run_block_size_write_test(test_dir: &str, block_size: usize, block_name: &str) -> Option<DiskTestResult> {
        // 检查磁盘空间
        let required_space = (block_size * 1000).max(50 * 1024 * 1024); // 至少50MB
        if !check_available_space(test_dir, required_space as u64) {
            return Some(DiskTestResult {
                test_name: format!("{} 写入测试错误", block_name),
                read_speed: "N/A".to_string(),
                write_speed: "错误".to_string(),
                read_iops: "N/A".to_string(),
                write_iops: "N/A".to_string(),
                total_speed: "N/A".to_string(),
                total_iops: "磁盘空间不足".to_string(),
            });
        }
        
        let test_file = PathBuf::from(test_dir).join(format!("write_test_{}.bin", block_name));
        
        let test_duration = Duration::from_secs(5); // 5秒测试
        let result = run_io_test_internal(&test_file, block_size, false, true, 5);
        
        // 清理测试文件
        let _ = std::fs::remove_file(&test_file);
        
        if let Some((speed, iops, latency)) = result {
            Some(DiskTestResult {
                test_name: format!("{} 写入", block_name),
                read_speed: "N/A".to_string(),
                write_speed: format_speed_with_unit(speed),
                read_iops: "N/A".to_string(),
                write_iops: format!("{:.1}k", iops / 1000.0),
                total_speed: format_speed_with_unit(speed),
                total_iops: format!("{:.1}k", iops / 1000.0),
            })
        } else {
            None
        }
    }

    /// 计算总计性能（将读写性能合并）
    pub fn calculate_total_performance(results: &[DiskTestResult]) -> Vec<DiskTestResult> {
        let mut total_results = Vec::new();
        let block_sizes = ["4K", "64K", "512K", "1M"];
        
        for block_size in &block_sizes {
            let read_result = results.iter().find(|r| r.test_name == format!("{} 读取", block_size));
            let write_result = results.iter().find(|r| r.test_name == format!("{} 写入", block_size));
            
            if let (Some(read), Some(write)) = (read_result, write_result) {
                let read_speed_val = parse_speed_value(&read.read_speed);
                let write_speed_val = parse_speed_value(&write.write_speed);
                let read_iops_val = parse_iops_value(&read.read_iops);
                let write_iops_val = parse_iops_value(&write.write_iops);
                
                let total_speed = read_speed_val + write_speed_val;
                let total_iops = read_iops_val + write_iops_val;
                
                total_results.push(DiskTestResult {
                    test_name: format!("{} 总计", block_size),
                    read_speed: read.read_speed.clone(),
                    write_speed: write.write_speed.clone(),
                    read_iops: read.read_iops.clone(),
                    write_iops: write.write_iops.clone(),
                    total_speed: format_speed_with_unit(total_speed),
                    total_iops: format!("{:.1}k", total_iops / 1000.0),
                });
            }
        }
        
        total_results
    }

    /// 根据块大小计算最优文件大小
    fn calculate_optimal_file_size(block_size: usize) -> usize {
        match block_size {
            4096 => 50 * 1024 * 1024,      // 4K -> 50MB
            65536 => 100 * 1024 * 1024,    // 64K -> 100MB
            524288 => 200 * 1024 * 1024,   // 512K -> 200MB
            1048576 => 500 * 1024 * 1024,  // 1M -> 500MB
            _ => 50 * 1024 * 1024,         // 默认50MB
        }
    }

    /// 格式化速度值（自动选择MB/s或GB/s）
    fn format_speed_with_unit(speed_mbps: f64) -> String {
        if speed_mbps >= 1024.0 {
            format!("{:.2} GB/s", speed_mbps / 1024.0)
        } else {
            format!("{:.2} MB/s", speed_mbps)
        }
    }

    /// 解析速度值（转换为MB/s）
    fn parse_speed_value(speed_str: &str) -> f64 {
        if speed_str == "N/A" || speed_str == "错误" {
            return 0.0;
        }
        
        if let Some(gb_pos) = speed_str.find("GB/s") {
            let value_str = &speed_str[..gb_pos].trim();
            value_str.parse::<f64>().unwrap_or(0.0) * 1024.0
        } else if let Some(mb_pos) = speed_str.find("MB/s") {
            let value_str = &speed_str[..mb_pos].trim();
            value_str.parse::<f64>().unwrap_or(0.0)
        } else {
            0.0
        }
    }

    /// 解析IOPS值
    fn parse_iops_value(iops_str: &str) -> f64 {
        if iops_str == "N/A" || iops_str == "错误" {
            return 0.0;
        }
        
        if let Some(k_pos) = iops_str.find('k') {
            let value_str = &iops_str[..k_pos].trim();
            value_str.parse::<f64>().unwrap_or(0.0) * 1000.0
        } else {
            iops_str.parse::<f64>().unwrap_or(0.0)
        }
    }
    pub fn run_read_only_test(test_dir: &str) -> Vec<DiskTestResult> {
        let mut results = Vec::new();
        
        // 检查磁盘空间
        if !check_available_space(test_dir, 30 * 1024 * 1024) {
            return vec![DiskTestResult {
                test_name: "读取测试错误".to_string(),
                read_speed: "错误".to_string(),
                write_speed: "磁盘空间不足".to_string(),
                read_iops: "N/A".to_string(),
                write_iops: "N/A".to_string(),
                total_speed: "N/A".to_string(),
                total_iops: "需要至少30MB空间".to_string(),
            }];
        }
        
        // 运行读取测试
        if let Some(result) = run_dedicated_read_test(test_dir) {
            results.push(result);
        }
        
        results
    }

    /// 运行纯写入测试
    pub fn run_write_only_test(test_dir: &str) -> Vec<DiskTestResult> {
        let mut results = Vec::new();
        
        // 检查磁盘空间
        if !check_available_space(test_dir, 30 * 1024 * 1024) {
            return vec![DiskTestResult {
                test_name: "写入测试错误".to_string(),
                read_speed: "错误".to_string(),
                write_speed: "磁盘空间不足".to_string(),
                read_iops: "N/A".to_string(),
                write_iops: "N/A".to_string(),
                total_speed: "N/A".to_string(),
                total_iops: "需要至少30MB空间".to_string(),
            }];
        }
        
        // 运行写入测试
        if let Some(result) = run_dedicated_write_test(test_dir) {
            results.push(result);
        }
        
        results
    }

    /// 实时读写测试 - 持续更新折线图数据
    fn run_realtime_read_write_test(test_dir: &str) -> Option<DiskTestResult> {
        let block_size = 4 * 1024; // 4KB
        let file_size = 20 * 1024 * 1024; // 20MB
        let read_test_file = PathBuf::from(test_dir).join("realtime_read.bin");
        let write_test_file = PathBuf::from(test_dir).join("realtime_write.bin");
        
        // 创建读取测试文件
        if !create_test_file(&read_test_file, file_size, block_size) {
            return None;
        }
        
        let test_duration = Duration::from_secs(10); // 10秒测试
        
        // 启动并发读写测试
        let read_handle = {
            let file_path = read_test_file.clone();
            thread::spawn(move || {
                run_continuous_read_test(&file_path, block_size, test_duration)
            })
        };
        
        let write_handle = {
            let file_path = write_test_file.clone();
            thread::spawn(move || {
                run_continuous_write_test(&file_path, block_size, test_duration)
            })
        };
        
        // 等待测试完成
        let read_result = read_handle.join().ok().flatten();
        let write_result = write_handle.join().ok().flatten();
        
        // 清理测试文件
        let _ = std::fs::remove_file(&read_test_file);
        let _ = std::fs::remove_file(&write_test_file);
        
        if let (Some((read_speed, read_iops, read_latency)), Some((write_speed, write_iops, write_latency))) = 
            (read_result, write_result) {
            Some(DiskTestResult {
                test_name: format!("实时读写测试 (R:{:.1}ms W:{:.1}ms)", 
                                   read_latency * 1000.0, write_latency * 1000.0),
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

    /// 专门的读取测试 - 类似Speedtest的下载测试
    fn run_dedicated_read_test(test_dir: &str) -> Option<DiskTestResult> {
        let block_size = 4 * 1024; // 4KB
        let file_size = 30 * 1024 * 1024; // 30MB
        let test_file = PathBuf::from(test_dir).join("read_test.bin");
        
        // 创建测试文件
        if !create_test_file(&test_file, file_size, block_size) {
            return None;
        }
        
        let test_duration = Duration::from_secs(8); // 8秒测试
        let result = run_continuous_read_test(&test_file, block_size, test_duration);
        
        // 清理测试文件
        let _ = std::fs::remove_file(&test_file);
        
        if let Some((speed, iops, latency)) = result {
            Some(DiskTestResult {
                test_name: format!("📖 读取性能测试 (延迟: {:.1}ms)", latency * 1000.0),
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

    /// 专门的写入测试 - 类似Speedtest的上传测试
    fn run_dedicated_write_test(test_dir: &str) -> Option<DiskTestResult> {
        let block_size = 4 * 1024; // 4KB
        let test_file = PathBuf::from(test_dir).join("write_test.bin");
        
        let test_duration = Duration::from_secs(8); // 8秒测试
        let result = run_continuous_write_test(&test_file, block_size, test_duration);
        
        // 清理测试文件
        let _ = std::fs::remove_file(&test_file);
        
        if let Some((speed, iops, latency)) = result {
            Some(DiskTestResult {
                test_name: format!("📤 写入性能测试 (延迟: {:.1}ms)", latency * 1000.0),
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

    /// 持续读取测试，实时更新数据
    fn run_continuous_read_test(
        file_path: &PathBuf,
        block_size: usize,
        duration: Duration,
    ) -> Option<(f64, f64, f64)> {
        let mut file = OpenOptions::new().read(true).open(file_path).ok()?;
        let mut buffer = vec![0u8; block_size];
        let mut rng = rand::thread_rng();
        
        let start_time = Instant::now();
        let mut bytes_processed = 0usize;
        let mut operations = 0u64;
        let mut total_latency = 0.0;
        
        // 获取文件大小
        let file_size = file.metadata().ok()?.len() as usize;
        
        // 实时更新计时器
        let mut last_update = Instant::now();
        let update_interval = Duration::from_millis(100); // 每100ms更新一次
        let mut last_bytes = 0usize;
        let mut last_ops = 0u64;
        
        while start_time.elapsed() < duration {
            let op_start = Instant::now();
            
            // 随机定位
            let max_offset = file_size.saturating_sub(block_size);
            if max_offset > 0 {
                let offset = rng.gen_range(0..max_offset);
                if file.seek(SeekFrom::Start(offset as u64)).is_err() {
                    continue;
                }
            }
            
            // 执行读取操作
            if file.read_exact(&mut buffer).is_ok() {
                bytes_processed += block_size;
                operations += 1;
                total_latency += op_start.elapsed().as_secs_f64();
                
                // 实时更新UI数据
                if last_update.elapsed() >= update_interval {
                    let elapsed_since_last = last_update.elapsed().as_secs_f64();
                    let bytes_delta = bytes_processed - last_bytes;
                    let ops_delta = operations - last_ops;
                    
                    if elapsed_since_last > 0.0 {
                        let current_read_speed = (bytes_delta as f64) / (1024.0 * 1024.0) / elapsed_since_last;
                        let current_read_iops = ops_delta as f64 / elapsed_since_last;
                        
                        // 更新实时数据（只更新读取部分）
                        super::update_realtime_data(current_read_speed, 0.0, current_read_iops, 0.0);
                    }
                    
                    last_update = Instant::now();
                    last_bytes = bytes_processed;
                    last_ops = operations;
                }
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

    /// 持续写入测试，实时更新数据
    fn run_continuous_write_test(
        file_path: &PathBuf,
        block_size: usize,
        duration: Duration,
    ) -> Option<(f64, f64, f64)> {
        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(file_path).ok()?;
            
        let buffer = create_test_data(block_size);
        
        let start_time = Instant::now();
        let mut bytes_processed = 0usize;
        let mut operations = 0u64;
        let mut total_latency = 0.0;
        
        // 实时更新计时器
        let mut last_update = Instant::now();
        let update_interval = Duration::from_millis(100);
        let mut last_bytes = 0usize;
        let mut last_ops = 0u64;
        let mut sync_counter = 0;
        
        while start_time.elapsed() < duration {
            let op_start = Instant::now();
            
            // 执行写入操作
            if file.write_all(&buffer).is_ok() {
                bytes_processed += block_size;
                operations += 1;
                total_latency += op_start.elapsed().as_secs_f64();
                sync_counter += 1;
                
                // 每32次操作同步一次
                if sync_counter >= 32 {
                    let _ = file.flush();
                    sync_counter = 0;
                }
                
                // 实时更新UI数据
                if last_update.elapsed() >= update_interval {
                    let elapsed_since_last = last_update.elapsed().as_secs_f64();
                    let bytes_delta = bytes_processed - last_bytes;
                    let ops_delta = operations - last_ops;
                    
                    if elapsed_since_last > 0.0 {
                        let current_write_speed = (bytes_delta as f64) / (1024.0 * 1024.0) / elapsed_since_last;
                        let current_write_iops = ops_delta as f64 / elapsed_since_last;
                        
                        // 更新实时数据（只更新写入部分）
                        super::update_realtime_data(0.0, current_write_speed, 0.0, current_write_iops);
                    }
                    
                    last_update = Instant::now();
                    last_bytes = bytes_processed;
                    last_ops = operations;
                }
            }
        }
        
        // 最终同步
        let _ = file.flush();
        
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
        let _file_size = 80 * 1024 * 1024; // 每线程80MB
        
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
                for line in output_str.lines().skip(1) // 跳过标题行
                {
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

    /// 内部 IO 测试实现 - 增强错误处理和实时反馈
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
        
        // 实时数据更新相关 - 增加更新频率
        let mut last_update = Instant::now();
        let update_interval = Duration::from_millis(100); // 每100ms更新一次，更频繁
        let mut last_bytes = 0usize;
        let mut last_ops = 0u64;
        
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
                
                // 实时数据更新 - 增强版本，更频繁更新
                if last_update.elapsed() >= update_interval {
                    let elapsed_since_last = last_update.elapsed().as_secs_f64();
                    let bytes_delta = bytes_processed - last_bytes;
                    let ops_delta = operations - last_ops;
                    
                    if elapsed_since_last > 0.0 {
                        let current_speed = (bytes_delta as f64) / (1024.0 * 1024.0) / elapsed_since_last;
                        let current_iops = ops_delta as f64 / elapsed_since_last;
                        
                        // 根据测试类型更新实时数据
                        if is_read {
                            super::update_realtime_data(current_speed, 0.0, current_iops, 0.0);
                        } else {
                            super::update_realtime_data(0.0, current_speed, 0.0, current_iops);
                        }
                    }
                    
                    last_update = Instant::now();
                    last_bytes = bytes_processed;
                    last_ops = operations;
                }
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
            
            // 最终更新一次实时数据
            if is_read {
                super::update_realtime_data(speed_mbps, 0.0, iops, 0.0);
            } else {
                super::update_realtime_data(0.0, speed_mbps, 0.0, iops);
            }
            
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