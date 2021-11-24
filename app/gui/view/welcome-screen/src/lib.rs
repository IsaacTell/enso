//! WelcomeScreen View.
//!
//! It is opened when the IDE launches without any project or entry point selected. It
//! displays a list of available projects, template cards and "new project" button.

#![warn(missing_docs)]


mod side_menu;
mod template_cards;

use ensogl::prelude::*;

use crate::side_menu::SideMenu;
use crate::template_cards::TemplateCards;

use enso_frp as frp;
use ensogl::application;
use ensogl::application::Application;
use ensogl::display;
use ensogl::display::DomSymbol;
use ensogl::system::web;
use ensogl::system::web::NodeInserter;
use std::rc::Rc;
use wasm_bindgen::closure::Closure;
use web_sys::HtmlDivElement;
use web_sys::MouseEvent;



// =================
// === Constants ===
// =================

/// Padding of Welcome Screen from the left border of the screen. In fractions of screen's width.
const PADDING_LEFT: f32 = 0.15;


// === Classes for CSS styles ===

const TEMPLATES_VIEW_ROOT: &str = "enso-internal-templates-view";
const CONTAINER: &str = "enso-internal-container";
const SIDE_MENU: &str = "enso-internal-side-menu";
const CONTENT: &str = "enso-internal-content";
const CARDS: &str = "enso-internal-cards";
const CARD: &str = "enso-internal-card";
const ROW: &str = "enso-internal-row";
const CARD_SPREADSHEETS: &str = "enso-internal-card-spreadsheets";
const CARD_GEO: &str = "enso-internal-card-geo";
const CARD_VISUALIZE: &str = "enso-internal-card-visualize";



// =============
// === Model ===
// =============


// === CSS Styles ===

static STYLESHEET: &str = include_str!("../style.css");


// === ClickClosure ===

/// Type alias for "on-click" event handlers on buttons.
type ClickClosure = Closure<dyn FnMut(MouseEvent)>;


// === Model ===

/// Model of Welcome Screen that generates HTML DOM elements.
#[derive(Clone, CloneRef, Debug)]
pub struct Model {
    application:    Application,
    logger:         Logger,
    dom:            DomSymbol,
    display_object: display::object::Instance,
    side_menu:      SideMenu,
    template_cards: TemplateCards,
}

impl Model {
    /// Constructor. `frp` is used to set up event handlers on buttons.
    pub fn new(app: &Application) -> Self {
        let application = app.clone_ref();
        let logger = Logger::new("WelcomeScreen");
        let display_object = display::object::Instance::new(&logger);

        let side_menu = SideMenu::new(&logger);
        let template_cards = TemplateCards::new(&logger);
        let dom = Self::create_dom(&logger, &side_menu, &template_cards);
        display_object.add_child(&dom);

        // Use `fullscreen_vis` layer to lock position when panning
        app.display.scene().layers.panel.add_exclusive(&dom);
        app.display.scene().dom.layers.fullscreen_vis.manage(&dom);

        let style = web::create_element("style");
        style.set_inner_html(STYLESHEET);
        dom.append_or_warn(&style, &logger);

        Self { application, logger, dom, display_object, side_menu, template_cards }
    }

    fn create_dom(
        logger: &Logger,
        side_menu: &SideMenu,
        template_cards: &TemplateCards,
    ) -> DomSymbol {
        let root = web::create_div();
        root.set_class_name(TEMPLATES_VIEW_ROOT);
        root.set_id("templates-view");

        let container = Self::create_content_container();
        container.append_or_warn(&side_menu.root_dom, logger);
        container.append_or_warn(&template_cards.root_dom, logger);
        root.append_or_warn(&container, logger);

        DomSymbol::new(&root)
    }

    fn create_content_container() -> HtmlDivElement {
        let container = web::create_div();
        container.set_class_name(CONTAINER);

        container
    }
}



// ===========
// === FRP ===
// ===========

ensogl::define_endpoints! {
    Input {}
    Output {}
}



// ============
// === View ===
// ============

/// View of the Welcome Screen.
#[derive(Clone, CloneRef, Debug)]
pub struct View {
    model: Model,
    frp:   Frp,
}

impl Deref for View {
    type Target = Frp;
    fn deref(&self) -> &Self::Target {
        &self.frp
    }
}

impl View {
    /// Constructor.
    pub fn new(app: &Application) -> Self {
        let frp = Frp::new();
        let model = Model::new(app);
        let network = &frp.network;

        frp::extend! { network
            // === Set view's position to the top-left corner of the viewport. ===

            let shape = app.display.scene().shape().clone_ref();
            position <- shape.map(|scene_size| {
                let padding_left = scene_size.width * PADDING_LEFT;
                let x = -scene_size.width / 2.0 + padding_left;
                let y =  scene_size.height / 2.0;
                Vector2(x, y)
            });
            eval position ((pos) model.display_object.set_position_xy(*pos));
        }

        Self { model, frp }
    }
}

impl display::Object for View {
    fn display_object(&self) -> &display::object::Instance {
        &self.model.display_object
    }
}

impl application::command::FrpNetworkProvider for View {
    fn network(&self) -> &frp::Network {
        &self.frp.network
    }
}

impl application::View for View {
    fn label() -> &'static str {
        "WelcomeScreen"
    }

    fn new(app: &Application) -> Self {
        Self::new(app)
    }

    fn app(&self) -> &Application {
        &self.model.application
    }
}
