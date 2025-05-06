pub struct JsonPointerPathBuilder {
    segments: Vec<String>,
}

impl JsonPointerPathBuilder {
    pub fn new() -> Self {
        Self {
            segments: Vec::new(),
        }
    }
}

impl JsonPointerPathBuilder {
    pub fn add_segment(&mut self, segment: String) -> &mut Self {
        if segment.contains("/") {
            let segment = segment.replace("/", "~1");
            self.segments.push(segment);
        } else {
            self.segments.push(segment);
        }
        self
    }

    pub fn back(&mut self) {
        self.segments.pop();
    }

    pub fn build(&self) -> String {
        self.segments.join("/")
    }
}