use anyhow::Context;

#[derive(Debug, Clone)]
pub struct CpuTimes {
    pub user: u64,
    pub nice: u64,
    pub system: u64,
    pub idle: u64,
    pub iowait: u64,
    pub irq: u64,
    pub softirq: u64,
    pub steal: u64,
    pub guest: u64,
    pub guest_nice: u64,
}

impl CpuTimes {
    // Made public for testing
    pub fn from_str(line: &str) -> anyhow::Result<Option<Self>> {
        let parts: Vec<&str> = line.split_whitespace().collect();
        // Expecting "cpu" followed by at least 10 numbers.
        // The first element must be "cpu".
        if parts.is_empty() || parts[0] != "cpu" {
            // This is not the aggregate CPU line we are looking for, or it's malformed.
            return Ok(None); 
        }

        // "cpu" is parts[0], so numbers start from parts[1]. We need 10 numbers.
        if parts.len() < 11 { 
            // Log or handle specific error for not enough data if this was the "cpu" line
            // For now, consistent with original logic, returning Ok(None) as if it's not the line we want.
            // A more robust error might be Err(...) if parts[0] == "cpu" but length is insufficient.
            return Ok(None); 
        }

        let user = parts[1].parse().context("Parsing user value")?;
        let nice = parts[2].parse().context("Parsing nice value")?;
        let system = parts[3].parse().context("Parsing system value")?;
        let idle = parts[4].parse().context("Parsing idle value")?;
        let iowait = parts[5].parse().context("Parsing iowait value")?;
        let irq = parts[6].parse().context("Parsing irq value")?;
        let softirq = parts[7].parse().context("Parsing softirq value")?;
        let steal = parts[8].parse().context("Parsing steal value")?;
        let guest = parts[9].parse().context("Parsing guest value")?;
        let guest_nice = parts[10].parse().context("Parsing guest_nice value")?;

        Ok(Some(CpuTimes {
            user,
            nice,
            system,
            idle,
            iowait,
            irq,
            softirq,
            steal,
            guest,
            guest_nice,
        }))
    }
}

pub async fn read_cpu_times() -> anyhow::Result<Option<CpuTimes>> {
    let content = tokio::fs::read_to_string("/proc/stat")
        .await
        .context("Failed to read /proc/stat")?;

    for line in content.lines() {
        if line.starts_with("cpu ") { // Note the space to distinguish from "cpu0", "cpu1", etc.
            return CpuTimes::from_str(line);
        }
    }
    Ok(None) // No line starting with "cpu " found
}

pub fn calculate_cpu_percentage(previous: &Option<CpuTimes>, current: &CpuTimes) -> f64 {
    let prev_opt = match previous {
        Some(p) => p,
        None => return 0.0, // No previous data, so cannot calculate a delta
    };

    let prev_idle = prev_opt.idle + prev_opt.iowait;
    let current_idle = current.idle + current.iowait;

    let prev_non_idle = prev_opt.user 
        + prev_opt.nice 
        + prev_opt.system 
        + prev_opt.irq 
        + prev_opt.softirq 
        + prev_opt.steal;
    let current_non_idle = current.user 
        + current.nice 
        + current.system 
        + current.irq 
        + current.softirq 
        + current.steal;

    let prev_total = prev_idle + prev_non_idle;
    let current_total = current_idle + current_non_idle;

    let total_delta = current_total.saturating_sub(prev_total);
    let idle_delta = current_idle.saturating_sub(prev_idle);

    if total_delta == 0 {
        return 0.0; // Avoid division by zero
    }

    let non_idle_delta = total_delta.saturating_sub(idle_delta);
    
    non_idle_delta as f64 * 100.0 / total_delta as f64
}


#[cfg(test)]
mod tests {
    use super::*;

    // Helper for float comparisons
    fn assert_approx_eq(a: f64, b: f64, epsilon: f64) {
        assert!((a - b).abs() < epsilon, "Floats not approximately equal: {} vs {}", a, b);
    }

    #[test]
    fn test_parse_proc_stat_valid_line() {
        let line = "cpu  1923489 22100 1234567 87654321 12345 0 56789 0 0 0";
        match CpuTimes::from_str(line) {
            Ok(Some(times)) => {
                assert_eq!(times.user, 1923489);
                assert_eq!(times.nice, 22100);
                assert_eq!(times.system, 1234567);
                assert_eq!(times.idle, 87654321);
                assert_eq!(times.iowait, 12345);
                assert_eq!(times.irq, 0);
                assert_eq!(times.softirq, 56789);
                assert_eq!(times.steal, 0);
                assert_eq!(times.guest, 0);
                assert_eq!(times.guest_nice, 0);
            }
            Ok(None) => panic!("Parsing valid line returned None."),
            Err(e) => panic!("Parsing valid line returned error: {:?}", e),
        }
    }

    #[test]
    fn test_parse_proc_stat_not_cpu_line() {
        let line = "cpu0 123 456 ..."; // Example for a specific core, not the aggregate
        match CpuTimes::from_str(line) {
            Ok(None) => { // Correctly returns None as it's not the "cpu " line
                // Test passed
            }
            Ok(Some(_)) => panic!("Parsing non-aggregate CPU line unexpectedly returned Some."),
            Err(e) => panic!("Parsing non-aggregate CPU line returned error: {:?}", e),
        }
    }
    
    #[test]
    fn test_parse_proc_stat_empty_line() {
        let line = "";
        match CpuTimes::from_str(line) {
            Ok(None) => { // Correctly returns None
                // Test passed
            }
            Ok(Some(_)) => panic!("Parsing empty line unexpectedly returned Some."),
            Err(e) => panic!("Parsing empty line returned error: {:?}", e),
        }
    }


    #[test]
    fn test_parse_proc_stat_invalid_line_short() {
        let line = "cpu  100 200 300";
        match CpuTimes::from_str(line) {
            Ok(None) => { // Correctly returns None as not enough fields
                // Test passed
            }
            Ok(Some(_)) => panic!("Parsing short line unexpectedly returned Some."),
            Err(e) => panic!("Parsing short line returned error: {:?}", e),
        }
    }

    #[test]
    fn test_parse_proc_stat_invalid_line_non_numeric() {
        let line = "cpu  100 abc 300 400 500 600 700 800 0 0";
        // This should return an Err because parsing "abc" will fail.
        assert!(CpuTimes::from_str(line).is_err(), "Parsing non-numeric line should return an error.");
    }

    #[test]
    fn test_calculate_cpu_percentage_basic() {
        let prev_times = Some(CpuTimes { user: 100, nice: 0, system: 100, idle: 100, iowait: 0, irq: 0, softirq: 0, steal: 0, guest: 0, guest_nice: 0 });
        let curr_times = CpuTimes { user: 150, nice: 0, system: 150, idle: 150, iowait: 0, irq: 0, softirq: 0, steal: 0, guest: 0, guest_nice: 0 };
        // PrevTotal = 100+0+100+100+0+0+0+0 = 300
        // CurrTotal = 150+0+150+150+0+0+0+0 = 450
        // total_delta = 450 - 300 = 150
        // PrevIdle = 100 (idle) + 0 (iowait) = 100
        // CurrIdle = 150 (idle) + 0 (iowait) = 150
        // idle_delta = 150 - 100 = 50
        // non_idle_delta = total_delta - idle_delta = 150 - 50 = 100
        // usage = (100 * 100.0) / 150 = 10000.0 / 150 = 66.666...
        let percentage = calculate_cpu_percentage(&prev_times, &curr_times);
        assert_approx_eq(percentage, 100.0 * 100.0 / 150.0, 0.001);
    }

    #[test]
    fn test_calculate_cpu_percentage_zero_delta() {
        let prev_times = Some(CpuTimes { user: 100, nice: 0, system: 100, idle: 100, iowait: 0, irq: 0, softirq: 0, steal: 0, guest: 0, guest_nice: 0 });
        let curr_times = CpuTimes { user: 100, nice: 0, system: 100, idle: 100, iowait: 0, irq: 0, softirq: 0, steal: 0, guest: 0, guest_nice: 0 };
        let percentage = calculate_cpu_percentage(&prev_times, &curr_times);
        assert_approx_eq(percentage, 0.0, 0.001);
    }

    #[test]
    fn test_calculate_cpu_percentage_initial() {
        let prev_times = None;
        let curr_times = CpuTimes { user: 150, nice: 0, system: 150, idle: 150, iowait: 0, irq: 0, softirq: 0, steal: 0, guest: 0, guest_nice: 0 };
        let percentage = calculate_cpu_percentage(&prev_times, &curr_times);
        assert_approx_eq(percentage, 0.0, 0.001);
    }
    
    #[test]
    fn test_calculate_cpu_percentage_full_usage() {
        let prev_times = Some(CpuTimes { user: 100, nice: 0, system: 0, idle: 0, iowait: 0, irq: 0, softirq: 0, steal: 0, guest: 0, guest_nice: 0 });
        let curr_times = CpuTimes { user: 200, nice: 0, system: 0, idle: 0, iowait: 0, irq: 0, softirq: 0, steal: 0, guest: 0, guest_nice: 0 };
        // PrevIdle = 0 (idle) + 0 (iowait) = 0
        // CurrIdle = 0 (idle) + 0 (iowait) = 0
        // idle_delta = 0
        // PrevNonIdle = 100
        // CurrNonIdle = 200
        // PrevTotal = 100
        // CurrTotal = 200
        // total_delta = 100
        // non_idle_delta = total_delta - idle_delta = 100 - 0 = 100
        // usage = (100 * 100.0) / 100 = 100.0
        let percentage = calculate_cpu_percentage(&prev_times, &curr_times);
        assert_approx_eq(percentage, 100.0, 0.001);
    }

    #[test]
    fn test_calculate_cpu_percentage_idle_time_increases_more_than_total() {
        // This can happen if other counters go down (e.g. guest times if not included in total)
        // or due to counter wraps, though u64 makes wraps very rare.
        // More practically, this test ensures saturating_sub works as expected.
        let prev_times = Some(CpuTimes { user: 100, nice: 0, system: 100, idle: 100, iowait: 0, irq: 0, softirq: 0, steal: 0, guest: 0, guest_nice: 0 }); // Total = 300, Idle = 100
        let curr_times = CpuTimes { user: 100, nice: 0, system: 100, idle: 200, iowait: 0, irq: 0, softirq: 0, steal: 0, guest: 0, guest_nice: 0 }; // Total = 400, Idle = 200
        // total_delta = 100
        // idle_delta = 100
        // non_idle_delta = 100 - 100 = 0
        // usage = 0.0
        let percentage = calculate_cpu_percentage(&prev_times, &curr_times);
        assert_approx_eq(percentage, 0.0, 0.001);
    }

    #[test]
    fn test_parse_cpu_line_various_valid_numbers() {
        let line = "cpu  0 0 0 0 0 0 0 0 0 0";
        assert!(CpuTimes::from_str(line).unwrap().is_some());
        let line_large = "cpu  1234567890123456789 1 2 3 4 5 6 7 8 9";
        assert!(CpuTimes::from_str(line_large).unwrap().is_some());
    }

}
