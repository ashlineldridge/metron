pub enum Config {
    Load(crate::load::Config),
    Server(crate::server::Config),
}
