pub struct HandlerStatus {
    code: HandlerStatusCode,
    message: Option<&'static str>,
    description: Option<&'static str>
}

impl From<HandlerStatusCode> for HandlerStatus {
    fn from(value: HandlerStatusCode) -> Self {
        Self {
            code: value,
            message: None,
            description: None
        }
    }
}

impl From<(HandlerStatusCode, &'static str)> for HandlerStatus {
    fn from(value: (HandlerStatusCode, &'static str)) -> Self {
        Self {
            code: value.0,
            message: Some(value.1),
            description: None
        }
    }
}

impl From<(HandlerStatusCode, &'static str, &'static str)> for HandlerStatus {
    fn from(value: (HandlerStatusCode, &'static str, &'static str)) -> Self {
        Self {
            code: value.0,
            message: Some(value.1),
            description: Some(value.2)
        }
    }
}

pub enum HandlerStatusCode {
    Ok,
    RequestCompleted,
    ServerError,
    ClientError,
    Disabled,
    Timeout,
    Continue
}