use std::any::{Any, TypeId};
use std::collections::HashMap;

pub struct Exchange<Input, Output, Metadata> {
    metadata: Option<Metadata>,
    input: Input,
    output: Output,
    input_listeners: Vec<Callback<Input>>,
    output_listeners: Vec<Callback<Output>>,
    attachments: Attachments,
}

impl<Input, Output, Metadata> Exchange<Input, Output, Metadata>
where
    Input: Default + Send + 'static,
    Output: Default + Send + 'static,
    Metadata: Send,
{
    pub fn new() -> Self {
        Self {
            metadata: None,
            input: Input::default(),
            output: Output::default(),
            input_listeners: vec![],
            output_listeners: vec![],
            attachments: Attachments::new(),
        }
    }

    pub fn add_metadata(&mut self, metadata: Metadata) {
        self.metadata = Some(metadata);
    }

    pub fn attachments(&self) -> &Attachments {
        &self.attachments
    }

    pub fn attachments_mut(&mut self) -> &mut Attachments {
        &mut self.attachments
    }

    pub fn add_input_listener(
        &mut self,
        callback: impl FnMut(&mut Input, &mut Attachments) + Send + 'static,
    ) {
        self.input_listeners.push(Callback::new(callback));
    }

    pub fn add_output_listener(
        &mut self,
        callback: impl FnMut(&mut Output, &mut Attachments) + Send + 'static,
    ) {
        self.output_listeners.push(Callback::new(callback));
    }

    fn execute_input_callbacks(&mut self) -> Result<(), ()> {
        self.input_listeners.iter_mut().for_each(|listener| {
            listener.invoke(&mut self.input, &mut self.attachments);
        });
        Ok(())
    }

    fn execute_output_callbacks(&mut self) -> Result<(), ()> {
        self.output_listeners.iter_mut().for_each(|listener| {
            listener.invoke(&mut self.output, &mut self.attachments);
        });
        Ok(())
    }

    pub fn save_input(&mut self, request: Input) {
        self.input = request;
    }

    pub fn input(&self) -> Result<&Input, ()> {
        Ok(&self.input)
    }

    pub fn input_mut(&mut self) -> Result<&mut Input, ()> {
        Ok(&mut self.input)
    }

    pub fn consume_request(&mut self) -> Result<Input, ()> {
        match self.execute_input_callbacks() {
            Ok(_) => {
                let consumed = std::mem::take(&mut self.input);
                Ok(consumed)
            }
            Err(_) => Err(()),
        }
    }

    pub fn save_output(&mut self, response: Output) {
        self.output = response;
    }

    pub fn output(&self) -> Result<&Output, ()> {
        Ok(&self.output)
    }

    pub fn output_mut(&mut self) -> Result<&mut Output, ()> {
        Ok(&mut self.output)
    }

    pub fn consume_output(&mut self) -> Result<Output, ()> {
        match self.execute_output_callbacks() {
            Ok(_) => {
                let consumed = std::mem::take(&mut self.output);
                Ok(consumed)
            }
            Err(_) => Err(()),
        }
    }
}

pub struct Attachments {
    attachments: HashMap<(AttachmentKey, TypeId), Box<dyn Any + Send>>,
}

impl Attachments {
    pub fn new() -> Self {
        Self {
            attachments: HashMap::new(),
        }
    }

    pub fn add_attachment<K>(&mut self, key: AttachmentKey, value: Box<dyn Any + Send>)
    where
        K: Send + 'static,
    {
        let type_id = TypeId::of::<K>();
        self.attachments.insert((key, type_id), value);
    }

    pub fn attachment<K>(&self, key: AttachmentKey) -> Option<&K>
    where
        K: Send + 'static,
    {
        let type_id = TypeId::of::<K>();
        if let Some(option_any) = self.attachments.get(&(key, type_id)) {
            option_any.downcast_ref::<K>()
        } else {
            None
        }
    }

    pub fn attachment_mut<K>(&mut self, key: AttachmentKey) -> Option<&mut K>
    where
        K: Send + 'static,
    {
        let type_id = TypeId::of::<K>();
        if let Some(option_any) = self.attachments.get_mut(&(key, type_id)) {
            option_any.downcast_mut::<K>()
        } else {
            None
        }
    }
}

// TODO - change how attachment keys work (probably string)
/* I wanted to make this struct use TypeId::of::<>() but it's not stable. */
#[derive(PartialOrd, PartialEq, Hash, Eq)]
pub struct AttachmentKey(pub u32);

pub struct Callback<P> {
    callback: Box<dyn FnMut(&mut P, &mut Attachments) + Send>,
}

impl<P> Callback<P>
where
    P: Send + 'static,
{
    pub fn new(callback: impl FnMut(&mut P, &mut Attachments) + Send + 'static) -> Self {
        Self {
            callback: Box::new(callback),
        }
    }

    pub fn invoke(&mut self, write: &mut P, attachments: &mut Attachments) {
        (self.callback)(write, attachments);
    }
}
