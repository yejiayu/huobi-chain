use admission_control::AdmissionControl;
use binding_macro::{cycles, service};
use protocol::traits::{ExecutorParams, ServiceResponse, ServiceSDK};
use protocol::types::{ServiceContext, SignedTransaction};

use multi_signature::MultiSignatureService;

pub struct AuthorizationService<AC, SDK> {
    _sdk:              SDK,
    multi_sig:         MultiSignatureService<SDK>,
    admission_control: AC,
}

#[service]
impl<AC, SDK> AuthorizationService<AC, SDK>
where
    AC: AdmissionControl,
    SDK: ServiceSDK,
{
    pub fn new(_sdk: SDK, multi_sig: MultiSignatureService<SDK>, admission_control: AC) -> Self {
        Self {
            _sdk,
            multi_sig,
            admission_control,
        }
    }

    #[cycles(21_000)]
    #[read]
    fn check_authorization(
        &self,
        ctx: ServiceContext,
        payload: SignedTransaction,
    ) -> ServiceResponse<()> {
        let resp = self
            .multi_sig
            .verify_signature(ctx.clone(), payload.clone());
        if resp.is_error() {
            return ServiceResponse::<()>::from_error(
                102,
                format!(
                    "verify transaction signature error {:?}",
                    resp.error_message
                ),
            );
        }

        if !self.admission_control.is_allowed(&ctx, payload) {
            return ServiceResponse::<()>::from_error(
                102,
                "The transaction is not allowed".to_owned(),
            );
        }

        ServiceResponse::from_succeed(())
    }
}
