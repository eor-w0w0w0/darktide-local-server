use regex::Regex;
use std::process::Command;

pub fn forward_port(port: u16) -> Option<String> {
    clear_endpoints(port);

    let address = format!("127.0.0.{}", get_endpoint());
    let output = Command::new("netsh")
        .args([
            "interface",
            "portproxy",
            "add",
            "v4tov4",
            &format!("listenaddress={}", address),
            "listenport=80",
            "connectaddress=127.0.0.1",
            &format!("connectport={}", port),
        ])
        .output()
        .ok()?;

    if output.status.success() {
        Some(address)
    } else {
        eprintln!(
            "Failed to forward port {}: {}",
            port,
            String::from_utf8_lossy(&output.stderr)
        );
        None
    }
}

pub fn remove_portforward(target_address: &str) {
    let _ = Command::new("netsh")
        .args([
            "interface",
            "portproxy",
            "delete",
            "v4tov4",
            &format!("listenaddress={}", target_address),
            "listenport=80",
        ])
        .output();
}

fn clear_endpoints(target_port: u16) {
    let output = match Command::new("netsh")
        .args(["interface", "portproxy", "show", "v4tov4"])
        .output()
    {
        Ok(output) => output,
        Err(_) => return,
    };

    if !output.status.success() {
        return;
    }

    let output_str = String::from_utf8_lossy(&output.stdout);
    let re = Regex::new(format!(r".*{}$", target_port).as_str()).unwrap();

    for line in output_str.lines().filter(|line| re.is_match(line)) {
        if let Some(address) = line.split_whitespace().next() {
            remove_portforward(address);
        }
    }
}

fn get_endpoint() -> u16 {
    let output = match Command::new("netsh")
        .args(["interface", "portproxy", "show", "v4tov4"])
        .output()
    {
        Ok(output) => output,
        Err(_) => return 80,
    };

    if !output.status.success() {
        return 80;
    }

    let output_str = String::from_utf8_lossy(&output.stdout);
    let mut endpoint = 80;

    loop {
        let address = format!("127.0.0.{}", endpoint);
        if !output_str.contains(&address) {
            return endpoint;
        }
        endpoint += 1;
    }
}
