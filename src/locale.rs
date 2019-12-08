/*!
https://tc39.es/proposal-intl-locale
*/

use language_tags::{LanguageTag, Result};
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub struct Locale {
    tag: LanguageTag,
}

#[cfg(windows)]
mod platform {
    use std::ffi::OsString;
    use std::os::windows::ffi::OsStringExt;
    use winapi::{ctypes::c_int, um::winnls};

    const MAX_LOCALE_NAME_LEN: usize = 85usize;

    #[inline]
    fn from_wide_string(vec: &[u16]) -> Result<String, OsString> {
        let s = OsString::from_wide(&vec).into_string()?;

        Ok(s.split('\0').next().unwrap().to_owned())
    }

    pub fn locale_name() -> Result<String, std::io::Error> {
        let mut buf = vec![0u16; MAX_LOCALE_NAME_LEN];

        let ret = unsafe {
            winnls::GetUserDefaultLocaleName(buf.as_mut_ptr(), MAX_LOCALE_NAME_LEN as c_int)
        };

        if ret == 0 {
            let err = std::io::Error::last_os_error();
            return Err(err);
        }

        buf.truncate(ret as usize - 1);

        if buf.len() == 0 {
            return Ok(String::new());
        }

        Ok(from_wide_string(&buf).unwrap())
    }
}

#[cfg(unix)]
mod platform {
    pub fn locale_name() -> Result<String, std::io::Error> {
        let posix_tagish =
            std::env::var("LANG").map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        let without_dangly_bits = posix_tagish.split(".").next().unwrap();
        Ok(without_dangly_bits.replace("_", "-"))
    }
}

thread_local! {
    pub static CURRENT_LOCALE: Rc<RefCell<Locale>> = Rc::new(RefCell::new(Locale::default()));
}

impl Default for Locale {
    fn default() -> Locale {
        if let Ok(v) = platform::locale_name() {
            if let Ok(tag) = v.parse() {
                return Locale { tag };
            }
        }

        Locale {
            tag: "und".parse().unwrap(),
        }
    }
}

impl Locale {
    pub fn current() -> Locale {
        CURRENT_LOCALE.with(|locale| locale.borrow().clone())
    }

    pub fn autoupdating_current() -> Rc<RefCell<Locale>> {
        CURRENT_LOCALE.with(|locale| Rc::clone(&locale))
    }

    pub fn set_current(new_locale: Locale) {
        CURRENT_LOCALE.with(|locale| *locale.borrow_mut() = new_locale);
    }

    pub fn new<S: AsRef<str>>(tag: S) -> Result<Locale> {
        let tag: LanguageTag = tag.as_ref().parse()?;
        Ok(Locale { tag })
    }

    pub fn base_name(&self) -> Option<String> {
        let mut out = match self.tag.language.as_ref() {
            Some(v) => v.to_string(),
            None => return None,
        };

        if let Some(v) = self.script() {
            out.push_str("-");
            out.push_str(v);
        }

        if let Some(v) = self.region() {
            out.push_str("-");
            out.push_str(v);
        }

        for variant in &self.tag.variants {
            out.push_str("-");
            out.push_str(variant);
        }

        Some(out)
    }

    pub fn calendar(&self) -> String {
        unimplemented!()
    }

    pub fn collation(&self) -> String {
        unimplemented!()
    }

    pub fn hour_cycle(&self) -> String {
        unimplemented!()
    }

    pub fn case_first(&self) -> String {
        unimplemented!()
    }

    pub fn numeric(&self) -> String {
        unimplemented!()
    }

    pub fn numbering_system(&self) -> String {
        unimplemented!()
    }

    pub fn language(&self) -> Option<&String> {
        self.tag.language.as_ref()
    }

    pub fn script(&self) -> Option<&String> {
        self.tag.script.as_ref()
    }

    pub fn region(&self) -> Option<&String> {
        self.tag.region.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn current_locale() {
        let locale = Locale::autoupdating_current();
        println!("{:?}", locale);

        Locale::set_current(Locale::new("de-u-co-phonebk-ka-shifted").unwrap());
        println!("{:?}", locale);

        Locale::set_current(Locale::new("und-Latn-t-und-cyrl").unwrap());
        println!("{:?}", locale);

        Locale::set_current(Locale::new("de-Latn-u-co-phonebk-ka-shifted-t-und-cyrl").unwrap());
        println!("{:?}", locale);
    }
}
