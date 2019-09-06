#[derive(Debug, PartialEq, Eq, Hash)]
pub enum Prefab {
    Glider,
    SmallExploder,
    Exploder,
    Spaceship,
    Tumbler,
    GliderGun,
}

impl Prefab {
    pub fn get_static_name(&self) -> &'static str {
        match self {
            Prefab::Glider => "Glider",
            Prefab::SmallExploder => "Small Exploder",
            Prefab::Exploder => "Exploder",
            Prefab::Spaceship => "Spaceship",
            Prefab::Tumbler => "Tumbler",
            Prefab::GliderGun => "Glider Gun",
        }
    }
}
