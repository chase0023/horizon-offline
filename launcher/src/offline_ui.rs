//! Offline profiles in the window: creating, renaming, deleting and choosing one. The card is
//! shared by Settings → Accounts and the first-run "Choose how to play" screen; the logic is
//! `horizon_launcher::account::Accounts`, so the CLI and MCP use the same profiles.
use super::*;
use crate::glass_ui::{self as ui, white};
use horizon_theme::glass::{ink, radius};

impl Launcher {
    /// Creates the profile named in the field and makes it who plays. Nothing here touches the
    /// network or the keychain.
    pub(super) fn offline_create(&mut self) {
        let name = self.offline_name.text().trim().to_owned();
        let created = self.accounts.add_offline(&name);
        let created = match created {
            Ok(profile) => self.accounts.save().map(|()| profile),
            Err(e) => Err(e),
        };
        match created {
            Ok(profile) => {
                self.offline_name.set(String::new());
                self.offline_error = None;
                self.offline_remove = None;
                self.owner_screen_open = false;
                if !self.settings.owner_notice_seen
                    || self.settings.account_choice_seen != Some(true)
                {
                    self.settings.owner_notice_seen = true;
                    self.settings.account_choice_seen = Some(true);
                    self.save_settings();
                }
                crate::glass_toast::say(format!(
                    "{} {}",
                    tr("Playing offline as"),
                    profile.name
                ));
            }
            Err(e) => self.offline_error = Some(tr(&e).to_owned()),
        }
    }

    /// Switches to a profile, keeping it as the last one chosen in `accounts.json`.
    pub(super) fn offline_select(&mut self, uuid: &str) {
        self.play_as(uuid);
    }

    pub(super) fn offline_start_rename(
        &mut self,
        uuid: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(profile) = self.accounts.find_offline(uuid) else {
            self.offline_error = Some(tr("That offline profile no longer exists.").to_owned());
            return;
        };
        self.offline_error = None;
        self.offline_remove = None;
        let handle = cx.focus_handle().tab_index(0).tab_stop(true);
        self.offline_rename = Some((
            uuid.to_owned(),
            line_edit::LineEdit::new(profile.name.clone(), 16, 24),
            handle.clone(),
        ));
        window.focus(&handle, cx);
    }

    pub(super) fn offline_cancel_rename(&mut self) {
        self.offline_rename = None;
        self.offline_error = None;
    }

    pub(super) fn offline_save_rename(&mut self) {
        let Some((uuid, field, _)) = &self.offline_rename else {
            return;
        };
        let name = field.text().trim().to_owned();
        let renamed = self.accounts.rename_offline(uuid, &name);
        match renamed {
            Ok(profile) => match self.accounts.save() {
                Ok(()) => {
                    self.offline_rename = None;
                    self.offline_error = None;
                    crate::glass_toast::say(format!(
                        "{} {}",
                        tr("Renamed to"),
                        profile.name
                    ));
                }
                Err(e) => self.offline_error = Some(e),
            },
            Err(e) => self.offline_error = Some(tr(&e).to_owned()),
        }
    }

    /// The first press asks, the second deletes.
    pub(super) fn offline_delete(&mut self, uuid: &str) {
        if self.offline_remove.as_deref() != Some(uuid) {
            self.offline_remove = Some(uuid.to_owned());
            return;
        }
        let removed = self.accounts.remove_offline(uuid);
        let removed = match removed {
            Ok(()) => self.accounts.save(),
            Err(e) => Err(e),
        };
        match removed {
            Ok(()) => {
                self.offline_remove = None;
                self.offline_error = None;
                if self.offline_rename.as_ref().is_some_and(|(id, ..)| id == uuid) {
                    self.offline_rename = None;
                }
                crate::glass_toast::say(tr("Offline profile deleted."));
            }
            Err(e) => self.offline_error = Some(tr(&e).to_owned()),
        }
    }

    /// The new-profile field. Enter creates; Escape leaves the field.
    pub(super) fn offline_name_field(
        &mut self,
        id: &'static str,
        placeholder: gpui::SharedString,
        cx: &mut Context<Self>,
    ) -> Stateful<Div> {
        let focused = self.offline_focused;
        let invalid = self.offline_error.is_some();
        self.offline_name.show(20);
        let field = ui::field(focused, invalid)
            .id(id)
            .track_focus(&self.offline_name_focus)
            .flex_1()
            .min_w_0()
            .cursor_text()
            .child(line_edit::content_with(
                &self.offline_name,
                &self.theme,
                focused,
                false,
                false,
                placeholder,
            ))
            .on_click(cx.listener(|this, _: &ClickEvent, window, cx| {
                window.focus(&this.offline_name_focus, cx);
                cx.notify();
            }))
            .on_key_down(cx.listener(|this, ev: &KeyDownEvent, window, cx| {
                match line_edit::handle(&mut this.offline_name, ev, cx) {
                    line_edit::Key::Enter => this.offline_create(),
                    line_edit::Key::Escape => window.blur(cx),
                    line_edit::Key::Pass => return,
                    _ => {}
                }
                cx.stop_propagation();
                cx.notify();
            }));
        line_edit::mouse(field, cx, |this| &mut this.offline_name)
    }

    /// The rename field of the profile being edited, with Save and Cancel.
    pub(super) fn offline_rename_field(&mut self, cx: &mut Context<Self>) -> Div {
        let focused = self.offline_rename_focused;
        let t = self.theme;
        let Some((_, _, handle)) = &self.offline_rename else {
            return div();
        };
        let handle = handle.clone();
        let content = line_edit::content(
            &self.offline_rename.as_ref().unwrap().1,
            &t,
            focused,
            false,
            false,
            "Offline name",
        );
        let input = ui::field(focused, false)
            .id("offline-rename")
            .track_focus(&handle)
            .flex_1()
            .min_w_0()
            .cursor_text()
            .child(content)
            .on_click(cx.listener(|this, _: &ClickEvent, window, cx| {
                if let Some((_, _, handle)) = &this.offline_rename {
                    let handle = handle.clone();
                    window.focus(&handle, cx);
                }
                cx.notify();
            }))
            .on_key_down(cx.listener(|this, ev: &KeyDownEvent, _window, cx| {
                let Some((_, field, _)) = this.offline_rename.as_mut() else {
                    return;
                };
                match line_edit::handle(field, ev, cx) {
                    line_edit::Key::Enter => this.offline_save_rename(),
                    line_edit::Key::Escape => this.offline_cancel_rename(),
                    line_edit::Key::Pass => return,
                    _ => {}
                }
                cx.stop_propagation();
                cx.notify();
            }));
        let input = line_edit::mouse(input, cx, |this| {
            &mut this.offline_rename.as_mut().unwrap().1
        });
        div()
            .flex()
            .items_center()
            .gap(px(8.))
            .w_full()
            .child(input)
            .child(ui::quiet("offline-rename-save", tr("Save"), 36., &t).on_click(
                cx.listener(|this, _: &ClickEvent, _, cx| {
                    this.offline_save_rename();
                    cx.notify();
                }),
            ))
            .child(
                ui::quiet("offline-rename-cancel", tr("Cancel"), 36., &t).on_click(
                    cx.listener(|this, _: &ClickEvent, _, cx| {
                        this.offline_cancel_rename();
                        cx.notify();
                    }),
                ),
            )
    }

    /// The offline side of the account choice: the new-profile field, the profiles saved here,
    /// and their Play as / Edit / Delete. `manage` hides Edit and Delete on the first-run screen.
    pub(super) fn glass_offline_choices(
        &mut self,
        manage: bool,
        cx: &mut Context<Self>,
    ) -> Div {
        let t = self.theme;
        let mut card = div().flex().flex_col().gap(px(10.));
        if self.accounts.offline_profiles.is_empty() {
            card = card.child(ui::text(14., FontWeight::NORMAL, ink::BODY).child(tr(
                "No offline profile yet. Pick a name and play without signing in.",
            )));
        } else {
            let now = self.accounts.selected_offline().map(|p| p.uuid.clone());
            let profiles: Vec<_> = self.accounts.offline_profiles.clone();
            for (i, profile) in profiles.into_iter().enumerate() {
                let playing = now.as_deref() == Some(profile.uuid.as_str());
                if manage
                    && self
                        .offline_rename
                        .as_ref()
                        .is_some_and(|(uuid, ..)| uuid == &profile.uuid)
                {
                    card = card.child(
                        div()
                            .flex()
                            .flex_col()
                            .gap(px(6.))
                            .child(
                                ui::text(12., FontWeight::NORMAL, ink::QUIET)
                                    .child(trf("Rename {name}", &[("name", &profile.name)])),
                            )
                            .child(self.offline_rename_field(cx)),
                    );
                    continue;
                }
                let uuid = profile.uuid.clone();
                let mut actions = div().flex().items_center().gap(px(6.));
                if playing {
                    actions = actions.child(
                        ui::text(13., FontWeight::MEDIUM, ink::FULL)
                            .child(tr("Playing")),
                    );
                } else {
                    let select = uuid.clone();
                    actions = actions.child(
                        ui::quiet(
                            SharedString::from(format!("offline-play-{i}")),
                            tr("Play as"),
                            32.,
                            &t,
                        )
                        .on_click(cx.listener(move |this, _: &ClickEvent, _, cx| {
                            this.offline_select(&select);
                            cx.notify();
                        })),
                    );
                }
                if manage {
                    let edit = uuid.clone();
                    actions = actions.child(
                        ui::quiet(
                            SharedString::from(format!("offline-edit-{i}")),
                            tr("Edit"),
                            32.,
                            &t,
                        )
                        .on_click(cx.listener(move |this, _: &ClickEvent, window, cx| {
                            this.offline_start_rename(&edit, window, cx);
                            cx.notify();
                        })),
                    );
                    let remove = uuid.clone();
                    let label = if self.offline_remove.as_deref() == Some(uuid.as_str()) {
                        tr("Confirm delete")
                    } else {
                        tr("Delete")
                    };
                    actions = actions.child(
                        ui::quiet(
                            SharedString::from(format!("offline-remove-{i}")),
                            label,
                            32.,
                            &t,
                        )
                        .on_click(cx.listener(move |this, _: &ClickEvent, _, cx| {
                            this.offline_delete(&remove);
                            cx.notify();
                        })),
                    );
                }
                card = card.child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(12.))
                        .p(px(10.))
                        .rounded(px(radius::CONTROL))
                        .bg(white(if playing { 0.07 } else { 0.035 }))
                        .child(
                            div()
                                .size(px(32.))
                                .rounded(px(8.))
                                .overflow_hidden()
                                .child(self.profile_heads.draw(
                                    Some(profile.uuid.as_str()),
                                    32.,
                                    self.scale,
                                    &t,
                                )),
                        )
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .flex_1()
                                .min_w_0()
                                .child(
                                    ui::text(14., FontWeight::MEDIUM, ink::FULL)
                                        .truncate()
                                        .child(profile.name.clone()),
                                )
                                .child(
                                    ui::text(12., FontWeight::NORMAL, ink::QUIET)
                                        .child(tr("Offline account · saved on this computer")),
                                ),
                        )
                        .child(actions),
                );
            }
        }
        card = card.child(
            div()
                .flex()
                .items_center()
                .gap(px(8.))
                .mt(px(4.))
                .child(self.offline_name_field(
                    "offline-new-name",
                    tr("Your offline name").into(),
                    cx,
                ))
                .child(
                    ui::button(
                        "offline-create".into(),
                        tr("Create profile").into(),
                        Kind::Primary,
                        &t,
                    )
                    .h(px(44.))
                    .px(px(18.))
                    .on_click(cx.listener(|this, _: &ClickEvent, _, cx| {
                        this.offline_create();
                        cx.notify();
                    })),
                ),
        );
        card = card.child(
            ui::text(12., FontWeight::NORMAL, ink::QUIET).child(tr(
                "Offline names use 3 to 16 letters, digits or underscores. An offline profile can play single-player and LAN worlds; online servers need a Microsoft account.",
            )),
        );
        if let Some(error) = &self.offline_error {
            card = card.child(
                ui::text(13., FontWeight::MEDIUM, ink::FULL)
                    .text_color(ui::color(horizon_theme::glass::DANGER))
                    .child(error.clone()),
            );
        }
        card
    }

    /// Settings' offline card: the profiles and their management, in the glass look.
    pub(super) fn glass_offline_card(&mut self, cx: &mut Context<Self>) -> Div {
        div()
            .flex()
            .flex_col()
            .rounded(px(16.))
            .bg(white(0.045))
            .border_1()
            .border_color(white(ui::edge::REST))
            .p(px(24.))
            .gap(px(12.))
            .child(ui::icon("user", 28., white(ink::BODY)))
            .child(
                div()
                    .font_family(horizon_theme::glass::DISPLAY_FONT)
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_size(px(22.))
                    .line_height(px(27.5))
                    .text_color(white(ink::FULL))
                    .child(tr("Offline profiles")),
            )
            .child(
                ui::text(14., FontWeight::NORMAL, 0.65)
                    .line_height(px(21.7))
                    .child(tr(
                        "Local profiles need no Microsoft account, email or password. They play \
                         single-player and LAN worlds; authenticated online servers still need a \
                         Microsoft account that owns Minecraft: Java Edition.",
                    )),
            )
            .child(self.glass_offline_choices(true, cx))
            .child(
                ui::text(12., FontWeight::NORMAL, 0.45)
                    .line_height(px(18.))
                    .child(tr(
                        "Profiles are saved in accounts.json on this computer. Renaming one makes \
                         a new offline identity, as the game derives the UUID from the name.",
                    )),
            )
            .child(
                ui::text(12., FontWeight::NORMAL, 0.45)
                    .line_height(px(18.))
                    .child(tr(
                        "Never enter a Microsoft password in an offline profile: Horizon never \
                         asks for one.",
                    )),
            )
    }

    /// The legacy look's offline rows: the same profiles, with the blocky widgets.
    pub(super) fn legacy_offline_rows(&mut self, cx: &mut Context<Self>) -> (Div, usize) {
        let t = self.theme;
        let mut rows = div().flex().flex_col();
        let mut n = 0;
        let now = self.accounts.selected_offline().map(|p| p.uuid.clone());
        let profiles: Vec<_> = self.accounts.offline_profiles.clone();
        for profile in profiles {
            let uuid = profile.uuid.clone();
            let playing = now.as_deref() == Some(uuid.as_str());
            let mut actions = div().flex().gap(px(4.));
            if !playing {
                let select = uuid.clone();
                actions = actions.child(
                    button(format!("offline-play-{}", profile.uuid), "Play as", Kind::Secondary, &t).on_click(
                        cx.listener(move |this, _: &ClickEvent, _, cx| {
                            this.offline_select(&select);
                            cx.notify();
                        }),
                    ),
                );
            }
            let edit = uuid.clone();
            actions = actions.child(
                button(format!("offline-edit-{}", profile.uuid), "Edit", Kind::Ghost, &t).on_click(cx.listener(
                    move |this, _: &ClickEvent, window, cx| {
                        this.offline_start_rename(&edit, window, cx);
                        cx.notify();
                    },
                )),
            );
            let remove = uuid.clone();
            actions = actions.child(
                button(
                    format!("offline-remove-{}", profile.uuid),
                    if self.offline_remove.as_deref() == Some(uuid.as_str()) {
                        "Confirm delete"
                    } else {
                        "Delete"
                    },
                    Kind::Ghost,
                    &t,
                )
                .on_click(cx.listener(move |this, _: &ClickEvent, _, cx| {
                    this.offline_delete(&remove);
                    cx.notify();
                })),
            );
            n += 1;
            rows = rows.child(account_row(
                Some(self.profile_heads.draw(Some(&profile.uuid), 32., self.scale, &t)),
                profile.name.clone(),
                Some(
                    div()
                        .child(tr(if playing {
                            "Offline account · plays"
                        } else {
                            "Offline account · saved on this computer"
                        }))
                        .into_any_element(),
                ),
                actions,
                &t,
            ));
            if self
                .offline_rename
                .as_ref()
                .is_some_and(|(id, ..)| id == &uuid)
            {
                rows = rows.child(self.offline_rename_field(cx));
            }
        }
        n += 1;
        rows = rows.child(
            div()
                .flex()
                .flex_col()
                .gap(px(6.))
                .child(self.legacy_offline_name_field(cx))
                .child(
                    div()
                        .flex()
                        .gap(px(4.))
                        .child(
                            button("offline-create", "Create profile", Kind::Secondary, &t)
                                .on_click(cx.listener(|this, _: &ClickEvent, _, cx| {
                                    this.offline_create();
                                    cx.notify();
                                })),
                        ),
                )
                .children(self.offline_error.as_ref().map(|e| {
                    div().text_color(t.danger).child(e.clone())
                })),
        );
        (rows, n)
    }

    /// The legacy look's new-profile field (the glass look draws `offline_name_field`).
    fn legacy_offline_name_field(&mut self, cx: &mut Context<Self>) -> Stateful<Div> {
        let t = self.theme;
        let focused = self.offline_focused;
        self.offline_name.show(30);
        let field = div()
            .flex()
            .items_center()
            .h(px(theme::FIELD))
            .px(px(12.))
            .w(px(320.))
            .overflow_hidden()
            .rounded(px(theme::RADIUS))
            .border_2()
            .border_color(if focused { t.accent } else { t.outline })
            .bg(t.well)
            .id("offline-new-name-legacy")
            .track_focus(&self.offline_name_focus)
            .cursor_text()
            .child(line_edit::content_with(
                &self.offline_name,
                &t,
                focused,
                false,
                false,
                tr("Your offline name").into(),
            ))
            .on_click(cx.listener(|this, _: &ClickEvent, window, cx| {
                window.focus(&this.offline_name_focus, cx);
                cx.notify();
            }))
            .on_key_down(cx.listener(|this, ev: &KeyDownEvent, window, cx| {
                match line_edit::handle(&mut this.offline_name, ev, cx) {
                    line_edit::Key::Enter => this.offline_create(),
                    line_edit::Key::Escape => window.blur(cx),
                    line_edit::Key::Pass => return,
                    _ => {}
                }
                cx.stop_propagation();
                cx.notify();
            }));
        line_edit::mouse(field, cx, |this| &mut this.offline_name)
    }
}
