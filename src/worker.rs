use log::info;
use tokio::process::Command;

use crate::Result;

pub struct Worker {

}
impl Worker {
    pub fn new() -> Self {
        Self {  }
    }

    pub async fn tempo2_fit(
        &self,
        par_file: &str,
        tim_file: &str,
    ) -> Result<()> {
        let result = Command::new("tempo2")
            .arg("-f")
            .arg(par_file)
            .arg(tim_file)
            .status()
            .await?;

        info!("{}", result);

        Ok(())
    }
}
