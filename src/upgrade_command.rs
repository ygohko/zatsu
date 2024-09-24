struct UpgradeCommand {
}

impl Command for UpgradeCommand {
    pub fn execute() -> Result<(), ZatsuError> {
        let repository = match Repository::load(".zatsu") {
            Ok(repository) => repository,
            Err(_) => {
                println!("Error: repository not found. To create repository, execute zatsu init.");
                return Err(ZatsuError::new(error::CODE_LOADING_REPOSITORY_FAILED));
            }
        };
        if repository.version != 1 {
            println!("Error: Repository is already up to date. Do nothing.");
            return Err(ZatsuError::new(error::CODE_GENERAL));
        }

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
