use anyhow::{anyhow, Context};
use helpers::{build_deploy_msg, FromExpr};
use models::casper::v1::deploy_service_client::DeployServiceClient;
use models::casper::v1::propose_service_client::ProposeServiceClient;
use models::casper::v1::{
    deploy_response,
    find_deploy_response,
    propose_response,
    rho_data_response,
};
use models::casper::{DataAtNameByBlockQuery, FindDeployQuery, ProposeQuery};
use models::rhoapi::expr::ExprInstance;
use models::rhoapi::{Expr, Par};
use secp256k1::SecretKey;

mod helpers;
mod models;

pub struct Client {
    wallet_key: SecretKey,
    deploy_client: DeployServiceClient<tonic::transport::Channel>,
    propose_client: ProposeServiceClient<tonic::transport::Channel>,
}

impl Client {
    pub async fn new(
        wallet_key: SecretKey,
        deploy_service_url: String,
        propose_service_url: String,
    ) -> anyhow::Result<Self> {
        let deploy_client = DeployServiceClient::connect(deploy_service_url)
            .await
            .context("failed to connect to deploy service")?;

        let propose_client = ProposeServiceClient::connect(propose_service_url)
            .await
            .context("failed to connect to propose service")?;

        Ok(Self {
            wallet_key,
            deploy_client,
            propose_client,
        })
    }

    pub async fn full_deploy(&mut self, code: String) -> anyhow::Result<String> {
        let msg = build_deploy_msg(&self.wallet_key, code);

        let resp = self
            .deploy_client
            .do_deploy(msg)
            .await
            .context("do_deploy grpc error")?
            .into_inner()
            .message
            .context("missing do_deploy responce")?;

        let result = match resp {
            deploy_response::Message::Result(result) => result,
            deploy_response::Message::Error(err) => {
                return Err(anyhow!("do_deploy error: {err:?}"))
            }
        };

        let suffix = result
            .strip_prefix("Success!\nDeployId is: ")
            .context("failed to extract deploy id")?;

        let deploy_id = hex::decode(suffix).context("failed to decode deploy id")?;

        let resp = self
            .propose_client
            .propose(ProposeQuery { is_async: false })
            .await
            .context("propose grpc error")?
            .into_inner()
            .message
            .context("missing propose responce")?;

        match resp {
            propose_response::Message::Result(_) => (),
            propose_response::Message::Error(err) => return Err(anyhow!("propose error: {err:?}")),
        }

        let resp = self
            .deploy_client
            .find_deploy(FindDeployQuery { deploy_id })
            .await
            .context("find_deploy grpc error")?
            .into_inner()
            .message
            .context("missing find_deploy responce")?;

        match resp {
            find_deploy_response::Message::BlockInfo(result) => Ok(result.block_hash),
            find_deploy_response::Message::Error(err) => Err(anyhow!("find_deploy error: {err:?}")),
        }
    }

    pub async fn get_channel_value<T>(&mut self, hash: String, channel: String) -> anyhow::Result<T>
    where
        T: FromExpr,
    {
        let mut par = Par::default();
        par.exprs.push(Expr {
            expr_instance: Some(ExprInstance::GString(channel)),
        });

        let resp = self
            .deploy_client
            .get_data_at_name(DataAtNameByBlockQuery {
                par: Some(par),
                block_hash: hash,
                use_pre_state_hash: false,
            })
            .await
            .context("get_data_at_name grpc error")?
            .into_inner()
            .message
            .context("missing get_data_at_name responce")?;

        let payload = match resp {
            rho_data_response::Message::Payload(payload) => payload,
            rho_data_response::Message::Error(err) => {
                return Err(anyhow!("get_data_at_name error: {err:?}"))
            }
        };

        let par = payload
            .par
            .into_iter()
            .last()
            .context("missing par in get_data_at_name")?;
        let expr = par
            .exprs
            .into_iter()
            .next()
            .context("missing exprs in get_data_at_name")?;
        let expr = expr
            .expr_instance
            .context("missing expr_instance in get_data_at_name")?;

        T::from(expr)
    }
}
