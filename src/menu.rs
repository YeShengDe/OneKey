use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MenuItem {
    SystemInfo,
    DiskTest,
    CpuTest,
    NetworkSpeedTest,
    CrossGFW,
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
            MenuItem::CrossGFW,
            MenuItem::OpenPort,
            MenuItem::ClosePort,
            MenuItem::K3s,
            MenuItem::K8s,
            MenuItem::TcpOptimization,
        ]
    }
    
    pub fn as_str(&self) -> &'static str {
        match self {
            MenuItem::SystemInfo => "1. 系统信息",
            MenuItem::DiskTest => "2. 硬盘测试",
            MenuItem::CpuTest => "3. CPU测试",
            MenuItem::NetworkSpeedTest => "4. 网速测试",
            MenuItem::CrossGFW => "5. 科学上网",
            MenuItem::OpenPort => "6. 开放端口",
            MenuItem::ClosePort => "7. 关闭端口",
            MenuItem::K3s => "8. k3s",
            MenuItem::K8s => "9. k8s",
            MenuItem::TcpOptimization => "0. tcp调优",
        }
    }
    
    pub fn description(&self) -> &'static str {
        match self {
            MenuItem::SystemInfo => "查看系统详细信息",
            MenuItem::DiskTest => "测试硬盘读写性能",
            MenuItem::CpuTest => "测试CPU性能",
            MenuItem::NetworkSpeedTest => "测试网络速度",
            MenuItem::CrossGFW => "科学上网",
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
    
    /// 通过数字键选择菜单项
    pub fn select_by_number(&mut self, number: char) -> bool {
        let index = match number {
            '1' => 0, // SystemInfo
            '2' => 1, // DiskTest
            '3' => 2, // CpuTest
            '4' => 3, // NetworkSpeedTest
            '5' => 4, // CrossGFW
            '6' => 5, // OpenPort
            '7' => 6, // ClosePort
            '8' => 7, // K3s
            '9' => 8, // K8s
            '0' => 9, // TcpOptimization
            _ => return false,
        };
        
        if index < self.items.len() {
            self.selected = index;
            true
        } else {
            false
        }
    }
}