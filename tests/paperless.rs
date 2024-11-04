

#[cfg(test)]
mod tests {
    use testcontainers::{core::{IntoContainerPort, WaitFor}, runners::AsyncRunner, GenericImage, ImageExt};

    #[tokio::test]
    async fn test_paperless() {
        let container = GenericImage::new("paperlessngx/paperless-ngx", "latest")
            .with_exposed_port(8080.tcp())
            .with_wait_for(WaitFor::message_on_stdout("Ready to accept connections"))
            .with_network("bridge")
            .with_env_var("DEBUG", "1")
            .start()
            .await
            .expect("Failed to start Paperless");
    }
}
