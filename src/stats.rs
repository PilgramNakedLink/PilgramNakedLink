use std::time::Duration;

#[derive(Debug)]
pub struct HopStats {
    /// The mean of the round-trip times of all queries.
    pub mean: Option<Duration>,
    /// The median of the round-trip times of all queries.
    pub median: Option<Duration>,
}

impl HopStats {
    pub(crate) fn from_durations(durations: &[Duration]) -> Self {
        let mean = time_mean(&durations);
        let median = time_median(&durations);

        Self { mean, median }
    }
}

fn time_mean(list: &[Duration]) -> Option<Duration> {
    if list.len() == 0 {
        return None;
    };
    let sum: Duration = Iterator::sum(list.iter());
    Some(Duration::from(sum) / (list.len() as u32))
}

fn time_median(list: &[Duration]) -> Option<Duration> {
    let len = list.len();

    if len == 0 {
        return None;
    };

    let mid = len / 2;

    if len % 2 == 0 {
        time_mean(&list[(mid - 1)..(mid + 1)])
    } else {
        Some(Duration::from(list[mid]))
    }
}
