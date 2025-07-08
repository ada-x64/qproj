// 𝒒𝒑𝒓𝒐𝒋 -- copyright (c) the contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

pub trait TextUtils {
    /// Length is the cutoff point. Tolerance specifies how many characters we
    /// can overshoot.
    fn wrap_text(&mut self, length: usize, tolernace: usize) -> &mut Self;
}
impl TextUtils for String {
    fn wrap_text(&mut self, length: usize, tolerance: usize) -> &mut Self {
        let mut last_whitespace = 0;
        let mut last_newline = 0;
        let whitespace = [' ', '\t', '\r'];
        for (i, c) in self.to_string().chars().enumerate() {
            let overshoot = (i - last_newline) as isize - length as isize;
            #[cfg(test)]
            println!(
                "'{}'\n~~> ({i},{c}); len={}; {};",
                &self[last_newline..i],
                i - last_newline,
                if overshoot > 0 {
                    format!(">({overshoot})")
                } else {
                    String::new()
                }
            );
            if c == '\n' {
                last_newline = i;
                last_whitespace = i;
                continue;
            }
            if whitespace.contains(&c) {
                last_whitespace = i;
            }
            if i - last_newline > length {
                if last_whitespace == i {
                    self.replace_range(i..=i, "\n");
                    last_newline = i;
                } else if overshoot > tolerance as isize {
                    self.replace_range(last_whitespace..=last_whitespace, "\n");
                    last_newline = last_whitespace;
                }
            }
        }
        self
    }
}

#[test]
fn test_wrap_text() {
    let mut text = (0..100).map(|_| "a ").collect::<String>();
    println!("{}", text.wrap_text(80, 5));
    let text = "type `bevy_platform::time::Instant` did not register the `ReflectSerialize` or `ReflectSerializeWithRegistry` type data. For certain types, this may need to be registered manually using `register_type_data` (stack: `bevy_time::time::Time<bevy_time::real::Real>` -> `bevy_time::real::Real` -> `bevy_platform::time::Instant`)";
    println!("{}", text.to_string().wrap_text(80, 5));
}
