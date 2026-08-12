pub struct CsdSettings {
    pub out_size: usize,
    pub in_size: usize,
}

impl Default for CsdSettings {
    fn default() -> Self {
        CsdSettings {
            out_size: 2,
            in_size: 2,
        }
    }
}

// TODO
// make real function
pub fn parse_csd_settings(file: &str) -> Option<CsdSettings> {
    Some(CsdSettings::default())
}
