mod state;

use crate::prelude::*;

use crate::executor::global::spawn_stream_handler;

use enso_frp as frp;
use ide_view as view;
use ide_view::graph_editor::component::node as node_view;
use ide_view::graph_editor::EdgeEndpoint;


type Logger = enso_logger::DefaultTraceLogger;


// ===============
// === Aliases ===
// ===============

type ViewNodeId = view::graph_editor::NodeId;
type AstNodeId = ast::Id;
type ViewConnection = view::graph_editor::EdgeId;
type AstConnection = controller::graph::Connection;



// =============
// === Model ===
// =============

#[derive(Clone, Debug)]
struct Model {
    logger:     Logger,
    controller: controller::ExecutedGraph,
    view:       view::graph_editor::GraphEditor,
    state:      Rc<state::State>,
}

impl Model {
    pub fn new(
        controller: controller::ExecutedGraph,
        view: view::graph_editor::GraphEditor,
    ) -> Self {
        let logger = Logger::new("presenter::Graph");
        let state = default();
        Self { logger, controller, view, state }
    }

    fn node_position_changed(&self, id: ViewNodeId, position: Vector2) {
        self.update_ast(
            || {
                let ast_id = self.state.node_position_changed(id, position)?;
                Some(self.controller.graph().set_node_position(ast_id, position))
            },
            "update node position",
        );
    }

    fn node_removed(&self, id: ViewNodeId) {
        self.update_ast(
            || {
                let ast_id = self.state.node_removed(id)?;
                Some(self.controller.graph().remove_node(ast_id))
            },
            "remove node",
        )
    }

    fn new_connection_created(&self, id: ViewConnection) {
        self.update_ast(
            || {
                let connection = self.view.model.edges.get_cloned_ref(&id)?;
                let ast_to_create = self.state.assign_connection_view(connection)?;
                Some(self.controller.connect(&ast_to_create))
            },
            "create connection",
        );
    }

    fn connection_removed(&self, id: ViewConnection) {
        self.update_ast(
            || {
                let ast_to_remove = self.state.connection_removed(id)?;
                Some(self.controller.disconnect(&ast_to_remove))
            },
            "delete connection",
        );
    }

    fn update_ast<F>(&self, f: F, action: &str)
    where F: FnOnce() -> Option<FallibleResult> {
        if let Some(Err(err)) = f() {
            error!(self.logger, "Failed to {action} in AST: {err}");
        }
    }

    fn all_types_of_node(
        &self,
        node: ViewNodeId,
    ) -> Vec<(ViewNodeId, ast::Id, Option<view::graph_editor::Type>)> {
        let subexpressions = self.state.subexpressions_of_node(node);
        subexpressions
            .iter()
            .map(|id| {
                let a_type = self.expression_type(*id);
                self.state.refresh_expression_type(*id, a_type.clone());
                (node, *id, a_type)
            })
            .collect()
    }

    fn all_method_pointers_of_node(
        &self,
        node: ViewNodeId,
    ) -> Vec<(ast::Id, Option<view::graph_editor::MethodPointer>)> {
        let subexpressions = self.state.subexpressions_of_node(node);
        subexpressions.iter().filter_map(|id| self.refresh_expression_method_pointer(*id)).collect()
    }

    fn refresh_expression_type(
        &self,
        id: ast::Id,
    ) -> Option<(ViewNodeId, ast::Id, Option<view::graph_editor::Type>)> {
        let a_type = self.expression_type(id);
        let node_view = self.state.refresh_expression_type(id, a_type.clone())?;
        Some((node_view, id, a_type))
    }

    fn refresh_expression_method_pointer(
        &self,
        id: ast::Id,
    ) -> Option<(ast::Id, Option<view::graph_editor::MethodPointer>)> {
        let method_pointer = self.expression_method(id);
        self.state.refresh_expression_method_pointer(id, method_pointer.clone())?;
        Some((id, method_pointer))
    }

    fn expression_type(&self, id: ast::Id) -> Option<view::graph_editor::Type> {
        let registry = self.controller.computed_value_info_registry();
        let info = registry.get(&id)?;
        Some(view::graph_editor::Type(info.typename.as_ref()?.clone_ref()))
    }

    fn expression_method(&self, id: ast::Id) -> Option<view::graph_editor::MethodPointer> {
        let registry = self.controller.computed_value_info_registry();
        let method_id = registry.get(&id)?.method_call?;
        let suggestion_db = self.controller.graph().suggestion_db.clone_ref();
        let method = suggestion_db.lookup_method_ptr(method_id).ok()?;
        Some(view::graph_editor::MethodPointer(Rc::new(method)))
    }
}


#[derive(Clone, Debug, Default)]
struct ViewUpdate {
    state:       Rc<state::State>,
    nodes:       Vec<controller::graph::Node>,
    trees:       HashMap<AstNodeId, controller::graph::NodeTrees>,
    connections: HashSet<AstConnection>,
}

impl ViewUpdate {
    fn new(model: &Model) -> FallibleResult<Self> {
        let displayed = model.state.clone_ref();
        let nodes = model.controller.graph().nodes()?;
        let connections_and_trees = model.controller.connections()?;
        let connections = connections_and_trees.connections.into_iter().collect();
        let trees = connections_and_trees.trees;
        Ok(Self { state: displayed, nodes, trees, connections })
    }

    fn nodes_to_remove(&self) -> Vec<ViewNodeId> {
        self.state.retain_nodes(&self.node_ids().collect())
    }

    fn nodes_to_add(&self) -> usize {
        self.node_ids().filter(|n| self.state.view_id_of_ast_node(*n).is_none()).count()
    }

    fn expressions_to_set(&self) -> Vec<(ViewNodeId, node_view::Expression)> {
        self.nodes
            .iter()
            .filter_map(|node| {
                let id = node.main_line.id();
                let trees = self.trees.get(&id).cloned().unwrap_or_default();
                self.state.refresh_node_expression(node, trees)
            })
            .collect()
    }

    fn positions_to_set(&self) -> Vec<(ViewNodeId, Vector2)> {
        self.nodes
            .iter()
            .filter_map(|node| {
                let id = node.main_line.id();
                let position = node.position()?.vector;
                let view_id = self.state.refresh_node_position(id, position)?;
                Some((view_id, position))
            })
            .collect()
    }

    fn connections_to_remove(&self) -> Vec<ViewConnection> {
        self.state.retain_connections(&self.connections)
    }

    fn connections_to_add(&self) -> Vec<(EdgeEndpoint, EdgeEndpoint)> {
        DEBUG!("connections_to_add: {self.connections:?}");
        let ast_conns = self.connections.iter();
        ast_conns
            .filter_map(|connection| self.state.refresh_connection(connection.clone()))
            .collect()
    }

    fn node_ids<'a>(&'a self) -> impl Iterator<Item = AstNodeId> + 'a {
        self.nodes.iter().map(controller::graph::Node::id)
    }
}

#[derive(Debug)]
pub struct Graph {
    network: frp::Network,
    model:   Rc<Model>,
}

impl Graph {
    pub fn new(
        controller: controller::ExecutedGraph,
        view: view::graph_editor::GraphEditor,
    ) -> Self {
        let network = frp::Network::new("presenter::Graph");
        let model = Rc::new(Model::new(controller, view));
        Self { network, model }.init()
    }

    pub fn init(self) -> Self {
        let logger = &self.model.logger;
        let network = &self.network;
        let model = &self.model;
        let view = &self.model.view.frp;
        frp::extend! { network
            update_view <- source::<()>();
            update_data <- update_view.map(
                f_!([logger,model] match ViewUpdate::new(&*model) {
                    Ok(update) => Rc::new(update),
                    Err(err) => {
                        error!(logger,"Failed to update view: {err:?}");
                        Rc::new(default())
                    }
                })
            );


            // === Refreshing Nodes ===

            remove_node <= update_data.map(|update| update.nodes_to_remove());
            update_node_expression <= update_data.map(|update| update.expressions_to_set());
            set_node_position <= update_data.map(|update| update.positions_to_set());
            view.remove_node <+ remove_node;
            view.set_node_expression <+ update_node_expression;
            view.set_node_position <+ set_node_position;

            view.add_node <+ update_data.map(|update| update.nodes_to_add()).repeat();
            added_node_update <- view.node_added.filter_map(f!((view_id)
                model.state.assign_newly_created_node(*view_id)
            ));
            init_node_expression <- added_node_update.filter_map(|update| Some((update.view_id?, update.expression.clone())));
            view.set_node_expression <+ init_node_expression;
            view.set_node_position <+ added_node_update.filter_map(|update| Some((update.view_id?, update.position)));


            // === Refreshing Connections ===

            remove_connection <= update_data.map(|update| update.connections_to_remove());
            add_connection <= update_data.map(|update| update.connections_to_add());
            view.remove_edge <+ remove_connection;
            view.connect_nodes <+ add_connection;


            // === Refreshing Expressions ===

            reset_node_types <- any(update_node_expression, init_node_expression)._0();
            set_expression_type <= reset_node_types.map(f!((view_id) model.all_types_of_node(*view_id)));
            set_method_pointer <= reset_node_types.map(f!((view_id) model.all_method_pointers_of_node(*view_id)));
            view.set_expression_usage_type <+ set_expression_type;
            view.set_method_pointer <+ set_method_pointer;

            update_expressions <- source::<Vec<ast::Id>>();
            update_expression <= update_expressions;
            view.set_expression_usage_type <+ update_expression.filter_map(f!((id) model.refresh_expression_type(*id)));
            view.set_method_pointer <+ update_expression.filter_map(f!((id) model.refresh_expression_method_pointer(*id)));


            // === Changes from the View ===

            eval view.node_position_set_batched(((node_id, position)) model.node_position_changed(*node_id, *position));
            eval view.node_removed((node_id) model.node_removed(*node_id));
            eval view.on_edge_endpoints_set((edge_id) model.new_connection_created(*edge_id));
            eval view.on_edge_endpoint_unset(((edge_id,_)) model.connection_removed(*edge_id));
        }

        update_view.emit(());
        self.setup_controller_notification_handlers(update_view, update_expressions);

        self
    }

    fn setup_controller_notification_handlers(
        &self,
        update_view: frp::Source<()>,
        update_expressions: frp::Source<Vec<ast::Id>>,
    ) {
        use crate::controller::graph::executed;
        use crate::controller::graph::Notification;
        let graph_notifications = self.model.controller.subscribe();
        self.spawn_sync_stream_handler(graph_notifications, move |notification, model| {
            info!(model.logger, "Received controller notification {notification:?}");
            match notification {
                executed::Notification::Graph(graph) => match graph {
                    Notification::Invalidate => update_view.emit(()),
                    Notification::PortsUpdate => update_view.emit(()),
                },
                executed::Notification::ComputedValueInfo(expressions) =>
                    update_expressions.emit(expressions),
                executed::Notification::EnteredNode(_) => {}
                executed::Notification::SteppedOutOfNode(_) => {}
            }
        })
    }

    fn spawn_sync_stream_handler<Stream, Function>(&self, stream: Stream, handler: Function)
    where
        Stream: StreamExt + Unpin + 'static,
        Function: Fn(Stream::Item, Rc<Model>) + 'static, {
        let model = Rc::downgrade(&self.model);
        spawn_stream_handler(model, stream, move |item, model| {
            handler(item, model);
            futures::future::ready(())
        })
    }
}
