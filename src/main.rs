use random_rails_generator::{build_app, Config};

fn main() -> anyhow::Result<()> {
    let config = Config {
        rails_path: "/Users/perryhertler/.local/share/mise/installs/ruby/3.3.5/bin/rails".to_string(),
        base_dir: "/Users/perryhertler/Software/tmp/gen-play".to_string(),
        app_name: "my_app".to_string(),
        num_packages: 120,
        codeowners_dotslash_path: "https://github.com/rubyatscale/codeowners-rs/releases/download/v0.2.1/codeowners".to_string(),
    };
    dbg!(&config);
    build_app(config)
}
