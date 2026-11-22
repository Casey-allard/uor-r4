#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Route {
    Serve = 0,
    Local = 1,
    NoGpu = 2,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Section {
    Header = 0,
    Tokenizer = 1,
    Spine = 2,
    Expert = 3,
    Table = 4,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum PageAction {
    Bind = 0,
    Fetch = 1,
    Drop = 2,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Manifest {
    pub spec: alloc::string::String,
    pub repo: alloc::string::String,
    pub revision: alloc::string::String,
    pub experts: u64,
    pub tableRows: u64,
    pub shards: alloc::vec::Vec<crate::Shard>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ref {
    pub branch: alloc::string::String,
    pub kappa: alloc::string::String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Start {
    Cold = 0,
    Warm = 1,
    Resume = 2,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Source {
    Device = 0,
    Peer = 1,
    Mirror = 2,
    Nowhere = 3,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Admission {
    Touch = 0,
    Insert = 1,
    EvictThenInsert = 2,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Decision {
    Accept = 0,
    Refuse = 1,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Message {
    pub role: alloc::string::String,
    pub content: alloc::string::String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Wallpaper {
    pub file: alloc::string::String,
    pub label: alloc::string::String,
    pub author: alloc::string::String,
    pub authorUrl: alloc::string::String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Capabilities {
    pub endpoints: alloc::vec::Vec<crate::Grant>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Request {
    pub model: alloc::string::String,
    pub messages: alloc::vec::Vec<crate::Message>,
    pub maxTokens: Option<u64>,
    pub seed: Option<u64>,
    pub temperature: alloc::string::String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Completion {
    pub id: alloc::string::String,
    pub created: alloc::string::String,
    pub model: alloc::string::String,
    pub text: alloc::string::String,
    pub fingerprint: alloc::string::String,
    pub receipt: alloc::string::String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Priority {
    First = 0,
    Fill = 1,
    Skip = 2,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Project {
    pub label: alloc::string::String,
    pub entries: alloc::vec::Vec<crate::Entry>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Obj {
    pub kind: alloc::string::String,
    pub label: alloc::string::String,
    pub kappa: alloc::string::String,
    pub bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Memo {
    pub model: alloc::vec::Vec<alloc::string::String>,
    pub engineKappa: alloc::string::String,
    pub promptKappa: alloc::string::String,
    pub paramsKappa: alloc::string::String,
    pub outputKappa: alloc::string::String,
    pub receipt: alloc::string::String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct View {
    pub headline: alloc::string::String,
    pub lede: alloc::string::String,
    pub promptPlaceholder: alloc::string::String,
    pub sendLabel: alloc::string::String,
    pub buildingLabel: alloc::string::String,
    pub previewLabel: alloc::string::String,
    pub snapshotLabel: alloc::string::String,
    pub rollbackLabel: alloc::string::String,
    pub refusedLabel: alloc::string::String,
    pub offlineLabel: alloc::string::String,
    pub publishLabel: alloc::string::String,
    pub publishingLabel: alloc::string::String,
    pub publishedLabel: alloc::string::String,
    pub publishRefusedLabel: alloc::string::String,
    pub holoLabel: alloc::string::String,
    pub downloadLabel: alloc::string::String,
    pub openLabel: alloc::string::String,
    pub runsLabel: alloc::string::String,
    pub holoTitle: alloc::string::String,
    pub holoLede: alloc::string::String,
    pub pickLabel: alloc::string::String,
    pub verifyingLabel: alloc::string::String,
    pub loadingLabel: alloc::string::String,
    pub servedLabel: alloc::string::String,
    pub sealedLabel: alloc::string::String,
    pub rederiveLabel: alloc::string::String,
    pub identicalLabel: alloc::string::String,
    pub noGpuLabel: alloc::string::String,
    pub modelLabel: alloc::string::String,
    pub appearanceLabel: alloc::string::String,
    pub darkLabel: alloc::string::String,
    pub lightLabel: alloc::string::String,
    pub immersiveLabel: alloc::string::String,
    pub wallpapers: alloc::vec::Vec<crate::Wallpaper>,
    pub localLabel: alloc::string::String,
    pub ladderLabel: alloc::string::String,
    pub seedModelLabel: alloc::string::String,
    pub largerLoadingLabel: alloc::string::String,
    pub promotedLabel: alloc::string::String,
    pub connectLabel: alloc::string::String,
    pub connectedLabel: alloc::string::String,
    pub listeningLabel: alloc::string::String,
    pub notConnectedLabel: alloc::string::String,
    pub runLabel: alloc::string::String,
    pub verifyLabel: alloc::string::String,
    pub baseUrlLabel: alloc::string::String,
    pub anyKeyLabel: alloc::string::String,
    pub modelIdLabel: alloc::string::String,
    pub testLabel: alloc::string::String,
    pub stayOpenLabel: alloc::string::String,
    pub askLabel: alloc::string::String,
    pub secondTabLabel: alloc::string::String,
    pub copyLabel: alloc::string::String,
    pub copiedLabel: alloc::string::String,
    pub macLabel: alloc::string::String,
    pub windowsLabel: alloc::string::String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    pub path: alloc::string::String,
    pub kappa: alloc::string::String,
    pub bytes: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Tier {
    Small = 0,
    Large = 1,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Staging {
    TrueRouting = 0,
    StagedReplace = 1,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Shard {
    pub label: alloc::string::String,
    pub bytes: u64,
    pub sha256: alloc::string::String,
    pub kappa: alloc::string::String,
    pub objects: alloc::string::String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Range {
    pub start: u64,
    pub stop: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Grant {
    pub endpoint: alloc::string::String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Preimages {
    pub prompt: alloc::vec::Vec<u8>,
    pub params: alloc::vec::Vec<u8>,
}

pub fn view() -> crate::View {
    { let _x_37 = crate::Wallpaper { file: alloc::string::String::from("alps.jpg"), label: alloc::string::String::from("Alpine Dawn"), author: alloc::string::String::from("Unsplash"), authorUrl: alloc::string::String::from("https://unsplash.com/?utm_source=Hologram_AI&utm_medium=referral") }; { let _x_42 = crate::Wallpaper { file: alloc::string::String::from("galaxy.jpg"), label: alloc::string::String::from("Galaxy"), author: alloc::string::String::from("Tiago Ferreira"), authorUrl: alloc::string::String::from("https://unsplash.com/@tiago_f_ferreira?utm_source=Hologram_AI&utm_medium=referral") }; { let _x_47 = crate::Wallpaper { file: alloc::string::String::from("aurora.jpg"), label: alloc::string::String::from("Aurora"), author: alloc::string::String::from("Lightscape"), authorUrl: alloc::string::String::from("https://unsplash.com/@lightscape?utm_source=Hologram_AI&utm_medium=referral") }; { let _x_49 = alloc::vec![_x_47]; { let _x_50 = { let mut __list = alloc::vec![_x_42]; __list.extend(_x_49); __list }; { let _x_51 = { let mut __list = alloc::vec![_x_37]; __list.extend(_x_50); __list }; { let _x_74 = crate::View { headline: alloc::string::String::from("Own Your Ideas"), lede: alloc::string::String::from("Describe it. It builds, runs and stays yours, in this browser, on no server."), promptPlaceholder: alloc::string::String::from("What do you want to build?"), sendLabel: alloc::string::String::from("Build"), buildingLabel: alloc::string::String::from("building in your browser"), previewLabel: alloc::string::String::from("Preview"), snapshotLabel: alloc::string::String::from("Sealed"), rollbackLabel: alloc::string::String::from("Go back"), refusedLabel: alloc::string::String::from("Refused: not what it claims to be"), offlineLabel: alloc::string::String::from("offline, working from your device"), publishLabel: alloc::string::String::from("Publish as a Hologram application"), publishingLabel: alloc::string::String::from("cooking your application through PrismPM"), publishedLabel: alloc::string::String::from("Published"), publishRefusedLabel: alloc::string::String::from("Refused: the evidence does not match this shell"), holoLabel: alloc::string::String::from("Your .holo application"), downloadLabel: alloc::string::String::from("Download"), openLabel: alloc::string::String::from("Open"), runsLabel: alloc::string::String::from("Runs on Hologram Desktop, Hologram Server and in any browser, with no server"), holoTitle: alloc::string::String::from("Run a Hologram application"), holoLede: alloc::string::String::from("The file is checked by PrismPM's own validator before it runs. Nothing leaves this browser."), pickLabel: alloc::string::String::from("Choose a .holo file"), verifyingLabel: alloc::string::String::from("verifying"), loadingLabel: alloc::string::String::from("getting the model, once"), servedLabel: alloc::string::String::from("Instant, from the seal"), sealedLabel: alloc::string::String::from("Sealed"), rederiveLabel: alloc::string::String::from("Check again"), identicalLabel: alloc::string::String::from("Checked, identical"), noGpuLabel: alloc::string::String::from("This browser cannot run the model. Try Chrome or Edge on a computer."), modelLabel: alloc::string::String::from("Qwen2.5 1.5B, on your device"), appearanceLabel: alloc::string::String::from("Appearance"), darkLabel: alloc::string::String::from("Dark"), lightLabel: alloc::string::String::from("Light"), immersiveLabel: alloc::string::String::from("Immersive"), wallpapers: _x_51, localLabel: alloc::string::String::from("On your device"), ladderLabel: alloc::string::String::from("The best model your device has ready"), seedModelLabel: alloc::string::String::from("SmolLM2 360M, on your device"), largerLoadingLabel: alloc::string::String::from("getting Qwen2.5 1.5B behind it"), promotedLabel: alloc::string::String::from("Now answering with Qwen2.5 1.5B"), connectLabel: alloc::string::String::from("Connect"), connectedLabel: alloc::string::String::from("Connected"), listeningLabel: alloc::string::String::from("Listening for the relay"), notConnectedLabel: alloc::string::String::from("Not connected"), runLabel: alloc::string::String::from("Run this once, on this computer"), verifyLabel: alloc::string::String::from("Verify the file"), baseUrlLabel: alloc::string::String::from("Base URL"), anyKeyLabel: alloc::string::String::from("Any key works, for example local"), modelIdLabel: alloc::string::String::from("Model"), testLabel: alloc::string::String::from("Send a test request"), stayOpenLabel: alloc::string::String::from("Nothing leaves your device. Keep this tab open."), askLabel: alloc::string::String::from("Your browser may ask to let this page reach your computer."), secondTabLabel: alloc::string::String::from("Another tab is already serving"), copyLabel: alloc::string::String::from("Copy"), copiedLabel: alloc::string::String::from("Copied"), macLabel: alloc::string::String::from("macOS / Linux"), windowsLabel: alloc::string::String::from("Windows") }; _x_74 } } } } } } }
}

pub fn anyEqual(x_1: &[alloc::string::String], x_2: &str) -> bool {
    match x_1 {
        [] => { let _x_30 = false; _x_30 },
        [head_19, tail_20 @ ..] => { let _x_39 = (head_19 == x_2); match _x_39 {
        false => { let _x_52 = anyEqual(&(tail_20), (x_2).as_ref()); _x_52 },
        true => _x_39,
    } },
    }
}

pub fn contentOf(message: &crate::Message) -> alloc::string::String {
    { let _x_8 = &(message).content; { let _x_20 = 2147483647; { let _x_13 = { let __value = _x_8; let __delimiter = alloc::string::String::from("\n"); let __maximum = usize::try_from(_x_20).ok(); if __delimiter.is_empty() { None } else { let __fields: alloc::vec::Vec<alloc::string::String> = __value.split(&__delimiter).map(alloc::string::String::from).collect(); __maximum.filter(|__maximum| __fields.len() <= *__maximum).map(|_| __fields) } }; match _x_13 {
        None => alloc::string::String::from(""),
        Some(val_16) => { let _x_25 = (val_16).join(&alloc::string::String::from("\n")); _x_25 },
    } } } }
}

pub fn createdOf(value: &crate::Completion) -> alloc::string::String {
    { let _x_8 = &(value).created; { let _x_20 = 2147483647; { let _x_13 = { let __value = _x_8; let __delimiter = alloc::string::String::from("\n"); let __maximum = usize::try_from(_x_20).ok(); if __delimiter.is_empty() { None } else { let __fields: alloc::vec::Vec<alloc::string::String> = __value.split(&__delimiter).map(alloc::string::String::from).collect(); __maximum.filter(|__maximum| __fields.len() <= *__maximum).map(|_| __fields) } }; match _x_13 {
        None => alloc::string::String::from(""),
        Some(val_16) => { let _x_25 = (val_16).join(&alloc::string::String::from("\n")); _x_25 },
    } } } }
}

pub fn done() -> alloc::string::String {
    alloc::string::String::from("[DONE]")
}

pub fn encodeCompletion(completion: &crate::Completion) -> alloc::string::String {
    { let _x_3 = idOf(&(completion)); { let _x_4 = escapeJson(_x_3); { let _x_6 = alloc::vec![alloc::string::String::from("\"")]; { let _x_7 = { let mut __list = alloc::vec![_x_4]; __list.extend(_x_6.clone()); __list }; { let _x_8 = { let mut __list = alloc::vec![alloc::string::String::from("\"")]; __list.extend(_x_7); __list }; { let _x_10 = (_x_8).join(&alloc::string::String::from("")); { let _x_12 = createdOf(&(completion)); { let _x_14 = modelOf(&(completion)); { let _x_15 = escapeJson(_x_14); { let _x_16 = { let mut __list = alloc::vec![_x_15]; __list.extend(_x_6.clone()); __list }; { let _x_17 = { let mut __list = alloc::vec![alloc::string::String::from("\"")]; __list.extend(_x_16); __list }; { let _x_18 = (_x_17).join(&alloc::string::String::from("")); { let _x_20 = fingerprintOf(&(completion)); { let _x_21 = escapeJson(_x_20); { let _x_22 = { let mut __list = alloc::vec![_x_21]; __list.extend(_x_6.clone()); __list }; { let _x_23 = { let mut __list = alloc::vec![alloc::string::String::from("\"")]; __list.extend(_x_22); __list }; { let _x_24 = (_x_23).join(&alloc::string::String::from("")); { let _x_26 = textOf(&(completion)); { let _x_27 = escapeJson(_x_26); { let _x_28 = { let mut __list = alloc::vec![_x_27]; __list.extend(_x_6.clone()); __list }; { let _x_29 = { let mut __list = alloc::vec![alloc::string::String::from("\"")]; __list.extend(_x_28); __list }; { let _x_30 = (_x_29).join(&alloc::string::String::from("")); { let _x_32 = alloc::vec![alloc::string::String::from(",\"refusal\":null},\"logprobs\":null,\"finish_reason\":\"stop\"}],\"usage\":null}")]; { let _x_33 = { let mut __list = alloc::vec![_x_30]; __list.extend(_x_32); __list }; { let _x_34 = { let mut __list = alloc::vec![alloc::string::String::from(",\"choices\":[{\"index\":0,\"message\":{\"role\":\"assistant\",\"content\":")]; __list.extend(_x_33); __list }; { let _x_35 = { let mut __list = alloc::vec![_x_24]; __list.extend(_x_34); __list }; { let _x_36 = { let mut __list = alloc::vec![alloc::string::String::from(",\"system_fingerprint\":")]; __list.extend(_x_35); __list }; { let _x_37 = { let mut __list = alloc::vec![_x_18]; __list.extend(_x_36); __list }; { let _x_38 = { let mut __list = alloc::vec![alloc::string::String::from(",\"model\":")]; __list.extend(_x_37); __list }; { let _x_39 = { let mut __list = alloc::vec![_x_12]; __list.extend(_x_38); __list }; { let _x_40 = { let mut __list = alloc::vec![alloc::string::String::from(",\"object\":\"chat.completion\",\"created\":")]; __list.extend(_x_39); __list }; { let _x_41 = { let mut __list = alloc::vec![_x_10]; __list.extend(_x_40); __list }; { let _x_42 = { let mut __list = alloc::vec![alloc::string::String::from("{\"id\":")]; __list.extend(_x_41); __list }; { let _x_43 = (_x_42).join(&alloc::string::String::from("")); _x_43 } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } }
}

pub fn encodeDelta(completion: &crate::Completion, delta: alloc::string::String) -> alloc::string::String {
    { let _x_3 = idOf(&(completion)); { let _x_4 = escapeJson(_x_3); { let _x_6 = alloc::vec![alloc::string::String::from("\"")]; { let _x_7 = { let mut __list = alloc::vec![_x_4]; __list.extend(_x_6.clone()); __list }; { let _x_8 = { let mut __list = alloc::vec![alloc::string::String::from("\"")]; __list.extend(_x_7); __list }; { let _x_10 = (_x_8).join(&alloc::string::String::from("")); { let _x_12 = createdOf(&(completion)); { let _x_14 = modelOf(&(completion)); { let _x_15 = escapeJson(_x_14); { let _x_16 = { let mut __list = alloc::vec![_x_15]; __list.extend(_x_6.clone()); __list }; { let _x_17 = { let mut __list = alloc::vec![alloc::string::String::from("\"")]; __list.extend(_x_16); __list }; { let _x_18 = (_x_17).join(&alloc::string::String::from("")); { let _x_20 = fingerprintOf(&(completion)); { let _x_21 = escapeJson(_x_20); { let _x_22 = { let mut __list = alloc::vec![_x_21]; __list.extend(_x_6.clone()); __list }; { let _x_23 = { let mut __list = alloc::vec![alloc::string::String::from("\"")]; __list.extend(_x_22); __list }; { let _x_24 = (_x_23).join(&alloc::string::String::from("")); { let _x_26 = escapeJson(delta); { let _x_27 = { let mut __list = alloc::vec![_x_26]; __list.extend(_x_6.clone()); __list }; { let _x_28 = { let mut __list = alloc::vec![alloc::string::String::from("\"")]; __list.extend(_x_27); __list }; { let _x_29 = (_x_28).join(&alloc::string::String::from("")); { let _x_31 = alloc::vec![alloc::string::String::from("},\"finish_reason\":null}]}")]; { let _x_32 = { let mut __list = alloc::vec![_x_29]; __list.extend(_x_31); __list }; { let _x_33 = { let mut __list = alloc::vec![alloc::string::String::from(",\"choices\":[{\"index\":0,\"delta\":{\"content\":")]; __list.extend(_x_32); __list }; { let _x_34 = { let mut __list = alloc::vec![_x_24]; __list.extend(_x_33); __list }; { let _x_35 = { let mut __list = alloc::vec![alloc::string::String::from(",\"system_fingerprint\":")]; __list.extend(_x_34); __list }; { let _x_36 = { let mut __list = alloc::vec![_x_18]; __list.extend(_x_35); __list }; { let _x_37 = { let mut __list = alloc::vec![alloc::string::String::from(",\"model\":")]; __list.extend(_x_36); __list }; { let _x_38 = { let mut __list = alloc::vec![_x_12]; __list.extend(_x_37); __list }; { let _x_39 = { let mut __list = alloc::vec![alloc::string::String::from(",\"object\":\"chat.completion.chunk\",\"created\":")]; __list.extend(_x_38); __list }; { let _x_40 = { let mut __list = alloc::vec![_x_10]; __list.extend(_x_39); __list }; { let _x_41 = { let mut __list = alloc::vec![alloc::string::String::from("{\"id\":")]; __list.extend(_x_40); __list }; { let _x_42 = (_x_41).join(&alloc::string::String::from("")); _x_42 } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } }
}

pub fn encodeError(message: alloc::string::String, kind: alloc::string::String) -> alloc::string::String {
    { let _x_3 = escapeJson(message); { let _x_5 = alloc::vec![alloc::string::String::from("\"")]; { let _x_6 = { let mut __list = alloc::vec![_x_3]; __list.extend(_x_5.clone()); __list }; { let _x_7 = { let mut __list = alloc::vec![alloc::string::String::from("\"")]; __list.extend(_x_6); __list }; { let _x_9 = (_x_7).join(&alloc::string::String::from("")); { let _x_11 = escapeJson(kind); { let _x_12 = { let mut __list = alloc::vec![_x_11]; __list.extend(_x_5.clone()); __list }; { let _x_13 = { let mut __list = alloc::vec![alloc::string::String::from("\"")]; __list.extend(_x_12); __list }; { let _x_14 = (_x_13).join(&alloc::string::String::from("")); { let _x_16 = alloc::vec![alloc::string::String::from(",\"param\":null,\"code\":null}}")]; { let _x_17 = { let mut __list = alloc::vec![_x_14]; __list.extend(_x_16); __list }; { let _x_18 = { let mut __list = alloc::vec![alloc::string::String::from(",\"type\":")]; __list.extend(_x_17); __list }; { let _x_19 = { let mut __list = alloc::vec![_x_9]; __list.extend(_x_18); __list }; { let _x_20 = { let mut __list = alloc::vec![alloc::string::String::from("{\"error\":{\"message\":")]; __list.extend(_x_19); __list }; { let _x_21 = (_x_20).join(&alloc::string::String::from("")); _x_21 } } } } } } } } } } } } } } }
}

pub fn encodeFinal(completion: &crate::Completion) -> alloc::string::String {
    { let _x_3 = idOf(&(completion)); { let _x_4 = escapeJson(_x_3); { let _x_6 = alloc::vec![alloc::string::String::from("\"")]; { let _x_7 = { let mut __list = alloc::vec![_x_4]; __list.extend(_x_6.clone()); __list }; { let _x_8 = { let mut __list = alloc::vec![alloc::string::String::from("\"")]; __list.extend(_x_7); __list }; { let _x_10 = (_x_8).join(&alloc::string::String::from("")); { let _x_12 = createdOf(&(completion)); { let _x_14 = modelOf(&(completion)); { let _x_15 = escapeJson(_x_14); { let _x_16 = { let mut __list = alloc::vec![_x_15]; __list.extend(_x_6.clone()); __list }; { let _x_17 = { let mut __list = alloc::vec![alloc::string::String::from("\"")]; __list.extend(_x_16); __list }; { let _x_18 = (_x_17).join(&alloc::string::String::from("")); { let _x_20 = fingerprintOf(&(completion)); { let _x_21 = escapeJson(_x_20); { let _x_22 = { let mut __list = alloc::vec![_x_21]; __list.extend(_x_6.clone()); __list }; { let _x_23 = { let mut __list = alloc::vec![alloc::string::String::from("\"")]; __list.extend(_x_22); __list }; { let _x_24 = (_x_23).join(&alloc::string::String::from("")); { let _x_26 = receiptOf(&(completion)); { let _x_27 = escapeJson(_x_26); { let _x_28 = { let mut __list = alloc::vec![_x_27]; __list.extend(_x_6.clone()); __list }; { let _x_29 = { let mut __list = alloc::vec![alloc::string::String::from("\"")]; __list.extend(_x_28); __list }; { let _x_30 = (_x_29).join(&alloc::string::String::from("")); { let _x_32 = alloc::vec![alloc::string::String::from("}}")]; { let _x_33 = { let mut __list = alloc::vec![_x_30]; __list.extend(_x_32); __list }; { let _x_34 = { let mut __list = alloc::vec![alloc::string::String::from(",\"choices\":[{\"index\":0,\"delta\":{},\"finish_reason\":\"stop\"}],\"hologram\":{\"receipt\":")]; __list.extend(_x_33); __list }; { let _x_35 = { let mut __list = alloc::vec![_x_24]; __list.extend(_x_34); __list }; { let _x_36 = { let mut __list = alloc::vec![alloc::string::String::from(",\"system_fingerprint\":")]; __list.extend(_x_35); __list }; { let _x_37 = { let mut __list = alloc::vec![_x_18]; __list.extend(_x_36); __list }; { let _x_38 = { let mut __list = alloc::vec![alloc::string::String::from(",\"model\":")]; __list.extend(_x_37); __list }; { let _x_39 = { let mut __list = alloc::vec![_x_12]; __list.extend(_x_38); __list }; { let _x_40 = { let mut __list = alloc::vec![alloc::string::String::from(",\"object\":\"chat.completion.chunk\",\"created\":")]; __list.extend(_x_39); __list }; { let _x_41 = { let mut __list = alloc::vec![_x_10]; __list.extend(_x_40); __list }; { let _x_42 = { let mut __list = alloc::vec![alloc::string::String::from("{\"id\":")]; __list.extend(_x_41); __list }; { let _x_43 = (_x_42).join(&alloc::string::String::from("")); _x_43 } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } }
}

pub fn encodeModels(ids: &[alloc::string::String]) -> alloc::string::String {
    { let _x_2 = modelEntries(&(ids)); { let _x_5 = alloc::vec![alloc::string::String::from("]}")]; { let _x_6 = { let mut __list = alloc::vec![_x_2]; __list.extend(_x_5); __list }; { let _x_7 = { let mut __list = alloc::vec![alloc::string::String::from("{\"object\":\"list\",\"data\":[")]; __list.extend(_x_6); __list }; { let _x_9 = (_x_7).join(&alloc::string::String::from("")); _x_9 } } } } }
}

pub fn encodeOpenRouterRequest(model: alloc::string::String, request: &crate::Request, stream: bool) -> alloc::string::String {
    { let _x_3 = escapeJson(model); { let _x_5 = alloc::vec![alloc::string::String::from("\"")]; { let _x_6 = { let mut __list = alloc::vec![_x_3]; __list.extend(_x_5); __list }; { let _x_7 = { let mut __list = alloc::vec![alloc::string::String::from("\"")]; __list.extend(_x_6); __list }; { let _x_9 = (_x_7).join(&alloc::string::String::from("")); { let _x_11 = &(request).messages; { let _x_12 = orMessages(&(_x_11)); { let _x_14 = maxTokensField(&(request)); { let _x_15 = seedField(&(request)); { let _x_16 = temperatureField(&(request)); { let _x_18 = streamText(stream); { let _x_20 = alloc::vec![alloc::string::String::from(",\"usage\":{\"include\":true},\"reasoning\":{\"enabled\":false},\"provider\":{\"require_parameters\":true}}")]; { let _x_21 = { let mut __list = alloc::vec![_x_18]; __list.extend(_x_20); __list }; { let _x_22 = { let mut __list = alloc::vec![alloc::string::String::from(",\"stream\":")]; __list.extend(_x_21); __list }; { let _x_23 = { let mut __list = alloc::vec![_x_16]; __list.extend(_x_22); __list }; { let _x_24 = { let mut __list = alloc::vec![_x_15]; __list.extend(_x_23); __list }; { let _x_25 = { let mut __list = alloc::vec![_x_14]; __list.extend(_x_24); __list }; { let _x_26 = { let mut __list = alloc::vec![alloc::string::String::from("]")]; __list.extend(_x_25); __list }; { let _x_27 = { let mut __list = alloc::vec![_x_12]; __list.extend(_x_26); __list }; { let _x_28 = { let mut __list = alloc::vec![alloc::string::String::from(",\"messages\":[")]; __list.extend(_x_27); __list }; { let _x_29 = { let mut __list = alloc::vec![_x_9]; __list.extend(_x_28); __list }; { let _x_30 = { let mut __list = alloc::vec![alloc::string::String::from("{\"model\":")]; __list.extend(_x_29); __list }; { let _x_31 = (_x_30).join(&alloc::string::String::from("")); _x_31 } } } } } } } } } } } } } } } } } } } } } } }
}

pub fn encodeRole(completion: &crate::Completion) -> alloc::string::String {
    { let _x_3 = idOf(&(completion)); { let _x_4 = escapeJson(_x_3); { let _x_6 = alloc::vec![alloc::string::String::from("\"")]; { let _x_7 = { let mut __list = alloc::vec![_x_4]; __list.extend(_x_6.clone()); __list }; { let _x_8 = { let mut __list = alloc::vec![alloc::string::String::from("\"")]; __list.extend(_x_7); __list }; { let _x_10 = (_x_8).join(&alloc::string::String::from("")); { let _x_12 = createdOf(&(completion)); { let _x_14 = modelOf(&(completion)); { let _x_15 = escapeJson(_x_14); { let _x_16 = { let mut __list = alloc::vec![_x_15]; __list.extend(_x_6.clone()); __list }; { let _x_17 = { let mut __list = alloc::vec![alloc::string::String::from("\"")]; __list.extend(_x_16); __list }; { let _x_18 = (_x_17).join(&alloc::string::String::from("")); { let _x_20 = fingerprintOf(&(completion)); { let _x_21 = escapeJson(_x_20); { let _x_22 = { let mut __list = alloc::vec![_x_21]; __list.extend(_x_6.clone()); __list }; { let _x_23 = { let mut __list = alloc::vec![alloc::string::String::from("\"")]; __list.extend(_x_22); __list }; { let _x_24 = (_x_23).join(&alloc::string::String::from("")); { let _x_26 = alloc::vec![alloc::string::String::from(",\"choices\":[{\"index\":0,\"delta\":{\"role\":\"assistant\",\"content\":\"\"},\"finish_reason\":null}]}")]; { let _x_27 = { let mut __list = alloc::vec![_x_24]; __list.extend(_x_26); __list }; { let _x_28 = { let mut __list = alloc::vec![alloc::string::String::from(",\"system_fingerprint\":")]; __list.extend(_x_27); __list }; { let _x_29 = { let mut __list = alloc::vec![_x_18]; __list.extend(_x_28); __list }; { let _x_30 = { let mut __list = alloc::vec![alloc::string::String::from(",\"model\":")]; __list.extend(_x_29); __list }; { let _x_31 = { let mut __list = alloc::vec![_x_12]; __list.extend(_x_30); __list }; { let _x_32 = { let mut __list = alloc::vec![alloc::string::String::from(",\"object\":\"chat.completion.chunk\",\"created\":")]; __list.extend(_x_31); __list }; { let _x_33 = { let mut __list = alloc::vec![_x_10]; __list.extend(_x_32); __list }; { let _x_34 = { let mut __list = alloc::vec![alloc::string::String::from("{\"id\":")]; __list.extend(_x_33); __list }; { let _x_35 = (_x_34).join(&alloc::string::String::from("")); _x_35 } } } } } } } } } } } } } } } } } } } } } } } } } } }
}

pub fn endpointReady(resident: bool, keyPresent: bool, online: bool) -> bool {
    match resident {
        false => match keyPresent {
        false => keyPresent,
        true => online,
    },
        true => resident,
    }
}

pub fn fingerprintOf(value: &crate::Completion) -> alloc::string::String {
    { let _x_8 = &(value).fingerprint; { let _x_20 = 2147483647; { let _x_13 = { let __value = _x_8; let __delimiter = alloc::string::String::from("\n"); let __maximum = usize::try_from(_x_20).ok(); if __delimiter.is_empty() { None } else { let __fields: alloc::vec::Vec<alloc::string::String> = __value.split(&__delimiter).map(alloc::string::String::from).collect(); __maximum.filter(|__maximum| __fields.len() <= *__maximum).map(|_| __fields) } }; match _x_13 {
        None => alloc::string::String::from(""),
        Some(val_16) => { let _x_25 = (val_16).join(&alloc::string::String::from("\n")); _x_25 },
    } } } }
}

pub fn idOf(value: &crate::Completion) -> alloc::string::String {
    { let _x_8 = &(value).id; { let _x_20 = 2147483647; { let _x_13 = { let __value = _x_8; let __delimiter = alloc::string::String::from("\n"); let __maximum = usize::try_from(_x_20).ok(); if __delimiter.is_empty() { None } else { let __fields: alloc::vec::Vec<alloc::string::String> = __value.split(&__delimiter).map(alloc::string::String::from).collect(); __maximum.filter(|__maximum| __fields.len() <= *__maximum).map(|_| __fields) } }; match _x_13 {
        None => alloc::string::String::from(""),
        Some(val_16) => { let _x_25 = (val_16).join(&alloc::string::String::from("\n")); _x_25 },
    } } } }
}

pub fn intersects(x_1: &[alloc::string::String], x_2: &[alloc::string::String]) -> bool {
    match x_1 {
        [] => { let _x_28 = false; _x_28 },
        [head_17, tail_18 @ ..] => { let _x_35 = anyEqual(&(x_2), (head_17).as_ref()); match _x_35 {
        false => { let _x_48 = intersects(&(tail_18), &(x_2)); _x_48 },
        true => _x_35,
    } },
    }
}

pub fn maxTokensField(request: &crate::Request) -> alloc::string::String {
    { let _x_16 = (request).maxTokens; match _x_16 {
        None => alloc::string::String::from(""),
        Some(val_19) => { let _x_29 = alloc::format!("{}", val_19); { let _x_31 = alloc::vec![_x_29]; { let _x_32 = { let mut __list = alloc::vec![alloc::string::String::from(",\"max_tokens\":")]; __list.extend(_x_31); __list }; { let _x_34 = (_x_32).join(&alloc::string::String::from("")); _x_34 } } } },
    } }
}

pub fn memoMatches(memo: &crate::Memo, candidates: &[alloc::string::String], promptKappa: &str, paramsKappa: &str) -> bool {
    { let _x_3 = &(memo).promptKappa; { let _x_4 = (_x_3 == promptKappa); match _x_4 {
        false => _x_4,
        true => { let _x_64 = &(memo).paramsKappa; { let _x_65 = (_x_64 == paramsKappa); match _x_65 {
        false => _x_65,
        true => { let _x_69 = &(memo).model; { let _x_70 = intersects(&(_x_69), &(candidates)); _x_70 } },
    } } },
    } } }
}

pub fn modelEntries(x_1: &[alloc::string::String]) -> alloc::string::String {
    match x_1 {
        [] => alloc::string::String::from(""),
        [head_73, tail_74 @ ..] => match tail_74 {
        [] => { let _x_142 = 2147483647; { let _x_143 = { let __value = head_73; let __delimiter = alloc::string::String::from("\n"); let __maximum = usize::try_from(_x_142).ok(); if __delimiter.is_empty() { None } else { let __fields: alloc::vec::Vec<alloc::string::String> = __value.split(&__delimiter).map(alloc::string::String::from).collect(); __maximum.filter(|__maximum| __fields.len() <= *__maximum).map(|_| __fields) } }; match _x_143 {
        None => { let _x_146 = modelEntry(alloc::string::String::from("")); _x_146 },
        Some(val_147) => { let _x_151 = (val_147).join(&alloc::string::String::from("\n")); { let _x_149 = modelEntry(_x_151); _x_149 } },
    } } },
        [head_134, tail_135 @ ..] => { let head_134 = head_134.clone(); { let _x_157 = 2147483647; { let _x_158 = { let __value = head_73; let __delimiter = alloc::string::String::from("\n"); let __maximum = usize::try_from(_x_157).ok(); if __delimiter.is_empty() { None } else { let __fields: alloc::vec::Vec<alloc::string::String> = __value.split(&__delimiter).map(alloc::string::String::from).collect(); __maximum.filter(|__maximum| __fields.len() <= *__maximum).map(|_| __fields) } }; { let _jp_159 = /* jp "_jp_159" inlined at its jump site */ (); match _x_158 {
        None => { let _y_160 = alloc::string::String::from(""); { let _x_161 = modelEntry(_y_160); { let _x_162 = modelEntries(&(tail_74)); { let _x_164 = alloc::vec![_x_162]; { let _x_165 = { let mut __list = alloc::vec![_x_161]; __list.extend(_x_164); __list }; { let _x_167 = (_x_165).join(&alloc::string::String::from(",")); _x_167 } } } } } },
        Some(val_170) => { let _x_173 = (val_170).join(&alloc::string::String::from("\n")); { let _y_160 = _x_173; { let _x_161 = modelEntry(_y_160); { let _x_162 = modelEntries(&(tail_74)); { let _x_164 = alloc::vec![_x_162]; { let _x_165 = { let mut __list = alloc::vec![_x_161]; __list.extend(_x_164); __list }; { let _x_167 = (_x_165).join(&alloc::string::String::from(",")); _x_167 } } } } } } },
    } } } } },
    },
    }
}

pub fn modelEntry(id: alloc::string::String) -> alloc::string::String {
    { let _x_3 = escapeJson(id); { let _x_5 = alloc::vec![alloc::string::String::from("\"")]; { let _x_6 = { let mut __list = alloc::vec![_x_3]; __list.extend(_x_5); __list }; { let _x_7 = { let mut __list = alloc::vec![alloc::string::String::from("\"")]; __list.extend(_x_6); __list }; { let _x_9 = (_x_7).join(&alloc::string::String::from("")); { let _x_11 = alloc::vec![alloc::string::String::from(",\"object\":\"model\",\"created\":0,\"owned_by\":\"browser\"}")]; { let _x_12 = { let mut __list = alloc::vec![_x_9]; __list.extend(_x_11); __list }; { let _x_13 = { let mut __list = alloc::vec![alloc::string::String::from("{\"id\":")]; __list.extend(_x_12); __list }; { let _x_14 = (_x_13).join(&alloc::string::String::from("")); _x_14 } } } } } } } } }
}

pub fn modelOf(value: &crate::Completion) -> alloc::string::String {
    { let _x_8 = &(value).model; { let _x_20 = 2147483647; { let _x_13 = { let __value = _x_8; let __delimiter = alloc::string::String::from("\n"); let __maximum = usize::try_from(_x_20).ok(); if __delimiter.is_empty() { None } else { let __fields: alloc::vec::Vec<alloc::string::String> = __value.split(&__delimiter).map(alloc::string::String::from).collect(); __maximum.filter(|__maximum| __fields.len() <= *__maximum).map(|_| __fields) } }; match _x_13 {
        None => alloc::string::String::from(""),
        Some(val_16) => { let _x_25 = (val_16).join(&alloc::string::String::from("\n")); _x_25 },
    } } } }
}

pub fn optionalDecimal(value: Option<u64>) -> alloc::string::String {
    match value {
        None => alloc::string::String::from("null"),
        Some(val_12) => { let _x_21 = alloc::format!("{}", val_12); _x_21 },
    }
}

pub fn orMessage(message: &crate::Message) -> alloc::string::String {
    { let _x_3 = roleOf(&(message)); { let _x_4 = escapeJson(_x_3); { let _x_6 = alloc::vec![alloc::string::String::from("\"")]; { let _x_7 = { let mut __list = alloc::vec![_x_4]; __list.extend(_x_6.clone()); __list }; { let _x_8 = { let mut __list = alloc::vec![alloc::string::String::from("\"")]; __list.extend(_x_7); __list }; { let _x_10 = (_x_8).join(&alloc::string::String::from("")); { let _x_12 = contentOf(&(message)); { let _x_13 = escapeJson(_x_12); { let _x_14 = { let mut __list = alloc::vec![_x_13]; __list.extend(_x_6.clone()); __list }; { let _x_15 = { let mut __list = alloc::vec![alloc::string::String::from("\"")]; __list.extend(_x_14); __list }; { let _x_16 = (_x_15).join(&alloc::string::String::from("")); { let _x_18 = alloc::vec![alloc::string::String::from("}")]; { let _x_19 = { let mut __list = alloc::vec![_x_16]; __list.extend(_x_18); __list }; { let _x_20 = { let mut __list = alloc::vec![alloc::string::String::from(",\"content\":")]; __list.extend(_x_19); __list }; { let _x_21 = { let mut __list = alloc::vec![_x_10]; __list.extend(_x_20); __list }; { let _x_22 = { let mut __list = alloc::vec![alloc::string::String::from("{\"role\":")]; __list.extend(_x_21); __list }; { let _x_23 = (_x_22).join(&alloc::string::String::from("")); _x_23 } } } } } } } } } } } } } } } } }
}

pub fn orMessages(x_1: &[crate::Message]) -> alloc::string::String {
    match x_1 {
        [] => alloc::string::String::from(""),
        [head_29, tail_30 @ ..] => match tail_30 {
        [] => { let _x_49 = orMessage(&(head_29)); _x_49 },
        [head_50, tail_51 @ ..] => { let head_50 = head_50.clone(); { let _x_53 = orMessage(&(head_29)); { let _x_54 = orMessages(&(tail_30)); { let _x_56 = alloc::vec![_x_54]; { let _x_57 = { let mut __list = alloc::vec![_x_53]; __list.extend(_x_56); __list }; { let _x_59 = (_x_57).join(&alloc::string::String::from(",")); _x_59 } } } } } },
    },
    }
}

pub fn paramsCanonical(request: &crate::Request) -> alloc::string::String {
    { let _x_2 = (request).maxTokens; { let _x_3 = optionalDecimal(_x_2); { let _x_5 = (request).seed; { let _x_6 = optionalDecimal(_x_5); { let _x_8 = temperatureText(&(request)); { let _x_11 = alloc::vec![alloc::string::String::from("}")]; { let _x_12 = { let mut __list = alloc::vec![_x_8]; __list.extend(_x_11); __list }; { let _x_13 = { let mut __list = alloc::vec![alloc::string::String::from(",\"temperature\":")]; __list.extend(_x_12); __list }; { let _x_14 = { let mut __list = alloc::vec![_x_6]; __list.extend(_x_13); __list }; { let _x_15 = { let mut __list = alloc::vec![alloc::string::String::from(",\"seed\":")]; __list.extend(_x_14); __list }; { let _x_16 = { let mut __list = alloc::vec![_x_3]; __list.extend(_x_15); __list }; { let _x_17 = { let mut __list = alloc::vec![alloc::string::String::from("{\"max_tokens\":")]; __list.extend(_x_16); __list }; { let _x_19 = (_x_17).join(&alloc::string::String::from("")); _x_19 } } } } } } } } } } } } }
}

pub fn preimages(request: &crate::Request) -> crate::Preimages {
    { let _x_1 = &(request).messages; { let _x_2 = renderPrompt(&(_x_1)); { let _x_3 = (_x_2).into_bytes(); { let _x_4 = paramsCanonical(&(request)); { let _x_5 = (_x_4).into_bytes(); { let _x_6 = crate::Preimages { prompt: _x_3, params: _x_5 }; _x_6 } } } } } }
}

pub fn receiptOf(value: &crate::Completion) -> alloc::string::String {
    { let _x_8 = &(value).receipt; { let _x_20 = 2147483647; { let _x_13 = { let __value = _x_8; let __delimiter = alloc::string::String::from("\n"); let __maximum = usize::try_from(_x_20).ok(); if __delimiter.is_empty() { None } else { let __fields: alloc::vec::Vec<alloc::string::String> = __value.split(&__delimiter).map(alloc::string::String::from).collect(); __maximum.filter(|__maximum| __fields.len() <= *__maximum).map(|_| __fields) } }; match _x_13 {
        None => alloc::string::String::from(""),
        Some(val_16) => { let _x_25 = (val_16).join(&alloc::string::String::from("\n")); _x_25 },
    } } } }
}

pub fn renderMessage(message: &crate::Message) -> alloc::string::String {
    { let _x_1 = roleOf(&(message)); { let _x_3 = contentOf(&(message)); { let _x_5 = alloc::vec![_x_3]; { let _x_6 = { let mut __list = alloc::vec![alloc::string::String::from(": ")]; __list.extend(_x_5); __list }; { let _x_7 = { let mut __list = alloc::vec![_x_1]; __list.extend(_x_6); __list }; { let _x_9 = (_x_7).join(&alloc::string::String::from("")); _x_9 } } } } } }
}

pub fn renderPrompt(x_1: &[crate::Message]) -> alloc::string::String {
    match x_1 {
        [] => alloc::string::String::from(""),
        [head_29, tail_30 @ ..] => match tail_30 {
        [] => { let _x_49 = renderMessage(&(head_29)); _x_49 },
        [head_50, tail_51 @ ..] => { let head_50 = head_50.clone(); { let _x_53 = renderMessage(&(head_29)); { let _x_54 = renderPrompt(&(tail_30)); { let _x_56 = alloc::vec![_x_54]; { let _x_57 = { let mut __list = alloc::vec![_x_53]; __list.extend(_x_56); __list }; { let _x_59 = (_x_57).join(&alloc::string::String::from("\n")); _x_59 } } } } } },
    },
    }
}

pub fn roleOf(message: &crate::Message) -> alloc::string::String {
    { let _x_8 = &(message).role; { let _x_20 = 2147483647; { let _x_13 = { let __value = _x_8; let __delimiter = alloc::string::String::from("\n"); let __maximum = usize::try_from(_x_20).ok(); if __delimiter.is_empty() { None } else { let __fields: alloc::vec::Vec<alloc::string::String> = __value.split(&__delimiter).map(alloc::string::String::from).collect(); __maximum.filter(|__maximum| __fields.len() <= *__maximum).map(|_| __fields) } }; match _x_13 {
        None => alloc::string::String::from(""),
        Some(val_16) => { let _x_25 = (val_16).join(&alloc::string::String::from("\n")); _x_25 },
    } } } }
}

pub fn route(hit: bool, gpuReady: bool) -> crate::Route {
    match hit {
        false => match gpuReady {
        false => { let _x_55 = crate::Route::NoGpu; _x_55 },
        true => { let _x_56 = crate::Route::Local; _x_56 },
    },
        true => { let _x_54 = crate::Route::Serve; _x_54 },
    }
}

pub fn seedField(request: &crate::Request) -> alloc::string::String {
    { let _x_16 = (request).seed; match _x_16 {
        None => alloc::string::String::from(""),
        Some(val_19) => { let _x_29 = alloc::format!("{}", val_19); { let _x_31 = alloc::vec![_x_29]; { let _x_32 = { let mut __list = alloc::vec![alloc::string::String::from(",\"seed\":")]; __list.extend(_x_31); __list }; { let _x_34 = (_x_32).join(&alloc::string::String::from("")); _x_34 } } } },
    } }
}

pub fn streamText(stream: bool) -> alloc::string::String {
    match stream {
        false => alloc::string::String::from("false"),
        true => alloc::string::String::from("true"),
    }
}

pub fn temperatureField(request: &crate::Request) -> alloc::string::String {
    { let _x_3 = &(request).temperature; { let _x_5 = _x_3 == ""; match _x_5 {
        false => { let _x_83 = 2147483647; { let _x_84 = { let __value = _x_3; let __delimiter = alloc::string::String::from("\n"); let __maximum = usize::try_from(_x_83).ok(); if __delimiter.is_empty() { None } else { let __fields: alloc::vec::Vec<alloc::string::String> = __value.split(&__delimiter).map(alloc::string::String::from).collect(); __maximum.filter(|__maximum| __fields.len() <= *__maximum).map(|_| __fields) } }; { let _jp_85 = /* jp "_jp_85" inlined at its jump site */ (); match _x_84 {
        None => { let _y_86 = alloc::string::String::from(""); { let _x_88 = alloc::vec![_y_86]; { let _x_89 = { let mut __list = alloc::vec![alloc::string::String::from(",\"temperature\":")]; __list.extend(_x_88); __list }; { let _x_90 = (_x_89).join(&alloc::string::String::from("")); _x_90 } } } },
        Some(val_92) => { let _x_94 = (val_92).join(&alloc::string::String::from("\n")); { let _y_86 = _x_94; { let _x_88 = alloc::vec![_y_86]; { let _x_89 = { let mut __list = alloc::vec![alloc::string::String::from(",\"temperature\":")]; __list.extend(_x_88); __list }; { let _x_90 = (_x_89).join(&alloc::string::String::from("")); _x_90 } } } } },
    } } } },
        true => alloc::string::String::from(""),
    } } }
}

pub fn temperatureText(request: &crate::Request) -> alloc::string::String {
    { let _x_3 = &(request).temperature; { let _x_5 = _x_3 == ""; match _x_5 {
        false => { let _x_61 = 2147483647; { let _x_62 = { let __value = _x_3; let __delimiter = alloc::string::String::from("\n"); let __maximum = usize::try_from(_x_61).ok(); if __delimiter.is_empty() { None } else { let __fields: alloc::vec::Vec<alloc::string::String> = __value.split(&__delimiter).map(alloc::string::String::from).collect(); __maximum.filter(|__maximum| __fields.len() <= *__maximum).map(|_| __fields) } }; match _x_62 {
        None => alloc::string::String::from(""),
        Some(val_64) => { let _x_66 = (val_64).join(&alloc::string::String::from("\n")); _x_66 },
    } } },
        true => alloc::string::String::from("null"),
    } } }
}

pub fn textOf(value: &crate::Completion) -> alloc::string::String {
    { let _x_8 = &(value).text; { let _x_20 = 2147483647; { let _x_13 = { let __value = _x_8; let __delimiter = alloc::string::String::from("\n"); let __maximum = usize::try_from(_x_20).ok(); if __delimiter.is_empty() { None } else { let __fields: alloc::vec::Vec<alloc::string::String> = __value.split(&__delimiter).map(alloc::string::String::from).collect(); __maximum.filter(|__maximum| __fields.len() <= *__maximum).map(|_| __fields) } }; match _x_13 {
        None => alloc::string::String::from(""),
        Some(val_16) => { let _x_25 = (val_16).join(&alloc::string::String::from("\n")); _x_25 },
    } } } }
}

pub fn warmup(localReady: bool, keyPresent: bool, online: bool) -> bool {
    match localReady {
        false => match keyPresent {
        false => keyPresent,
        true => online,
    },
        true => { let _x_35 = false; _x_35 },
    }
}

pub fn admitPage(listed: &[alloc::string::String], kappa: &str, derived: &str) -> bool {
    { let _x_40 = 2147483647; { let _x_20 = { let __value = kappa; let __delimiter = alloc::string::String::from("\n"); let __maximum = usize::try_from(_x_40).ok(); if __delimiter.is_empty() { None } else { let __fields: alloc::vec::Vec<alloc::string::String> = __value.split(&__delimiter).map(alloc::string::String::from).collect(); __maximum.filter(|__maximum| __fields.len() <= *__maximum).map(|_| __fields) } }; { let _jp_32 = /* jp "_jp_32" inlined at its jump site */ (); match _x_20 {
        None => { let _y_25 = alloc::string::String::from(""); { let _x_26 = anyEqual(&(listed), (_y_25).as_ref()); match _x_26 {
        false => _x_26,
        true => { let _x_51 = (kappa == derived); _x_51 },
    } } },
        Some(val_23) => { let _x_53 = (val_23).join(&alloc::string::String::from("\n")); { let _y_25 = _x_53; { let _x_26 = anyEqual(&(listed), (_y_25).as_ref()); match _x_26 {
        false => _x_26,
        true => { let _x_51 = (kappa == derived); _x_51 },
    } } } },
    } } } }
}

pub fn expertPage(start: u64, length: u64, experts: u64, expert: u64) -> Result<crate::Range, crate::ComputeError> {
    Ok({ let _x_5 = stride(length, experts); { let _x_6 = ((expert) as u64).checked_mul(_x_5).ok_or(crate::ComputeError::MulOverflow)?; { let _x_21 = ((start) as u64).checked_add(_x_6).ok_or(crate::ComputeError::AddOverflow)?; { let _x_8 = 1; { let _x_26 = ((expert) as u64).checked_add(_x_8).ok_or(crate::ComputeError::AddOverflow)?; { let _x_12 = ((_x_26) as u64).checked_mul(_x_5).ok_or(crate::ComputeError::MulOverflow)?; { let _x_30 = ((start) as u64).checked_add(_x_12).ok_or(crate::ComputeError::AddOverflow)?; { let _x_14 = crate::Range { start: _x_21, stop: _x_30 }; _x_14 } } } } } } } })
}

pub fn fetchSource(onDevice: bool, onMirror: bool, peerFaster: bool) -> crate::Source {
    match onDevice {
        false => match peerFaster {
        false => match onMirror {
        false => { let _x_100 = crate::Source::Nowhere; _x_100 },
        true => { let _x_101 = crate::Source::Mirror; _x_101 },
    },
        true => { let _x_99 = crate::Source::Peer; _x_99 },
    },
        true => { let _x_95 = crate::Source::Device; _x_95 },
    }
}

pub fn firstTokenReady(spinePresent: bool, promptPagesPresent: bool) -> bool {
    match spinePresent {
        false => spinePresent,
        true => promptPagesPresent,
    }
}

pub fn loaderStart(shellOnDevice: bool, snapshotOnDevice: bool) -> crate::Start {
    match snapshotOnDevice {
        false => match shellOnDevice {
        false => { let _x_55 = crate::Start::Cold; _x_55 },
        true => { let _x_56 = crate::Start::Warm; _x_56 },
    },
        true => { let _x_54 = crate::Start::Resume; _x_54 },
    }
}

pub fn manifestRepo(manifest: &crate::Manifest) -> alloc::string::String {
    { let _x_8 = &(manifest).repo; { let _x_20 = 2147483647; { let _x_13 = { let __value = _x_8; let __delimiter = alloc::string::String::from("\n"); let __maximum = usize::try_from(_x_20).ok(); if __delimiter.is_empty() { None } else { let __fields: alloc::vec::Vec<alloc::string::String> = __value.split(&__delimiter).map(alloc::string::String::from).collect(); __maximum.filter(|__maximum| __fields.len() <= *__maximum).map(|_| __fields) } }; match _x_13 {
        None => alloc::string::String::from(""),
        Some(val_16) => { let _x_25 = (val_16).join(&alloc::string::String::from("\n")); _x_25 },
    } } } }
}

pub fn manifestRevision(manifest: &crate::Manifest) -> alloc::string::String {
    { let _x_8 = &(manifest).revision; { let _x_20 = 2147483647; { let _x_13 = { let __value = _x_8; let __delimiter = alloc::string::String::from("\n"); let __maximum = usize::try_from(_x_20).ok(); if __delimiter.is_empty() { None } else { let __fields: alloc::vec::Vec<alloc::string::String> = __value.split(&__delimiter).map(alloc::string::String::from).collect(); __maximum.filter(|__maximum| __fields.len() <= *__maximum).map(|_| __fields) } }; match _x_13 {
        None => alloc::string::String::from(""),
        Some(val_16) => { let _x_25 = (val_16).join(&alloc::string::String::from("\n")); _x_25 },
    } } } }
}

pub fn manifestSpec(manifest: &crate::Manifest) -> alloc::string::String {
    { let _x_8 = &(manifest).spec; { let _x_20 = 2147483647; { let _x_13 = { let __value = _x_8; let __delimiter = alloc::string::String::from("\n"); let __maximum = usize::try_from(_x_20).ok(); if __delimiter.is_empty() { None } else { let __fields: alloc::vec::Vec<alloc::string::String> = __value.split(&__delimiter).map(alloc::string::String::from).collect(); __maximum.filter(|__maximum| __fields.len() <= *__maximum).map(|_| __fields) } }; match _x_13 {
        None => alloc::string::String::from(""),
        Some(val_16) => { let _x_25 = (val_16).join(&alloc::string::String::from("\n")); _x_25 },
    } } } }
}

pub fn objEntry(obj: &crate::Obj) -> alloc::string::String {
    { let _x_2 = objKind(&(obj)); { let _x_3 = escapeJson(_x_2); { let _x_5 = objLabel(&(obj)); { let _x_6 = escapeJson(_x_5); { let _x_7 = objKappa(&(obj)); { let _x_8 = escapeJson(_x_7); { let _x_13 = (obj).bytes; { let _x_14 = alloc::format!("{}", _x_13); { let _x_17 = alloc::vec![alloc::string::String::from("]")]; { let _x_18 = { let mut __list = alloc::vec![_x_14]; __list.extend(_x_17); __list }; { let _x_19 = { let mut __list = alloc::vec![alloc::string::String::from("\",")]; __list.extend(_x_18); __list }; { let _x_20 = { let mut __list = alloc::vec![_x_8]; __list.extend(_x_19); __list }; { let _x_21 = { let mut __list = alloc::vec![alloc::string::String::from("\",\"")]; __list.extend(_x_20); __list }; { let _x_22 = { let mut __list = alloc::vec![_x_6]; __list.extend(_x_21); __list }; { let _x_23 = { let mut __list = alloc::vec![alloc::string::String::from("\",\"")]; __list.extend(_x_22); __list }; { let _x_24 = { let mut __list = alloc::vec![_x_3]; __list.extend(_x_23); __list }; { let _x_25 = { let mut __list = alloc::vec![alloc::string::String::from("[\"")]; __list.extend(_x_24); __list }; { let _x_27 = (_x_25).join(&alloc::string::String::from("")); _x_27 } } } } } } } } } } } } } } } } } }
}

pub fn objKappa(obj: &crate::Obj) -> alloc::string::String {
    { let _x_8 = &(obj).kappa; { let _x_20 = 2147483647; { let _x_13 = { let __value = _x_8; let __delimiter = alloc::string::String::from("\n"); let __maximum = usize::try_from(_x_20).ok(); if __delimiter.is_empty() { None } else { let __fields: alloc::vec::Vec<alloc::string::String> = __value.split(&__delimiter).map(alloc::string::String::from).collect(); __maximum.filter(|__maximum| __fields.len() <= *__maximum).map(|_| __fields) } }; match _x_13 {
        None => alloc::string::String::from(""),
        Some(val_16) => { let _x_25 = (val_16).join(&alloc::string::String::from("\n")); _x_25 },
    } } } }
}

pub fn objKind(obj: &crate::Obj) -> alloc::string::String {
    { let _x_8 = &(obj).kind; { let _x_20 = 2147483647; { let _x_13 = { let __value = _x_8; let __delimiter = alloc::string::String::from("\n"); let __maximum = usize::try_from(_x_20).ok(); if __delimiter.is_empty() { None } else { let __fields: alloc::vec::Vec<alloc::string::String> = __value.split(&__delimiter).map(alloc::string::String::from).collect(); __maximum.filter(|__maximum| __fields.len() <= *__maximum).map(|_| __fields) } }; match _x_13 {
        None => alloc::string::String::from(""),
        Some(val_16) => { let _x_25 = (val_16).join(&alloc::string::String::from("\n")); _x_25 },
    } } } }
}

pub fn objLabel(obj: &crate::Obj) -> alloc::string::String {
    { let _x_8 = &(obj).label; { let _x_20 = 2147483647; { let _x_13 = { let __value = _x_8; let __delimiter = alloc::string::String::from("\n"); let __maximum = usize::try_from(_x_20).ok(); if __delimiter.is_empty() { None } else { let __fields: alloc::vec::Vec<alloc::string::String> = __value.split(&__delimiter).map(alloc::string::String::from).collect(); __maximum.filter(|__maximum| __fields.len() <= *__maximum).map(|_| __fields) } }; match _x_13 {
        None => alloc::string::String::from(""),
        Some(val_16) => { let _x_25 = (val_16).join(&alloc::string::String::from("\n")); _x_25 },
    } } } }
}

pub fn packRank(part: crate::Section) -> u64 {
    match part {
        crate::Section::Header => { let _x_44 = 0; _x_44 },
        crate::Section::Tokenizer => { let _x_46 = 1; _x_46 },
        crate::Section::Spine => { let _x_48 = 2; _x_48 },
        crate::Section::Expert => { let _x_50 = 3; _x_50 },
        crate::Section::Table => { let _x_52 = 4; _x_52 },
    }
}

pub fn packed(part: crate::Section) -> bool {
    match part {
        crate::Section::Table => { let _x_31 = false; _x_31 },
        _ => { let _x_23 = true; _x_23 },
    }
}

pub fn pageAction(resident: bool, staging: crate::Staging) -> crate::PageAction {
    match resident {
        false => match staging {
        crate::Staging::TrueRouting => { let _x_40 = crate::PageAction::Fetch; _x_40 },
        crate::Staging::StagedReplace => { let _x_41 = crate::PageAction::Drop; _x_41 },
    },
        true => { let _x_39 = crate::PageAction::Bind; _x_39 },
    }
}

pub fn pageBytes(rowBytes: u64, rows: u64) -> Result<u64, crate::ComputeError> {
    Ok({ let _x_2 = ((rowBytes) as u64).checked_mul(rows).ok_or(crate::ComputeError::MulOverflow)?; _x_2 })
}

pub fn pageStart(start: u64, rowBytes: u64, rows: u64, index: u64) -> Result<u64, crate::ComputeError> {
    Ok({ let _x_13 = ((rowBytes) as u64).checked_mul(rows).ok_or(crate::ComputeError::MulOverflow)?; { let _x_6 = ((index) as u64).checked_mul(_x_13).ok_or(crate::ComputeError::MulOverflow)?; { let _x_15 = ((start) as u64).checked_add(_x_6).ok_or(crate::ComputeError::AddOverflow)?; _x_15 } } })
}

pub fn poolAdmit(present: bool, spaceLeft: bool) -> crate::Admission {
    match present {
        false => match spaceLeft {
        false => { let _x_55 = crate::Admission::EvictThenInsert; _x_55 },
        true => { let _x_56 = crate::Admission::Insert; _x_56 },
    },
        true => { let _x_54 = crate::Admission::Touch; _x_54 },
    }
}

pub fn prefetchOrder(predicted: bool, popular: bool) -> crate::Priority {
    match predicted {
        false => match popular {
        false => { let _x_55 = crate::Priority::Skip; _x_55 },
        true => { let _x_56 = crate::Priority::Fill; _x_56 },
    },
        true => { let _x_54 = crate::Priority::First; _x_54 },
    }
}

pub fn promote(current: crate::Tier, largeResident: bool, largeFast: bool) -> crate::Tier {
    match current {
        crate::Tier::Small => match largeResident {
        false => current,
        true => match largeFast {
        false => current,
        true => { let _x_73 = crate::Tier::Large; _x_73 },
    },
    },
        crate::Tier::Large => current,
    }
}

pub fn rootPreimage(manifest: &crate::Manifest) -> alloc::string::String {
    { let _x_5 = (manifest).experts; { let _x_6 = alloc::format!("{}", _x_5); { let _x_9 = manifestRepo(&(manifest)); { let _x_10 = escapeJson(_x_9); { let _x_12 = alloc::vec![alloc::string::String::from("\"")]; { let _x_13 = { let mut __list = alloc::vec![_x_10]; __list.extend(_x_12.clone()); __list }; { let _x_14 = { let mut __list = alloc::vec![alloc::string::String::from("\"")]; __list.extend(_x_13); __list }; { let _x_16 = (_x_14).join(&alloc::string::String::from("")); { let _x_18 = manifestRevision(&(manifest)); { let _x_19 = escapeJson(_x_18); { let _x_20 = { let mut __list = alloc::vec![_x_19]; __list.extend(_x_12.clone()); __list }; { let _x_21 = { let mut __list = alloc::vec![alloc::string::String::from("\"")]; __list.extend(_x_20); __list }; { let _x_22 = (_x_21).join(&alloc::string::String::from("")); { let _x_24 = &(manifest).shards; { let _x_25 = shardEntries(&(_x_24)); { let _x_28 = manifestSpec(&(manifest)); { let _x_29 = escapeJson(_x_28); { let _x_30 = { let mut __list = alloc::vec![_x_29]; __list.extend(_x_12.clone()); __list }; { let _x_31 = { let mut __list = alloc::vec![alloc::string::String::from("\"")]; __list.extend(_x_30); __list }; { let _x_32 = (_x_31).join(&alloc::string::String::from("")); { let _x_34 = (manifest).tableRows; { let _x_35 = alloc::format!("{}", _x_34); { let _x_37 = alloc::vec![alloc::string::String::from("}")]; { let _x_38 = { let mut __list = alloc::vec![_x_35]; __list.extend(_x_37); __list }; { let _x_39 = { let mut __list = alloc::vec![alloc::string::String::from(",\"table_rows\":")]; __list.extend(_x_38); __list }; { let _x_40 = { let mut __list = alloc::vec![_x_32]; __list.extend(_x_39); __list }; { let _x_41 = { let mut __list = alloc::vec![alloc::string::String::from(",\"spec\":")]; __list.extend(_x_40); __list }; { let _x_42 = { let mut __list = alloc::vec![alloc::string::String::from("]")]; __list.extend(_x_41); __list }; { let _x_43 = { let mut __list = alloc::vec![_x_25]; __list.extend(_x_42); __list }; { let _x_44 = { let mut __list = alloc::vec![alloc::string::String::from(",\"shards\":[")]; __list.extend(_x_43); __list }; { let _x_45 = { let mut __list = alloc::vec![_x_22]; __list.extend(_x_44); __list }; { let _x_46 = { let mut __list = alloc::vec![alloc::string::String::from(",\"revision\":")]; __list.extend(_x_45); __list }; { let _x_47 = { let mut __list = alloc::vec![_x_16]; __list.extend(_x_46); __list }; { let _x_48 = { let mut __list = alloc::vec![alloc::string::String::from(",\"repo\":")]; __list.extend(_x_47); __list }; { let _x_49 = { let mut __list = alloc::vec![_x_6]; __list.extend(_x_48); __list }; { let _x_50 = { let mut __list = alloc::vec![alloc::string::String::from("{\"experts\":")]; __list.extend(_x_49); __list }; { let _x_51 = (_x_50).join(&alloc::string::String::from("")); _x_51 } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } }
}

pub fn sha256Text(shard: &crate::Shard) -> alloc::string::String {
    { let _x_3 = &(shard).sha256; { let _x_5 = _x_3 == ""; match _x_5 {
        false => { let _x_43 = shardSha256(&(shard)); { let _x_44 = escapeJson(_x_43); { let _x_46 = alloc::vec![alloc::string::String::from("\"")]; { let _x_47 = { let mut __list = alloc::vec![_x_44]; __list.extend(_x_46); __list }; { let _x_48 = { let mut __list = alloc::vec![alloc::string::String::from("\"")]; __list.extend(_x_47); __list }; { let _x_49 = (_x_48).join(&alloc::string::String::from("")); _x_49 } } } } } },
        true => alloc::string::String::from("null"),
    } } }
}

pub fn shardEntries(shards: &[crate::Shard]) -> alloc::string::String {
    { let _x_2 = shardEntriesFrom(&(shards), alloc::string::String::from("")); _x_2 }
}

pub fn shardEntriesFrom(x_1: &[crate::Shard], x_2: alloc::string::String) -> alloc::string::String {
    match x_1 {
        [] => x_2.clone(),
        [head_26, tail_27 @ ..] => { let _x_55 = x_2.clone() == ""; match _x_55 {
        false => { let _x_73 = shardEntry(&(head_26)); { let _x_75 = alloc::vec![_x_73]; { let _x_76 = { let mut __list = alloc::vec![x_2.clone()]; __list.extend(_x_75); __list }; { let _x_78 = (_x_76).join(&alloc::string::String::from(",")); { let _x_79 = shardEntriesFrom(&(tail_27), _x_78); _x_79 } } } } },
        true => { let _x_80 = shardEntry(&(head_26)); { let _x_81 = shardEntriesFrom(&(tail_27), _x_80); _x_81 } },
    } },
    }
}

pub fn shardEntry(shard: &crate::Shard) -> alloc::string::String {
    { let _x_5 = (shard).bytes; { let _x_6 = alloc::format!("{}", _x_5); { let _x_9 = shardKappa(&(shard)); { let _x_10 = escapeJson(_x_9); { let _x_12 = alloc::vec![alloc::string::String::from("\"")]; { let _x_13 = { let mut __list = alloc::vec![_x_10]; __list.extend(_x_12.clone()); __list }; { let _x_14 = { let mut __list = alloc::vec![alloc::string::String::from("\"")]; __list.extend(_x_13); __list }; { let _x_16 = (_x_14).join(&alloc::string::String::from("")); { let _x_18 = shardLabel(&(shard)); { let _x_19 = escapeJson(_x_18); { let _x_20 = { let mut __list = alloc::vec![_x_19]; __list.extend(_x_12.clone()); __list }; { let _x_21 = { let mut __list = alloc::vec![alloc::string::String::from("\"")]; __list.extend(_x_20); __list }; { let _x_22 = (_x_21).join(&alloc::string::String::from("")); { let _x_24 = shardObjects(&(shard)); { let _x_25 = escapeJson(_x_24); { let _x_26 = { let mut __list = alloc::vec![_x_25]; __list.extend(_x_12.clone()); __list }; { let _x_27 = { let mut __list = alloc::vec![alloc::string::String::from("\"")]; __list.extend(_x_26); __list }; { let _x_28 = (_x_27).join(&alloc::string::String::from("")); { let _x_30 = sha256Text(&(shard)); { let _x_32 = alloc::vec![alloc::string::String::from("}")]; { let _x_33 = { let mut __list = alloc::vec![_x_30]; __list.extend(_x_32); __list }; { let _x_34 = { let mut __list = alloc::vec![alloc::string::String::from(",\"sha256\":")]; __list.extend(_x_33); __list }; { let _x_35 = { let mut __list = alloc::vec![_x_28]; __list.extend(_x_34); __list }; { let _x_36 = { let mut __list = alloc::vec![alloc::string::String::from(",\"objects\":")]; __list.extend(_x_35); __list }; { let _x_37 = { let mut __list = alloc::vec![_x_22]; __list.extend(_x_36); __list }; { let _x_38 = { let mut __list = alloc::vec![alloc::string::String::from(",\"name\":")]; __list.extend(_x_37); __list }; { let _x_39 = { let mut __list = alloc::vec![_x_16]; __list.extend(_x_38); __list }; { let _x_40 = { let mut __list = alloc::vec![alloc::string::String::from(",\"kappa\":")]; __list.extend(_x_39); __list }; { let _x_41 = { let mut __list = alloc::vec![_x_6]; __list.extend(_x_40); __list }; { let _x_42 = { let mut __list = alloc::vec![alloc::string::String::from("{\"bytes\":")]; __list.extend(_x_41); __list }; { let _x_43 = (_x_42).join(&alloc::string::String::from("")); _x_43 } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } }
}

pub fn shardKappa(shard: &crate::Shard) -> alloc::string::String {
    { let _x_8 = &(shard).kappa; { let _x_20 = 2147483647; { let _x_13 = { let __value = _x_8; let __delimiter = alloc::string::String::from("\n"); let __maximum = usize::try_from(_x_20).ok(); if __delimiter.is_empty() { None } else { let __fields: alloc::vec::Vec<alloc::string::String> = __value.split(&__delimiter).map(alloc::string::String::from).collect(); __maximum.filter(|__maximum| __fields.len() <= *__maximum).map(|_| __fields) } }; match _x_13 {
        None => alloc::string::String::from(""),
        Some(val_16) => { let _x_25 = (val_16).join(&alloc::string::String::from("\n")); _x_25 },
    } } } }
}

pub fn shardLabel(shard: &crate::Shard) -> alloc::string::String {
    { let _x_8 = &(shard).label; { let _x_20 = 2147483647; { let _x_13 = { let __value = _x_8; let __delimiter = alloc::string::String::from("\n"); let __maximum = usize::try_from(_x_20).ok(); if __delimiter.is_empty() { None } else { let __fields: alloc::vec::Vec<alloc::string::String> = __value.split(&__delimiter).map(alloc::string::String::from).collect(); __maximum.filter(|__maximum| __fields.len() <= *__maximum).map(|_| __fields) } }; match _x_13 {
        None => alloc::string::String::from(""),
        Some(val_16) => { let _x_25 = (val_16).join(&alloc::string::String::from("\n")); _x_25 },
    } } } }
}

pub fn shardObjects(shard: &crate::Shard) -> alloc::string::String {
    { let _x_8 = &(shard).objects; { let _x_20 = 2147483647; { let _x_13 = { let __value = _x_8; let __delimiter = alloc::string::String::from("\n"); let __maximum = usize::try_from(_x_20).ok(); if __delimiter.is_empty() { None } else { let __fields: alloc::vec::Vec<alloc::string::String> = __value.split(&__delimiter).map(alloc::string::String::from).collect(); __maximum.filter(|__maximum| __fields.len() <= *__maximum).map(|_| __fields) } }; match _x_13 {
        None => alloc::string::String::from(""),
        Some(val_16) => { let _x_25 = (val_16).join(&alloc::string::String::from("\n")); _x_25 },
    } } } }
}

pub fn shardSha256(shard: &crate::Shard) -> alloc::string::String {
    { let _x_8 = &(shard).sha256; { let _x_20 = 2147483647; { let _x_13 = { let __value = _x_8; let __delimiter = alloc::string::String::from("\n"); let __maximum = usize::try_from(_x_20).ok(); if __delimiter.is_empty() { None } else { let __fields: alloc::vec::Vec<alloc::string::String> = __value.split(&__delimiter).map(alloc::string::String::from).collect(); __maximum.filter(|__maximum| __fields.len() <= *__maximum).map(|_| __fields) } }; match _x_13 {
        None => alloc::string::String::from(""),
        Some(val_16) => { let _x_25 = (val_16).join(&alloc::string::String::from("\n")); _x_25 },
    } } } }
}

pub fn stride(length: u64, experts: u64) -> u64 {
    { let _x_2 = 0; { let _x_5 = if experts == 0 { _x_2 } else { length / experts }; _x_5 } }
}

pub fn tablePage(start: u64, stop: u64, rowBytes: u64, rows: u64, index: u64) -> Result<crate::Range, crate::ComputeError> {
    Ok({ let _x_1 = pageStart(start, rowBytes, rows, index)?; { let _x_20 = ((rowBytes) as u64).checked_mul(rows).ok_or(crate::ComputeError::MulOverflow)?; { let _x_22 = ((_x_1) as u64).checked_add(_x_20).ok_or(crate::ComputeError::AddOverflow)?; { let _x_7 = (_x_22 <= stop); match _x_7 {
        false => { let _x_41 = crate::Range { start: _x_1, stop: stop }; _x_41 },
        true => { let _x_42 = crate::Range { start: _x_1, stop: _x_22 }; _x_42 },
    } } } } })
}

pub fn holoPath(kappa: alloc::string::String) -> alloc::string::String {
    { let _x_4 = alloc::vec![alloc::string::String::from("/")]; { let _x_5 = { let mut __list = alloc::vec![kappa]; __list.extend(_x_4); __list }; { let _x_6 = { let mut __list = alloc::vec![alloc::string::String::from("/holo/")]; __list.extend(_x_5); __list }; { let _x_8 = (_x_6).join(&alloc::string::String::from("")); _x_8 } } } }
}

pub fn publishDecision(closure: alloc::string::String, built: alloc::string::String, attestation: Option<alloc::string::String>) -> crate::Decision {
    match attestation {
        None => { let _x_30 = crate::Decision::Refuse; _x_30 },
        Some(val_19) => { let _x_39 = (closure == built); match _x_39 {
        false => { let _x_52 = crate::Decision::Refuse; _x_52 },
        true => { let _x_53 = crate::Decision::Accept; _x_53 },
    } },
    }
}

pub fn publishPreimage(application: alloc::string::String, version: alloc::string::String, closure: alloc::string::String, attestation: alloc::string::String) -> alloc::string::String {
    { let _x_3 = escapeJson(application); { let _x_5 = alloc::vec![alloc::string::String::from("\"")]; { let _x_6 = { let mut __list = alloc::vec![_x_3]; __list.extend(_x_5.clone()); __list }; { let _x_7 = { let mut __list = alloc::vec![alloc::string::String::from("\"")]; __list.extend(_x_6); __list }; { let _x_9 = (_x_7).join(&alloc::string::String::from("")); { let _x_11 = escapeJson(attestation); { let _x_12 = { let mut __list = alloc::vec![_x_11]; __list.extend(_x_5.clone()); __list }; { let _x_13 = { let mut __list = alloc::vec![alloc::string::String::from("\"")]; __list.extend(_x_12); __list }; { let _x_14 = (_x_13).join(&alloc::string::String::from("")); { let _x_16 = escapeJson(closure); { let _x_17 = { let mut __list = alloc::vec![_x_16]; __list.extend(_x_5.clone()); __list }; { let _x_18 = { let mut __list = alloc::vec![alloc::string::String::from("\"")]; __list.extend(_x_17); __list }; { let _x_19 = (_x_18).join(&alloc::string::String::from("")); { let _x_21 = escapeJson(version); { let _x_22 = { let mut __list = alloc::vec![_x_21]; __list.extend(_x_5.clone()); __list }; { let _x_23 = { let mut __list = alloc::vec![alloc::string::String::from("\"")]; __list.extend(_x_22); __list }; { let _x_24 = (_x_23).join(&alloc::string::String::from("")); { let _x_26 = alloc::vec![alloc::string::String::from("}")]; { let _x_27 = { let mut __list = alloc::vec![_x_24]; __list.extend(_x_26); __list }; { let _x_28 = { let mut __list = alloc::vec![alloc::string::String::from(",\"schema\":\"hologram-forge/published/1\",\"version\":")]; __list.extend(_x_27); __list }; { let _x_29 = { let mut __list = alloc::vec![_x_19]; __list.extend(_x_28); __list }; { let _x_30 = { let mut __list = alloc::vec![alloc::string::String::from(",\"closure\":")]; __list.extend(_x_29); __list }; { let _x_31 = { let mut __list = alloc::vec![_x_14]; __list.extend(_x_30); __list }; { let _x_32 = { let mut __list = alloc::vec![alloc::string::String::from(",\"attestation\":")]; __list.extend(_x_31); __list }; { let _x_33 = { let mut __list = alloc::vec![_x_9]; __list.extend(_x_32); __list }; { let _x_34 = { let mut __list = alloc::vec![alloc::string::String::from("{\"application\":")]; __list.extend(_x_33); __list }; { let _x_35 = (_x_34).join(&alloc::string::String::from("")); _x_35 } } } } } } } } } } } } } } } } } } } } } } } } } } }
}

pub fn sourceManifest(leanManifest: alloc::string::String, coverage: alloc::string::String, kernel: alloc::string::String, modelId: alloc::string::String, semanticId: alloc::string::String, sourceId: alloc::string::String) -> alloc::string::String {
    { let _x_3 = escapeJson(leanManifest); { let _x_5 = alloc::vec![alloc::string::String::from("\"")]; { let _x_6 = { let mut __list = alloc::vec![_x_3]; __list.extend(_x_5.clone()); __list }; { let _x_7 = { let mut __list = alloc::vec![alloc::string::String::from("\"")]; __list.extend(_x_6); __list }; { let _x_9 = (_x_7).join(&alloc::string::String::from("")); { let _x_11 = escapeJson(coverage); { let _x_12 = { let mut __list = alloc::vec![_x_11]; __list.extend(_x_5.clone()); __list }; { let _x_13 = { let mut __list = alloc::vec![alloc::string::String::from("\"")]; __list.extend(_x_12); __list }; { let _x_14 = (_x_13).join(&alloc::string::String::from("")); { let _x_16 = escapeJson(kernel); { let _x_17 = { let mut __list = alloc::vec![_x_16]; __list.extend(_x_5.clone()); __list }; { let _x_18 = { let mut __list = alloc::vec![alloc::string::String::from("\"")]; __list.extend(_x_17); __list }; { let _x_19 = (_x_18).join(&alloc::string::String::from("")); { let _x_21 = escapeJson(modelId); { let _x_22 = { let mut __list = alloc::vec![_x_21]; __list.extend(_x_5.clone()); __list }; { let _x_23 = { let mut __list = alloc::vec![alloc::string::String::from("\"")]; __list.extend(_x_22); __list }; { let _x_24 = (_x_23).join(&alloc::string::String::from("")); { let _x_26 = escapeJson(semanticId); { let _x_27 = { let mut __list = alloc::vec![_x_26]; __list.extend(_x_5.clone()); __list }; { let _x_28 = { let mut __list = alloc::vec![alloc::string::String::from("\"")]; __list.extend(_x_27); __list }; { let _x_29 = (_x_28).join(&alloc::string::String::from("")); { let _x_31 = escapeJson(sourceId); { let _x_32 = { let mut __list = alloc::vec![_x_31]; __list.extend(_x_5.clone()); __list }; { let _x_33 = { let mut __list = alloc::vec![alloc::string::String::from("\"")]; __list.extend(_x_32); __list }; { let _x_34 = (_x_33).join(&alloc::string::String::from("")); { let _x_36 = alloc::vec![alloc::string::String::from(",\"version\":4}")]; { let _x_37 = { let mut __list = alloc::vec![_x_34]; __list.extend(_x_36); __list }; { let _x_38 = { let mut __list = alloc::vec![alloc::string::String::from(",\"source_id\":")]; __list.extend(_x_37); __list }; { let _x_39 = { let mut __list = alloc::vec![_x_29]; __list.extend(_x_38); __list }; { let _x_40 = { let mut __list = alloc::vec![alloc::string::String::from("}],\"semantic_id\":")]; __list.extend(_x_39); __list }; { let _x_41 = { let mut __list = alloc::vec![_x_24]; __list.extend(_x_40); __list }; { let _x_42 = { let mut __list = alloc::vec![alloc::string::String::from("},{\"kind\":\"model-document\",\"sha256\":")]; __list.extend(_x_41); __list }; { let _x_43 = { let mut __list = alloc::vec![_x_19]; __list.extend(_x_42); __list }; { let _x_44 = { let mut __list = alloc::vec![alloc::string::String::from("},{\"kind\":\"lean4-prod-kernel-ir\",\"sha256\":")]; __list.extend(_x_43); __list }; { let _x_45 = { let mut __list = alloc::vec![_x_14]; __list.extend(_x_44); __list }; { let _x_46 = { let mut __list = alloc::vec![alloc::string::String::from("},{\"kind\":\"lean4-prod-coverage\",\"sha256\":")]; __list.extend(_x_45); __list }; { let _x_47 = { let mut __list = alloc::vec![_x_9]; __list.extend(_x_46); __list }; { let _x_48 = { let mut __list = alloc::vec![alloc::string::String::from("{\"files\":[{\"kind\":\"lexlean-build-manifest\",\"sha256\":")]; __list.extend(_x_47); __list }; { let _x_49 = (_x_48).join(&alloc::string::String::from("")); _x_49 } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } }
}

pub fn admits(x_1: &[crate::Grant], x_2: &str) -> bool {
    match x_1 {
        [] => { let _x_31 = false; _x_31 },
        [head_20, tail_21 @ ..] => { let _x_40 = grantEndpoint(&(head_20)); { let _x_41 = (_x_40 == x_2); match _x_41 {
        false => { let _x_54 = admits(&(tail_21), (x_2).as_ref()); _x_54 },
        true => _x_41,
    } } },
    }
}

pub fn entryKappa(entry: &crate::Entry) -> alloc::string::String {
    { let _x_8 = &(entry).kappa; { let _x_20 = 2147483647; { let _x_13 = { let __value = _x_8; let __delimiter = alloc::string::String::from("\n"); let __maximum = usize::try_from(_x_20).ok(); if __delimiter.is_empty() { None } else { let __fields: alloc::vec::Vec<alloc::string::String> = __value.split(&__delimiter).map(alloc::string::String::from).collect(); __maximum.filter(|__maximum| __fields.len() <= *__maximum).map(|_| __fields) } }; match _x_13 {
        None => alloc::string::String::from(""),
        Some(val_16) => { let _x_25 = (val_16).join(&alloc::string::String::from("\n")); _x_25 },
    } } } }
}

pub fn entryLine(entry: &crate::Entry) -> alloc::string::String {
    { let _x_3 = entryPath(&(entry)); { let _x_4 = escapeJson(_x_3); { let _x_6 = alloc::vec![alloc::string::String::from("\"")]; { let _x_7 = { let mut __list = alloc::vec![_x_4]; __list.extend(_x_6.clone()); __list }; { let _x_8 = { let mut __list = alloc::vec![alloc::string::String::from("\"")]; __list.extend(_x_7); __list }; { let _x_10 = (_x_8).join(&alloc::string::String::from("")); { let _x_12 = entryKappa(&(entry)); { let _x_13 = escapeJson(_x_12); { let _x_14 = { let mut __list = alloc::vec![_x_13]; __list.extend(_x_6.clone()); __list }; { let _x_15 = { let mut __list = alloc::vec![alloc::string::String::from("\"")]; __list.extend(_x_14); __list }; { let _x_16 = (_x_15).join(&alloc::string::String::from("")); { let _x_20 = (entry).bytes; { let _x_21 = alloc::format!("{}", _x_20); { let _x_23 = alloc::vec![alloc::string::String::from("]")]; { let _x_24 = { let mut __list = alloc::vec![_x_21]; __list.extend(_x_23); __list }; { let _x_25 = { let mut __list = alloc::vec![alloc::string::String::from(",")]; __list.extend(_x_24); __list }; { let _x_26 = { let mut __list = alloc::vec![_x_16]; __list.extend(_x_25); __list }; { let _x_27 = { let mut __list = alloc::vec![alloc::string::String::from(",")]; __list.extend(_x_26); __list }; { let _x_28 = { let mut __list = alloc::vec![_x_10]; __list.extend(_x_27); __list }; { let _x_29 = { let mut __list = alloc::vec![alloc::string::String::from("[")]; __list.extend(_x_28); __list }; { let _x_30 = (_x_29).join(&alloc::string::String::from("")); _x_30 } } } } } } } } } } } } } } } } } } } } }
}

pub fn entryLines(x_1: &[crate::Entry]) -> alloc::string::String {
    match x_1 {
        [] => alloc::string::String::from(""),
        [head_29, tail_30 @ ..] => match tail_30 {
        [] => { let _x_49 = entryLine(&(head_29)); _x_49 },
        [head_50, tail_51 @ ..] => { let head_50 = head_50.clone(); { let _x_53 = entryLine(&(head_29)); { let _x_54 = entryLines(&(tail_30)); { let _x_56 = alloc::vec![_x_54]; { let _x_57 = { let mut __list = alloc::vec![_x_53]; __list.extend(_x_56); __list }; { let _x_59 = (_x_57).join(&alloc::string::String::from(",")); _x_59 } } } } } },
    },
    }
}

pub fn entryPath(entry: &crate::Entry) -> alloc::string::String {
    { let _x_8 = &(entry).path; { let _x_20 = 2147483647; { let _x_13 = { let __value = _x_8; let __delimiter = alloc::string::String::from("\n"); let __maximum = usize::try_from(_x_20).ok(); if __delimiter.is_empty() { None } else { let __fields: alloc::vec::Vec<alloc::string::String> = __value.split(&__delimiter).map(alloc::string::String::from).collect(); __maximum.filter(|__maximum| __fields.len() <= *__maximum).map(|_| __fields) } }; match _x_13 {
        None => alloc::string::String::from(""),
        Some(val_16) => { let _x_25 = (val_16).join(&alloc::string::String::from("\n")); _x_25 },
    } } } }
}

pub fn escapeBackslash(value: alloc::string::String) -> alloc::string::String {
    { let _x_19 = 2147483647; { let _x_12 = { let __value = value; let __delimiter = alloc::string::String::from("\\"); let __maximum = usize::try_from(_x_19).ok(); if __delimiter.is_empty() { None } else { let __fields: alloc::vec::Vec<alloc::string::String> = __value.split(&__delimiter).map(alloc::string::String::from).collect(); __maximum.filter(|__maximum| __fields.len() <= *__maximum).map(|_| __fields) } }; match _x_12 {
        None => alloc::string::String::from(""),
        Some(val_15) => { let _x_24 = (val_15).join(&alloc::string::String::from("\\\\")); _x_24 },
    } } }
}

pub fn escapeJson(value: alloc::string::String) -> alloc::string::String {
    { let _x_8 = escapeReturn(value); { let _x_20 = 2147483647; { let _x_13 = { let __value = _x_8; let __delimiter = alloc::string::String::from("\t"); let __maximum = usize::try_from(_x_20).ok(); if __delimiter.is_empty() { None } else { let __fields: alloc::vec::Vec<alloc::string::String> = __value.split(&__delimiter).map(alloc::string::String::from).collect(); __maximum.filter(|__maximum| __fields.len() <= *__maximum).map(|_| __fields) } }; match _x_13 {
        None => alloc::string::String::from(""),
        Some(val_16) => { let _x_25 = (val_16).join(&alloc::string::String::from("\\t")); _x_25 },
    } } } }
}

pub fn escapeNewline(value: alloc::string::String) -> alloc::string::String {
    { let _x_8 = escapeQuote(value); { let _x_20 = 2147483647; { let _x_13 = { let __value = _x_8; let __delimiter = alloc::string::String::from("\n"); let __maximum = usize::try_from(_x_20).ok(); if __delimiter.is_empty() { None } else { let __fields: alloc::vec::Vec<alloc::string::String> = __value.split(&__delimiter).map(alloc::string::String::from).collect(); __maximum.filter(|__maximum| __fields.len() <= *__maximum).map(|_| __fields) } }; match _x_13 {
        None => alloc::string::String::from(""),
        Some(val_16) => { let _x_25 = (val_16).join(&alloc::string::String::from("\\n")); _x_25 },
    } } } }
}

pub fn escapeQuote(value: alloc::string::String) -> alloc::string::String {
    { let _x_8 = escapeBackslash(value); { let _x_20 = 2147483647; { let _x_13 = { let __value = _x_8; let __delimiter = alloc::string::String::from("\""); let __maximum = usize::try_from(_x_20).ok(); if __delimiter.is_empty() { None } else { let __fields: alloc::vec::Vec<alloc::string::String> = __value.split(&__delimiter).map(alloc::string::String::from).collect(); __maximum.filter(|__maximum| __fields.len() <= *__maximum).map(|_| __fields) } }; match _x_13 {
        None => alloc::string::String::from(""),
        Some(val_16) => { let _x_25 = (val_16).join(&alloc::string::String::from("\\\"")); _x_25 },
    } } } }
}

pub fn escapeReturn(value: alloc::string::String) -> alloc::string::String {
    { let _x_8 = escapeNewline(value); { let _x_20 = 2147483647; { let _x_13 = { let __value = _x_8; let __delimiter = alloc::string::String::from("\r"); let __maximum = usize::try_from(_x_20).ok(); if __delimiter.is_empty() { None } else { let __fields: alloc::vec::Vec<alloc::string::String> = __value.split(&__delimiter).map(alloc::string::String::from).collect(); __maximum.filter(|__maximum| __fields.len() <= *__maximum).map(|_| __fields) } }; match _x_13 {
        None => alloc::string::String::from(""),
        Some(val_16) => { let _x_25 = (val_16).join(&alloc::string::String::from("\\r")); _x_25 },
    } } } }
}

pub fn grantEndpoint(grant: &crate::Grant) -> alloc::string::String {
    { let _x_8 = &(grant).endpoint; { let _x_20 = 2147483647; { let _x_13 = { let __value = _x_8; let __delimiter = alloc::string::String::from("\n"); let __maximum = usize::try_from(_x_20).ok(); if __delimiter.is_empty() { None } else { let __fields: alloc::vec::Vec<alloc::string::String> = __value.split(&__delimiter).map(alloc::string::String::from).collect(); __maximum.filter(|__maximum| __fields.len() <= *__maximum).map(|_| __fields) } }; match _x_13 {
        None => alloc::string::String::from(""),
        Some(val_16) => { let _x_25 = (val_16).join(&alloc::string::String::from("\n")); _x_25 },
    } } } }
}

pub fn headOf(x_1: &[crate::Ref], x_2: alloc::string::String) -> Option<alloc::string::String> {
    match x_1 {
        [] => None,
        [head_21, tail_22 @ ..] => { let _x_42 = refBranch(&(head_21)); { let _x_43 = (_x_42 == x_2.clone()); match _x_43 {
        false => { let _x_56 = headOf(&(tail_22), x_2.clone()); _x_56 },
        true => { let _x_57 = refKappa(&(head_21)); { let _x_58 = Some(_x_57); _x_58 } },
    } } },
    }
}

pub fn networkDecision(capabilities: &crate::Capabilities, origin: alloc::string::String) -> crate::Decision {
    { let _x_1 = &(capabilities).endpoints; { let _x_2 = admits(&(_x_1), (origin).as_ref()); match _x_2 {
        false => { let _x_25 = crate::Decision::Refuse; _x_25 },
        true => { let _x_26 = crate::Decision::Accept; _x_26 },
    } } }
}

pub fn previewPath(kappa: alloc::string::String) -> alloc::string::String {
    { let _x_4 = alloc::vec![alloc::string::String::from("/")]; { let _x_5 = { let mut __list = alloc::vec![kappa]; __list.extend(_x_4); __list }; { let _x_6 = { let mut __list = alloc::vec![alloc::string::String::from("/p/")]; __list.extend(_x_5); __list }; { let _x_8 = (_x_6).join(&alloc::string::String::from("")); _x_8 } } } }
}

pub fn projectName(project: &crate::Project) -> alloc::string::String {
    { let _x_8 = &(project).label; { let _x_20 = 2147483647; { let _x_13 = { let __value = _x_8; let __delimiter = alloc::string::String::from("\n"); let __maximum = usize::try_from(_x_20).ok(); if __delimiter.is_empty() { None } else { let __fields: alloc::vec::Vec<alloc::string::String> = __value.split(&__delimiter).map(alloc::string::String::from).collect(); __maximum.filter(|__maximum| __fields.len() <= *__maximum).map(|_| __fields) } }; match _x_13 {
        None => alloc::string::String::from(""),
        Some(val_16) => { let _x_25 = (val_16).join(&alloc::string::String::from("\n")); _x_25 },
    } } } }
}

pub fn refBranch(__prod_ref: &crate::Ref) -> alloc::string::String {
    { let _x_8 = &(__prod_ref).branch; { let _x_20 = 2147483647; { let _x_13 = { let __value = _x_8; let __delimiter = alloc::string::String::from("\n"); let __maximum = usize::try_from(_x_20).ok(); if __delimiter.is_empty() { None } else { let __fields: alloc::vec::Vec<alloc::string::String> = __value.split(&__delimiter).map(alloc::string::String::from).collect(); __maximum.filter(|__maximum| __fields.len() <= *__maximum).map(|_| __fields) } }; match _x_13 {
        None => alloc::string::String::from(""),
        Some(val_16) => { let _x_25 = (val_16).join(&alloc::string::String::from("\n")); _x_25 },
    } } } }
}

pub fn refKappa(__prod_ref: &crate::Ref) -> alloc::string::String {
    { let _x_8 = &(__prod_ref).kappa; { let _x_20 = 2147483647; { let _x_13 = { let __value = _x_8; let __delimiter = alloc::string::String::from("\n"); let __maximum = usize::try_from(_x_20).ok(); if __delimiter.is_empty() { None } else { let __fields: alloc::vec::Vec<alloc::string::String> = __value.split(&__delimiter).map(alloc::string::String::from).collect(); __maximum.filter(|__maximum| __fields.len() <= *__maximum).map(|_| __fields) } }; match _x_13 {
        None => alloc::string::String::from(""),
        Some(val_16) => { let _x_25 = (val_16).join(&alloc::string::String::from("\n")); _x_25 },
    } } } }
}

pub fn restoreDecision(derived: alloc::string::String, expected: alloc::string::String) -> crate::Decision {
    { let _x_3 = (derived == expected); match _x_3 {
        false => { let _x_26 = crate::Decision::Refuse; _x_26 },
        true => { let _x_27 = crate::Decision::Accept; _x_27 },
    } }
}

pub fn snapshotPreimage(project: &crate::Project, parent: alloc::string::String) -> alloc::string::String {
    { let _x_2 = &(project).entries; { let _x_3 = entryLines(&(_x_2)); { let _x_6 = projectName(&(project)); { let _x_7 = escapeJson(_x_6); { let _x_9 = alloc::vec![alloc::string::String::from("\"")]; { let _x_10 = { let mut __list = alloc::vec![_x_7]; __list.extend(_x_9.clone()); __list }; { let _x_11 = { let mut __list = alloc::vec![alloc::string::String::from("\"")]; __list.extend(_x_10); __list }; { let _x_13 = (_x_11).join(&alloc::string::String::from("")); { let _x_15 = escapeJson(parent); { let _x_16 = { let mut __list = alloc::vec![_x_15]; __list.extend(_x_9.clone()); __list }; { let _x_17 = { let mut __list = alloc::vec![alloc::string::String::from("\"")]; __list.extend(_x_16); __list }; { let _x_18 = (_x_17).join(&alloc::string::String::from("")); { let _x_20 = alloc::vec![alloc::string::String::from("}")]; { let _x_21 = { let mut __list = alloc::vec![_x_18]; __list.extend(_x_20); __list }; { let _x_22 = { let mut __list = alloc::vec![alloc::string::String::from(",\"parent\":")]; __list.extend(_x_21); __list }; { let _x_23 = { let mut __list = alloc::vec![_x_13]; __list.extend(_x_22); __list }; { let _x_24 = { let mut __list = alloc::vec![alloc::string::String::from("],\"name\":")]; __list.extend(_x_23); __list }; { let _x_25 = { let mut __list = alloc::vec![_x_3]; __list.extend(_x_24); __list }; { let _x_26 = { let mut __list = alloc::vec![alloc::string::String::from("{\"entries\":[")]; __list.extend(_x_25); __list }; { let _x_27 = (_x_26).join(&alloc::string::String::from("")); _x_27 } } } } } } } } } } } } } } } } } } } }
}

