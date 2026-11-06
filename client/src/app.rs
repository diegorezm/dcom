use crate::mock::{self, Data, Message};
use crate::theme::{self, ACCENT, HOVER, MENTION, MUTED, RAIL, RAISED, SURFACE, TEXT};
use eframe::egui::text::{LayoutJob, TextFormat};
use eframe::egui::{
    self, Align, Align2, Color32, CornerRadius, FontId, Galley, Margin, Painter, Pos2, Rect,
    RichText, Sense, Stroke, Ui, Vec2, pos2, vec2,
};
use egui_phosphor::regular as icon;
use std::ffi::OsString;
use std::sync::Arc;
use std::time::Instant;

const RAIL_WIDTH: f32 = 92.0;
const LIST_WIDTH: f32 = 340.0;
const AVATAR: f32 = 44.0;
const AVATAR_GAP: f32 = 12.0;
const TIME_WIDTH: f32 = 46.0;
const BUBBLE_X: f32 = 14.0;
const BUBBLE_Y: f32 = 10.0;
const MAX_BUBBLE_TEXT: f32 = 620.0;
const NAME_HEIGHT: f32 = 21.0;
const GROUP_GAP: f32 = 18.0;
const ROW_GAP: f32 = 5.0;
/// A largura de quebra anda em degraus para não remedir o histórico a cada pixel de resize.
const WRAP_STEP: f32 = 24.0;
const BENCH_SECONDS: f32 = 10.0;

/// Posição vertical de cada mensagem, para desenhar só as visíveis.
#[derive(Default)]
struct RowLayout {
    key: (usize, usize, usize, u32),
    tops: Vec<f32>,
}

struct Bench {
    output: OsString,
    started: Instant,
    frames: u32,
}

pub struct App {
    data: Data,
    space: usize,
    conversation: usize,
    search: String,
    draft: String,
    rows: RowLayout,
    bench: Option<Bench>,
}

impl App {
    pub fn new(bench_output: Option<OsString>) -> Self {
        Self {
            data: mock::data(),
            space: 1,
            conversation: 0,
            search: String::new(),
            draft: String::new(),
            rows: RowLayout::default(),
            bench: bench_output.map(|output| Bench {
                output,
                started: Instant::now(),
                frames: 0,
            }),
        }
    }

    fn rail(&mut self, ui: &mut Ui) {
        ui.add_space(22.0);
        let (logo, _) = ui.allocate_exact_size(vec2(RAIL_WIDTH, 44.0), Sense::hover());
        ui.painter().text(
            logo.center(),
            Align2::CENTER_CENTER,
            "dc",
            theme::semibold(30.0),
            TEXT,
        );
        ui.add_space(22.0);

        for index in 0..self.data.spaces.len() {
            let space = &self.data.spaces[index];
            let (rect, response) = ui.allocate_exact_size(vec2(RAIL_WIDTH, 84.0), Sense::click());
            let selected = self.space == index;
            let painter = ui.painter();
            if selected || response.hovered() {
                let fill = if selected { RAISED } else { HOVER };
                painter.rect_filled(rect.shrink2(vec2(8.0, 2.0)), 16.0, fill);
            }
            let tile = Rect::from_center_size(rect.center() - vec2(0.0, 11.0), Vec2::splat(40.0));
            if index == 0 {
                painter.text(
                    tile.center(),
                    Align2::CENTER_CENTER,
                    icon::CHATS_CIRCLE,
                    theme::regular(30.0),
                    TEXT,
                );
            } else {
                avatar(painter, tile, space.color, space.name, 12.0, 16.0);
            }
            painter.text(
                pos2(rect.center().x, rect.bottom() - 18.0),
                Align2::CENTER_CENTER,
                space.name,
                theme::regular(12.5),
                if selected { TEXT } else { MUTED },
            );
            if space.unread > 0 {
                badge(painter, tile.right_top() + vec2(0.0, 2.0), space.unread);
            }
            if index == 0 {
                let y = rect.bottom() + 8.0;
                let x = rect.center().x;
                painter.hline(x - 18.0..=x + 18.0, y, Stroke::new(1.0, RAISED));
                ui.add_space(16.0);
            }
            if response.clicked() {
                self.space = index;
                self.conversation = 0;
            }
        }

        let (add, response) = ui.allocate_exact_size(vec2(RAIL_WIDTH, 60.0), Sense::click());
        ui.painter().text(
            add.center(),
            Align2::CENTER_CENTER,
            icon::PLUS_CIRCLE,
            theme::regular(26.0),
            if response.hovered() { TEXT } else { MUTED },
        );
        response.on_hover_text("Criar ou entrar em um servidor");
    }

    fn list(&mut self, ui: &mut Ui) {
        egui::Frame::new()
            .fill(RAISED)
            .corner_radius(16.0)
            .inner_margin(Margin::symmetric(14, 11))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new(icon::MAGNIFYING_GLASS)
                            .font(theme::regular(19.0))
                            .color(MUTED),
                    );
                    ui.add(
                        egui::TextEdit::singleline(&mut self.search)
                            .frame(egui::Frame::NONE)
                            .hint_text("Buscar")
                            .font(theme::regular(15.0))
                            .text_color(TEXT)
                            .desired_width(f32::INFINITY),
                    );
                });
            });
        ui.add_space(14.0);

        let query = self.search.to_lowercase();
        egui::ScrollArea::vertical()
            .id_salt(("list", self.space))
            .auto_shrink(false)
            .show(ui, |ui| {
                ui.spacing_mut().item_spacing.y = 4.0;
                for index in 0..self.data.spaces[self.space].conversations.len() {
                    let conversation = &self.data.spaces[self.space].conversations[index];
                    if !conversation.name.to_lowercase().contains(&query) {
                        continue;
                    }
                    let selected = index == self.conversation;
                    if conversation_row(ui, conversation, selected).clicked() {
                        self.conversation = index;
                    }
                }
            });
    }

    fn header(&self, ui: &mut Ui) {
        let conversation = &self.data.spaces[self.space].conversations[self.conversation];
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                let prefix = if conversation.is_channel { "# " } else { "" };
                ui.label(
                    RichText::new(format!("{prefix}{}", conversation.name))
                        .font(theme::semibold(21.0))
                        .color(TEXT),
                );
                ui.label(
                    RichText::new(&conversation.subtitle)
                        .font(theme::regular(13.5))
                        .color(MUTED),
                );
            });
            ui.with_layout(egui::Layout::right_to_left(Align::Center), |ui| {
                ui.spacing_mut().item_spacing.x = 18.0;
                for glyph in [icon::USERS, icon::MAGNIFYING_GLASS] {
                    ui.label(RichText::new(glyph).font(theme::regular(23.0)).color(MUTED));
                }
            });
        });
    }

    fn composer(&mut self, ui: &mut Ui) {
        egui::Frame::new()
            .fill(RAISED)
            .corner_radius(18.0)
            .inner_margin(Margin::symmetric(16, 12))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new(icon::PAPERCLIP)
                            .font(theme::regular(21.0))
                            .color(MUTED),
                    );
                    let edit = ui.add(
                        egui::TextEdit::singleline(&mut self.draft)
                            .frame(egui::Frame::NONE)
                            .hint_text("Escreva uma mensagem")
                            .font(theme::regular(15.0))
                            .text_color(TEXT)
                            .desired_width(ui.available_width() - 40.0),
                    );
                    let (send, response) =
                        ui.allocate_exact_size(Vec2::splat(30.0), Sense::click());
                    ui.painter().circle_filled(send.center(), 15.0, ACCENT);
                    ui.painter().text(
                        send.center(),
                        Align2::CENTER_CENTER,
                        icon::PAPER_PLANE_RIGHT,
                        theme::regular(16.0),
                        Color32::WHITE,
                    );
                    let enter = edit.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
                    if (enter || response.clicked()) && !self.draft.trim().is_empty() {
                        let body = std::mem::take(&mut self.draft);
                        self.data.spaces[self.space].conversations[self.conversation]
                            .messages
                            .push(Message {
                                author: mock::ME,
                                body,
                                time: "agora".to_owned(),
                            });
                    }
                    if enter {
                        edit.request_focus();
                    }
                });
            });
    }

    fn messages(&mut self, ui: &mut Ui) {
        let wrap = (ui.available_width() - AVATAR - AVATAR_GAP - TIME_WIDTH - BUBBLE_X * 2.0)
            .clamp(120.0, MAX_BUBBLE_TEXT);
        let wrap = (wrap / WRAP_STEP).floor() * WRAP_STEP;
        let messages = &self.data.spaces[self.space].conversations[self.conversation].messages;
        let key = (self.space, self.conversation, messages.len(), wrap as u32);
        if self.rows.key != key || self.rows.tops.is_empty() {
            self.rows.key = key;
            self.rows.tops = measure(ui, messages, wrap);
        }
        let tops = &self.rows.tops;
        let people = &self.data.people;
        let total = *tops.last().unwrap_or(&0.0);
        let bench = self
            .bench
            .as_ref()
            .map(|b| b.started.elapsed().as_secs_f32());

        egui::ScrollArea::vertical()
            .id_salt(("messages", self.space, self.conversation))
            .auto_shrink(false)
            .stick_to_bottom(true)
            .show_viewport(ui, |ui, viewport| {
                ui.set_height(total);
                let origin = ui.max_rect().min;

                if let Some(seconds) = bench {
                    let span = (total - viewport.height()).max(0.0);
                    let phase = (seconds / BENCH_SECONDS).fract();
                    let target = Rect::from_min_size(
                        origin + vec2(0.0, span * phase),
                        vec2(1.0, viewport.height()),
                    );
                    ui.scroll_to_rect(target, Some(Align::TOP));
                }

                let first = tops
                    .partition_point(|&top| top < viewport.top())
                    .saturating_sub(1);
                let last = tops
                    .partition_point(|&top| top < viewport.bottom())
                    .min(messages.len());
                for index in first..last {
                    let message = &messages[index];
                    let opens = opens_group(messages, index);
                    let gap = if opens { GROUP_GAP } else { ROW_GAP };
                    let top = origin.y + tops[index] + gap;
                    let bottom = origin.y + tops[index + 1];
                    let person = &people[message.author];
                    let x = origin.x;

                    if opens {
                        let rect = Rect::from_min_size(pos2(x, top), Vec2::splat(AVATAR));
                        avatar(ui.painter(), rect, person.color, person.name, 13.0, 17.0);
                    }

                    let galley = ui.painter().layout_job(message_job(&message.body, wrap));
                    let name = opens.then(|| {
                        ui.painter().layout_no_wrap(
                            person.name.to_owned(),
                            theme::semibold(13.5),
                            MUTED,
                        )
                    });
                    let inner = galley
                        .size()
                        .x
                        .max(name.as_ref().map_or(0.0, |n| n.size().x));
                    let left = x + AVATAR + AVATAR_GAP;
                    let bubble = Rect::from_min_max(
                        pos2(left, top),
                        pos2(left + inner + BUBBLE_X * 2.0, bottom),
                    );
                    ui.painter().rect_filled(bubble, 16.0, RAISED);
                    let mut text_top = top + BUBBLE_Y;
                    if let Some(name) = name {
                        ui.painter()
                            .galley(pos2(bubble.left() + BUBBLE_X, text_top), name, MUTED);
                        text_top += NAME_HEIGHT;
                    }
                    let text = Rect::from_min_size(
                        pos2(bubble.left() + BUBBLE_X, text_top),
                        galley.size(),
                    );
                    ui.put(text, egui::Label::new(galley));
                    ui.painter().text(
                        pos2(bubble.right() + 8.0, bubble.bottom() - 4.0),
                        Align2::LEFT_BOTTOM,
                        &message.time,
                        theme::regular(11.5),
                        MUTED,
                    );
                }
            });
    }

    fn bench_frame(&mut self, ctx: &egui::Context) {
        let Some(bench) = &mut self.bench else { return };
        bench.frames += 1;
        let elapsed = bench.started.elapsed().as_secs_f32();
        ctx.request_repaint();
        if elapsed >= BENCH_SECONDS {
            let fps = bench.frames as f32 / elapsed;
            let _ = std::fs::write(&bench.output, format!("{fps:.0} fps\n"));
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
    }
}

impl eframe::App for App {
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        egui::Rgba::from(RAIL).to_array()
    }

    fn ui(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
        ui.spacing_mut().item_spacing = vec2(8.0, 6.0);
        ui.style_mut().visuals.selection.bg_fill = ACCENT.linear_multiply(0.5);
        ui.style_mut().visuals.text_cursor.stroke.color = ACCENT;

        egui::Panel::left("rail")
            .exact_size(RAIL_WIDTH)
            .resizable(false)
            .show_separator_line(false)
            .frame(egui::Frame::new().fill(RAIL))
            .show(ui, |ui| {
                ui.spacing_mut().item_spacing.y = 4.0;
                self.rail(ui);
            });
        egui::Panel::left("list")
            .exact_size(LIST_WIDTH)
            .resizable(false)
            .show_separator_line(false)
            .frame(
                egui::Frame::new()
                    .fill(SURFACE)
                    .corner_radius(CornerRadius {
                        nw: 26,
                        sw: 26,
                        ne: 0,
                        se: 0,
                    })
                    .inner_margin(Margin {
                        left: 16,
                        right: 12,
                        top: 24,
                        bottom: 12,
                    }),
            )
            .show(ui, |ui| self.list(ui));
        egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(SURFACE).inner_margin(Margin {
                left: 20,
                right: 28,
                top: 22,
                bottom: 20,
            }))
            .show(ui, |ui| {
                self.header(ui);
                ui.add_space(6.0);
                egui::Panel::bottom("composer")
                    .show_separator_line(false)
                    .resizable(false)
                    .frame(egui::Frame::new().inner_margin(Margin {
                        left: 0,
                        right: 0,
                        top: 14,
                        bottom: 0,
                    }))
                    .show(ui, |ui| self.composer(ui));
                egui::CentralPanel::default()
                    .frame(egui::Frame::NONE)
                    .show(ui, |ui| self.messages(ui));
            });

        let ctx = ui.ctx().clone();
        self.bench_frame(&ctx);
    }
}

fn opens_group(messages: &[Message], index: usize) -> bool {
    index == 0 || messages[index - 1].author != messages[index].author
}

fn measure(ui: &Ui, messages: &[Message], wrap: f32) -> Vec<f32> {
    let mut tops = Vec::with_capacity(messages.len() + 1);
    let mut y = 0.0;
    tops.push(y);
    for (index, message) in messages.iter().enumerate() {
        let opens = opens_group(messages, index);
        let text = ui
            .painter()
            .layout_job(message_job(&message.body, wrap))
            .size()
            .y;
        y += if opens {
            GROUP_GAP + NAME_HEIGHT
        } else {
            ROW_GAP
        } + text
            + BUBBLE_Y * 2.0;
        tops.push(y);
    }
    tops
}

fn message_job(body: &str, wrap: f32) -> LayoutJob {
    let plain = TextFormat::simple(theme::regular(15.0), TEXT);
    let mention = TextFormat::simple(theme::regular(15.0), MENTION);
    let mut job = LayoutJob::default();
    job.wrap.max_width = wrap;
    let mut rest = body;
    while let Some(start) = rest.find('@') {
        let end = rest[start + 1..]
            .find(|c: char| !c.is_alphanumeric())
            .map_or(rest.len(), |offset| start + 1 + offset);
        job.append(&rest[..start], 0.0, plain.clone());
        job.append(&rest[start..end], 0.0, mention.clone());
        rest = &rest[end..];
    }
    job.append(rest, 0.0, plain);
    job
}

fn one_line(ui: &Ui, text: &str, font: FontId, color: Color32, width: f32) -> Arc<Galley> {
    let mut job = LayoutJob::simple_singleline(text.to_owned(), font, color);
    job.wrap.max_width = width;
    job.wrap.max_rows = 1;
    job.wrap.break_anywhere = true;
    ui.painter().layout_job(job)
}

fn conversation_row(
    ui: &mut Ui,
    conversation: &mock::Conversation,
    selected: bool,
) -> egui::Response {
    let width = ui.available_width();
    let (rect, response) = ui.allocate_exact_size(vec2(width, 78.0), Sense::click());
    if !ui.is_rect_visible(rect) {
        return response;
    }
    let painter = ui.painter();
    if selected || response.hovered() {
        painter.rect_filled(rect, 20.0, if selected { RAISED } else { HOVER });
    }
    let tile = Rect::from_min_size(rect.min + vec2(10.0, 12.0), Vec2::splat(54.0));
    let label = if conversation.is_channel {
        "#"
    } else {
        conversation.name.as_str()
    };
    avatar(painter, tile, conversation.color, label, 15.0, 21.0);

    let left = tile.right() + 12.0;
    let time = painter.layout_no_wrap(conversation.time.to_owned(), theme::regular(12.0), MUTED);
    let time_pos = pos2(rect.right() - 14.0 - time.size().x, rect.top() + 16.0);
    let name = one_line(
        ui,
        &conversation.name,
        theme::semibold(15.5),
        TEXT,
        time_pos.x - left - 8.0,
    );
    let right_slot = if conversation.unread > 0 { 44.0 } else { 14.0 };
    let last = conversation.messages.last().map_or("", |m| m.body.as_str());
    let preview = one_line(
        ui,
        last,
        theme::regular(13.5),
        MUTED,
        rect.right() - right_slot - left,
    );
    let painter = ui.painter();
    painter.galley(time_pos, time, MUTED);
    painter.galley(pos2(left, rect.top() + 14.0), name, TEXT);
    painter.galley(pos2(left, rect.top() + 42.0), preview, MUTED);
    if conversation.unread > 0 {
        badge(
            painter,
            pos2(rect.right() - 25.0, rect.top() + 51.0),
            conversation.unread,
        );
    }
    response
}

fn avatar(painter: &Painter, rect: Rect, color: Color32, name: &str, radius: f32, size: f32) {
    painter.rect_filled(rect, radius, color);
    let initial: String = name.chars().take(1).flat_map(char::to_uppercase).collect();
    painter.text(
        rect.center(),
        Align2::CENTER_CENTER,
        initial,
        theme::semibold(size),
        Color32::from_rgb(0x1B, 0x1D, 0x24),
    );
}

fn badge(painter: &Painter, center: Pos2, count: u32) {
    painter.circle_filled(center, 11.0, ACCENT);
    painter.text(
        center,
        Align2::CENTER_CENTER,
        count.to_string(),
        theme::semibold(11.5),
        Color32::WHITE,
    );
}
