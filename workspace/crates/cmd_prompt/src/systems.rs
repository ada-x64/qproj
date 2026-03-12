use bevy::{input::mouse::MouseScrollUnit, text::LineHeight};
use itertools::Itertools;

use crate::prelude::*;

// The font propogation / char width calculation is not ideal.
// This incurs a frame-delay between changing the font and rewrapping.
// denote a frame change with ';', successive events with '->', and concurrent events with ','
// update_font -> resize ; update_char_width -> resize
// This happens because the TerminalCharWidth entity's ComputedNode is updated the frame _after_ updating
// the text element. Cannot manually trigger a rerender here.
// We _could_ make the parent node invisible, then override the inserted text span's Visibility components.
//

/// Propogates font changes from the [`VtUi`] to the targeted TerminalCharWidth entity.
pub fn update_font(
    q_font: Query<(&TextFont, &VtCharWidthTarget), Changed<TextFont>>,
    q_cw: Query<Entity, With<VtCharWidth>>,
    mut commands: Commands,
) {
    trace!("update_font");
    for (font, target) in q_font {
        let cw_id = c!(q_cw.get(target.target()));
        commands.entity(cw_id).insert(font.clone());
    }
}

/// Measure the character width of the monospace font by using a detached,
/// invisible Node containing a single Text(" ")
pub fn update_char_width(
    q: Query<(Entity, &ComputedNode, &VtCharWidth), Changed<ComputedNode>>,
    mut commands: Commands,
) {
    trace!("update_char_width");
    for (entity, node, cw) in q {
        commands
            .entity(entity)
            .insert(VtCharWidth::new(cw.target(), node.content_size().x));
    }
}

pub fn resize(
    q_ui: Query<
        (
            &VtUi,
            &ComputedNode,
            &TextFont,
            &LineHeight,
            &VtCharWidthTarget,
        ),
        Changed<ComputedNode>,
    >,
    q_width: Query<&VtCharWidth>,
    mut commands: Commands,
) {
    trace!("resize");
    for (vt_ui, node, font, line_height, cw_target) in q_ui.iter() {
        let cw = c!(q_width.get(cw_target.entity()));
        let size = node.size();
        let line_height = match line_height {
            LineHeight::Px(px) => *px,
            LineHeight::RelativeToFont(rel) => rel * font.font_size,
        };
        let cols = (size.x / cw.value()).floor() as usize;
        let rows = (size.y / line_height).floor() as usize;
        commands
            .entity(vt_ui.target())
            .insert(VtSize { cols, rows });
        debug!("Got node size: {size:?}");
        debug!("Set new term size to {cols} x {rows}");
    }
}

#[derive(Debug, Bundle, PartialEq, Clone, Copy)]
struct TextSpanStyleBundle {
    color: TextColor,
    bg: TextBackgroundColor,
}
impl Default for TextSpanStyleBundle {
    fn default() -> Self {
        Self {
            color: TextColor(Color::WHITE),
            bg: TextBackgroundColor(Color::BLACK),
        }
    }
}

/// Given a row and its corresponding lines, spawn the text spans.
fn generate_textspan_ui(
    terminfo: &TermInfoItem,
    ui_id: Entity,
    row_and_line: Option<(&VtRow, &VtLine)>,
) -> Vec<(TextSpan, TextSpanStyleBundle, ChildOf)> {
    trace!("spawn_row_ui");
    if let Some((row, line)) = row_and_line {
        let mut spans = line
            .cells()
            .iter()
            .skip(row.offset)
            .take(terminfo.size.cols)
            .copied()
            .pad_using(terminfo.size.cols, |_| VtCell::default())
            .fold(
                Vec::<(TextSpan, TextSpanStyleBundle, ChildOf)>::new(),
                |mut spans, cell| {
                    let color = TextColor(cell.style.color);
                    let bg = TextBackgroundColor(cell.style.background);
                    let style_bundle = TextSpanStyleBundle { color, bg };
                    let new = (TextSpan::new(cell.value), style_bundle, ChildOf(ui_id));
                    if let Some(last) = spans.last_mut() {
                        if last.1 != style_bundle {
                            spans.push(new);
                        } else {
                            last.0.0.push(cell.value);
                        }
                    } else {
                        spans.push(new);
                    };
                    spans
                },
            );
        if let Some((span, _, _)) = spans.last_mut() {
            span.0.push('\n')
        }
        spans
    } else {
        vec![(
            TextSpan::new(" ".repeat(terminfo.size.cols)),
            TextSpanStyleBundle::default(),
            ChildOf(ui_id),
        )]
    }
}

/// Translates from [`VtViewportRow`] entities to the [`VtUi`]-based render.
pub fn update_layout_ui(
    q: Query<
        (TermInfo, &VtUiTarget),
        Or<(Changed<VtSize>, Changed<VtViewport>, Changed<VtScrollPos>)>,
    >,
    q_lines: Query<&VtLine>,
    q_viewport: Query<(&VtViewportRow, Option<Ref<VtRow>>)>,
    mut commands: Commands,
) {
    trace!("update_layout_ui");
    for (terminfo, ui_target) in q {
        let ui_id = ui_target.target();
        commands.entity(ui_id).despawn_children();
        let mut spans = vec![];
        for (_, maybe_row) in q_viewport.iter_many(terminfo.viewport.iter()) {
            let mut new_spans = if let Some(row) = maybe_row {
                let line = c!(q_lines.get(row.line()));
                generate_textspan_ui(&terminfo, ui_id, Some((row.as_ref(), line)))
            } else {
                generate_textspan_ui(&terminfo, ui_id, None)
            };
            spans.append(&mut new_spans);
        }
        commands.spawn_batch(spans);
    }
}

// pub fn update_layout(
//     q: Query<
//         LayoutQueryData,
//         (
//             Or<(Changed<VtSize>, Changed<VtViewport>, Changed<VtScrollPos>)>,
//             With<Terminal>,
//         ),
//     >,
//     q_lines: Query<(&VtLine, &Children)>,
//     q_vspans: Query<&VirtualTextSpan>,
//     q_rows: Query<&VtRow>,
//     q_wrapper: Query<Entity, With<VtTextWrapper>>,
//     mut commands: Commands,
// ) {
//     trace!("update layout");
//     for data in q {
//         let wrapper_id = c!(data.children.iter().find(|c| q_wrapper.contains(*c)));
//         let new_children = data
//             .layout
//             .iter()
//             .rev()
//             .skip(**data.scroll_pos)
//             .take(**data.term_height)
//             .filter_map(|id| {
//                 debug!(?id);
//                 let row = r!(q_rows.get(id));
//                 let (line, vspans) = r!(q_lines.get(r!(row.line)));
//                 let textval = line
//                     .chars()
//                     .skip(row.offset)
//                     .take(**data.term_width)
//                     .collect::<String>();
//                 let ret = vspans
//                     .iter()
//                     .filter_map(|child| q_vspans.get(child).ok())
//                     .enumerate()
//                     .fold(vec![], |mut accum, (i, vspan)| {
//                         let mut text = textval
//                             .chars()
//                             .skip(vspan.offset)
//                             .take(vspan.range)
//                             .collect::<String>();

//                         if i == vspans.len() - 1 {
//                             text += "\n";
//                         }
//                         let id = commands
//                             .spawn((
//                                 TextSpan(text),
//                                 TextColor(vspan.style.color),
//                                 TextBackgroundColor(vspan.style.background),
//                                 data.font.clone(),
//                             ))
//                             .id();
//                         accum.push(id);
//                         accum
//                     });
//                 debug!(?ret);
//                 Some(ret)
//             })
//             .rev()
//             .flatten()
//             .collect::<Vec<Entity>>();
//         commands.entity(wrapper_id).despawn_children();
//         commands.entity(wrapper_id).add_children(&new_children);
//     }
// }

pub(crate) fn on_scroll(
    trigger: On<Pointer<Scroll>>,
    q: Query<(&LineHeight, &TextFont), With<Terminal>>,
    mut commands: Commands,
) {
    debug!(?trigger);
    let (line_height, text_font) = r!(q.get(trigger.entity));
    let delta = match trigger.unit {
        MouseScrollUnit::Line => trigger.y,
        MouseScrollUnit::Pixel => {
            let line_height = match line_height {
                LineHeight::Px(line_height) => *line_height,
                LineHeight::RelativeToFont(rel) => rel * text_font.font_size,
            };
            line_height / trigger.y
        }
    };
    commands.write_message(TermMsg::scroll(trigger.entity, delta as isize));
}

pub(crate) fn scroll_viewport(
    terminfo: Query<TermInfo, Changed<VtScrollPos>>,
    q_viewport: Query<&VtViewportRow>,
    q_rows: Query<&VtRow>,
) {
    // recreate viewport.
    // ideally only get diff from scrollback.
}

// #[test]
// fn test_text_nodes() {
//     let mut app = App::new();
//     app.add_plugins(test_harness);
//     app.add_step(0, |mut commands: Commands| {
//         let term_id = commands
//             .spawn((
//                 Terminal,
//                 TextFont {
//                     font_size: 10.,
//                     ..Default::default()
//                 },
//                 Node {
//                     width: px(100),
//                     height: px(50),
//                     ..Default::default()
//                 },
//                 TerminalScrollPos(10),
//             ))
//             .id();
//         for i in 0..100 {
//             commands.write_message(TermMsg::write(term_id, i.to_string() + "\n"));
//         }
//         commands.set_state(Step(1));
//     });
//     app.add_step(
//         1,
//         (|mut commands: Commands,
//           terminfo: Query<TermInfo>,
//           window: Query<(&Children, &TerminalScrollPos)>,
//           terminal: Query<(&TerminalLayout, &TermHeight), With<Terminal>>,
//           text_spans: Query<&TextSpan>,
//           rows: Query<&TerminalRow>,
//           lines: Query<&TerminalLine>| {
//             let window = window.single();
//             if window.is_err() {
//                 error!("TerminalWindow with Children not found!");
//                 commands.write_message(AppExit::error());
//                 return;
//             }
//             let (children, scroll_pos) = window.unwrap();
//             let spans = children
//                 .iter()
//                 .filter_map(|child_id| Some(r!(text_spans.get(child_id)).0.clone()))
//                 .collect::<Vec<_>>();
//             let mut lines = layout
//                 .iter()
//                 .filter_map(|row_id| {
//                     let row = r!(rows.get(row_id));
//                     let line_id = r!(row.line);
//                     let line = r!(lines.get(line_id));
//                     Some(line.value.clone())
//                 })
//                 .rev()
//                 .skip(**scroll_pos)
//                 .take(**num_rows)
//                 .collect::<Vec<_>>();
//             lines.reverse();
//             let expected = (85..=89).map(|i| i.to_string()).collect::<Vec<String>>();
//             info!(?spans, ?lines, ?expected);
//             if spans == lines && spans == expected {
//                 commands.write_message(AppExit::Success);
//             } else {
//                 commands.write_message(AppExit::error());
//             }
//         })
//         .after(TerminalSystems::PostMutation),
//     );
//     assert!(app.run().is_success());
// }
