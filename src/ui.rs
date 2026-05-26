pub trait ProgressReporter: Send + Sync {
    /// Update the message displayed by the progress reporter.
    fn set_message(&self, msg: &str);

    /// Increment the progress by a specific amount.
    fn inc(&self, delta: u64);

    /// Set the total length of the progress (if applicable).
    fn set_length(&self, len: u64);

    /// Mark the progress as complete and clear the reporter.
    fn finish(&self);

    /// Mark the progress as complete and display a final message.
    fn finish_with_message(&self, msg: &str);
}

/// A progress reporter that does nothing. Useful for tests or headless environments.
pub struct NoOpReporter;

impl ProgressReporter for NoOpReporter {
    fn set_message(&self, _msg: &str) {}
    fn inc(&self, _delta: u64) {}
    fn set_length(&self, _len: u64) {}
    fn finish(&self) {}
    fn finish_with_message(&self, _msg: &str) {}
}
