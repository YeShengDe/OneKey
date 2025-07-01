pub fn get_info() -> String {
    let mut content = String::from("=== 网速测试 ===\n\n");
    
    content.push_str("常用网速测试工具：\n\n");
    
    // speedtest-cli
    content.push_str("1. 使用 speedtest-cli\n");
    content.push_str("   安装:\n");
    content.push_str("   ```\n");
    content.push_str("   # 通过 pip 安装\n");
    content.push_str("   pip install speedtest-cli\n\n");
    content.push_str("   # 或通过包管理器\n");
    content.push_str("   apt-get install speedtest-cli\n");
    content.push_str("   ```\n\n");
    
    content.push_str("   使用方法:\n");
    content.push_str("   ```\n");
    content.push_str("   # 基本测试\n");
    content.push_str("   speedtest-cli\n\n");
    content.push_str("   # 列出附近的服务器\n");
    content.push_str("   speedtest-cli --list\n\n");
    content.push_str("   # 指定服务器测试\n");
    content.push_str("   speedtest-cli --server 12345\n");
    content.push_str("   ```\n\n");
    
    // iperf3
    content.push_str("2. 使用 iperf3 测试\n");
    content.push_str("   安装:\n");
    content.push_str("   ```\n");
    content.push_str("   apt-get install iperf3\n");
    content.push_str("   ```\n\n");
    
    content.push_str("   使用方法:\n");
    content.push_str("   ```\n");
    content.push_str("   # 服务器端\n");
    content.push_str("   iperf3 -s\n\n");
    content.push_str("   # 客户端测试\n");
    content.push_str("   iperf3 -c server_ip\n\n");
    content.push_str("   # 反向测试（测试下载速度）\n");
    content.push_str("   iperf3 -c server_ip -R\n");
    content.push_str("   ```\n\n");
    
    // 网络延迟测试
    content.push_str("3. 网络延迟测试\n");
    content.push_str("   ```\n");
    content.push_str("   # Ping 测试\n");
    content.push_str("   ping -c 10 8.8.8.8\n\n");
    content.push_str("   # MTR 路由跟踪\n");
    content.push_str("   apt-get install mtr\n");
    content.push_str("   mtr google.com\n");
    content.push_str("   ```\n\n");
    
    // 带宽测试脚本
    content.push_str("4. 一键测速脚本\n");
    content.push_str("   ```\n");
    content.push_str("   # Bench.sh 脚本\n");
    content.push_str("   wget -qO- bench.sh | bash\n\n");
    content.push_str("   # Superbench 脚本\n");
    content.push_str("   bash <(curl -Lso- https://git.io/superbench)\n");
    content.push_str("   ```\n\n");
    
    content.push_str("提示: 网速测试会消耗流量，请注意VPS的流量限制。\n");
    
    content
}