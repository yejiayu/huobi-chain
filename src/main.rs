use asset::AssetService;
use derive_more::{Display, From};
use metadata::MetadataService;
use muta::MutaBuilder;
use node_manager::NodeManagerService;
use protocol::traits::{Service, ServiceMapping, ServiceSDK};
use protocol::{ProtocolError, ProtocolErrorKind, ProtocolResult};
use riscv::RiscvService;

struct DefaultServiceMapping;

impl ServiceMapping for DefaultServiceMapping {
    fn get_service<SDK: 'static + ServiceSDK>(
        &self,
        name: &str,
        sdk: SDK,
    ) -> ProtocolResult<Box<dyn Service>> {
        let service = match name {
            "asset" => Box::new(AssetService::new(sdk)?) as Box<dyn Service>,
            "metadata" => Box::new(MetadataService::new(sdk)?) as Box<dyn Service>,
            "riscv" => Box::new(RiscvService::init(sdk)?) as Box<dyn Service>,
            "node_manager" => Box::new(NodeManagerService::new(sdk)?) as Box<dyn Service>,
            _ => {
                return Err(MappingError::NotFoundService {
                    service: name.to_owned(),
                }
                .into())
            }
        };

        Ok(service)
    }

    fn list_service_name(&self) -> Vec<String> {
        vec![
            "asset".to_owned(),
            "metadata".to_owned(),
            "riscv".to_owned(),
            "node_manager".to_owned(),
        ]
    }
}

fn main() {
    let matches = clap::App::new("Huobi-chain")
        .version("v0.2.0")
        .author("Muta Dev <muta@nervos.org>")
        .arg(
            clap::Arg::from_usage("-c --config=[FILE] 'a required file for the configuration'")
                .default_value("./config/chain.toml"),
        )
        .arg(
            clap::Arg::from_usage("-g --genesis=[FILE] 'a required file for the genesis'")
                .default_value("./config/genesis.toml"),
        )
        .get_matches();

    let config_path = matches.value_of("config").unwrap();
    let genesis_path = matches.value_of("genesis").unwrap();

    let builder = MutaBuilder::new();

    // set configs
    let builder = builder
        .config_path(&config_path)
        .genesis_path(&genesis_path);

    // set service-mapping
    let builer = builder.service_mapping(DefaultServiceMapping {});

    let muta = builer.build().unwrap();

    muta.run().unwrap()
}

#[derive(Debug, Display, From)]
pub enum MappingError {
    #[display(fmt = "service {:?} was not found", service)]
    NotFoundService { service: String },
}
impl std::error::Error for MappingError {}

impl From<MappingError> for ProtocolError {
    fn from(err: MappingError) -> ProtocolError {
        ProtocolError::new(ProtocolErrorKind::Service, Box::new(err))
    }
}
