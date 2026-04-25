use async_trait::async_trait;

#[async_trait]
pub trait Provider {
    fn id(&self) -> &str;

    async fn chat(&self, model: &str);
}
