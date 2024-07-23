use crate::ZatsuError;

pub trait Command {
    fn execute(&self) -> Result<(), ZatsuError>;
}
