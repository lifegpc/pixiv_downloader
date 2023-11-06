use super::check::CheckUnknown;
use super::image_urls::ImageUrls;
use super::tag::Tag;
use json::JsonValue;
use proc_macros::check_json_keys;

#[derive(Clone)]
pub struct PixivAppIllust {
    data: JsonValue,
}

impl PixivAppIllust {
    pub fn new(data: JsonValue) -> Self {
        Self { data }
    }

    pub fn id(&self) -> Option<u64> {
        self.data["id"].as_u64()
    }

    pub fn title(&self) -> Option<&str> {
        self.data["title"].as_str()
    }

    pub fn typ(&self) -> Option<&str> {
        self.data["type"].as_str()
    }

    pub fn image_urls(&self) -> ImageUrls {
        return ImageUrls::new(self.data["image_urls"].clone());
    }

    pub fn caption(&self) -> Option<&str> {
        self.data["caption"].as_str()
    }

    pub fn caption_is_empty(&self) -> bool {
        self.caption().unwrap_or("").is_empty()
    }

    pub fn restrict(&self) -> Option<u64> {
        self.data["restrict"].as_u64()
    }

    pub fn user_id(&self) -> Option<u64> {
        self.data["user"]["id"].as_u64()
    }

    pub fn user_name(&self) -> Option<&str> {
        self.data["user"]["name"].as_str()
    }

    pub fn user_account(&self) -> Option<&str> {
        self.data["user"]["account"].as_str()
    }

    pub fn user_profile_image_urls_medium(&self) -> Option<&str> {
        self.data["user"]["profile_image_urls"]["medium"].as_str()
    }

    pub fn user_is_followed(&self) -> Option<bool> {
        self.data["user"]["is_followed"].as_bool()
    }

    pub fn tags(&self) -> Vec<Tag> {
        let mut tags = Vec::new();
        for tag in self.data["tags"].members() {
            tags.push(Tag::new(tag.clone()));
        }
        tags
    }

    pub fn create_date(&self) -> Option<&str> {
        self.data["create_date"].as_str()
    }

    pub fn page_count(&self) -> Option<u64> {
        self.data["page_count"].as_u64()
    }

    pub fn width(&self) -> Option<u64> {
        self.data["width"].as_u64()
    }

    pub fn height(&self) -> Option<u64> {
        self.data["height"].as_u64()
    }

    pub fn sanity_level(&self) -> Option<u64> {
        self.data["sanity_level"].as_u64()
    }

    pub fn x_restrict(&self) -> Option<u64> {
        self.data["x_restrict"].as_u64()
    }

    pub fn original_image_url(&self) -> Option<&str> {
        self.data["meta_single_page"]["original_image_url"].as_str()
    }

    pub fn meta_pages(&self) -> Vec<ImageUrls> {
        let mut meta_pages = Vec::new();
        for meta_page in self.data["meta_pages"].members() {
            meta_pages.push(ImageUrls::new(meta_page["image_urls"].clone()));
        }
        meta_pages
    }

    pub fn total_view(&self) -> Option<u64> {
        self.data["total_view"].as_u64()
    }

    pub fn total_bookmarks(&self) -> Option<u64> {
        self.data["total_bookmarks"].as_u64()
    }

    pub fn is_bookmarked(&self) -> Option<bool> {
        self.data["is_bookmarked"].as_bool()
    }

    pub fn visible(&self) -> Option<bool> {
        self.data["visible"].as_bool()
    }

    pub fn is_muted(&self) -> Option<bool> {
        self.data["is_muted"].as_bool()
    }

    pub fn total_comments(&self) -> Option<u64> {
        self.data["total_comments"].as_u64()
    }

    pub fn illust_ai_type(&self) -> Option<u64> {
        self.data["illust_ai_type"].as_u64()
    }
}

impl CheckUnknown for PixivAppIllust {
    fn check_unknown(&self) -> Result<(), String> {
        check_json_keys!(
            "id"+,
            "title"+,
            "type"+typ,
            "image_urls": ["square_medium", "medium", "large"],
            "caption"+,
            "restrict"+,
            "user": [
                "id"+,
                "name"+,
                "account"+,
                "profile_image_urls": ["medium"],
                "is_followed"+
            ],
            "tags",
            "tools",
            "create_date"+,
            "page_count"+,
            "width"+,
            "height"+,
            "sanity_level"+,
            "x_restrict"+,
            "series",
            "meta_single_page": ["original_image_url"],
            "meta_pages",
            "total_view"+,
            "total_bookmarks"+,
            "is_bookmarked"+,
            "visible"+,
            "is_muted"+,
            "total_comments"+,
            "illust_ai_type"+,
            "illust_book_style",
            "comment_access_control"
        );
        for i in self.tags() {
            i.check_unknown()?;
        }
        for i in self.meta_pages() {
            i.check_unknown()?;
        }
        Ok(())
    }
}

impl std::fmt::Debug for PixivAppIllust {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PixivAppIllust")
            .field("id", &self.id())
            .field("title", &self.title())
            .field("type", &self.typ())
            .field("image_urls", &self.image_urls())
            .field("caption", &self.caption())
            .field("restrict", &self.restrict())
            .field("user_id", &self.user_id())
            .field("user_name", &self.user_name())
            .field("user_account", &self.user_account())
            .field(
                "user_profile_image_urls_medium",
                &self.user_profile_image_urls_medium(),
            )
            .field("user_is_followed", &self.user_is_followed())
            .field("tags", &self.tags())
            .field("create_date", &self.create_date())
            .field("page_count", &self.page_count())
            .field("width", &self.width())
            .field("height", &self.height())
            .field("sanity_level", &self.sanity_level())
            .field("x_restrict", &self.x_restrict())
            .field("original_image_url", &self.original_image_url())
            .field("meta_pages", &self.meta_pages())
            .field("total_view", &self.total_view())
            .field("total_bookmarks", &self.total_bookmarks())
            .field("is_bookmarked", &self.is_bookmarked())
            .field("visible", &self.visible())
            .field("is_muted", &self.is_muted())
            .field("total_comments", &self.total_comments())
            .field("illust_ai_type", &self.illust_ai_type())
            .finish_non_exhaustive()
    }
}
