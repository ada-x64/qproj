use bevy::{ecs::query::QueryData, input::mouse::MouseScrollUnit, text::LineHeight};

use crate::prelude::*;

// The font propogation / char width calculation is not ideal.
// This incurs a frame-delay between changing the font and rewrapping.
// denote a frame change with ';', successive events with '->', and concurrent events with ','
// update_font -> resize ; update_char_width -> resize
// This happens because the TerminalCharWidth entity's ComputedNode is updated the frame _after_ updating
// the text element. Cannot manually trigger a rerender here.
// We _could_ make the parent node invisible, then override the inserted text span's Visibility components.
//

/// Propogates font changes from the TerminalWindow to the targeted TerminalCharWidth entity.
pub fn update_font(
    q_font: Query<
        (
            &TextFont,
            &TextColor,
            &LineHeight,
            &TerminalCharWidthTarget,
            &Children,
        ),
        Changed<TextFont>,
    >,
    q_cw: Query<Entity, With<TerminalCharWidth>>,
    q_wrapper: Query<Entity, With<TerminalTextWrapper>>,
    mut commands: Commands,
) {
    for (font, color, lineheight, target, children) in q_font {
        let entity = c!(q_cw.get(target.target()));
        commands.entity(entity).insert(font.clone());
        let wrapper = c!(q_wrapper.iter().find(|c| children.contains(c)));
        commands
            .entity(wrapper)
            .insert((*lineheight, font.clone(), *color));
    }
}
/// Measure the character width of the monospace font by using a detached,
/// invisible Node containing a single Text(" ")
pub fn update_char_width(
    q: Query<(Entity, &ComputedNode, &TerminalCharWidth), Changed<ComputedNode>>,
    mut commands: Commands,
) {
    for (entity, node, cw) in q {
        commands
            .entity(entity)
            .insert(TerminalCharWidth::new(cw.target(), node.content_size().x));
    }
}

pub fn resize(
    q_window: Query<
        (
            Entity,
            &ComputedNode,
            &TextFont,
            &LineHeight,
            &TerminalCharWidthTarget,
        ),
        (Changed<ComputedNode>, With<TerminalWindow>),
    >,
    q_width: Query<&TerminalCharWidth>,
    mut commands: Commands,
) {
    trace!("resize");
    for (window_id, node, font, line_height, cw_target) in q_window.iter() {
        let cw = c!(q_width.get(cw_target.entity()));
        let size = node.size();
        let line_height = match line_height {
            LineHeight::Px(px) => *px,
            LineHeight::RelativeToFont(rel) => rel * font.font_size,
        };
        let width = (size.x / cw.value()).floor() as usize;
        let height = (size.y / line_height).floor() as usize;
        commands.entity(window_id).insert(TermWidth(width));
        commands.entity(window_id).insert(TermHeight(height));
        debug!("Got node size: {size:?}");
        debug!("Set new term size to {width} x {height}");
    }
}

#[derive(QueryData, Debug)]
pub struct LayoutQueryData<'a> {
    window_id: Entity,
    term_width: &'a TermWidth,
    term_height: &'a TermHeight,
    layout: &'a TerminalLayout,
    scroll_pos: &'a TerminalScrollPos,
    line_height: &'a LineHeight,
    font: &'a TextFont,
    color: &'a TextColor,
    children: &'a Children,
}

pub fn update_layout(
    q: Query<
        LayoutQueryData,
        (
            Or<(
                Changed<TermHeight>,
                Changed<TermWidth>,
                Changed<TerminalLayout>,
                Changed<TerminalScrollPos>,
            )>,
            With<TerminalWindow>,
        ),
    >,
    q_lines: Query<(&TerminalLine, &Children)>,
    q_vspans: Query<&VirtualTextSpan>,
    q_rows: Query<&TerminalRow>,
    q_wrapper: Query<Entity, With<TerminalTextWrapper>>,
    mut commands: Commands,
) {
    trace!("update layout");
    for data in q {
        let wrapper_id = c!(data.children.iter().find(|c| q_wrapper.contains(*c)));
        let new_children = data
            .layout
            .iter()
            .rev()
            .skip(**data.scroll_pos)
            .take(**data.term_height)
            .filter_map(|id| {
                debug!(?id);
                let row = r!(q_rows.get(id));
                let (line, vspans) = r!(q_lines.get(r!(row.line)));
                let textval = line
                    .chars()
                    .skip(row.offset)
                    .take(**data.term_width)
                    .collect::<String>();
                let ret = vspans
                    .iter()
                    .filter_map(|child| q_vspans.get(child).ok())
                    .enumerate()
                    .fold(vec![], |mut accum, (i, vspan)| {
                        let color = vspan.color.unwrap_or(**data.color);
                        let mut text = textval
                            .chars()
                            .skip(vspan.start)
                            .take(vspan.range)
                            .collect::<String>();

                        if i == vspans.len() - 1 {
                            text += "\n";
                        }
                        let id = commands
                            .spawn((TextSpan(text), TextColor(color), data.font.clone()))
                            .id();

                        if let Some(bg) = vspan.background_color {
                            commands.entity(id).insert(TextBackgroundColor(bg));
                        }
                        accum.push(id);
                        accum
                    });
                debug!(?ret);
                Some(ret)
            })
            .rev()
            .flatten()
            .collect::<Vec<Entity>>();
        commands.entity(wrapper_id).despawn_children();
        commands.entity(wrapper_id).add_children(&new_children);
    }
}

pub(crate) fn scroll_terminal_window(
    trigger: On<Pointer<Scroll>>,
    q: Query<(&LineHeight, &TextFont), With<TerminalWindow>>,
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

#[test]
fn test_text_nodes() {
    let mut app = App::new();
    app.add_plugins(test_harness);
    app.add_step(0, |mut commands: Commands| {
        let term_id = commands.spawn(Terminal).id();
        let term_window_id = commands
            .spawn((
                TextFont {
                    font_size: 10.,
                    ..Default::default()
                },
                Node {
                    width: px(100),
                    height: px(50),
                    ..Default::default()
                },
                TerminalWindow(term_id),
                TerminalScrollPos(10),
            ))
            .id();
        for i in 0..100 {
            commands.write_message(TermMsg::writeln(term_window_id, i.to_string()));
        }
        commands.set_state(Step(1));
    });
    app.add_step(
        1,
        (|mut commands: Commands,
          window: Query<(&Children, &TerminalWindow, &TerminalScrollPos)>,
          terminal: Query<(&TerminalLayout, &TermHeight), With<Terminal>>,
          text_spans: Query<&TextSpan>,
          rows: Query<&TerminalRow>,
          lines: Query<&TerminalLine>| {
            let window = window.single();
            if window.is_err() {
                error!("TerminalWindow with Children not found!");
                commands.write_message(AppExit::error());
                return;
            }
            let (children, window, scroll_pos) = window.unwrap();
            let spans = children
                .iter()
                .filter_map(|child_id| Some(r!(text_spans.get(child_id)).0.clone()))
                .collect::<Vec<_>>();
            let (layout, num_rows) = r!(terminal.get(window.0));
            let mut lines = layout
                .iter()
                .filter_map(|row_id| {
                    let row = r!(rows.get(row_id));
                    let line_id = r!(row.line);
                    let line = r!(lines.get(line_id));
                    Some(line.value.clone())
                })
                .rev()
                .skip(**scroll_pos)
                .take(**num_rows)
                .collect::<Vec<_>>();
            lines.reverse();
            let expected = (85..=89).map(|i| i.to_string()).collect::<Vec<String>>();
            info!(?spans, ?lines, ?expected);
            if spans == lines && spans == expected {
                commands.write_message(AppExit::Success);
            } else {
                commands.write_message(AppExit::error());
            }
        })
        .after(TerminalSystems::PostMutation),
    );
    assert!(app.run().is_success());
}
