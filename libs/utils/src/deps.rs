// 𝒒𝒑𝒓𝒐𝒋 -- copyright (c) the contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

#[macro_export]
macro_rules! as_expr {
    ($e:expr) => {
        $e
    };
}

/// Adds an optional plugin.
#[macro_export]
macro_rules! optional_plugin {
    ($plugin:ident) => {
        if !app.is_plugin_added::<$plugin>() {
            app.add_plugins(as_expr!($plugin));
        }
    };
    ($plugin:ty) => {
        if !app.is_plugin_added::<$plugin>() {
            app.add_plugins(as_expr!($plugin));
        }
    };
    ($plugin:ty, $init:expr) => {
        if !app.is_plugin_added::<$plugin>() {
            app.add_plugins($init);
        }
    };
}

/// Adds plugins if they do not already exist. Useful for adding dependencies to
/// a plugin. Inputs:
/// ### Syntax
/// ```rust, ignore
/// plugin_deps!(app,
///     (
///         // Empty / marker struct
///         Plugin1,
///         // Plugins with initialization expressions
///         (Plugin2, Plugin2 {...})
///         (Plugin3, Plugin3::default())
///         // ...etc
///     )
/// );
/// ```
#[macro_export]
macro_rules! plugin_deps {
    ($app:expr, $($items:tt)*) => {
        $crate::plugin_deps_inner!($app, $($items)*)
    };
}

#[macro_export]
macro_rules! plugin_deps_inner {
    // Base case: empty
    ($app:expr,) => {};

    // Handle simple plugin (identifier)
    ($app:expr, $plugin:ident $(, $($rest:tt)*)?) => {
        if !$app.is_plugin_added::<$plugin>() {
            $app.add_plugins($crate::as_expr!($plugin));
        }
        $(
            $crate::plugin_deps_inner!($app, $($rest)*);
        )?
    };

    // Handle plugin with initialization expression
    ($app:expr, ($plugin:ty, $init:expr) $(, $($rest:tt)*)?) => {
        if !$app.is_plugin_added::<$plugin>() {
            $app.add_plugins($crate::as_expr!($init));
        }
        $(
            $crate::plugin_deps_inner!($app, $($rest)*);
        )?
    };
}
