use std::fmt::{Display, Formatter};
use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, Not};

#[derive(Debug)]
pub struct HandlerExecutionError {
    message: String,
}

impl Display for HandlerExecutionError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Fatal error: {}", self.message)
    }
}

pub struct HandlerStatus {
    code: Code,
    message: Option<&'static str>,
    details: Option<&'static str>,
}

impl HandlerStatus {
    pub fn code(&self) -> Code {
        self.code
    }
    pub fn message(&self) -> &'static str {
        self.message.unwrap_or_else(|| "")
    }

    pub fn details(&self) -> &'static str {
        self.details.unwrap_or_else(|| "")
    }

    pub fn new(code: Code) -> HandlerStatus {
        Self {
            code,
            message: None,
            details: None,
        }
    }

    pub fn set_message(mut self, message: &'static str) -> HandlerStatus {
        self.message = Some(message);
        self
    }

    pub fn set_details(mut self, description: &'static str) -> HandlerStatus {
        self.details = Some(description);
        self
    }
}

#[derive(Clone, Copy)]
pub struct Code(pub i32);

impl Code {
    pub const OK: Self = Self(1);
    pub const REQUEST_COMPLETED: Self = Self(1 << 1);
    pub const SERVER_ERROR: Self = Self(1 << 2);
    pub const CLIENT_ERROR: Self = Self(1 << 3);
    pub const DISABLED: Self = Self(1 << 4);
    pub const TIMEOUT: Self = Self(1 << 5);
    pub const CONTINUE: Self = Self(1 << 6);

    pub fn any_flags(&self, flags: Code) -> bool {
        self.0 & flags.0 != 0
    }

    pub fn any_flags_clear(&self, flags: Code) -> bool {
        self.0 & flags.0 != flags.0
    }

    pub fn all_flags(&self, flags: Code) -> bool {
        self.0 & flags.0 == flags.0
    }

    pub fn all_flags_clear(&self, flags: Code) -> bool {
        self.0 & flags.0 == 0
    }
}

impl PartialEq for Code {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl BitOrAssign for Code {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0
    }
}

impl BitAndAssign for Code {
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0
    }
}

impl Not for Code {
    type Output = Self;

    fn not(self) -> Self::Output {
        Self(!self.0)
    }
}

impl BitAnd for Code {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

impl BitOr for Code {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

#[cfg(test)]
mod tests {
    use crate::status::Code;

    #[test]
    fn test_status_flags() {
        let mut test_code = Code::OK;
        test_code |= Code::REQUEST_COMPLETED;
        assert!(test_code.all_flags(Code::OK | Code::REQUEST_COMPLETED));
        assert!(test_code.all_flags_clear(Code::DISABLED | Code::TIMEOUT | Code::CONTINUE | Code::CLIENT_ERROR | Code::SERVER_ERROR));
    }

    #[test]
    fn test_status_constructs() {
        let test_code1 = Code::OK | Code::SERVER_ERROR;
        let test_code2 = Code::from(test_code1 | Code::REQUEST_COMPLETED);
        assert!(test_code2.all_flags(Code::OK | Code::SERVER_ERROR | Code::REQUEST_COMPLETED));
    }
}
