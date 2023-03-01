use lazy_static::lazy_static;
use std::collections::HashMap;

macro_rules! net_info {
    ($name:literal, $subgraph:literal) => {{
        NetInfo {
            name: $name,
            subgraph: concat!(
                "https://api.thegraph.com/subgraphs/name/tornadocash/",
                $subgraph,
                "-tornado-subgraph"
            ),
        }
    }};
}

lazy_static! {
    pub static ref NET_INFO_MAP: HashMap<u32, NetInfo> = {
        let mut map = HashMap::new();
        map.insert(1, net_info!("ethereum", "mainnet"));
        map.insert(5, net_info!("goerli", "goerli"));
        map.insert(56, net_info!("binancesmartchain", "bsc"));
        map.insert(100, net_info!("gnosischain", "xdai"));
        map.insert(137, net_info!("polygon", "matic"));
        map.insert(42161, net_info!("arbitrum", "arbitrum"));
        map.insert(43114, net_info!("avalanche", "avalanche"));
        map.insert(10, net_info!("optimism", "optimism"));
        map
    };
}

#[derive(Debug, Clone, Default)]
pub struct NetInfo {
    pub name: &'static str,
    pub subgraph: &'static str,
}
