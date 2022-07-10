use super::error::FanboxAPIError;

/// Check if have data that we don't handle
pub trait CheckUnkown {
    fn check_unknown(&self) -> Result<(), FanboxAPIError>;
}
