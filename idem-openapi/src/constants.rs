pub(crate) mod format {
    pub const DATE: &str = "date";
    pub const DATE_TIME: &str = "date-time";
    pub const EMAIL: &str = "email";
    pub const IPV4: &str = "ipv4";
    pub const IPV6: &str = "ipv6";
    pub const UUID: &str = "uuid";
}

pub(crate) mod pattern {
    pub const EMAIL_REGEX: &str = r#"^[\w-\.]+@([\w-]+\.)+[\w-]{2,4}$"#;
    pub const DATE_TIME_REGEX: &str =
        r#"^((?:(\d{4}-\d{2}-\d{2})T(\d{2}:\d{2}:\d{2}(?:\.\d+)?))(Z|[\+-]\d{2}:\d{2})?)$"#;
    pub const DATE_REGEX: &str = r#"^(\d{4}-\d{2}-\d{2})$"#;
    pub const UUID_REGEX: &str = r#"^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"#;
    pub const IPV6_REGEX: &str = r#"^((?:[0-9a-fA-F]{1,4}:){7}[0-9a-fA-F]{1,4})$"#;

    // technically, this regex allows you to have an invalid ip. i.e. 999.999.999.999.
    // is it worth adding more constraints?
    pub const IPV4_REGEX: &str = r#"^((?:[0-9]{1,3}\.){3}[0-9]{1,3})$"#;
}