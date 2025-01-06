pub trait Publisher {
    async fn post_text(&self, text: &str) -> Result<(), Box<dyn std::error::Error>>;
    async fn post_audio(&self, text: &str, audio: &str) -> Result<(), Box<dyn std::error::Error>>;

}
