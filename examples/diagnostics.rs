use std::net::{Ipv4Addr, Ipv6Addr};
use std::time::Duration;

use tokio_multicast::{diagnose_multicast, diagnose_multicast_with_config, MulticastDiagnosticConfig};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let json = args.iter().any(|arg| arg == "--json");
    let ipv4_only = args.iter().any(|arg| arg == "--ipv4-only");
    let ipv6_only = args.iter().any(|arg| arg == "--ipv6-only");
    let timeout_ms = parse_option(&args, "--timeout-ms").and_then(|value| value.parse::<u64>().ok());
    let ipv4_group = parse_option(&args, "--ipv4-group").and_then(|value| value.parse::<Ipv4Addr>().ok());
    let ipv6_group = parse_option(&args, "--ipv6-group").and_then(|value| value.parse::<Ipv6Addr>().ok());
    let ipv6_interface =
        parse_option(&args, "--ipv6-ifindex").and_then(|value| value.parse::<u32>().ok());

    let report = if timeout_ms.is_none()
        && ipv4_group.is_none()
        && ipv6_group.is_none()
        && ipv6_interface.is_none()
    {
        diagnose_multicast()
    } else {
        let mut config = MulticastDiagnosticConfig::default();
        if let Some(timeout_ms) = timeout_ms {
            config.timeout = Duration::from_millis(timeout_ms);
        }
        if let Some(ipv4_group) = ipv4_group {
            config.ipv4_group = ipv4_group;
        }
        if let Some(ipv6_group) = ipv6_group {
            config.ipv6_group = ipv6_group;
        }
        if let Some(ipv6_interface) = ipv6_interface {
            config.ipv6_interface = Some(ipv6_interface);
        }
        diagnose_multicast_with_config(&config)
    };

    if json {
        println!("{}", render_json(&report));
    } else {
        println!("overall supported: {}", report.supported());
        if !ipv6_only {
            print_probe(&report.ipv4);
        }
        if !ipv4_only {
            print_probe(&report.ipv6);
        }
    }

    let selected_supported = if ipv4_only && !ipv6_only {
        report.ipv4.supported
    } else if ipv6_only && !ipv4_only {
        report.ipv6.supported
    } else {
        report.supported()
    };

    if !selected_supported {
        std::process::exit(2);
    }
}

fn print_probe(probe: &tokio_multicast::ProbeResult) {
    println!("{} supported: {}", probe.label, probe.supported);
    if let Some(error_kind) = probe.error_kind {
        println!(
            "{} error kind: {}",
            probe.label,
            probe_error_kind_name(error_kind)
        );
    }
    println!(
        "{} stages: socket_created={} bound={} joined={} loopback_sent={} loopback_received={}",
        probe.label,
        probe.stages.socket_created,
        probe.stages.bound,
        probe.stages.joined,
        probe.stages.loopback_sent,
        probe.stages.loopback_received
    );
    if let Some(details) = &probe.details {
        println!("{} details: {details}", probe.label);
    }
    if let Some(error) = &probe.error {
        println!("{} error: {error}", probe.label);
    }
}

fn parse_option<'a>(args: &'a [String], name: &str) -> Option<&'a str> {
    args.windows(2)
        .find(|pair| pair[0] == name)
        .map(|pair| pair[1].as_str())
}

fn render_json(report: &tokio_multicast::MulticastDiagnostics) -> String {
    format!(
        concat!(
            "{{",
            "\"supported\":{},",
            "\"ipv4\":{},",
            "\"ipv6\":{}",
            "}}"
        ),
        report.supported(),
        render_probe_json(&report.ipv4),
        render_probe_json(&report.ipv6)
    )
}

fn render_probe_json(probe: &tokio_multicast::ProbeResult) -> String {
    format!(
        concat!(
            "{{",
            "\"label\":\"{}\",",
            "\"supported\":{},",
            "\"error_kind\":{},",
            "\"stages\":{{",
            "\"socket_created\":{},",
            "\"bound\":{},",
            "\"joined\":{},",
            "\"loopback_sent\":{},",
            "\"loopback_received\":{}",
            "}},",
            "\"details\":{},",
            "\"error\":{}",
            "}}"
        ),
        escape_json(probe.label),
        probe.supported,
        option_error_kind_json(probe.error_kind),
        probe.stages.socket_created,
        probe.stages.bound,
        probe.stages.joined,
        probe.stages.loopback_sent,
        probe.stages.loopback_received,
        option_json(&probe.details),
        option_json(&probe.error)
    )
}

fn option_json(value: &Option<String>) -> String {
    match value {
        Some(value) => format!("\"{}\"", escape_json(value)),
        None => "null".to_string(),
    }
}

fn option_error_kind_json(value: Option<tokio_multicast::ProbeErrorKind>) -> String {
    match value {
        Some(value) => format!("\"{}\"", probe_error_kind_name(value)),
        None => "null".to_string(),
    }
}

fn probe_error_kind_name(value: tokio_multicast::ProbeErrorKind) -> &'static str {
    match value {
        tokio_multicast::ProbeErrorKind::Unavailable => "unavailable",
        tokio_multicast::ProbeErrorKind::SocketCreate => "socket_create",
        tokio_multicast::ProbeErrorKind::Bind => "bind",
        tokio_multicast::ProbeErrorKind::Join => "join",
        tokio_multicast::ProbeErrorKind::Send => "send",
        tokio_multicast::ProbeErrorKind::Receive => "receive",
        tokio_multicast::ProbeErrorKind::InvalidData => "invalid_data",
        tokio_multicast::ProbeErrorKind::Other => "other",
    }
}

fn escape_json(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
}
