use static_toml::static_toml;

static_toml! {
    #[static_toml(values_ident = Tile)]
    pub(crate) const TILES = include_toml!("data/tiles.toml");
}
