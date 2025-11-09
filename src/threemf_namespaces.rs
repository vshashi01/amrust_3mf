/////////////////////////////////////////////////////////////////////////////////////
///  namespaces related to the Core specification
pub const CORE_NS: &str = "http://schemas.microsoft.com/3dmanufacturing/core/2015/02";

pub const CORE_TRIANGLESET_NS: &str =
    "http://schemas.microsoft.com/3dmanufacturing/trianglesets/2021/07";
pub const CORE_TRIANGLESET_PREFIX: &str = "t";
/////////////////////////////////////////////////////////////////////////////////////

////////////////////////////////////////////////////////////////////////////////////
/// namspaces related to the Production extension
pub const PROD_NS: &str = "http://schemas.microsoft.com/3dmanufacturing/production/2015/06";
pub const PROD_PREFIX: &str = "p";

////////////////////////////////////////////////////////////////////////////////////
/// namespaces related to the Beam Lattice extension
pub const BEAM_LATTICE_NS: &str =
    "http://schemas.microsoft.com/3dmanufacturing/beamlattice/2017/02";
pub const BEAM_LATTICE_PREFIX: &str = "b";

pub const BEAM_LATTICE_BALLS_NS: &str =
    "http://schemas.microsoft.com/3dmanufacturing/beamlattice/balls/2020/07";
pub const BEAM_LATTICE_BALLS_PREFIX: &str = "b2";

/// Enum representing the different XML namespaces used in 3MF
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ThreemfNamespace {
    Core,
    Prod,
    BeamLattice,
    CoreTriangleSet,
}

impl ThreemfNamespace {
    pub fn uri(&self) -> &'static str {
        match self {
            Self::Core => CORE_NS,
            Self::Prod => PROD_NS,
            Self::BeamLattice => BEAM_LATTICE_NS,
            Self::CoreTriangleSet => CORE_TRIANGLESET_NS,
        }
    }

    pub fn prefix(&self) -> Option<&'static str> {
        match self {
            Self::Core => None, // default namespace
            Self::Prod => Some(PROD_PREFIX),
            Self::BeamLattice => Some(BEAM_LATTICE_PREFIX),
            Self::CoreTriangleSet => Some(CORE_TRIANGLESET_PREFIX),
        }
    }

    pub fn xmlns_declaration(&self) -> String {
        match self.prefix() {
            Some(prefix) => format!(r#" xmlns:{}="{}""#, prefix, self.uri()),
            None => format!(r#" xmlns="{}""#, self.uri()),
        }
    }
}
