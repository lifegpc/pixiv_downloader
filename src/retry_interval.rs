use crate::dur::Dur;
use crate::dur::DurType;
use crate::ext::json::ToJson;
use crate::gettext;
use crate::list::NonTailList;
use json::JsonValue;
use std::str::FromStr;
use std::time::Duration;

pub fn parse_retry_interval_from_str(s: &str) -> Result<NonTailList<Duration>, &'static str> {
    let l: Vec<Dur> = NonTailList::<Dur>::from_str(s)?.into();
    let mut r = Vec::new();
    for i in l {
        r.push(i.to_duration(DurType::Second));
    }
    Ok(NonTailList::from(r))
}

pub fn parse_retry_interval_from_json<T: ToJson>(
    s: T,
) -> Result<NonTailList<Duration>, &'static str> {
    let obj = s.to_json();
    if obj.is_none() {
        return Err(gettext("Failed to get JSON object."));
    }
    let obj = obj.unwrap();
    let mut r = NonTailList::<Duration>::default();
    if obj.is_number() {
        match obj.as_u64() {
            Some(num) => {
                r += Duration::new(num, 0);
            }
            None => match obj.as_f64() {
                Some(num) => {
                    r += Duration::from_secs_f64(num);
                }
                None => {
                    return Err(gettext("Failed to parse JSON number."));
                }
            },
        }
    } else if obj.is_array() {
        for v in obj.members() {
            if v.is_number() {
                match v.as_u64() {
                    Some(num) => {
                        r += Duration::new(num, 0);
                    }
                    None => match v.as_f64() {
                        Some(num) => {
                            r += Duration::from_secs_f64(num);
                        }
                        None => {
                            return Err(gettext("Failed to parse JSON number."));
                        }
                    },
                }
            } else {
                return Err(gettext("Unsupported JSON type."));
            }
        }
    } else {
        return Err(gettext("Unsupported JSON type."));
    }
    Ok(r)
}

pub fn check_retry_interval(s: &JsonValue) -> bool {
    parse_retry_interval_from_json(s).is_ok()
}

#[test]
fn test_parse_retry_interval() {
    let l = parse_retry_interval_from_str("2, 3, 4").unwrap();
    assert_eq!(
        l,
        vec![
            Duration::new(2, 0),
            Duration::new(3, 0),
            Duration::new(4, 0)
        ]
    );
    let l = parse_retry_interval_from_json(json::parse("123").unwrap()).unwrap();
    assert_eq!(l, vec![Duration::new(123, 0)]);
    let l = parse_retry_interval_from_json(json::parse("123.7").unwrap()).unwrap();
    assert_eq!(l, vec![Duration::new(123, 700_000_000)]);
    let l = parse_retry_interval_from_json(json::array![123, 123.7, 230]).unwrap();
    assert_eq!(
        l,
        vec![
            Duration::new(123, 0),
            Duration::new(123, 700_000_000),
            Duration::new(230, 0)
        ]
    );
}
