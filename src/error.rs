use std::error::Error;
use std::fmt::Display;

#[derive(Debug)]
pub struct DeploymentIdDashError {
    pub id: String,
}

impl Display for DeploymentIdDashError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "Deployment ID must not contain dashes ({})", self.id)
    }
}

impl Error for DeploymentIdDashError {}
