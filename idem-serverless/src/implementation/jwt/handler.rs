use idem_handler::exchange::Exchange;
use idem_handler::handler::Handler;
use idem_handler::status::{Code, HandlerExecutionError, HandlerStatus};
use lambda_http::aws_lambda_events::apigw::{ApiGatewayProxyRequest, ApiGatewayProxyResponse};
use std::future::Future;
use std::pin::Pin;
use jsonwebtoken::{Algorithm, Validation};
use lambda_http::Context;

pub type LambdaExchange = Exchange<ApiGatewayProxyRequest, ApiGatewayProxyResponse, Context>;
pub type HandlerOutput<'a> = Pin<Box<dyn Future<Output = Result<HandlerStatus, HandlerExecutionError>> + Send + 'a>>;

pub struct JwtValidationHandler;

impl JwtValidationHandler {
}


impl Handler<ApiGatewayProxyRequest, ApiGatewayProxyResponse, Context> for JwtValidationHandler {
    fn process<'handler, 'exchange, 'result>(
        &'handler self,
        exchange: &'exchange mut LambdaExchange,
    ) -> HandlerOutput<'result>
    where
        'handler: 'result,
        'exchange: 'result,
        Self: 'result,
    {
        Box::pin(async move {

            let mut validation = Validation::new(Algorithm::RS256);
            validation.set_audience(&["me"]);
            validation.set_required_spec_claims(&["exp", "sub", "aud"]);


            Ok(HandlerStatus::new(Code::OK)) })
    }
}


mod test {
    use std::error::Error;
    use base64::Engine;
    use base64::prelude::BASE64_URL_SAFE_NO_PAD;
    use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
    use rsa::pkcs1::EncodeRsaPrivateKey;
    use rsa::RsaPrivateKey;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize, Deserialize)]
    struct Claims {
        sub: String,
        exp: usize,
    }

    fn b64_decode(s: &str) -> Result<Vec<u8>, Box<dyn Error>> {
        Ok(BASE64_URL_SAFE_NO_PAD.decode(s)?)
    }

    fn rsa_private_key_from_jwk(jwk: &serde_json::Value) -> Result<RsaPrivateKey, Box<dyn Error>> {
        let n = rsa::BigUint::from_bytes_be(&b64_decode(jwk["n"].as_str().unwrap())?);
        let e = rsa::BigUint::from_bytes_be(&b64_decode(jwk["e"].as_str().unwrap())?);
        let d = rsa::BigUint::from_bytes_be(&b64_decode(jwk["d"].as_str().unwrap())?);
        let p = rsa::BigUint::from_bytes_be(&b64_decode(jwk["p"].as_str().unwrap())?);
        let q = rsa::BigUint::from_bytes_be(&b64_decode(jwk["q"].as_str().unwrap())?);
        Ok(RsaPrivateKey::from_components(n, e, d, vec![p, q]).unwrap())
    }

    #[test]
    fn test_key_gen() {
        let jwk_private_key = r#"
        {
            "p": "5uX_xavee0ATqxU9IlvNzCh5Q3wXjI8sIlvGHQTaHzwSTSkBqpRQHFQXzFQUlPvEJkvAc9wi6ofIt7VdJXgPEwkLiEuAb7oretTD079BM37fwijk97olTGdWEjCHFV0AvFvMPeSk6XtB-eSdHd55Odia63ZtvNKYI-pctRtSlfc",
            "kty": "RSA",
            "q": "yBRrrB9LqapmHbnVADqvt6LwiFmWH3ulFoC-XkuJz0nFV5GqE9VHvoPHsHERW83cjWDR3O-1wCsEkxH0Ai11TQYaA6OWAiwROJUezRnvZHDt8tsgu_se0SJODjTVqr0Wo1yhtAPqHIN0bzXgnDRurTg_LKjZxTcXCeobdk9-3sU",
            "d": "P_Ou2z_MCHm9xOiSCPMfLtfwn1E4lbb8vH7fnomYZYsX91tHfU_JgOSBbq9DFrKvRG4OYLi8l4j_Uxs17tIyaPHJtWebafF0VuJy3y6TJqrmCyVVSa5glKbq6Bi3bGbwxl75B5Fx9OyoADmSrXg_Td-zL31OEBfQGt_5Yn6l2iZYDMkPedYj5xYqUxEkqx6GcNEM4CL-EZf05GEMiq_qUlTSM5eISj0Nk5dSN7O79VrVvFpCXgN4df5QuoN9r_0EuGeAtV-knHVYqpR2RC342GUbzUdNpOYGVzUSs47E-LJLLonNnBvW0JHUNzC9DtFv45Y9yKVj_XsLUgjEcFIzmQ",
            "e": "AQAB",
            "kid": "DDbt045YVtnjCkzHUv-rFN4wPfGD3Upk9_da_yweZ1c",
            "qi": "JkxIDydvNZv8Ct9TNwoK5ar2tHkk3Pf3BqZsuyR7j3EzyfrZSVqS6qR6km7gL3H9KhImroTdAJlTwlM5ZqXnZpNRa9lqBUgn2h2SKEjjqlacqqITiSNgayQzC6G9kyams9_uSwy6-JjweGosjrWK74730BpscGYKpndFNk4qK6Y",
            "dp": "pTSNQ7bMMa1QJUnF-w5qehe_Y9ym0Mgj0NWPM3YkRtLpWVHswkrp4sr8WBMUwuA8oRX0NjGcvee3YlIeuk9jocAIA1XaKJaww2r2Tkv6b8joenheEy2ZwEfzmoIkNNHdU-fug55TrEanlw_Opu9mF1B2z-BldgPMHW5zNJW_ClM",
            "alg": "RS256",
            "dq": "ClZTwc7UH-333K1PPfXKQlieyMyoHvRKcUExlLmeYyFSmtWhzeiFDmjMlmchGHcoX_2SmjGgWE9gqyCQVNR4bQRVr75x76bLNPsvXjVq0uuqv5Nmu4-b5f45vi4oo-ulEcelayGQpOx9xYkpE6j51uVDDlGi_rd77z0zMgelbGk",
            "n": "tHYa58pgzbOMxt-jEvuPMbw_ymgHv4j7nkqUiLzfIdKgJDCed6zrq7ikqzX0Ach9YiYaX-iwzjp5LwRueAmgEmNMf76ULtVml1O2yKY_IQ6tzg-L8XJL2MxPZV7pDV_awg-q47ArR2DMuLZNQgZKmjH8-IsIwZ8oVtvnYXOCe7-dwFTwcTR1X1aPibjFnXVuLtqFYVgpMwn-bywwk-5PQoo3Lhi-usdIeFxqXNRp5NnupakKh7mQ9GrCCpF---MRNUa8pwlqgRqvuzNP5PnBngPrHnHSZbCzopFaoqRkY1FAlGhNkkoqE5MMNIaoa_lWTmR_hiTqPn50-IDWrSuZEw"
        }
        "#;
        let jwk: serde_json::Value = serde_json::from_str(jwk_private_key).unwrap();
        let mut private_key = rsa_private_key_from_jwk(&jwk).unwrap();

        let der = private_key.to_pkcs1_der().unwrap().as_bytes().to_vec();
        let encoding_key = EncodingKey::from_rsa_der(&der);
        let claims = Claims {
            sub: "user123".to_string(),
            exp: 2000000000,
        };
        let mut header = Header::new(Algorithm::RS256);
        header.kid = jwk.get("kid").and_then(|v| v.as_str()).map(|s| s.to_string());
        let token = encode(&header, &claims, &encoding_key).unwrap();
        println!("Generated JWT token:\n{}", token);
        assert!(!token.is_empty())
    }
}
