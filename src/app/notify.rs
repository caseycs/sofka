use super::*;

impl App {
    /// `:notify` — toggle a watch-notification on the selected object. While
    /// active, every state change the watch sees for it (the same transitions
    /// the timeline records: rollout progress, readiness, phase, restarts,
    /// waiting reasons, conditions) flashes, rings the terminal bell, and
    /// emits an OSC 9 desktop notification. Each notify is its own bounded
    /// single-object watch, so it keeps firing no matter which view is open,
    /// until toggled off or the session ends. Nothing touches disk.
    pub(super) fn toggle_notify(&mut self) {
        if matches!(self.kind_plural.as_str(), "helm" | "helmhistory") {
            self.flash_warn("notify is not available for Helm views");
            return;
        }
        let Some(kind) = self.kind.clone() else {
            self.flash_warn("select a resource first");
            return;
        };
        let Some((key, name, ns)) = self.selected_ref().map(|obj| {
            (
                row_key(obj),
                obj.metadata.name.clone().unwrap_or_default(),
                obj.metadata.namespace.clone().unwrap_or_default(),
            )
        }) else {
            self.flash_warn("no selection to notify on");
            return;
        };
        let id = format!("{}/{key}", self.kind_plural);
        if let Some(handle) = self.notify_tasks.remove(&id) {
            handle.abort();
            self.flash = format!(
                "notify off: {key} ({} still active)",
                self.notify_tasks.len()
            );
            self.flash_err = false;
            return;
        }
        if name.is_empty() {
            self.flash_warn("object has no name to watch");
            return;
        }
        let label = format!("{}/{name}", trim_s(&self.kind_plural));
        let plural = self.kind_plural.clone();
        let client = self.cluster.client.clone();
        let ar = kind.ar.clone();
        let namespaced = kind.namespaced;
        let tx = self.tx.clone();

        let handle = tokio::spawn(async move {
            let api: Api<DynamicObject> = if namespaced && !ns.is_empty() {
                Api::namespaced_with(client, &ns, &ar)
            } else {
                Api::all_with(client, &ar)
            };
            let cfg = watcher::Config::default()
                .any_semantic()
                .fields(&format!("metadata.name={name}"));
            let mut stream = watcher(api, cfg).boxed();
            let mut prev: Option<DynamicObject> = None;
            // The initial list describes the state the user just looked at —
            // only changes after that are news.
            let mut synced = false;
            while let Some(event) = stream.next().await {
                match event {
                    Ok(watcher::Event::Apply(o)) | Ok(watcher::Event::InitApply(o)) => {
                        let news: Vec<String> = match &prev {
                            Some(p) => crate::timeline::transitions(p, &o, &plural)
                                .into_iter()
                                .map(|(_, text)| format!("{label}: {text}"))
                                .collect(),
                            None if synced => vec![format!("{label}: created")],
                            None => Vec::new(),
                        };
                        for text in news {
                            if tx.send(Msg::Notify(text)).await.is_err() {
                                return;
                            }
                        }
                        prev = Some(o);
                    }
                    Ok(watcher::Event::Delete(_)) => {
                        prev = None;
                        if tx
                            .send(Msg::Notify(format!("{label}: deleted")))
                            .await
                            .is_err()
                        {
                            return;
                        }
                    }
                    Ok(watcher::Event::Init) => {}
                    Ok(watcher::Event::InitDone) => synced = true,
                    // The watcher self-heals; a transient error is not a
                    // state change worth ringing a bell for.
                    Err(_) => {}
                }
            }
        });
        self.notify_tasks.insert(id, handle);
        self.flash = format!(
            "notify on: {key} — changes flash + bell ({} active)",
            self.notify_tasks.len()
        );
        self.flash_err = false;
    }

    /// The message the main loop should ring the bell / emit a desktop
    /// notification for, if one arrived since the last frame.
    pub fn take_notification(&mut self) -> Option<String> {
        self.pending_notify.take()
    }
}

/// The terminal escape sequences that deliver one `:notify` event, per the
/// `[notify]` config: an optional BEL, then the chosen desktop-notification
/// protocol(s). Control characters are stripped so object content can't
/// smuggle sequences; BEL-terminated OSC matches every terminal's documented
/// examples.
///
/// - `osc9` — iTerm2-style, body only: iTerm2, Ghostty, kitty, WezTerm,
///   foot, Windows Terminal.
/// - `osc777` — rxvt-style `notify;title;body`: Ghostty (which recommends
///   it), kitty, WezTerm, foot, urxvt.
///
/// An unknown `desktop` value behaves as the default `osc9` (it warned at
/// config load).
pub fn notification_sequence(text: &str, cfg: &crate::config::NotifyConfig) -> String {
    let clean: String = text.chars().filter(|c| !c.is_control()).collect();
    let mut seq = String::new();
    if cfg.bell {
        seq.push('\x07');
    }
    let desktop = match cfg.desktop.as_str() {
        d @ ("osc9" | "osc777" | "both" | "off") => d,
        _ => "osc9",
    };
    if matches!(desktop, "osc9" | "both") {
        seq.push_str(&format!("\x1b]9;sofka: {clean}\x07"));
    }
    if matches!(desktop, "osc777" | "both") {
        seq.push_str(&format!("\x1b]777;notify;sofka;{clean}\x07"));
    }
    seq
}
