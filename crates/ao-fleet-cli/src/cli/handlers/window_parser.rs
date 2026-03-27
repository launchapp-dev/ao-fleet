use anyhow::{Result, anyhow};

use ao_fleet_core::WeekdayWindow;

pub fn parse_window(value: &str) -> Result<WeekdayWindow> {
    let parts = value.split(',').map(str::trim).collect::<Vec<_>>();
    if parts.len() != 3 {
        return Err(anyhow!("window must be formatted as weekday,start,end"));
    }

    let weekday = parts[0]
        .parse::<u8>()
        .map_err(|error| anyhow!("invalid weekday '{}': {error}", parts[0]))?;
    let start_hour = parts[1]
        .parse::<u8>()
        .map_err(|error| anyhow!("invalid start hour '{}': {error}", parts[1]))?;
    let end_hour = parts[2]
        .parse::<u8>()
        .map_err(|error| anyhow!("invalid end hour '{}': {error}", parts[2]))?;

    Ok(WeekdayWindow { weekdays: vec![weekday], start_hour, end_hour })
}
