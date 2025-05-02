use std::any::{Any, TypeId};
use std::collections::HashMap;

pub struct Exchange<Input, Output, Metadata> {
    metadata: Option<Metadata>,
    input: Option<Input>,
    output: Option<Output>,
    input_listeners: Vec<Callback<Input>>,
    output_listeners: Vec<Callback<Output>>,
    attachments: Attachments,
}

impl<Input, Output, Metadata> Exchange<Input, Output, Metadata>
where
    Input: Send + 'static,
    Output: Send + 'static,
    Metadata: Send,
{
    pub fn new() -> Self {
        Self {
            metadata: None,
            input: None,
            output: None,
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
        if let Some(input) = &mut self.input {
            for mut callback in &mut self.input_listeners.drain(..) {
                callback.invoke(input, &mut self.attachments);
            }
            Ok(())
        } else {
            Err(())
        }
    }

    fn execute_output_callbacks(&mut self) -> Result<(), ()> {
        if let Some(output) = &mut self.output {
            for mut callback in &mut self.output_listeners.drain(..) {
                callback.invoke(output, &mut self.attachments);
            }
            Ok(())
        } else {
            Err(())
        }
    }

    pub fn save_input(&mut self, request: Input) {
        self.input = Some(request);
    }

    pub fn input(&self) -> Result<&Input, ()> {
        match &self.input {
            Some(out) => Ok(out),
            None => Err(()),
        }
    }

    pub fn input_mut(&mut self) -> Result<&mut Input, ()> {
        match &mut self.input {
            Some(out) => Ok(out),
            None => Err(()),
        }
    }

    pub fn take_request(&mut self) -> Result<Input, ()> {
        if let Ok(_) = self.execute_input_callbacks() {
            self.input.take().ok_or_else(|| ())
        } else {
            Err(())
        }
    }

    pub fn save_output(&mut self, response: Output) {
        self.output = Some(response);
    }

    pub fn output(&self) -> Result<&Output, ()> {
        match &self.output {
            Some(out) => Ok(out),
            None => Err(()),
        }
    }

    pub fn output_mut(&mut self) -> Result<&mut Output, ()> {
        match &mut self.output {
            Some(out) => Ok(out),
            None => Err(()),
        }
    }

    pub fn take_output(&mut self) -> Result<Output, ()> {
        if let Ok(_) = self.execute_output_callbacks() {
            self.output.take().ok_or_else(|| ())
        } else {
            Err(())
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
pub struct AttachmentKey(pub &'static str);

pub struct Callback<T> {
    callback: Box<dyn FnMut(&mut T, &mut Attachments) + Send>,
}

impl<T> Callback<T>
where
    T: Send + 'static,
{
    pub fn new(callback: impl FnMut(&mut T, &mut Attachments) + Send + 'static) -> Self {
        Self {
            callback: Box::new(callback),
        }
    }

    pub fn invoke(&mut self, write: &mut T, attachments: &mut Attachments) {
        (self.callback)(write, attachments);
    }
}
