use super::error::FanboxAPIError;

/// Check if have data that we don't handle
pub trait CheckUnknown {
    fn check_unknown(&self) -> Result<(), FanboxAPIError>;
}
