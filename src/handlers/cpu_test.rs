pub fn get_info() -> String {
    let mut content = String::from("=== CPU测试 ===\n\n");
    
    content.push_str("常用CPU测试工具：\n\n");
    
    // sysbench 测试
    content.push_str("1. 使用 sysbench 测试\n");
    content.push_str("   安装:\n");
    content.push_str("   ```\n");
    content.push_str("   apt-get install sysbench  # Debian/Ubuntu\n");
    content.push_str("   yum install sysbench      # CentOS/RHEL\n");
    content.push_str("   ```\n\n");
    
    content.push_str("   CPU性能测试:\n");
    content.push_str("   ```\n");
    content.push_str("   # 单线程测试\n");
    content.push_str("   sysbench cpu --cpu-max-prime=20000 run\n\n");
    content.push_str("   # 多线程测试\n");
    content.push_str("   sysbench cpu --cpu-max-prime=20000 --threads=4 run\n");
    content.push_str("   ```\n\n");
    
    // stress 测试
    content.push_str("2. 使用 stress 压力测试\n");
    content.push_str("   安装:\n");
    content.push_str("   ```\n");
    content.push_str("   apt-get install stress\n");
    content.push_str("   ```\n\n");
    
    content.push_str("   压力测试:\n");
    content.push_str("   ```\n");
    content.push_str("   # 4核心压力测试，持续60秒\n");
    content.push_str("   stress --cpu 4 --timeout 60s\n\n");
    content.push_str("   # 包含内存压力\n");
    content.push_str("   stress --cpu 2 --vm 2 --vm-bytes 256M --timeout 60s\n");
    content.push_str("   ```\n\n");
    
    // stress-ng 测试
    content.push_str("3. 使用 stress-ng (更强大的压力测试)\n");
    content.push_str("   ```\n");
    content.push_str("   apt-get install stress-ng\n");
    content.push_str("   stress-ng --cpu 4 --cpu-method all --verify --timeout 60s\n");
    content.push_str("   ```\n\n");
    
    // 7-zip 基准测试
    content.push_str("4. 使用 7-zip 基准测试\n");
    content.push_str("   ```\n");
    content.push_str("   apt-get install p7zip-full\n");
    content.push_str("   7z b\n");
    content.push_str("   ```\n\n");
    
    content.push_str("提示: 运行CPU测试时会占用大量系统资源，请谨慎使用。\n");
    
    content
}