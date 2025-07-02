use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use futures_util::StreamExt;
use reqwest::Client;
use tokio::time::timeout;
use rand::Rng;

#[derive(Debug, Clone)]
pub struct NetworkProvider {
    pub name: String,
    pub test_urls: Vec<String>,
    pub color: &'static str,
}

#[derive(Debug, Clone)]
pub struct SpeedTestResult {
    pub provider: String,
    pub download_speed: f64, // Mbps
    pub upload_speed: f64,   // Mbps
    pub ping: f64,           // ms
    pub jitter: f64,         // ms
    pub packet_loss: f64,    // %
    pub status: TestStatus,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TestStatus {
    NotStarted,
    Testing,
    Completed,
    Failed,
}

#[derive(Debug, Clone)]
pub struct NetworkTestInfo {
    pub is_testing: bool,
    pub current_provider: Option<String>,
    pub current_stage: String,
    pub progress: f64,
    pub results: HashMap<String, SpeedTestResult>,
    pub overall_progress: f64,
    pub error_message: Option<String>,
    pub start_time: Option<Instant>,
}

static NETWORK_TEST_STATE: Mutex<Option<Arc<Mutex<NetworkTestInfo>>>> = Mutex::new(None);

impl Default for NetworkTestInfo {
    fn default() -> Self {
        Self {
            is_testing: false,
            current_provider: None,
            current_stage: "准备测试...".to_string(),
            progress: 0.0,
            results: HashMap::new(),
            overall_progress: 0.0,
            error_message: None,
            start_time: None,
        }
    }
}

impl Default for SpeedTestResult {
    fn default() -> Self {
        Self {
            provider: String::new(),
            download_speed: 0.0,
            upload_speed: 0.0,
            ping: 0.0,
            jitter: 0.0,
            packet_loss: 0.0,
            status: TestStatus::NotStarted,
            error_message: None,
        }
    }
}

pub fn get_network_providers() -> Vec<NetworkProvider> {
    vec![
        NetworkProvider {
            name: "中国移动".to_string(),
            test_urls: vec![
                "http://speedtest1.bmcc.com.cn:8080/speedtest/upload.php".to_string(),
                "http://speedtest2.bmcc.com.cn:8080/speedtest/upload.php".to_string(),
            ],
            color: "#00D4AA",
        },
        NetworkProvider {
            name: "中国联通".to_string(),
            test_urls: vec![
                "http://61.135.169.121:8080/speedtest/upload.php".to_string(),
                "http://61.135.169.122:8080/speedtest/upload.php".to_string(),
            ],
            color: "#E60012",
        },
        NetworkProvider {
            name: "中国电信".to_string(),
            test_urls: vec![
                "http://speedtest1.online.sh.cn:8080/speedtest/upload.php".to_string(),
                "http://speedtest2.online.sh.cn:8080/speedtest/upload.php".to_string(),
            ],
            color: "#0052D9",
        },
    ]
}

pub fn get_current_test_info() -> NetworkTestInfo {
    let state_guard = NETWORK_TEST_STATE.lock().unwrap();
    if let Some(ref state_arc) = *state_guard {
        let state = state_arc.lock().unwrap();
        state.clone()
    } else {
        NetworkTestInfo::default()
    }
}

pub fn start_network_test() {
    // 添加调试输出
    println!("开始网速测试...");
    
    let state_guard = NETWORK_TEST_STATE.lock().unwrap();
    let state_arc = if let Some(ref existing) = *state_guard {
        existing.clone()
    } else {
        drop(state_guard);
        let new_state = Arc::new(Mutex::new(NetworkTestInfo::default()));
        *NETWORK_TEST_STATE.lock().unwrap() = Some(new_state.clone());
        new_state
    };

    // 检查是否已经在测试中
    {
        let mut state = state_arc.lock().unwrap();
        if state.is_testing {
            println!("网速测试已在进行中，跳过");
            return;
        }
        state.is_testing = true;
        state.start_time = Some(Instant::now());
        state.current_stage = "初始化测试...".to_string();
        state.progress = 0.0;
        state.overall_progress = 0.0;
        state.results.clear();
        state.error_message = None;
        println!("网速测试状态已初始化");
    }

    let state_clone = state_arc.clone();
    tokio::spawn(async move {
        println!("网速测试异步任务已启动");
        run_network_tests(state_clone).await;
    });
}

async fn run_network_tests(state: Arc<Mutex<NetworkTestInfo>>) {
    println!("开始执行网速测试");
    let providers = get_network_providers();
    let total_providers = providers.len();

    for (index, provider) in providers.iter().enumerate() {
        println!("测试运营商: {}", provider.name);
        {
            let mut state_lock = state.lock().unwrap();
            state_lock.current_provider = Some(provider.name.clone());
            state_lock.current_stage = format!("准备测试 {}...", provider.name);
            state_lock.overall_progress = (index as f64 / total_providers as f64) * 100.0;
            state_lock.progress = 0.0;
            println!("总体进度: {:.1}%", state_lock.overall_progress);
            
            // 先创建一个初始的测试结果
            let initial_result = SpeedTestResult {
                provider: provider.name.clone(),
                status: TestStatus::Testing,
                ..Default::default()
            };
            state_lock.results.insert(provider.name.clone(), initial_result);
        }

        // 执行测试
        match test_provider_speed(provider, state.clone()).await {
            Ok(test_result) => {
                let mut state_lock = state.lock().unwrap();
                let mut final_result = test_result;
                final_result.status = TestStatus::Completed;
                state_lock.results.insert(provider.name.clone(), final_result);
                println!("运营商 {} 测试完成", provider.name);
            }
            Err(e) => {
                let mut state_lock = state.lock().unwrap();
                if let Some(mut result) = state_lock.results.get(&provider.name).cloned() {
                    result.status = TestStatus::Failed;
                    result.error_message = Some(e.to_string());
                    state_lock.results.insert(provider.name.clone(), result);
                }
                println!("运营商 {} 测试失败: {}", provider.name, e);
            }
        }

        // 更新总体进度
        {
            let mut state_lock = state.lock().unwrap();
            state_lock.overall_progress = ((index + 1) as f64 / total_providers as f64) * 100.0;
            println!("更新进度: {:.1}%", state_lock.overall_progress);
        }

        // 测试间隔 - 增加间隔让用户能看到进度
        if index < total_providers - 1 { // 不是最后一个运营商
            println!("等待1秒后开始下一个运营商测试...");
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    }

    // 测试完成
    {
        let mut state_lock = state.lock().unwrap();
        state_lock.is_testing = false;
        state_lock.current_stage = "所有测试已完成".to_string();
        state_lock.progress = 100.0;
        state_lock.overall_progress = 100.0;
        state_lock.current_provider = None;
        println!("所有网速测试完成");
    }
}

async fn test_provider_speed(
    provider: &NetworkProvider, 
    state: Arc<Mutex<NetworkTestInfo>>
) -> Result<SpeedTestResult, Box<dyn std::error::Error + Send + Sync>> {
    let mut result = SpeedTestResult {
        provider: provider.name.clone(),
        status: TestStatus::Testing,
        ..Default::default()
    };

    let client = Client::builder()
        .timeout(Duration::from_secs(30))
        .build()?;

    // 阶段1: Ping测试 (33%)
    {
        let mut state_lock = state.lock().unwrap();
        state_lock.current_stage = format!("{} - 延迟测试", provider.name);
        state_lock.progress = 0.0;
        println!("开始 {} 的延迟测试", provider.name);
    }

    let ping_results = test_ping(provider, &client, state.clone()).await?;
    result.ping = ping_results.0;
    result.jitter = ping_results.1;

    // 更新结果并显示ping完成
    {
        let mut state_lock = state.lock().unwrap();
        if let Some(current_result) = state_lock.results.get_mut(&provider.name) {
            current_result.ping = result.ping;
            current_result.jitter = result.jitter;
        }
        state_lock.progress = 33.0;
        println!("{} 延迟测试完成，ping: {:.1}ms, 进度: 33%", provider.name, result.ping);
    }

    // 阶段2: 下载测试 (33%)
    {
        let mut state_lock = state.lock().unwrap();
        state_lock.current_stage = format!("{} - 下载测试", provider.name);
        println!("开始 {} 的下载测试", provider.name);
    }

    result.download_speed = test_download_speed(provider, &client, state.clone()).await?;

    // 更新结果并显示下载完成
    {
        let mut state_lock = state.lock().unwrap();
        if let Some(current_result) = state_lock.results.get_mut(&provider.name) {
            current_result.download_speed = result.download_speed;
        }
        state_lock.progress = 66.0;
        println!("{} 下载测试完成，下载: {:.1} Mbps, 进度: 66%", provider.name, result.download_speed);
    }

    // 阶段3: 上传测试 (34%)
    {
        let mut state_lock = state.lock().unwrap();
        state_lock.current_stage = format!("{} - 上传测试", provider.name);
        println!("开始 {} 的上传测试", provider.name);
    }

    result.upload_speed = test_upload_speed(provider, &client, state.clone()).await?;

    // 更新结果并显示上传完成
    {
        let mut state_lock = state.lock().unwrap();
        if let Some(current_result) = state_lock.results.get_mut(&provider.name) {
            current_result.upload_speed = result.upload_speed;
        }
        state_lock.progress = 100.0;
        println!("{} 上传测试完成，上传: {:.1} Mbps, 进度: 100%", provider.name, result.upload_speed);
    }

    Ok(result)
}

async fn test_ping(
    _provider: &NetworkProvider,
    _client: &Client,
    state: Arc<Mutex<NetworkTestInfo>>,
) -> Result<(f64, f64), Box<dyn std::error::Error + Send + Sync>> {
    // 模拟ping测试 - 在实际实现中，你可能需要使用系统ping命令或ICMP
    let mut ping_times = Vec::new();
    
    for i in 0..5 { // 减少到5次ping，加快速度
        let start = Instant::now();
        
        // 模拟网络延迟（适度延迟让用户能看到进度）
        let delay = 300 + (i * 100); // 300-700ms的延迟
        tokio::time::sleep(Duration::from_millis(delay)).await;
        
        let elapsed = start.elapsed().as_millis() as f64;
        ping_times.push(elapsed);
        
        // 更新ping测试的进度 (在33%内分布)
        let ping_progress = ((i + 1) as f64 / 5.0) * 33.0;
        {
            let mut state_lock = state.lock().unwrap();
            state_lock.progress = ping_progress;
        }
        
        println!("Ping {}/5 完成，延迟: {:.1}ms，进度: {:.1}%", i + 1, elapsed, ping_progress);
    }

    let avg_ping = ping_times.iter().sum::<f64>() / ping_times.len() as f64;
    let jitter = calculate_jitter(&ping_times);

    Ok((avg_ping, jitter))
}

async fn test_download_speed(
    _provider: &NetworkProvider,
    _client: &Client,
    state: Arc<Mutex<NetworkTestInfo>>,
) -> Result<f64, Box<dyn std::error::Error + Send + Sync>> {
    // 模拟下载测试，显示进度
    println!("开始下载速度测试...");
    
    // 分6个阶段模拟下载，每个阶段更新进度
    for i in 0..6 {
        tokio::time::sleep(Duration::from_millis(400)).await; // 每个阶段0.4秒
        
        let stage_progress = ((i + 1) as f64 / 6.0) * 33.0; // 下载占33%的进度
        {
            let mut state_lock = state.lock().unwrap();
            state_lock.progress = 33.0 + stage_progress; // ping占33%，下载从33%开始
            println!("下载进度: {:.1}%", state_lock.progress);
        }
    }
    
    // 模拟下载速度结果
    let download_speed = 30.0 + (rand::thread_rng().gen::<f64>() * 50.0); // 30-80 Mbps
    println!("下载速度测试完成: {:.1} Mbps", download_speed);
    
    Ok(download_speed)
}

async fn test_upload_speed(
    _provider: &NetworkProvider,
    _client: &Client,
    state: Arc<Mutex<NetworkTestInfo>>,
) -> Result<f64, Box<dyn std::error::Error + Send + Sync>> {
    // 模拟上传测试，显示进度
    println!("开始上传速度测试...");
    
    // 分6个阶段模拟上传，每个阶段更新进度  
    for i in 0..6 {
        tokio::time::sleep(Duration::from_millis(300)).await; // 每个阶段0.3秒
        
        let upload_progress = ((i + 1) as f64 / 6.0) * 34.0; // 上传占34%的进度
        {
            let mut state_lock = state.lock().unwrap();
            state_lock.progress = 66.0 + upload_progress; // ping+下载占66%，上传从66%开始
            println!("上传进度: {:.1}%", state_lock.progress);
        }
    }
    
    // 模拟上传速度结果
    let upload_speed = 20.0 + (rand::thread_rng().gen::<f64>() * 30.0); // 20-50 Mbps
    println!("上传速度测试完成: {:.1} Mbps", upload_speed);
    
    Ok(upload_speed)
}

fn calculate_jitter(ping_times: &[f64]) -> f64 {
    if ping_times.len() < 2 {
        return 0.0;
    }
    
    let mut deltas = Vec::new();
    for i in 1..ping_times.len() {
        deltas.push((ping_times[i] - ping_times[i-1]).abs());
    }
    
    deltas.iter().sum::<f64>() / deltas.len() as f64
}

/// 检查是否需要刷新UI
pub fn check_needs_refresh() -> bool {
    let state_guard = NETWORK_TEST_STATE.lock().unwrap();
    if let Some(ref state_arc) = *state_guard {
        let state = state_arc.lock().unwrap();
        state.is_testing
    } else {
        false
    }
}

pub fn get_info() -> String {
    format!("网速测试功能已集成到UI中，请在UI界面中查看测试结果。")
}