struct UpgradeCommand {
}

impl Command for UpgradeCommand {
    pub fn execute() -> Result<(), ZatsuError> {
        // TODO: Move objects directory.

        // TODO: Copy objects into new new directory.

        // TODO: Update hashes of entries.
    }
}

impl UpgradeCommand {
    pub fn new() -> Self {
        Self {}
    }
}
