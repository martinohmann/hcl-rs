use hcl::{
    eval::{Context, Evaluate},
    expr::Traversal,
    Map, Result,
};
use serde::{Deserialize, Serialize};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let input = r#"
        listener "http" {
          bind = "127.0.0.1"
          port = 80
        }

        upstream "service" {
          ip = "127.0.0.1"
          port = 3000
        }

        route "domain.com" {
          listener = listener.http
          upstreams = [upstream.service]
        }

        route "anotherroute" {
          listener = listener.http
          upstreams = [upstream.service]
        }
        "#;

    let config: Config = hcl::from_str(input)?;

    serde_json::to_writer_pretty(std::io::stdout(), &config)?;
    Ok(())
}

#[derive(Serialize, Deserialize)]
struct RawConfig {
    #[serde(rename = "listener")]
    listeners: Map<String, Listener>,
    #[serde(rename = "upstream")]
    upstreams: Map<String, Upstream>,
    #[serde(rename = "route")]
    raw_routes: Map<String, RawRoute>,
}

impl RawConfig {
    fn resolve(&self) -> Result<Config> {
        let mut ctx = Context::new();
        ctx.declare_var("upstream", hcl::to_value(&self.upstreams)?);

        let mut routes = Vec::with_capacity(self.raw_routes.len());

        for raw_route in self.raw_routes.values() {
            let route = raw_route.resolve(&ctx)?;
            routes.push(route);
        }

        let listeners = self.listeners.values().cloned().collect();

        Ok(Config { listeners, routes })
    }
}

#[derive(Serialize, Deserialize)]
struct RawRoute {
    listener: Traversal,
    upstreams: Vec<Traversal>,
}

impl RawRoute {
    fn resolve(&self, ctx: &Context) -> Result<Route> {
        let mut upstreams = Vec::with_capacity(self.upstreams.len());

        for traversal in &self.upstreams {
            let value = traversal.evaluate(&ctx)?;
            let upstream = hcl::from_value(value)?;
            upstreams.push(upstream);
        }

        Ok(Route { upstreams })
    }
}

#[derive(Serialize, Clone, Debug)]
struct Config {
    listeners: Vec<Listener>,
    routes: Vec<Route>,
}

impl<'de> Deserialize<'de> for Config {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let raw_config = RawConfig::deserialize(deserializer)?;
        raw_config.resolve().map_err(serde::de::Error::custom)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct Route {
    upstreams: Vec<Upstream>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct Listener {
    bind: String,
    port: usize,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct Upstream {
    ip: String,
    port: usize,
}
