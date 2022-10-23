use flagset::FlagSet;

flagset::flags! {
    /// Speicfy which part should not be updated.
    pub enum PixivArtworkLock: u8 {
        Title = 1,
        Author = 2,
        Description = 4,
        IsNsfw = 8,
    }
}

pub struct PixivArtwork {
    /// The artwork ID
    pub id: u64,
    /// The artwork title
    pub title: String,
    /// The artwork author
    pub author: String,
    /// The author's UID
    pub uid: u64,
    /// The artwork description
    pub description: String,
    /// The artwork's page count
    pub count: u64,
    /// Whether the artwork is NSFW
    pub is_nsfw: bool,
    /// Specify which part should not be updated.
    pub lock: FlagSet<PixivArtworkLock>,
}
