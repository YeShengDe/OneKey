use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MenuItem {
    SystemInfo,
    DiskTest,
    CpuTest,
    NetworkSpeedTest,
    SingBoxScript,
    XrayScript,
    OpenPort,
    ClosePort,
    K3s,
    K8s,
    TcpOptimization,
}

impl MenuItem {
    pub fn all() -> Vec<MenuItem> {
        vec![
            MenuItem::SystemInfo,
            MenuItem::DiskTest,
            MenuItem::CpuTest,
            MenuItem::NetworkSpeedTest,
            MenuItem::SingBoxScript,
            MenuItem::XrayScript,
            MenuItem::OpenPort,
            MenuItem::ClosePort,
            MenuItem::K3s,
            MenuItem::K8s,
            MenuItem::TcpOptimization,
        ]
    }
    
    pub fn as_str(&self) -> &'static str {
        match self {
            MenuItem::SystemInfo => "系统信息",
            MenuItem::DiskTest => "硬盘测试",
            MenuItem::CpuTest => "CPU测试",
            MenuItem::NetworkSpeedTest => "网速测试",
            MenuItem::SingBoxScript => "sing-box一键脚本",
            MenuItem::XrayScript => "xray一键脚本",
            MenuItem::OpenPort => "开放端口",
            MenuItem::ClosePort => "关闭端口",
            MenuItem::K3s => "k3s",
            MenuItem::K8s => "k8s",
            MenuItem::TcpOptimization => "tcp调优",
        }
    }
    
    pub fn description(&self) -> &'static str {
        match self {
            MenuItem::SystemInfo => "查看系统详细信息",
            MenuItem::DiskTest => "测试硬盘读写性能",
            MenuItem::CpuTest => "测试CPU性能",
            MenuItem::NetworkSpeedTest => "测试网络速度",
            MenuItem::SingBoxScript => "安装和配置sing-box",
            MenuItem::XrayScript => "安装和配置xray",
            MenuItem::OpenPort => "开放防火墙端口",
            MenuItem::ClosePort => "关闭防火墙端口",
            MenuItem::K3s => "部署轻量级Kubernetes",
            MenuItem::K8s => "部署完整版Kubernetes",
            MenuItem::TcpOptimization => "优化TCP网络参数",
        }
    }
}

impl fmt::Display for MenuItem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

pub struct Menu {
    items: Vec<MenuItem>,
    selected: usize,
}

impl Menu {
    pub fn new() -> Self {
        Self {
            items: MenuItem::all(),
            selected: 0,
        }
    }
    
    pub fn next(&mut self) {
        if self.selected < self.items.len() - 1 {
            self.selected += 1;
        }
    }
    
    pub fn previous(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
        }
    }
    
    pub fn selected_item(&self) -> MenuItem {
        self.items[self.selected]
    }
    
    pub fn selected_index(&self) -> usize {
        self.selected
    }
    
    pub fn items(&self) -> &[MenuItem] {
        &self.items
    }
}