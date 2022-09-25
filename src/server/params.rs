use super::traits::GetRequestParams;
use crate::error::PixivDownloaderError;
use crate::ext::try_err::TryErr;
use crate::gettext;
use bytes::{Buf, BytesMut};
use hyper::{body::HttpBody, Body, Request};
use multipart::server::{Multipart, ReadEntryResult};
use std::collections::HashMap;
use std::io::Read;

/// Parameters from request.
pub struct RequestParams {
    /// Parameters.
    pub params: HashMap<String, Vec<String>>,
}

#[allow(dead_code)]
impl RequestParams {
    /// Get parameter.
    /// * `name` - Parameter name.
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

    /// Get parameter and return it as boolean.
    /// * `name` - Parameter name.
    pub fn get_bool<S: AsRef<str> + ?Sized>(
        &self,
        name: &S,
    ) -> Result<Option<bool>, PixivDownloaderError> {
        match self.get(name) {
            Some(v) => {
                let v = v.trim();
                if v == "true" {
                    Ok(Some(true))
                } else if v == "false" {
                    Ok(Some(false))
                } else {
                    match v.parse::<i64>() {
                        Ok(v) => return Ok(Some(v != 0)),
                        Err(_) => {}
                    }
                    Err(gettext("Invalid boolean value.").into())
                }
            }
            None => Ok(None),
        }
    }

    /// Get parameter and return it as [u64].
    /// * `name` - Parameter name.
    pub fn get_u64<S: AsRef<str> + ?Sized>(
        &self,
        name: &S,
    ) -> Result<Option<u64>, PixivDownloaderError> {
        match self.get(name) {
            Some(v) => {
                let v = v.trim();
                match v.parse::<u64>() {
                    Ok(v) => Ok(Some(v)),
                    Err(_) => Err(gettext("Invalid unsigned 64bit integer value.").into()),
                }
            }
            None => Ok(None),
        }
    }

    /// Get all parameters with same name.
    /// * `name` - Parameter name.
    pub fn get_all<S: AsRef<str> + ?Sized>(&self, name: &S) -> Option<&Vec<String>> {
        match self.params.get(name.as_ref()) {
            Some(v) => {
                if v.len() > 0 {
                    Some(v)
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
