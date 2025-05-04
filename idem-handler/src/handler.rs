use crate::HandlerOutput;
use crate::exchange::Exchange;

pub trait Handler<Input, Output, Metadata>: Send
where
    Input: Send,
    Output: Send,
    Metadata: Send,
{
    fn exec<'handler, 'exchange, 'result>(
        &'handler self,
        exchange: &'exchange mut Exchange<Input, Output, Metadata>,
    ) -> HandlerOutput<'result>
    where
        'handler: 'result,
        'exchange: 'result,
        Self: 'result;
}

#[cfg(test)]
mod test {
    use crate::HandlerOutput;
    use crate::exchange::Exchange;
    use crate::handler::Handler;
    use crate::status::{Code, HandlerStatus};

    pub struct TestHandler;

    #[derive(Default)]
    pub struct TestInput
    where
        Self: Send,
    {
        pub test_input: String,
    }

    #[derive(Default)]
    pub struct TestOutput
    where
        Self: Send,
    {
        pub test_output: String,
    }

    impl Handler<TestInput, TestOutput, ()> for TestHandler {
        fn exec<'handler, 'exchange, 'result>(
            &'handler self,
            exchange: &'exchange mut Exchange<TestInput, TestOutput, ()>,
        ) -> HandlerOutput<'result>
        where
            'handler: 'result,
            'exchange: 'result,
            Self: 'result,
        {
            Box::pin(async move {

                // Simple exchange of input and output values
                let consumed_input = exchange.take_request().unwrap();
                let mut output = TestOutput::default();
                output.test_output = consumed_input.test_input;
                exchange.save_output(output);
                Ok(HandlerStatus::new(Code::OK))
            })
        }
    }

    #[tokio::test(flavor = "current_thread")]
    async fn test_handler() {
        let handler = TestHandler;
        let mut exchange: Exchange<TestInput, TestOutput, ()> = Exchange::new();
        let mut input = TestInput::default();
        input.test_input = "test".to_string();
        exchange.save_input(input);
        handler.exec(&mut exchange).await.expect("Failed to execute test");
        let output = exchange.take_output().unwrap();
        assert_eq!(output.test_output, "test");
    }
}
