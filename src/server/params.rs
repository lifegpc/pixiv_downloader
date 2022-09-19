use super::traits::GetRequestParams;
use crate::error::PixivDownloaderError;
use crate::ext::try_err::TryErr;
use crate::gettext;
use bytes::{Buf, BytesMut};
use hyper::{body::HttpBody, Body, Request};
use multipart::server::{Multipart, ReadEntryResult};
use std::collections::HashMap;
use std::io::Read;

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
            let ct = ct.to_str()?.to_owned();
            let cts = ct.to_lowercase();
            if cts == "application/x-www-form-urlencoded" {
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
            } else if cts.starts_with("multipart/form-data") {
                let mut body = BytesMut::new();
                loop {
                    if let Some(d) = self.body_mut().data().await {
                        body.extend_from_slice(&d?);
                    } else {
                        break;
                    }
                }
                let mut r = body.reader();
                let boundary = ct
                    .find("boundary=")
                    .try_err(gettext("Failed to find boundary."))?;
                let boundary = &ct[boundary + 9..];
                let params2 = Multipart::with_body(&mut r, boundary);
                let mut entry = params2.into_entry();
                loop {
                    match entry {
                        ReadEntryResult::Entry(mut data) => {
                            if data.is_text() {
                                let mut s = String::new();
                                data.data.read_to_string(&mut s)?;
                                let name = data.headers.name.to_string();
                                match params.get_mut(&name) {
                                    Some(l) => {
                                        l.push(s);
                                    }
                                    None => {
                                        params.insert(name, vec![s]);
                                    }
                                }
                            }
                            entry = data.next_entry();
                        }
                        ReadEntryResult::End(_) => {
                            break;
                        }
                        ReadEntryResult::Error(_, e) => {
                            return Err(PixivDownloaderError::from(e));
                        }
                    }
                }
            }
        }
        Ok(RequestParams { params })
    }
}
