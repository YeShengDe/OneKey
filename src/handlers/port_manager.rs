pub fn get_open_port_info() -> String {
    let mut content = String::from("=== 开放端口 ===\n\n");
    
    content.push_str("不同系统的端口开放方法：\n\n");
    
    // iptables
    content.push_str("1. 使用 iptables (传统方法)\n");
    content.push_str("   ```\n");
    content.push_str("   # 开放单个端口\n");
    content.push_str("   iptables -A INPUT -p tcp --dport 8080 -j ACCEPT\n");
    content.push_str("   iptables -A INPUT -p udp --dport 8080 -j ACCEPT\n\n");
    content.push_str("   # 开放端口范围\n");
    content.push_str("   iptables -A INPUT -p tcp --dport 8000:9000 -j ACCEPT\n\n");
    content.push_str("   # 限制来源IP\n");
    content.push_str("   iptables -A INPUT -p tcp -s 192.168.1.100 --dport 22 -j ACCEPT\n\n");
    content.push_str("   # 保存规则\n");
    content.push_str("   iptables-save > /etc/iptables/rules.v4\n");
    content.push_str("   ```\n\n");
    
    // firewalld
    content.push_str("2. 使用 firewalld (CentOS/RHEL 7+)\n");
    content.push_str("   ```\n");
    content.push_str("   # 开放端口\n");
    content.push_str("   firewall-cmd --zone=public --add-port=8080/tcp --permanent\n");
    content.push_str("   firewall-cmd --zone=public --add-port=8080/udp --permanent\n\n");
    content.push_str("   # 开放端口范围\n");
    content.push_str("   firewall-cmd --zone=public --add-port=8000-9000/tcp --permanent\n\n");
    content.push_str("   # 重载配置\n");
    content.push_str("   firewall-cmd --reload\n\n");
    content.push_str("   # 查看已开放端口\n");
    content.push_str("   firewall-cmd --list-ports\n");
    content.push_str("   ```\n\n");
    
    // ufw
    content.push_str("3. 使用 ufw (Ubuntu/Debian)\n");
    content.push_str("   ```\n");
    content.push_str("   # 开放端口\n");
    content.push_str("   ufw allow 8080/tcp\n");
    content.push_str("   ufw allow 8080/udp\n\n");
    content.push_str("   # 开放端口范围\n");
    content.push_str("   ufw allow 8000:9000/tcp\n\n");
    content.push_str("   # 限制来源IP\n");
    content.push_str("   ufw allow from 192.168.1.100 to any port 22\n\n");
    content.push_str("   # 查看状态\n");
    content.push_str("   ufw status\n");
    content.push_str("   ```\n\n");
    
    // 检查端口
    content.push_str("4. 检查端口状态\n");
    content.push_str("   ```\n");
    content.push_str("   # 查看监听端口\n");
    content.push_str("   netstat -tlnp\n");
    content.push_str("   ss -tlnp\n\n");
    content.push_str("   # 测试端口连通性\n");
    content.push_str("   telnet localhost 8080\n");
    content.push_str("   nc -zv localhost 8080\n");
    content.push_str("   ```\n\n");
    
    content.push_str("提示: 开放端口前请确认服务已正确配置，避免安全风险。\n");
    
    content
}

pub fn get_close_port_info() -> String {
    let mut content = String::from("=== 关闭端口 ===\n\n");
    
    content.push_str("不同系统的端口关闭方法：\n\n");
    
    // iptables
    content.push_str("1. 使用 iptables\n");
    content.push_str("   ```\n");
    content.push_str("   # 删除允许规则\n");
    content.push_str("   iptables -D INPUT -p tcp --dport 8080 -j ACCEPT\n");
    content.push_str("   iptables -D INPUT -p udp --dport 8080 -j ACCEPT\n\n");
    content.push_str("   # 添加拒绝规则\n");
    content.push_str("   iptables -A INPUT -p tcp --dport 8080 -j DROP\n\n");
    content.push_str("   # 查看现有规则\n");
    content.push_str("   iptables -L -n --line-numbers\n\n");
    content.push_str("   # 根据行号删除规则\n");
    content.push_str("   iptables -D INPUT 5\n\n");
    content.push_str("   # 保存规则\n");
    content.push_str("   iptables-save > /etc/iptables/rules.v4\n");
    content.push_str("   ```\n\n");
    
    // firewalld
    content.push_str("2. 使用 firewalld\n");
    content.push_str("   ```\n");
    content.push_str("   # 关闭端口\n");
    content.push_str("   firewall-cmd --zone=public --remove-port=8080/tcp --permanent\n");
    content.push_str("   firewall-cmd --zone=public --remove-port=8080/udp --permanent\n\n");
    content.push_str("   # 关闭端口范围\n");
    content.push_str("   firewall-cmd --zone=public --remove-port=8000-9000/tcp --permanent\n\n");
    content.push_str("   # 重载配置\n");
    content.push_str("   firewall-cmd --reload\n");
    content.push_str("   ```\n\n");
    
    // ufw
    content.push_str("3. 使用 ufw\n");
    content.push_str("   ```\n");
    content.push_str("   # 删除允许规则\n");
    content.push_str("   ufw delete allow 8080/tcp\n");
    content.push_str("   ufw delete allow 8080/udp\n\n");
    content.push_str("   # 添加拒绝规则\n");
    content.push_str("   ufw deny 8080/tcp\n\n");
    content.push_str("   # 根据编号删除规则\n");
    content.push_str("   ufw status numbered\n");
    content.push_str("   ufw delete 5\n");
    content.push_str("   ```\n\n");
    
    // 停止服务
    content.push_str("4. 停止监听服务\n");
    content.push_str("   ```\n");
    content.push_str("   # 查找占用端口的进程\n");
    content.push_str("   lsof -i :8080\n");
    content.push_str("   fuser 8080/tcp\n\n");
    content.push_str("   # 停止服务\n");
    content.push_str("   systemctl stop service_name\n\n");
    content.push_str("   # 终止进程\n");
    content.push_str("   kill -9 PID\n");
    content.push_str("   ```\n\n");
    
    content.push_str("提示: 关闭端口前请确认不会影响正常服务。\n");
    
    content
}