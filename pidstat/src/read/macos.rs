use super::{ReadPidOptions, ReadStatsError, ReadStatsOptions, ReadTidOptions, Stats};

impl ReadPidOptions<'_> {
    pub async fn read_pid(&self) -> Vec<usize> {
        todo!()
    }
}

impl ReadTidOptions {
    pub async fn read_tid(&self) -> Result<Vec<usize>, ReadStatsError> {
        todo!()
    }
}

impl ReadStatsOptions {
    pub async fn read_stats(&self) -> Result<Stats, ReadStatsError> {
        todo!()
    }
}
