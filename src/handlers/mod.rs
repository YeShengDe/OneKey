pub mod command;
pub mod cpu_test;
pub mod disk_test;
pub mod k3s;
pub mod k8s;
pub mod network_test;
pub mod port_manager;
pub mod sing_box;
pub mod system_info;
pub mod tcp_optimizer;
pub mod xray;

use crate::menu::MenuItem;

/// 根据菜单项获取对应的内容
pub fn get_content(item: MenuItem) -> String {
    match item {
        MenuItem::SystemInfo => system_info::get_info(),
        MenuItem::DiskTest => disk_test::get_info(),
        MenuItem::CpuTest => cpu_test::get_info(),
        MenuItem::NetworkSpeedTest => network_test::get_info(),
        MenuItem::SingBoxScript => sing_box::get_info(),
        MenuItem::XrayScript => xray::get_info(),
        MenuItem::OpenPort => port_manager::get_open_port_info(),
        MenuItem::ClosePort => port_manager::get_close_port_info(),
        MenuItem::K3s => k3s::get_info(),
        MenuItem::K8s => k8s::get_info(),
        MenuItem::TcpOptimization => tcp_optimizer::get_info(),
    }
}