use std::{process::Command, sync::Arc};

use regex::Regex;

use crate::constants::PortForward;

/// Forward a port using netsh portproxy
pub fn forward_port(port: u16) -> Arc<PortForward> {
    clear_endpoints(port);
    let endpoint = get_endpoint();
    let target_address = Arc::new(PortForward {
        port,
        address: format!("127.0.0.{endpoint}"),
    });

    let output = Command::new("netsh")
        .args(&[
            "interface",
            "portproxy",
            "add",
            "v4tov4",
            &format!("listenaddress={}", &target_address.address),
            "listenport=80",
            "connectaddress=127.0.0.1",
            &format!("connectport={}", port),
        ])
        .output()
        .expect("failed to retrieve portproxy");
    let _outputStr = String::from_utf8_lossy(&output.stdout);
    if output.status.success() {
        return target_address;
    }
    eprintln!(
        "Failed to forward port {}: {}",
        port,
        String::from_utf8_lossy(&output.stderr)
    );
    return Arc::new(PortForward {
        port,
        address: String::new(),
    }); // Return
}

pub fn remove_portforward(target_address: String) {
    let _output = Command::new("netsh")
        .args(&[
            "interface",
            "portproxy",
            "delete",
            "v4tov4",
            &format!("listenaddress={}", &target_address),
            "listenport=80",
        ])
        .output()
        .expect("failed to retrieve portproxy");
}

pub fn clear_endpoints(targetPort: u16) {
    let output = Command::new("netsh")
        .args(&["interface", "portproxy", "show", "v4tov4"])
        .output()
        .expect("failed to retrieve portproxy");
    if output.status.success() {
        let outputStr = String::from_utf8_lossy(&output.stdout);
        let existing_forwards = outputStr.lines();
        if existing_forwards.count() > 0 {
            let re = Regex::new(format!(r".*{}$", targetPort).as_str()).unwrap();
            let decimal_lines: Vec<&str> =
                outputStr.lines().filter(|line| re.is_match(line)).collect();

            for line in decimal_lines {
                let parts: Vec<&str> = line.split_whitespace().collect();
                remove_portforward(parts[0].to_string());
            }
        }
    }
}
// Cycle through the endpoints until we find a match, starting with 127.0.0.80
pub fn get_endpoint() -> u16 {
    let output = Command::new("netsh")
        .args(&["interface", "portproxy", "show", "v4tov4"])
        .output()
        .expect("failed to retrieve portproxy");
    if output.status.success() {
        let outputStr = String::from_utf8_lossy(&output.stdout);
        let mut targetAddr = 80;
        let mut re = Regex::new(format!("127.0.0.{}", targetAddr).as_str()).unwrap();
        while re.is_match(&outputStr) {
            targetAddr += 1;
            re = Regex::new(format!("127.0.0.{}", targetAddr).as_str()).unwrap();
        }
        return targetAddr;
    }
    return 80;
}
