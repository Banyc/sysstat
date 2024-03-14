use super::{ReadPidOptions, ReadStatsError, ReadStatsOptions, ReadTasksOptions, Stats};

impl ReadPidOptions<'_> {
    pub async fn read_pid(&self) -> Vec<usize> {
        todo!()
    }
}

impl ReadTasksOptions {
    pub async fn read_tid(&self) -> Result<Vec<usize>, ReadStatsError> {
        todo!()
    }
}

impl ReadStatsOptions {
    pub async fn read(&self) -> Result<Stats, ReadStatsError> {
        todo!()
    }
}
