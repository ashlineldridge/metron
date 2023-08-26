use proto::agent::Config;

fn main() {
    let agent_config = Config {
        port: 100,
        name: "foobar".to_owned(),
    };

    println!("Hello, world! {:?}", agent_config);
}
