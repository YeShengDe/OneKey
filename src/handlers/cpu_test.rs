use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Instant;
use std::collections::HashMap;
use rayon::prelude::*;

// CPU 测试结果结构
#[derive(Debug, Clone)]
pub struct CpuTestResult {
    pub test_name: String,
    pub single_core_score: u32,
    pub multi_core_score: u32,
    pub duration_ms: u64,
    pub details: HashMap<String, String>,
}

// CPU 测试信息状态
#[derive(Debug, Clone)]
pub struct CpuTestInfo {
    pub is_testing: bool,
    pub current_test: String,
    pub progress: u8,  // 0-100
    pub results: Vec<CpuTestResult>,
    pub last_update: Instant,
    pub error_message: Option<String>,
    pub cpu_info: String,
    pub test_start_time: Option<Instant>,
    pub current_test_phase: String,
    pub total_test_phases: usize,
    pub current_phase_index: usize,
    pub animation_frame: usize,
    pub single_core_current_score: u32,
    pub multi_core_current_score: u32,
    pub estimated_single_core: u32,
    pub estimated_multi_core: u32,
}

impl Default for CpuTestInfo {
    fn default() -> Self {
        Self {
            is_testing: false,
            current_test: String::new(),
            progress: 0,
            results: Vec::new(),
            last_update: Instant::now(),
            error_message: None,
            cpu_info: get_cpu_info(),
            test_start_time: None,
            current_test_phase: String::new(),
            total_test_phases: 8, // 总共8个测试阶段
            current_phase_index: 0,
            animation_frame: 0,
            single_core_current_score: 0,
            multi_core_current_score: 0,
            estimated_single_core: 0,
            estimated_multi_core: 0,
        }
    }
}

// 全局状态管理
static CPU_TEST_INFO: Mutex<Option<CpuTestInfo>> = Mutex::new(None);
static NEEDS_UI_REFRESH: AtomicBool = AtomicBool::new(false);
static CPU_TEST_STARTED: AtomicBool = AtomicBool::new(false);

/// 获取CPU测试信息
pub fn get_info() -> String {
    if let Ok(global_info) = CPU_TEST_INFO.lock() {
        if let Some(ref info) = global_info.as_ref() {
            return format_cpu_test_info(info);
        }
    }
    
    // 首次创建CPU测试信息时，自动启动测试
    let mut info = CpuTestInfo::default();
    info.cpu_info = get_cpu_info();
    
    let formatted = format_cpu_test_info(&info);
    
    // 保存到全局状态
    if let Ok(mut global_info) = CPU_TEST_INFO.lock() {
        *global_info = Some(info);
    }
    
    // 自动启动CPU测试
    start_cpu_test();
    
    formatted
}

/// 获取当前CPU测试状态
pub fn get_current_test_info() -> CpuTestInfo {
    if let Ok(global_info) = CPU_TEST_INFO.lock() {
        if let Some(info) = global_info.as_ref() {
            return info.clone();
        }
    }
    CpuTestInfo::default()
}

/// 启动CPU测试
pub fn start_cpu_test() {
    if CPU_TEST_STARTED.swap(true, Ordering::Relaxed) {
        return; // 测试已在进行中
    }
    
    // 初始化测试状态
    if let Ok(mut global_info) = CPU_TEST_INFO.lock() {
        let mut info = CpuTestInfo::default();
        info.is_testing = true;
        info.test_start_time = Some(Instant::now());
        *global_info = Some(info);
    }
    
    // 在新线程中运行测试
    thread::spawn(|| {
        run_async_cpu_tests();
    });
}

/// 检查是否需要刷新UI
pub fn check_needs_refresh() -> bool {
    NEEDS_UI_REFRESH.swap(false, Ordering::Relaxed)
}

/// 更新动画帧
pub fn update_animation_frame() {
    if let Ok(mut global_info) = CPU_TEST_INFO.lock() {
        if let Some(ref mut info) = global_info.as_mut() {
            if info.is_testing {
                info.animation_frame = (info.animation_frame + 1) % 8;
                info.last_update = Instant::now();
                NEEDS_UI_REFRESH.store(true, Ordering::Relaxed);
            }
        }
    }
}

// 内部函数实现

fn run_async_cpu_tests() {
    // 更严谨的测试阶段定义
    let test_phases = vec![
        ("🔢 整数运算基准", 10),
        ("🧮 浮点运算基准", 25),
        ("⚡ SIMD矢量计算", 40),
        ("🔐 加密哈希运算", 55),
        ("📦 数据压缩算法", 70),
        ("💾 内存带宽测试", 85),
        ("🎯 多线程并发", 95),
    ];
    
    let mut all_results = Vec::new();
    
    for (i, (phase_name, progress)) in test_phases.iter().enumerate() {
        update_test_status(phase_name, *progress, true);
        update_test_phase(phase_name, i);
        
        // 执行严谨的基准测试
        let result = match i {
            0 => cpu_benchmarks::run_rigorous_integer_benchmark(),
            1 => cpu_benchmarks::run_rigorous_floating_point_benchmark(),
            2 => cpu_benchmarks::run_rigorous_simd_benchmark(),
            3 => cpu_benchmarks::run_rigorous_cryptographic_benchmark(),
            4 => cpu_benchmarks::run_rigorous_compression_benchmark(),
            5 => cpu_benchmarks::run_rigorous_memory_benchmark(),
            6 => cpu_benchmarks::run_rigorous_multithreading_benchmark(),
            _ => None,
        };
        
        if let Some(test_result) = result {
            all_results.push(test_result);
        }
        
        // 更新估计分数
        update_estimated_scores(&all_results);
    }
    
    // 计算最终分数（使用Geekbench风格的加权）
    let final_result = cpu_benchmarks::calculate_geekbench_scores(&all_results);
    all_results.push(final_result);
    
    // 完成测试
    update_test_status("CPU测试完成", 100, false);
    update_test_results(all_results);
    CPU_TEST_STARTED.store(false, Ordering::Relaxed);
}

fn update_test_status(current_test: &str, progress: u8, is_testing: bool) {
    if let Ok(mut global_info) = CPU_TEST_INFO.lock() {
        if let Some(ref mut info) = global_info.as_mut() {
            info.current_test = current_test.to_string();
            info.progress = progress.min(100); // 确保进度不超过100%
            info.is_testing = is_testing;
            info.last_update = Instant::now();
            
            if is_testing {
                if info.test_start_time.is_none() {
                    info.test_start_time = Some(Instant::now());
                }
                info.current_test_phase = current_test.to_string();
                info.animation_frame = (info.animation_frame + 1) % 8;
            }
            
            NEEDS_UI_REFRESH.store(true, Ordering::Relaxed);
        }
    }
}

fn update_test_phase(phase_name: &str, phase_index: usize) {
    if let Ok(mut global_info) = CPU_TEST_INFO.lock() {
        if let Some(ref mut info) = global_info.as_mut() {
            info.current_phase_index = phase_index;
            info.current_test_phase = phase_name.to_string();
            NEEDS_UI_REFRESH.store(true, Ordering::Relaxed);
        }
    }
}

fn update_test_results(results: Vec<CpuTestResult>) {
    if let Ok(mut global_info) = CPU_TEST_INFO.lock() {
        if let Some(ref mut info) = global_info.as_mut() {
            info.results = results;
            NEEDS_UI_REFRESH.store(true, Ordering::Relaxed);
        }
    }
}

fn update_estimated_scores(results: &[CpuTestResult]) {
    if let Ok(mut global_info) = CPU_TEST_INFO.lock() {
        if let Some(ref mut info) = global_info.as_mut() {
            // 基于已完成的测试估算最终分数
            let completed_tests = results.len() as f64;
            let _total_tests = 7.0; // 不包括准备阶段
            
            if completed_tests > 0.0 {
                let avg_single = results.iter().map(|r| r.single_core_score as f64).sum::<f64>() / completed_tests;
                let avg_multi = results.iter().map(|r| r.multi_core_score as f64).sum::<f64>() / completed_tests;
                
                // 估算最终分数（基于平均值和权重）
                info.estimated_single_core = (avg_single * 1.2) as u32; // 加权估算
                info.estimated_multi_core = (avg_multi * 1.1) as u32;
                
                // 更新当前分数为最新测试的结果
                if let Some(latest) = results.last() {
                    info.single_core_current_score = latest.single_core_score;
                    info.multi_core_current_score = latest.multi_core_score;
                }
            }
            
            NEEDS_UI_REFRESH.store(true, Ordering::Relaxed);
        }
    }
}

fn update_test_status_with_error(error: String) {
    if let Ok(mut global_info) = CPU_TEST_INFO.lock() {
        if let Some(ref mut info) = global_info.as_mut() {
            info.error_message = Some(error);
            info.is_testing = false;
            info.progress = 0;
            NEEDS_UI_REFRESH.store(true, Ordering::Relaxed);
        }
    }
    CPU_TEST_STARTED.store(false, Ordering::Relaxed);
}

fn calculate_final_scores(results: &[CpuTestResult]) -> CpuTestResult {
    let weights = HashMap::from([
        ("整数运算测试", 1.2),
        ("浮点运算测试", 1.3),
        ("矢量运算测试", 1.1),
        ("加密算法测试", 1.0),
        ("压缩算法测试", 1.0),
        ("内存带宽测试", 0.8),
        ("综合性能测试", 1.4),
    ]);
    
    let mut weighted_single = 0.0;
    let mut weighted_multi = 0.0;
    let mut total_weight = 0.0;
    
    for result in results {
        if let Some(&weight) = weights.get(result.test_name.as_str()) {
            weighted_single += result.single_core_score as f64 * weight;
            weighted_multi += result.multi_core_score as f64 * weight;
            total_weight += weight;
        }
    }
    
    let final_single = if total_weight > 0.0 {
        (weighted_single / total_weight) as u32
    } else {
        0
    };
    
    let final_multi = if total_weight > 0.0 {
        (weighted_multi / total_weight) as u32
    } else {
        0
    };
    
    let mut details = HashMap::new();
    details.insert("测试项目数".to_string(), results.len().to_string());
    details.insert("总耗时".to_string(), format!("{}秒", results.iter().map(|r| r.duration_ms).sum::<u64>() / 1000));
    details.insert("平均单核".to_string(), format!("{}", results.iter().map(|r| r.single_core_score).sum::<u32>() / results.len() as u32));
    details.insert("平均多核".to_string(), format!("{}", results.iter().map(|r| r.multi_core_score).sum::<u32>() / results.len() as u32));
    
    CpuTestResult {
        test_name: "综合评分".to_string(),
        single_core_score: final_single,
        multi_core_score: final_multi,
        duration_ms: results.iter().map(|r| r.duration_ms).sum(),
        details,
    }
}

fn get_cpu_info() -> String {
    let mut info = String::new();
    
    // 获取CPU信息
    if let Ok(contents) = std::fs::read_to_string("/proc/cpuinfo") {
        let lines: Vec<&str> = contents.lines().collect();
        
        // 提取关键信息
        let model_name = lines.iter()
            .find(|line| line.starts_with("model name"))
            .and_then(|line| line.split(':').nth(1))
            .map(|s| s.trim())
            .unwrap_or("未知");
            
        let cpu_cores = lines.iter()
            .find(|line| line.starts_with("cpu cores"))
            .and_then(|line| line.split(':').nth(1))
            .map(|s| s.trim())
            .unwrap_or("未知");
            
        let cpu_mhz = lines.iter()
            .find(|line| line.starts_with("cpu MHz"))
            .and_then(|line| line.split(':').nth(1))
            .map(|s| s.trim())
            .unwrap_or("未知");
            
        info.push_str(&format!("处理器: {}\n", model_name));
        info.push_str(&format!("核心数: {} 核\n", cpu_cores));
        info.push_str(&format!("当前频率: {} MHz\n", cpu_mhz));
    }
    
    // 获取逻辑核心数
    let logical_cores = num_cpus::get();
    info.push_str(&format!("逻辑核心: {} 个\n", logical_cores));
    
    info
}

fn format_cpu_test_info(info: &CpuTestInfo) -> String {
    let mut output = String::new();
    
    output.push_str("=== Geekbench 风格 CPU 性能测试 ===\n\n");
    
    if info.is_testing {
        let clamped_progress = info.progress.min(100);
        output.push_str(&format!("状态: {}\n", info.current_test));
        output.push_str(&format!("进度: {}%\n", clamped_progress));
        
        // 进度条
        let progress_bar_length = 40;
        let filled = (clamped_progress as usize * progress_bar_length) / 100;
        let empty = progress_bar_length - filled;
        output.push_str(&format!("[{}{}]\n\n", 
            "█".repeat(filled),
            "░".repeat(empty)
        ));
        
        // 实时分数估算
        if info.estimated_single_core > 0 {
            output.push_str("预估分数:\n");
            output.push_str(&format!("单核心: {} 分\n", info.estimated_single_core));
            output.push_str(&format!("多核心: {} 分\n\n", info.estimated_multi_core));
        }
        
        output.push_str("正在执行CPU性能测试，请稍候...\n");
    } else if let Some(ref error) = info.error_message {
        output.push_str(&format!("错误: {}\n\n", error));
        output.push_str(&info.cpu_info);
    } else if !info.results.is_empty() {
        output.push_str("测试完成！\n\n");
        output.push_str("CPU 性能测试结果:\n");
        output.push_str("=".repeat(60).as_str());
        output.push_str("\n\n");
        
        // 显示最终综合评分
        if let Some(final_result) = info.results.iter().find(|r| r.test_name == "综合评分") {
            output.push_str(&format!("🏆 综合评分:\n"));
            output.push_str(&format!("   单核心: {} 分\n", final_result.single_core_score));
            output.push_str(&format!("   多核心: {} 分\n\n", final_result.multi_core_score));
        }
        
        // 显示详细测试结果
        output.push_str("详细测试结果:\n");
        output.push_str("-".repeat(60).as_str());
        output.push_str("\n\n");
        
        for result in &info.results {
            if result.test_name != "综合评分" {
                output.push_str(&format!("{}:\n", result.test_name));
                output.push_str(&format!("  单核: {} 分 | 多核: {} 分 | 耗时: {}ms\n\n", 
                    result.single_core_score, 
                    result.multi_core_score,
                    result.duration_ms
                ));
            }
        }
        
        output.push_str("\n💡 说明: 分数越高表示性能越好。\n");
        output.push_str("这些分数可以与其他设备进行对比。\n");
    } else {
        output.push_str("准备开始CPU测试...\n\n");
        output.push_str(&info.cpu_info);
        output.push_str("\n");
        output.push_str("测试项目:\n");
        output.push_str("• 整数运算性能\n");
        output.push_str("• 浮点运算性能\n");
        output.push_str("• 矢量运算性能\n");
        output.push_str("• 加密算法性能\n");
        output.push_str("• 压缩算法性能\n");
        output.push_str("• 内存带宽测试\n");
        output.push_str("• 综合性能评估\n\n");
        output.push_str("💡 提示: 测试将持续约30-60秒，期间CPU使用率会较高。\n");
    }
    
    output
}

// CPU基准测试模块
mod cpu_benchmarks {
    use super::*;
    use std::f64::consts::PI;
    use rand::prelude::*;
    
    pub fn run_integer_test() -> Option<CpuTestResult> {
        let start_time = Instant::now();
        
        // 单核心整数测试
        let single_score = {
            let iterations = 1_000_000;
            let start = Instant::now();
            
            let mut result = 0u64;
            for i in 0..iterations {
                result = result.wrapping_add(fibonacci_iterative(i % 20));
                result = result.wrapping_mul(prime_check(i % 1000 + 1));
            }
            
            let duration = start.elapsed().as_millis() as f64;
            ((iterations as f64 / duration) * 10.0) as u32
        };
        
        // 多核心整数测试
        let multi_score = {
            let iterations = 1_000_000;
            let start = Instant::now();
            
            let _result: u64 = (0..iterations)
                .into_par_iter()
                .map(|i| {
                    let mut res = fibonacci_iterative(i % 20);
                    res = res.wrapping_mul(prime_check(i % 1000 + 1));
                    res
                })
                .sum();
            
            let duration = start.elapsed().as_millis() as f64;
            ((iterations as f64 / duration) * 10.0 * num_cpus::get() as f64) as u32
        };
        
        Some(CpuTestResult {
            test_name: "整数运算测试".to_string(),
            single_core_score: single_score,
            multi_core_score: multi_score,
            duration_ms: start_time.elapsed().as_millis() as u64,
            details: HashMap::new(),
        })
    }
    
    pub fn run_floating_point_test() -> Option<CpuTestResult> {
        let start_time = Instant::now();
        
        // 单核心浮点测试
        let single_score = {
            let iterations = 500_000;
            let start = Instant::now();
            
            let mut _result = 0.0f64;
            for i in 0..iterations {
                let x = i as f64 * PI / 180.0;
                _result += (x.sin() * x.cos()).sqrt() + x.tan().abs();
            }
            
            let duration = start.elapsed().as_millis() as f64;
            ((iterations as f64 / duration) * 20.0) as u32
        };
        
        // 多核心浮点测试
        let multi_score = {
            let iterations = 500_000;
            let start = Instant::now();
            
            let _result: f64 = (0..iterations)
                .into_par_iter()
                .map(|i| {
                    let x = i as f64 * PI / 180.0;
                    (x.sin() * x.cos()).sqrt() + x.tan().abs()
                })
                .sum();
            
            let duration = start.elapsed().as_millis() as f64;
            ((iterations as f64 / duration) * 20.0 * num_cpus::get() as f64) as u32
        };
        
        Some(CpuTestResult {
            test_name: "浮点运算测试".to_string(),
            single_core_score: single_score,
            multi_core_score: multi_score,
            duration_ms: start_time.elapsed().as_millis() as u64,
            details: HashMap::new(),
        })
    }
    
    pub fn run_vector_test() -> Option<CpuTestResult> {
        let start_time = Instant::now();
        
        // 矢量运算测试（模拟SIMD操作）
        let single_score = {
            let size = 100_000;
            let start = Instant::now();
            
            let mut vec1: Vec<f32> = (0..size).map(|i| i as f32).collect();
            let vec2: Vec<f32> = (0..size).map(|i| (i * 2) as f32).collect();
            
            for _ in 0..100 {
                for i in 0..size {
                    vec1[i] = (vec1[i] * vec2[i]).sqrt() + vec1[i].sin();
                }
            }
            
            let duration = start.elapsed().as_millis() as f64;
            ((size as f64 * 100.0 / duration) * 5.0) as u32
        };
        
        // 多核心矢量测试
        let multi_score = {
            let size = 100_000;
            let start = Instant::now();
            
            let mut vec1: Vec<f32> = (0..size).map(|i| i as f32).collect();
            let vec2: Vec<f32> = (0..size).map(|i| (i * 2) as f32).collect();
            
            for _ in 0..100 {
                vec1.par_iter_mut()
                    .zip(vec2.par_iter())
                    .for_each(|(a, b)| {
                        *a = (*a * b).sqrt() + a.sin();
                    });
            }
            
            let duration = start.elapsed().as_millis() as f64;
            ((size as f64 * 100.0 / duration) * 5.0 * num_cpus::get() as f64) as u32
        };
        
        Some(CpuTestResult {
            test_name: "矢量运算测试".to_string(),
            single_core_score: single_score,
            multi_core_score: multi_score,
            duration_ms: start_time.elapsed().as_millis() as u64,
            details: HashMap::new(),
        })
    }
    
    pub fn run_encryption_test() -> Option<CpuTestResult> {
        let start_time = Instant::now();
        
        // 模拟加密操作（简单的哈希计算）
        let single_score = {
            let iterations = 50_000;
            let start = Instant::now();
            
            let mut hasher = std::collections::hash_map::DefaultHasher::new();
            for i in 0..iterations {
                use std::hash::{Hash, Hasher};
                let data = format!("test_data_{}", i);
                data.hash(&mut hasher);
                let _hash = hasher.finish();
            }
            
            let duration = start.elapsed().as_millis() as f64;
            ((iterations as f64 / duration) * 30.0) as u32
        };
        
        // 多核心加密测试
        let multi_score = {
            let iterations = 50_000;
            let start = Instant::now();
            
            let _results: Vec<u64> = (0..iterations)
                .into_par_iter()
                .map(|i| {
                    use std::hash::{Hash, Hasher};
                    let mut hasher = std::collections::hash_map::DefaultHasher::new();
                    let data = format!("test_data_{}", i);
                    data.hash(&mut hasher);
                    hasher.finish()
                })
                .collect();
            
            let duration = start.elapsed().as_millis() as f64;
            ((iterations as f64 / duration) * 30.0 * num_cpus::get() as f64) as u32
        };
        
        Some(CpuTestResult {
            test_name: "加密算法测试".to_string(),
            single_core_score: single_score,
            multi_core_score: multi_score,
            duration_ms: start_time.elapsed().as_millis() as u64,
            details: HashMap::new(),
        })
    }
    
    pub fn run_compression_test() -> Option<CpuTestResult> {
        let start_time = Instant::now();
        
        // 模拟压缩算法（字符串压缩）
        let test_data = "a".repeat(10000) + &"b".repeat(10000) + &"c".repeat(10000);
        
        let single_score = {
            let iterations = 1000;
            let start = Instant::now();
            
            for _ in 0..iterations {
                let _compressed = simple_compress(&test_data);
            }
            
            let duration = start.elapsed().as_millis() as f64;
            ((iterations as f64 / duration) * 25.0) as u32
        };
        
        // 多核心压缩测试
        let multi_score = {
            let iterations = 1000;
            let start = Instant::now();
            
            let _results: Vec<String> = (0..iterations)
                .into_par_iter()
                .map(|_| simple_compress(&test_data))
                .collect();
            
            let duration = start.elapsed().as_millis() as f64;
            ((iterations as f64 / duration) * 25.0 * num_cpus::get() as f64) as u32
        };
        
        Some(CpuTestResult {
            test_name: "压缩算法测试".to_string(),
            single_core_score: single_score,
            multi_core_score: multi_score,
            duration_ms: start_time.elapsed().as_millis() as u64,
            details: HashMap::new(),
        })
    }
    
    pub fn run_memory_bandwidth_test() -> Option<CpuTestResult> {
        let start_time = Instant::now();
        
        // 内存带宽测试
        let single_score = {
            let size = 10_000_000usize; // 10M elements
            let start = Instant::now();
            
            let mut data: Vec<u64> = (0..size).map(|i| i as u64).collect();
            
            // 顺序访问
            for _ in 0..10 {
                for i in 0..size {
                    data[i] = data[i].wrapping_mul(2).wrapping_add(1);
                }
            }
            
            let duration = start.elapsed().as_millis() as f64;
            ((size as f64 * 10.0 / duration) * 0.1) as u32
        };
        
        // 多核心内存测试
        let multi_score = {
            let size = 10_000_000usize;
            let start = Instant::now();
            
            let mut data: Vec<u64> = (0..size).map(|i| i as u64).collect();
            
            for _ in 0..10 {
                data.par_iter_mut().for_each(|x| {
                    *x = x.wrapping_mul(2).wrapping_add(1);
                });
            }
            
            let duration = start.elapsed().as_millis() as f64;
            ((size as f64 * 10.0 / duration) * 0.1 * num_cpus::get() as f64) as u32
        };
        
        Some(CpuTestResult {
            test_name: "内存带宽测试".to_string(),
            single_core_score: single_score,
            multi_core_score: multi_score,
            duration_ms: start_time.elapsed().as_millis() as u64,
            details: HashMap::new(),
        })
    }
    
    pub fn run_comprehensive_test() -> Option<CpuTestResult> {
        let start_time = Instant::now();
        
        // 综合测试（混合多种操作）
        let single_score = {
            let start = Instant::now();
            let mut rng = thread_rng();
            
            let mut result = 0u64;
            for i in 0..100_000 {
                // 混合整数、浮点、内存操作
                let x = rng.gen::<f64>();
                let int_op = fibonacci_iterative(i % 15);
                let float_op = (x * PI).sin();
                result = result.wrapping_add(int_op).wrapping_add(float_op as u64);
            }
            
            let duration = start.elapsed().as_millis() as f64;
            ((100_000.0 / duration) * 15.0) as u32
        };
        
        // 多核心综合测试
        let multi_score = {
            let start = Instant::now();
            
            let _result: u64 = (0..100_000)
                .into_par_iter()
                .map(|i| {
                    let mut rng = thread_rng();
                    let x = rng.gen::<f64>();
                    let int_op = fibonacci_iterative(i % 15);
                    let float_op = (x * PI).sin();
                    int_op.wrapping_add(float_op as u64)
                })
                .sum();
            
            let duration = start.elapsed().as_millis() as f64;
            ((100_000.0 / duration) * 15.0 * num_cpus::get() as f64) as u32
        };
        
        Some(CpuTestResult {
            test_name: "综合性能测试".to_string(),
            single_core_score: single_score,
            multi_core_score: multi_score,
            duration_ms: start_time.elapsed().as_millis() as u64,
            details: HashMap::new(),
        })
    }
    
    // 辅助函数
    fn fibonacci_iterative(n: u64) -> u64 {
        if n <= 1 { return n; }
        let mut a = 0;
        let mut b = 1;
        for _ in 2..=n {
            let temp = a + b;
            a = b;
            b = temp;
        }
        b
    }
    
    fn prime_check(n: u64) -> u64 {
        if n < 2 { return 0; }
        for i in 2..=(n as f64).sqrt() as u64 {
            if n % i == 0 { return 0; }
        }
        1
    }
    
    fn simple_compress(data: &str) -> String {
        // 简单的RLE压缩
        let mut result = String::new();
        let chars: Vec<char> = data.chars().collect();
        if chars.is_empty() { return result; }
        
        let mut current_char = chars[0];
        let mut count = 1;
        
        for &ch in &chars[1..] {
            if ch == current_char {
                count += 1;
            } else {
                result.push_str(&format!("{}{}", count, current_char));
                current_char = ch;
                count = 1;
            }
        }
        result.push_str(&format!("{}{}", count, current_char));
        result
    }
    
    // ============ 严谨的CPU基准测试算法 ============
    
    /// 严谨的整数运算基准测试（基于素数筛选和斐波那契数列）
    pub fn run_rigorous_integer_benchmark() -> Option<CpuTestResult> {
        let start_time = Instant::now();
        
        // 单核测试：埃拉托斯特尼筛法 + 斐波那契数列
        let single_core_score = {
            let test_start = Instant::now();
            let _primes_count = sieve_of_eratosthenes(1_000_000); // 增加到100万
            let _fib_result = fibonacci_recursive(40); // 增加到40
            let elapsed_ms = test_start.elapsed().as_millis() as f64;
            
            // 基于GeekBench 5的计分标准：基准分1000分，基准时间约5秒
            let base_time_ms = 5000.0; // 基准时间5秒
            let base_score = 1000.0;   // 基准分数1000分
            
            // 分数与时间成反比，时间越短分数越高
            if elapsed_ms > 0.0 {
                (base_score * base_time_ms / elapsed_ms) as u32
            } else {
                1000
            }
        };
        
        // 多核测试：并行素数筛选
        let multi_core_score = {
            use rayon::prelude::*;
            let test_start = Instant::now();
            
            // 增加计算量，让测试更有挑战性
            let ranges: Vec<_> = (0..16).map(|i| (i * 62500, (i + 1) * 62500)).collect();
            let _total_primes: usize = ranges.par_iter()
                .map(|&(start, end)| count_primes_in_range(start, end))
                .sum();
                
            let elapsed_ms = test_start.elapsed().as_millis() as f64;
            
            // 多核基准分数通常是单核的4-8倍
            let base_time_ms = 2000.0; // 多核基准时间2秒
            let base_score = 4000.0;   // 多核基准分数4000分
            
            if elapsed_ms > 0.0 {
                (base_score * base_time_ms / elapsed_ms) as u32
            } else {
                4000
            }
        };
        
        Some(CpuTestResult {
            test_name: "整数运算基准".to_string(),
            single_core_score,
            multi_core_score,
            duration_ms: start_time.elapsed().as_millis() as u64,
            details: HashMap::new(),
        })
    }
    
    /// 严谨的浮点运算基准测试（科学计算和三角函数）
    pub fn run_rigorous_floating_point_benchmark() -> Option<CpuTestResult> {
        let start_time = Instant::now();
        
        // 单核测试：蒙特卡洛方法计算π + 复杂三角函数
        let single_core_score = {
            let test_start = Instant::now();
            let _pi_estimate = monte_carlo_pi(5_000_000); // 增加到500万次
            let _trig_sum = complex_trigonometric_operations(1_000_000); // 增加到100万次
            let elapsed_ms = test_start.elapsed().as_millis() as f64;
            
            // 浮点运算基准分数
            let base_time_ms = 4000.0; // 基准时间4秒
            let base_score = 1200.0;   // 基准分数1200分
            
            if elapsed_ms > 0.0 {
                (base_score * base_time_ms / elapsed_ms) as u32
            } else {
                1200
            }
        };
        
        // 多核测试：并行蒙特卡洛
        let multi_score = {
            use rayon::prelude::*;
            let test_start = Instant::now();
            
            let _pi_estimates: Vec<f64> = (0..8).into_par_iter()
                .map(|_| monte_carlo_pi(1_250_000)) // 每个线程125万次
                .collect();
            
            let elapsed_ms = test_start.elapsed().as_millis() as f64;
            
            let base_time_ms = 1500.0; // 多核基准时间1.5秒
            let base_score = 5000.0;   // 多核基准分数5000分
            
            if elapsed_ms > 0.0 {
                (base_score * base_time_ms / elapsed_ms) as u32
            } else {
                5000
            }
        };
        
        Some(CpuTestResult {
            test_name: "浮点运算基准".to_string(),
            single_core_score,
            multi_core_score: multi_score,
            duration_ms: start_time.elapsed().as_millis() as u64,
            details: HashMap::new(),
        })
    }
    
    /// 严谨的SIMD矢量计算基准测试
    pub fn run_rigorous_simd_benchmark() -> Option<CpuTestResult> {
        let start_time = Instant::now();
        
        // 模拟SIMD操作：向量点积和矩阵乘法
        let single_core_score = {
            let test_start = Instant::now();
            let _vector_ops = vector_dot_product_operations(500_000); // 增加到50万
            let _matrix_ops = matrix_multiplication_benchmark(200); // 增加到200x200矩阵
            let elapsed_ms = test_start.elapsed().as_millis() as f64;
            
            let base_time_ms = 3000.0; // 基准时间3秒
            let base_score = 900.0;    // 基准分数900分
            
            if elapsed_ms > 0.0 {
                (base_score * base_time_ms / elapsed_ms) as u32
            } else {
                900
            }
        };
        
        // 多核SIMD测试
        let multi_score = {
            use rayon::prelude::*;
            let test_start = Instant::now();
            
            let _results: Vec<f64> = (0..8).into_par_iter()
                .map(|_| vector_dot_product_operations(125_000) + matrix_multiplication_benchmark(100))
                .collect();
            
            let elapsed_ms = test_start.elapsed().as_millis() as f64;
            
            let base_time_ms = 1200.0; // 多核基准时间1.2秒
            let base_score = 4500.0;   // 多核基准分数4500分
            
            if elapsed_ms > 0.0 {
                (base_score * base_time_ms / elapsed_ms) as u32
            } else {
                4500
            }
        };
        
        Some(CpuTestResult {
            test_name: "SIMD矢量计算".to_string(),
            single_core_score,
            multi_core_score: multi_score,
            duration_ms: start_time.elapsed().as_millis() as u64,
            details: HashMap::new(),
        })
    }
    
    /// 严谨的加密哈希运算基准测试
    pub fn run_rigorous_cryptographic_benchmark() -> Option<CpuTestResult> {
        let start_time = Instant::now();
        
        // 单核测试：SHA-256模拟 + 简单加密操作
        let single_core_score = {
            let test_start = Instant::now();
            let _hash_ops = sha256_simulation_benchmark(100_000); // 增加到10万次
            let _encrypt_ops = simple_encryption_benchmark(500_000); // 增加到50万次
            let elapsed_ms = test_start.elapsed().as_millis() as f64;
            
            let base_time_ms = 3500.0; // 基准时间3.5秒
            let base_score = 800.0;    // 基准分数800分
            
            if elapsed_ms > 0.0 {
                (base_score * base_time_ms / elapsed_ms) as u32
            } else {
                800
            }
        };
        
        // 多核测试：并行哈希计算
        let multi_score = {
            use rayon::prelude::*;
            let test_start = Instant::now();
            
            let _results: Vec<f64> = (0..8).into_par_iter()
                .map(|_| sha256_simulation_benchmark(25_000) + simple_encryption_benchmark(125_000) as f64)
                .collect();
            
            let elapsed_ms = test_start.elapsed().as_millis() as f64;
            
            let base_time_ms = 1800.0; // 多核基准时间1.8秒
            let base_score = 3800.0;   // 多核基准分数3800分
            
            if elapsed_ms > 0.0 {
                (base_score * base_time_ms / elapsed_ms) as u32
            } else {
                3800
            }
        };
        
        Some(CpuTestResult {
            test_name: "加密哈希运算".to_string(),
            single_core_score,
            multi_core_score: multi_score,
            duration_ms: start_time.elapsed().as_millis() as u64,
            details: HashMap::new(),
        })
    }
    
    /// 严谨的数据压缩算法基准测试
    pub fn run_rigorous_compression_benchmark() -> Option<CpuTestResult> {
        let start_time = Instant::now();
        
        // 单核测试：LZ77风格压缩 + 霍夫曼编码模拟
        let single_core_score = {
            let test_start = Instant::now();
            let _compression_ratio = lz77_compression_simulation(100_000); // 增加到10万
            let _huffman_ops = huffman_encoding_simulation(200_000); // 增加到20万
            let elapsed_ms = test_start.elapsed().as_millis() as f64;
            
            let base_time_ms = 4500.0; // 基准时间4.5秒
            let base_score = 700.0;    // 基准分数700分
            
            if elapsed_ms > 0.0 {
                (base_score * base_time_ms / elapsed_ms) as u32
            } else {
                700
            }
        };
        
        // 多核测试：并行数据压缩
        let multi_score = {
            use rayon::prelude::*;
            let test_start = Instant::now();
            
            let _results: Vec<f64> = (0..8).into_par_iter()
                .map(|_| lz77_compression_simulation(25_000) + huffman_encoding_simulation(50_000))
                .collect();
            
            let elapsed_ms = test_start.elapsed().as_millis() as f64;
            
            let base_time_ms = 2000.0; // 多核基准时间2秒
            let base_score = 3200.0;   // 多核基准分数3200分
            
            if elapsed_ms > 0.0 {
                (base_score * base_time_ms / elapsed_ms) as u32
            } else {
                3200
            }
        };
        
        Some(CpuTestResult {
            test_name: "数据压缩算法".to_string(),
            single_core_score,
            multi_core_score: multi_score,
            duration_ms: start_time.elapsed().as_millis() as u64,
            details: HashMap::new(),
        })
    }
    
    /// 严谨的内存带宽基准测试
    pub fn run_rigorous_memory_benchmark() -> Option<CpuTestResult> {
        let start_time = Instant::now();
        
        // 单核测试：内存密集型操作
        let single_core_score = {
            let test_start = Instant::now();
            let _memory_ops = memory_intensive_operations(1_000_000); // 增加到100万
            let _cache_performance = cache_performance_test(500_000); // 增加到50万
            let elapsed_ms = test_start.elapsed().as_millis() as f64;
            
            let base_time_ms = 6000.0; // 基准时间6秒
            let base_score = 600.0;    // 基准分数600分
            
            if elapsed_ms > 0.0 {
                (base_score * base_time_ms / elapsed_ms) as u32
            } else {
                600
            }
        };
        
        // 多核测试：并行内存访问
        let multi_score = {
            use rayon::prelude::*;
            let test_start = Instant::now();
            
            let _results: Vec<f64> = (0..8).into_par_iter()
                .map(|_| memory_intensive_operations(250_000) + cache_performance_test(125_000))
                .collect();
            
            let elapsed_ms = test_start.elapsed().as_millis() as f64;
            
            let base_time_ms = 2500.0; // 多核基准时间2.5秒
            let base_score = 2800.0;   // 多核基准分数2800分
            
            if elapsed_ms > 0.0 {
                (base_score * base_time_ms / elapsed_ms) as u32
            } else {
                2800
            }
        };
        
        Some(CpuTestResult {
            test_name: "内存带宽测试".to_string(),
            single_core_score,
            multi_core_score: multi_score,
            duration_ms: start_time.elapsed().as_millis() as u64,
            details: HashMap::new(),
        })
    }
    
    /// 严谨的多线程并发基准测试
    pub fn run_rigorous_multithreading_benchmark() -> Option<CpuTestResult> {
        let start_time = Instant::now();
        
        // 单核分数（作为基准）
        let single_core_score = {
            let test_start = Instant::now();
            let _sequential_ops = sequential_processing_benchmark(100_000); // 增加到10万
            let elapsed_ms = test_start.elapsed().as_millis() as f64;
            
            let base_time_ms = 5000.0; // 基准时间5秒
            let base_score = 500.0;    // 基准分数500分
            
            if elapsed_ms > 0.0 {
                (base_score * base_time_ms / elapsed_ms) as u32
            } else {
                500
            }
        };
        
        // 多核测试：真正的并发性能测试
        let multi_core_score = {
            use rayon::prelude::*;
            let test_start = Instant::now();
            
            // 模拟生产者-消费者模式和并发计算
            let _concurrent_results: Vec<f64> = (0..16).into_par_iter()
                .map(|_| concurrent_processing_benchmark(20_000)) // 增加到2万
                .collect();
            
            let elapsed_ms = test_start.elapsed().as_millis() as f64;
            
            let base_time_ms = 1500.0; // 多核基准时间1.5秒
            let base_score = 6000.0;   // 多核基准分数6000分
            
            if elapsed_ms > 0.0 {
                (base_score * base_time_ms / elapsed_ms) as u32
            } else {
                6000
            }
        };
        
        Some(CpuTestResult {
            test_name: "多线程并发".to_string(),
            single_core_score,
            multi_core_score,
            duration_ms: start_time.elapsed().as_millis() as u64,
            details: HashMap::new(),
        })
    }
    
    /// 计算GeekBench风格的加权综合分数
    pub fn calculate_geekbench_scores(results: &[CpuTestResult]) -> CpuTestResult {
        // GeekBench 5的权重分配（近似）
        let weights = [
            ("整数运算基准", 0.20),
            ("浮点运算基准", 0.20),
            ("SIMD矢量计算", 0.15),
            ("加密哈希运算", 0.15),
            ("数据压缩算法", 0.10),
            ("内存带宽测试", 0.10),
            ("多线程并发", 0.10),
        ];
        
        let mut weighted_single = 0.0;
        let mut weighted_multi = 0.0;
        let mut total_weight = 0.0;
        
        for result in results {
            if let Some((_, weight)) = weights.iter().find(|(name, _)| result.test_name.contains(name)) {
                weighted_single += result.single_core_score as f64 * weight;
                weighted_multi += result.multi_core_score as f64 * weight;
                total_weight += weight;
            }
        }
        
        // 确保权重归一化
        if total_weight > 0.0 {
            weighted_single /= total_weight;
            weighted_multi /= total_weight;
        }
        
        CpuTestResult {
            test_name: "综合评分".to_string(),
            single_core_score: weighted_single as u32,
            multi_core_score: weighted_multi as u32,
            duration_ms: 0,
            details: HashMap::new(),
        }
    }
    
    // ============ 基准测试辅助函数 ============
    
    /// 埃拉托斯特尼筛法计算素数
    fn sieve_of_eratosthenes(limit: usize) -> usize {
        let mut is_prime = vec![true; limit + 1];
        is_prime[0] = false;
        if limit > 0 { is_prime[1] = false; }
        
        for i in 2..=((limit as f64).sqrt() as usize) {
            if is_prime[i] {
                for j in ((i * i)..=limit).step_by(i) {
                    is_prime[j] = false;
                }
            }
        }
        
        is_prime.iter().filter(|&&x| x).count()
    }
    
    /// 计算指定范围内的素数个数
    fn count_primes_in_range(start: usize, end: usize) -> usize {
        (start..end).filter(|&n| is_prime(n)).count()
    }
    
    /// 判断是否为素数
    fn is_prime(n: usize) -> bool {
        if n < 2 { return false; }
        if n == 2 { return true; }
        if n % 2 == 0 { return false; }
        
        for i in (3..=((n as f64).sqrt() as usize)).step_by(2) {
            if n % i == 0 { return false; }
        }
        true
    }
    
    /// 递归斐波那契数列（CPU密集型）
    fn fibonacci_recursive(n: u32) -> u64 {
        match n {
            0 => 0,
            1 => 1,
            _ => fibonacci_recursive(n - 1) + fibonacci_recursive(n - 2),
        }
    }
    
    /// 蒙特卡洛方法估算π
    fn monte_carlo_pi(iterations: usize) -> f64 {
        let mut inside_circle = 0;
        for i in 0..iterations {
            let x = ((i * 1103515245 + 12345) % (1 << 31)) as f64 / (1 << 31) as f64;
            let y = (((i + 1) * 1103515245 + 12345) % (1 << 31)) as f64 / (1 << 31) as f64;
            
            if x * x + y * y <= 1.0 {
                inside_circle += 1;
            }
        }
        4.0 * inside_circle as f64 / iterations as f64
    }
    
    /// 复杂三角函数运算
    fn complex_trigonometric_operations(iterations: usize) -> f64 {
        let mut sum = 0.0;
        for i in 0..iterations {
            let x = i as f64 * 0.001;
            sum += (x.sin() * x.cos()).powf(0.5) + x.tan().abs().sqrt();
        }
        sum
    }
    
    /// 向量点积运算基准
    fn vector_dot_product_operations(size: usize) -> f64 {
        let vec_a: Vec<f64> = (0..size).map(|i| i as f64 * 0.1).collect();
        let vec_b: Vec<f64> = (0..size).map(|i| (i + 1) as f64 * 0.2).collect();
        
        vec_a.iter().zip(vec_b.iter()).map(|(a, b)| a * b).sum()
    }
    
    /// 矩阵乘法基准测试
    fn matrix_multiplication_benchmark(size: usize) -> f64 {
        let matrix_a: Vec<Vec<f64>> = (0..size).map(|i| 
            (0..size).map(|j| (i * size + j) as f64).collect()
        ).collect();
        
        let matrix_b: Vec<Vec<f64>> = (0..size).map(|i| 
            (0..size).map(|j| ((i + 1) * (j + 1)) as f64).collect()
        ).collect();
        
        let mut result = 0.0;
        for i in 0..size {
            for j in 0..size {
                for k in 0..size {
                    result += matrix_a[i][k] * matrix_b[k][j];
                }
            }
        }
        result
    }
    
    /// SHA-256模拟基准测试
    fn sha256_simulation_benchmark(iterations: usize) -> f64 {
        let mut hash_sum = 0u64;
        for i in 0..iterations {
            // 模拟SHA-256的部分计算过程
            let mut h = i as u64;
            for _ in 0..64 {
                h = h.wrapping_mul(1103515245).wrapping_add(12345);
                h ^= h >> 16;
                h = h.wrapping_mul(0x85ebca6b);
                h ^= h >> 13;
                h = h.wrapping_mul(0xc2b2ae35);
                h ^= h >> 16;
            }
            hash_sum = hash_sum.wrapping_add(h);
        }
        hash_sum as f64
    }
    
    /// 简单加密操作基准测试
    fn simple_encryption_benchmark(iterations: usize) -> u64 {
        let mut encrypted_sum = 0u64;
        let key = 0x1234567890ABCDEFu64;
        
        for i in 0..iterations {
            let data = i as u64;
            let encrypted = data ^ key;
            let rotated = encrypted.rotate_left(13);
            encrypted_sum = encrypted_sum.wrapping_add(rotated);
        }
        encrypted_sum
    }
    
    /// LZ77压缩算法模拟
    fn lz77_compression_simulation(data_size: usize) -> f64 {
        let data: Vec<u8> = (0..data_size).map(|i| (i % 256) as u8).collect();
        let mut compressed_size = 0;
        let mut i = 0;
        
        while i < data.len() {
            let mut best_length = 0;
            let mut _best_distance = 0;
            
            // 查找最长匹配
            for distance in 1..=i.min(255) {
                if i >= distance {
                    let mut length = 0;
                    while length < 255 && i + length < data.len() && 
                          data[i + length] == data[i - distance + length] {
                        length += 1;
                    }
                    if length > best_length {
                        best_length = length;
                        _best_distance = distance;
                    }
                }
            }
            
            if best_length >= 3 {
                compressed_size += 3; // 距离+长度+标志
                i += best_length;
            } else {
                compressed_size += 1; // 原始字节
                i += 1;
            }
        }
        
        data_size as f64 / compressed_size as f64 // 压缩率
    }
    
    /// 霍夫曼编码模拟
    fn huffman_encoding_simulation(data_size: usize) -> f64 {
        let data: Vec<u8> = (0..data_size).map(|i| (i % 256) as u8).collect();
        
        // 统计频率
        let mut freq = [0u32; 256];
        for &byte in &data {
            freq[byte as usize] += 1;
        }
        
        // 模拟霍夫曼树构建的计算复杂度
        let mut complexity = 0.0;
        for f in freq.iter() {
            if *f > 0 {
                complexity += (*f as f64) * (*f as f64).log2();
            }
        }
        
        complexity
    }
    
    /// 内存密集型操作基准
    fn memory_intensive_operations(size: usize) -> f64 {
        let mut data: Vec<f64> = (0..size).map(|i| i as f64).collect();
        
        // 随机访问模式
        for i in 0..size / 2 {
            let idx1 = (i * 1103515245 + 12345) % size;
            let idx2 = ((i + 1) * 1103515245 + 12345) % size;
            data.swap(idx1, idx2);
        }
        
        data.iter().sum()
    }
    
    /// 缓存性能测试
    fn cache_performance_test(size: usize) -> f64 {
        let data: Vec<f64> = (0..size).map(|i| i as f64).collect();
        let mut sum = 0.0;
        
        // 顺序访问（缓存友好）
        for &value in &data {
            sum += value;
        }
        
        // 随机访问（缓存不友好）
        for i in 0..size / 4 {
            let idx = (i * 1103515245 + 12345) % size;
            sum += data[idx];
        }
        
        sum
    }
    
    /// 顺序处理基准测试
    fn sequential_processing_benchmark(iterations: usize) -> f64 {
        let mut result = 0.0;
        for i in 0..iterations {
            result += (i as f64).sqrt() + (i as f64).sin();
        }
        result
    }
    
    /// 并发处理基准测试
    fn concurrent_processing_benchmark(iterations: usize) -> f64 {
        let mut result = 0.0;
        for i in 0..iterations {
            // 模拟复杂计算
            let x = i as f64;
            result += x.powf(2.5) + (x * 3.14159).cos() + (x / 2.718).ln_1p();
        }
        result
    }
}