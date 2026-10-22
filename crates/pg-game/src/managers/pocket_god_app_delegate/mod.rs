//! `PocketGodAppDelegate` (superclass `NSObject`).

#![allow(unused_variables, unused_imports)]

use crate::prelude::*;

pub mod app_delegate;
pub use app_delegate::*;

/// `PocketGodAppDelegate` ivars.
#[derive(Component, Clone, Debug, Default)]
pub struct PocketGodAppDelegate {
    /// `mPocketGodViewController`: `@"PocketGodViewController"`
    pub pocket_god_view_controller: Option<Entity>,

    /// `mImageView`: `@"UIImageView"`
    pub image_view: Option<Entity>,

    /// `mbStartupComplete`: `B`
    pub startup_complete: bool,

    /// `mbDashboardOpen`: `B`
    pub dashboard_open: bool,

    /// `mbModalViewOpen`: `B`
    pub modal_view_open: bool,

    /// `window`: `@"UIWindow"`
    pub window: Option<Entity>,
}

pub const POCKET_GOD_APP_DELEGATE: Class = Class {
    name: "PocketGodAppDelegate",
    superclass: None,
    ..Class::NS_OBJECT
};
