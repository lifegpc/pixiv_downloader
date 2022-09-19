use super::traits::GetRequestParams;
use crate::error::PixivDownloaderError;
use bytes::BytesMut;
use hyper::{body::HttpBody, Body, Request};
use std::collections::HashMap;

pub struct RequestParams {
    pub params: HashMap<String, Vec<String>>,
}

impl RequestParams {
    pub fn get<S: AsRef<str> + ?Sized>(&self, name: &S) -> Option<&str> {
        match self.params.get(name.as_ref()) {
            Some(v) => {
                if v.len() > 0 {
                    Some(&v[0])
                } else {
                    None
                }
            }
            None => None,
        }
    }
}

#[async_trait]
impl GetRequestParams for Request<Body> {
    async fn get_params(&mut self) -> Result<RequestParams, PixivDownloaderError> {
        let mut params = HashMap::new();
        if let Some(query) = self.uri().query() {
            params = urlparse::parse_qs(query);
        }
        if let Some(ct) = self.headers().get(hyper::header::CONTENT_TYPE) {
            if ct == "application/x-www-form-urlencoded" {
                let mut body = BytesMut::new();
                loop {
                    if let Some(d) = self.body_mut().data().await {
                        body.extend_from_slice(&d?);
                    } else {
                        break;
                    }
                }
                let body = String::from_utf8(body.to_vec())?;
                let params2 = urlparse::parse_qs(&body);
                for (k, v) in params2 {
                    match params.get_mut(&k) {
                        Some(l) => {
                            l.extend(v);
                        }
                        None => {
                            params.insert(k, v);
                        }
                    }
                }
            }
        }
        Ok(RequestParams { params })
    }
}
