use protocol::{traits::ServiceResponse, types::Address, Bytes};

pub trait ChainInterface {
    fn get_storage(&self, key: &Bytes) -> Bytes;

    fn set_storage(&mut self, key: Bytes, val: Bytes);

    fn service_call(
        &mut self,
        service: &str,
        method: &str,
        payload: &str,
        current_cycle: u64,
        readonly: bool,
    ) -> ServiceResponse<(String, u64)>;

    fn contract_call(
        &mut self,
        address: Address,
        args: Bytes,
        current_cycle: u64,
    ) -> ServiceResponse<(String, u64)>;
}
