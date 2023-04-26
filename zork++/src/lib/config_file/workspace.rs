use serde::Deserialize;

/// [`WorkspaceAttribute`] - Defines the characteristics of a `Zork++` workspace.
///
/// A `Zork++` workspace is a similar concept to Cargo workspaces, where the managed by
/// the workspace collection of projects shares dependencies, a common output directory,
/// metadata attributes... Commands applied to a workspace are propagated down to the
/// workspace members.
///
/// This allows the user to divide a project into smaller pieces, or create new organization
/// structures for more complex configurations.
///
/// * `members` - A collection of the names by which the dependent projects (every member of the ws)
/// has been defined in their own `zork**.toml` config file in the **project_name** key
///
/// ### Tests
///
/// ```rust
/// use zork::config_file::workspace::{
///     WorkspaceAttribute
/// };
///

/// ```
///
/// > Note: TOML table are toml commented (#) to allow us to parse
/// the inner attributes as the direct type that they belongs to.
/// That commented tables aren't the real TOML, they are just there
/// for testing and exemplification purposes of the inner attributes
/// of the configuration file.
///
/// For a test over a real example, please look at the
/// [`zork::config_file::ZorkConfigFile`] doc-test
#[derive(Deserialize, Debug, PartialEq, Default)]
#[serde(deny_unknown_fields)]
pub struct WorkspaceAttribute<'a> {
    #[serde(borrow)]
    pub members: Vec<&'a str>
}