use std::net::IpAddr;

use ryvos_core::error::{Result, RyvosError};

pub fn is_ip_blocked_for_egress(ip: &IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => {
            let octets = v4.octets();
            if octets[0] == 127 {
                return true;
            }
            if octets[0] == 169 && octets[1] == 254 {
                return true;
            }
            if octets[0] == 10 {
                return true;
            }
            if octets[0] == 172 && (16..=31).contains(&octets[1]) {
                return true;
            }
            if octets[0] == 192 && octets[1] == 168 {
                return true;
            }
            if octets[0] == 0 {
                return true;
            }
            if octets[0] >= 224 {
                return true;
            }
            false
        }
        IpAddr::V6(v6) => {
            if v6.is_loopback() {
                return true;
            }
            let segments = v6.segments();
            if (segments[0] & 0xfe00) == 0xfc00 {
                return true;
            }
            if (segments[0] & 0xffc0) == 0xfe80 {
                return true;
            }
            if let Some(v4) = v6.to_ipv4_mapped() {
                return is_ip_blocked_for_egress(&IpAddr::V4(v4));
            }
            false
        }
    }
}

pub async fn validate_url_for_egress(url_str: &str) -> Result<()> {
    let parsed = reqwest::Url::parse(url_str)
        .map_err(|e| RyvosError::ToolValidation(format!("Invalid URL: {}", e)))?;
    let host = parsed
        .host_str()
        .ok_or_else(|| RyvosError::ToolValidation("URL missing host".to_string()))?;
    let port = parsed.port_or_known_default().unwrap_or(80);

    if let Ok(ip) = host.parse::<IpAddr>() {
        if is_ip_blocked_for_egress(&ip) {
            return Err(RyvosError::ToolExecution {
                tool: "egress".to_string(),
                message: format!(
                    "Egress blocked: target IP {} is forbidden by SSRF security policy",
                    ip
                ),
            });
        }
        return Ok(());
    }

    let socket_addrs = tokio::net::lookup_host(format!("{}:{}", host, port))
        .await
        .map_err(|e| RyvosError::ToolExecution {
            tool: "egress".to_string(),
            message: format!("DNS resolution failed for {}: {}", host, e),
        })?;

    let mut found_any = false;
    for addr in socket_addrs {
        found_any = true;
        let ip = addr.ip();
        if is_ip_blocked_for_egress(&ip) {
            return Err(RyvosError::ToolExecution {
                tool: "egress".to_string(),
                message: format!("Egress blocked: host {} resolved to forbidden IP {}", host, ip),
            });
        }
    }

    if !found_any {
        return Err(RyvosError::ToolExecution {
            tool: "egress".to_string(),
            message: format!("No IP addresses resolved for host {}", host),
        });
    }

    Ok(())
}

pub async fn validate_safe_egress_url(url_str: &str) -> Result<()> {
    validate_url_for_egress(url_str).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_loopback_blocked() {
        let ip: IpAddr = "127.0.0.1".parse().unwrap();
        assert!(is_ip_blocked_for_egress(&ip));
        let ip6: IpAddr = "::1".parse().unwrap();
        assert!(is_ip_blocked_for_egress(&ip6));
    }

    #[test]
    fn test_cloud_metadata_blocked() {
        let ip: IpAddr = "169.254.169.254".parse().unwrap();
        assert!(is_ip_blocked_for_egress(&ip));
    }

    #[test]
    fn test_rfc1918_blocked() {
        let ip10: IpAddr = "10.1.2.3".parse().unwrap();
        let ip172: IpAddr = "172.16.5.6".parse().unwrap();
        let ip192: IpAddr = "192.168.1.1".parse().unwrap();
        assert!(is_ip_blocked_for_egress(&ip10));
        assert!(is_ip_blocked_for_egress(&ip172));
        assert!(is_ip_blocked_for_egress(&ip192));
    }

    #[test]
    fn test_public_allowed() {
        let ip: IpAddr = "1.1.1.1".parse().unwrap();
        assert!(!is_ip_blocked_for_egress(&ip));
    }

    #[tokio::test]
    async fn test_validate_url_local() {
        let res = validate_url_for_egress("http://127.0.0.1:8080/test").await;
        assert!(res.is_err());
    }

    #[tokio::test]
    async fn test_validate_url_metadata() {
        let res = validate_url_for_egress("http://169.254.169.254/latest/meta-data/").await;
        assert!(res.is_err());
    }
}
