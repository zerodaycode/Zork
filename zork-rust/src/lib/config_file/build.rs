use serde::*;


#[derive(Deserialize, Debug, PartialEq)]
pub struct BuildAttribute {
    pub output_dir: Option< String >
}