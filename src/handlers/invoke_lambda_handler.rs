use crate::exchange::{Exchange, Handler};
use aws_config::BehaviorVersion;
use aws_sdk_lambda::primitives::Blob;
use aws_sdk_lambda::Client as LambdaClient;
use lambda_http::aws_lambda_events::apigw::{ApiGatewayProxyRequest, ApiGatewayProxyResponse};
use std::collections::HashMap;
use std::future::Future;
use std::ops::Add;
use std::pin::Pin;

pub(crate) struct FunctionName(pub String);

#[derive(Debug, Clone, Default)]
pub(crate) struct AWSLambdaFunctionProxyHandler {
    lambda_client: Option<LambdaClient>,
    paths: HashMap<String, String>,
}

impl AWSLambdaFunctionProxyHandler {
    pub(crate) async fn new() -> Self {
        let mut test_path: HashMap<String, String> = HashMap::new();
        test_path.insert(
            "/path/to/resource@POST".to_string(),
            "arn:aws:lambda:ca-central-1:173982495217:function:test-lambda-function-destination"
                .to_string(),
        );
        Self {
            lambda_client: Some(LambdaClient::new(
                &aws_config::load_defaults(BehaviorVersion::latest()).await,
            )),
            paths: test_path,
        }
    }
}

impl Handler<ApiGatewayProxyRequest, ApiGatewayProxyResponse, HashMap<String, String>>
    for AWSLambdaFunctionProxyHandler
{
    fn process<'i1, 'i2, 'o>(
        &'i1 self,
        exchange: &'i2 mut Exchange<
            ApiGatewayProxyRequest,
            ApiGatewayProxyResponse,
            HashMap<String, String>,
        >,
    ) -> Pin<Box<dyn Future<Output = Result<(), ()>> + Send + 'o>>
    where
        'i1: 'o,
        'i2: 'o,
        Self: 'o,
    {
        let client = self.lambda_client.clone();
        Box::pin(async move {
            match exchange.consume_request() {
                Ok(request) => {
                    let payload = serde_json::to_string(&request).unwrap();
                    let path = match request.path {
                        Some(path) => path,
                        None => todo!(),
                    };
                    let method = request.http_method;
                    let function_key = path.add("@").add(method.as_str());
                    let function_name = match self.paths.get(&function_key) {
                        None => todo!(),
                        Some(function) => function.clone(),
                    };
                    let proxy_blob = Blob::new(payload);
                    match client
                        .unwrap()
                        .invoke()
                        .function_name(&function_name)
                        .payload(proxy_blob)
                        .send()
                        .await
                    {
                        Ok(response) => {
                            if response.function_error().is_some() {
                                todo!("Handle function failure")
                            }

                            let response_payload_bytes = response.payload.unwrap().into_inner();
                            let lambda_response: ApiGatewayProxyResponse =
                                match serde_json::from_slice(&response_payload_bytes) {
                                    Ok(response) => response,
                                    Err(_) => todo!("failed to get response from lambda function call.")
                                };
                            exchange.save_output(lambda_response);
                            Ok(())
                        }
                        Err(_) => todo!("Handle SDK error"),
                    }
                }
                Err(_) => todo!("Handle no request failure"),
            }
        })
    }
}
