use std::convert::{TryFrom, TryInto};
use std::ffi::CStr;
use std::time::SystemTime;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Ord)]
pub enum EntryType {
    Empty,
    BootTime,
    OldTime,
    NewTime,
    UserProcess,
    InitProcess,
    LoginProcess,
    DeadProcess,
}

#[derive(Debug)]
pub struct TryFromNonUtmpxError;

impl Into<i16> for EntryType {
    fn into(self) -> i16 {
        use EntryType::*;
        let val = match self {
            Empty => crochet_utmp_sys::EMPTY,
            BootTime => crochet_utmp_sys::BOOT_TIME,
            OldTime => crochet_utmp_sys::OLD_TIME,
            NewTime => crochet_utmp_sys::NEW_TIME,
            UserProcess => crochet_utmp_sys::USER_PROCESS,
            InitProcess => crochet_utmp_sys::INIT_PROCESS,
            LoginProcess => crochet_utmp_sys::LOGIN_PROCESS,
            DeadProcess => crochet_utmp_sys::DEAD_PROCESS,
        };
        val.try_into().unwrap()
    }
}

impl TryFrom<i16> for EntryType {
    type Error = TryFromNonUtmpxError;
    fn try_from(item: i16) -> Result<Self, Self::Error> {
        let item = item as u32;
        use EntryType::*;
        match item {
            crochet_utmp_sys::EMPTY => Ok(Empty),
            crochet_utmp_sys::BOOT_TIME => Ok(BootTime),
            crochet_utmp_sys::OLD_TIME => Ok(OldTime),
            crochet_utmp_sys::NEW_TIME => Ok(NewTime),
            crochet_utmp_sys::USER_PROCESS => Ok(UserProcess),
            crochet_utmp_sys::INIT_PROCESS => Ok(InitProcess),
            crochet_utmp_sys::LOGIN_PROCESS => Ok(LoginProcess),
            crochet_utmp_sys::DEAD_PROCESS => Ok(DeadProcess),
            _ => Err(TryFromNonUtmpxError),
        }
    }
}

impl PartialOrd for EntryType {
    fn partial_cmp(&self, other: &EntryType) -> Option<std::cmp::Ordering> {
        let lhs: i16 = (*self).into();
        let rhs: i16 = (*other).into();
        lhs.partial_cmp(&rhs)
    }
}

pub struct NewEntry {
    id: String,
    kind: EntryType,
    user: String,
    line: String,
    time: SystemTime,
}

impl NewEntry {
    fn into_raw(self) -> crochet_utmp_sys::utmpx {
        use std::mem::MaybeUninit;
        use std::ptr;

        let mut x: crochet_utmp_sys::utmpx = unsafe { MaybeUninit::zeroed().assume_init() };
        unsafe {
            ptr::copy_nonoverlapping(
                self.id.as_ptr() as *const _,
                x.ut_id.as_mut_ptr(),
                self.id.len().max(31),
            );
            ptr::copy_nonoverlapping(
                self.user.as_ptr() as *const _,
                x.ut_user.as_mut_ptr(),
                self.user.len().max(31),
            );
            ptr::copy_nonoverlapping(
                self.line.as_ptr() as *const _,
                x.ut_line.as_mut_ptr(),
                self.line.len().max(31),
            );
        }
        x.ut_type = self.kind.into();
        let duration = self.time.duration_since(SystemTime::UNIX_EPOCH).unwrap();
        x.ut_tv.tv_sec = duration.as_secs().try_into().unwrap();
        x.ut_tv.tv_usec = duration.subsec_micros().try_into().unwrap();
        x
    }
}

#[derive(Debug)]
pub struct Entry<'u> {
    kind: Option<EntryType>,
    user: &'u CStr,
    line: &'u CStr,
    time: SystemTime,
    raw: &'u crochet_utmp_sys::utmpx,
}

impl<'u> Entry<'u> {
    pub fn from_raw(u: &'u crochet_utmp_sys::utmpx) -> Self {
        Self {
            kind: u.ut_type.try_into().ok(),
            user: unsafe { CStr::from_ptr(u.ut_user.as_ptr()) },
            line: unsafe { CStr::from_ptr(u.ut_line.as_ptr()) },
            time: std::time::SystemTime::UNIX_EPOCH
                .checked_add(std::time::Duration::from_secs(
                    u.ut_tv.tv_sec.try_into().unwrap(),
                ))
                .unwrap()
                .checked_add(std::time::Duration::from_micros(
                    u.ut_tv.tv_usec.try_into().unwrap(),
                ))
                .unwrap(),
            raw: u,
        }
    }
}

pub struct Utmp {
    _priv: (),
}

impl Utmp {
    pub fn new() -> Self {
        unsafe {
            crochet_utmp_sys::setutxent();
        }
        Self { _priv: () }
    }

    pub fn next(&'_ self) -> Option<Entry<'_>> {
        let entry = unsafe { crochet_utmp_sys::getutxent().as_ref() };
        entry.map(|u| Entry::from_raw(u))
    }

    pub fn put(&self, entry: NewEntry) -> Option<Entry<'_>> {
        let raw = entry.into_raw();
        unsafe {
            crochet_utmp_sys::pututxline(&raw)
                .as_ref()
                .map(Entry::from_raw)
        }
    }
}

impl Drop for Utmp {
    fn drop(&mut self) {
        unsafe {
            crochet_utmp_sys::endutxent();
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    pub fn finds_an_entry() {
        let u = Utmp::new();
        let entry = u.next();
        assert!(entry.is_some())
    }

    #[test]
    pub fn all_parse() {
        let u = Utmp::new();
        let mut entry = u.next();
        while entry.is_some() {
            eprintln!("{:?}", entry);
            entry = u.next();
        }
    }

    #[test]
    pub fn pollute() {
        let u = Utmp::new();
        let entry = NewEntry {
            id: "test".to_string(),
            user: "test".to_string(),
            line: "test".to_string(),
            kind: EntryType::BootTime,
            time: SystemTime::now(),
        };
        u.put(entry);
    }
}
