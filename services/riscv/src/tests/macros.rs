macro_rules! service {
    ($service:expr, $method:ident, $ctx:expr) => {{
        let resp = $service.$method($ctx);
        assert!(!resp.is_error(), resp.error_message);

        resp.succeed_data
    }};
    ($service:expr, $method:ident, $ctx:expr, $payload:expr) => {{
        let resp = $service.$method($ctx, $payload);
        assert!(!resp.is_error(), resp.error_message);

        resp.succeed_data
    }};
}
