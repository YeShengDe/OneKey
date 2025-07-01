pub fn get_info() -> String {
    let mut content = String::from("=== TCP 调优 ===\n\n");
    
    content.push_str("优化 TCP 参数可以显著提升网络性能\n\n");
    
    // 查看当前设置
    content.push_str("1. 查看当前 TCP 设置\n");
    content.push_str("   ```\n");
    content.push_str("   # 查看所有网络参数\n");
    content.push_str("   sysctl -a | grep net\n\n");
    content.push_str("   # 查看 TCP 相关参数\n");
    content.push_str("   sysctl -a | grep tcp\n\n");
    content.push_str("   # 查看当前拥塞控制算法\n");
    content.push_str("   sysctl net.ipv4.tcp_congestion_control\n");
    content.push_str("   sysctl net.ipv4.tcp_available_congestion_control\n");
    content.push_str("   ```\n\n");
    
    // 基础优化
    content.push_str("2. 基础 TCP 优化参数\n");
    content.push_str("   编辑 /etc/sysctl.conf 添加:\n");
    content.push_str("   ```\n");
    content.push_str("   # TCP 缓冲区大小\n");
    content.push_str("   net.core.rmem_default = 262144\n");
    content.push_str("   net.core.rmem_max = 134217728\n");
    content.push_str("   net.core.wmem_default = 262144\n");
    content.push_str("   net.core.wmem_max = 134217728\n");
    content.push_str("   net.ipv4.tcp_rmem = 4096 87380 134217728\n");
    content.push_str("   net.ipv4.tcp_wmem = 4096 65536 134217728\n\n");
    content.push_str("   # TCP 连接数\n");
    content.push_str("   net.core.somaxconn = 32768\n");
    content.push_str("   net.ipv4.tcp_max_syn_backlog = 8192\n\n");
    content.push_str("   # TCP KeepAlive\n");
    content.push_str("   net.ipv4.tcp_keepalive_time = 60\n");
    content.push_str("   net.ipv4.tcp_keepalive_intvl = 10\n");
    content.push_str("   net.ipv4.tcp_keepalive_probes = 6\n");
    content.push_str("   ```\n\n");
    
    // BBR 优化
    content.push_str("3. 启用 BBR 拥塞控制\n");
    content.push_str("   ```\n");
    content.push_str("   # 检查内核版本 (需要 4.9+)\n");
    content.push_str("   uname -r\n\n");
    content.push_str("   # 加载 BBR 模块\n");
    content.push_str("   modprobe tcp_bbr\n");
    content.push_str("   echo \"tcp_bbr\" >> /etc/modules-load.d/modules.conf\n\n");
    content.push_str("   # 启用 BBR\n");
    content.push_str("   echo \"net.core.default_qdisc = fq\" >> /etc/sysctl.conf\n");
    content.push_str("   echo \"net.ipv4.tcp_congestion_control = bbr\" >> /etc/sysctl.conf\n\n");
    content.push_str("   # 应用设置\n");
    content.push_str("   sysctl -p\n");
    content.push_str("   ```\n\n");
    
    // 高性能优化
    content.push_str("4. 高性能服务器优化\n");
    content.push_str("   ```\n");
    content.push_str("   # 文件描述符限制\n");
    content.push_str("   echo \"* soft nofile 1000000\" >> /etc/security/limits.conf\n");
    content.push_str("   echo \"* hard nofile 1000000\" >> /etc/security/limits.conf\n\n");
    content.push_str("   # 更多 TCP 优化\n");
    content.push_str("   net.ipv4.tcp_fastopen = 3\n");
    content.push_str("   net.ipv4.tcp_tw_reuse = 1\n");
    content.push_str("   net.ipv4.tcp_fin_timeout = 30\n");
    content.push_str("   net.ipv4.tcp_max_tw_buckets = 5000\n");
    content.push_str("   net.ipv4.tcp_syncookies = 1\n");
    content.push_str("   net.ipv4.tcp_synack_retries = 2\n");
    content.push_str("   net.ipv4.ip_local_port_range = 10000 65535\n");
    content.push_str("   ```\n\n");
    
    // 验证优化
    content.push_str("5. 验证优化效果\n");
    content.push_str("   ```\n");
    content.push_str("   # 查看 BBR 是否启用\n");
    content.push_str("   lsmod | grep bbr\n");
    content.push_str("   sysctl net.ipv4.tcp_congestion_control\n\n");
    content.push_str("   # 网络性能测试\n");
    content.push_str("   iperf3 -s  # 服务端\n");
    content.push_str("   iperf3 -c server_ip -t 30  # 客户端\n\n");
    content.push_str("   # 查看 TCP 统计\n");
    content.push_str("   ss -s\n");
    content.push_str("   netstat -s | grep -i tcp\n");
    content.push_str("   ```\n\n");
    
    // 注意事项
    content.push_str("6. 注意事项\n");
    content.push_str("   - BBR 需要内核 4.9 或更高版本\n");
    content.push_str("   - 修改前建议备份原始配置\n");
    content.push_str("   - 某些参数可能需要重启生效\n");
    content.push_str("   - 根据实际网络环境调整参数\n\n");
    
    content.push_str("提示: 应用配置后使用 'sysctl -p' 使其生效。\n");
    
    content
}