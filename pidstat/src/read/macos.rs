use super::{ReadStatsError, ReadStatsOptions, ReadTasksOptions, Stats};

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
