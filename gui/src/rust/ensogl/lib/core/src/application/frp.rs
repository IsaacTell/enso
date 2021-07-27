//! FRP utilities for defining application components.

/// Generate a set of structures allowing for nice management of FRP inputs, outputs, and commands.
///
/// Given the definition:
///
/// ```compile_fail
///     define_endpoints! { [GLOBAL_OPTS]
///         Input { [INPUT_OPTS]
///             input1 (f32),
///             input2 (),
///         }
///         Output { [OUTPUT_OPTS]
///             output1 (String),
///             output2 (bool),
///             output3 (),
///         }
///     }
/// ```
///
/// The `INPUT_OPTS`, `OUTPUT_OPTS`, and `GLOBAL_OPTS` are optional and will be passed directly to
/// input, output, or both FRP network definitions, respectively. For example, you can add the
/// `TRACE_ALL` option this way.
///
/// The code presented below will be generated. Please note that additional fields are added
/// automatically. In particular, an output `focused(bool)`, and inputs `focus()`, `defocus()`,
/// and `set_focus(bool)` are always defined and connected. They are mainly used for shortcut
/// manager to send commands only to focused GUI elements.
///
/// ```compile_fail
///     /// Frp network and endpoints.
///     #[derive(Debug, Clone, CloneRef)]
///     #[allow(missing_docs)]
///     pub struct Frp {
///         pub network: frp::Network,
///         pub output: FrpEndpoints,
///     }
///
///     impl Frp {
///         /// Constructor.
///         pub fn new(network: frp::Network, output: FrpEndpoints) -> Self {
///             Self { network, output }
///         }
///
///         /// Create Frp with network, inputs and outputs.
///         pub fn new_network() -> Self {
///             let network = frp::Network::new();
///             let frp_inputs = FrpInputs::new(&network);
///             let frp_endpoints = FrpEndpoints::new(&network, frp_inputs);
///             Self::new(network, frp_endpoints)
///         }
///     }
///
///     impl Deref for Frp {
///         type Target = FrpEndpoints;
///         fn deref(&self) -> &Self::Target {
///             &self.output
///         }
///     }
///
///     /// Frp inputs.
///     #[derive(Debug, Clone, CloneRef)]
///     #[allow(missing_docs)]
///     #[allow(unused_parens)]
///     pub struct FrpInputs {
///         pub focus     : frp::Source<()>,
///         pub defocus   : frp::Source<()>,
///         pub set_focus : frp::Source<bool>,
///         pub input1    : frp::Source<f32>,
///         pub input2    : frp::Source<()>,
///     }
///
///     #[allow(unused_parens)]
///     impl FrpInputs {
///         /// Constructor.
///         pub fn new(network: &frp::Network) -> Self {
///             frp::extend! { network
///                 focus     <- source();
///                 defocus   <- source();
///                 set_focus <- source();
///                 input1    <- source();
///                 input2    <- source ( ) ;
///             }
///             Self {focus,defocus,set_focus,input1,input2}
///         }
///
///         #[allow(missing_docs)]
///         pub fn focus(&self) {
///             self.focus.emit(());
///         }
///
///         #[allow(missing_docs)]
///         pub fn defocus(&self) {
///             self.defocus.emit(());
///         }
///
///         #[allow(missing_docs)]
///         pub fn set_focus(&self, t1: impl IntoParam<bool>) {
///             self.set_focus.emit(t1);
///         }
///
///         #[allow(missing_docs)]
///         pub fn input1(&self, t1: impl IntoParam<f32>) {
///             self.input1.emit(t1);
///         }
///
///         #[allow(missing_docs)]
///         pub fn input2(&self) {
///             self.input2.emit(());
///         }
///     }
///     /// Frp outputs.
///     #[derive(Debug, Clone, CloneRef)]
///     #[allow(missing_docs)]
///     pub struct FrpEndpoints {
///         pub input         : FrpInputs,
///         pub(crate) source : FrpOutputsSource,
///         pub status_map    : Rc<RefCell<HashMap<String,frp::Sampler<bool>>>>,
///         pub command_map   : Rc<RefCell<HashMap<String,Command>>>,
///         pub focused       : frp::Sampler<bool>,
///         pub output1       : frp::Sampler<String>,
///         pub output2       : frp::Sampler<bool>,
///         pub output3       : frp::Sampler<>,
///     }
///
///     impl Deref for FrpEndpoints {
///         type Target = FrpInputs;
///         fn deref(&self) -> &Self::Target {
///             &self.input
///         }
///     }
///
///     impl FrpEndpoints {
///         /// Constructor.
///         pub fn new(network: &frp::Network, input: FrpInputs) -> Self {
///             use ::ensogl_core::application::command::*;
///             let source = FrpOutputsSource::new(network);
///             let mut status_map: HashMap<String, frp::Sampler<bool>> = default();
///             let mut command_map: HashMap<String, Command> = default();
///             frp::extend! { network
///                 focused <- source.focused.sampler();
///                 output1 <- source.output1.sampler();
///                 output2 <- source.output2.sampler();
///                 output3 <- source.output3.sampler();
///                 focus_events <- bool(&input.defocus,&input.focus);
///                 focused      <- any(&input.set_focus,&focus_events);
///                 source.focused <+ focused;
///             }
///
///             status_map.insert("focused".into(), focused.clone_ref());
///             status_map.insert("output2".into(), output2.clone_ref());
///             command_map.insert("focus".into(), Command::new((input.focus).clone_ref()));
///             command_map.insert("defocus".into(), Command::new((input.defocus).clone_ref()));
///             command_map.insert("input2".into(), Command::new((input.input2).clone_ref()));
///             let status_map  = Rc::new(RefCell::new(status_map));
///             let command_map = Rc::new(RefCell::new(command_map));
///             Self {source,input,status_map,command_map,focused,output1,output2,output3}
///         }
///     }
///
///     /// Frp output setters.
///     #[derive(Debug, Clone, CloneRef)]
///     pub(crate) struct FrpOutputsSource {
///         focused : frp::Any<bool>,
///         output1 : frp::Any<String>,
///         output2 : frp::Any<bool>,
///         output3 : frp::Any<>,
///     }
///
///     impl FrpOutputsSource {
///         /// Constructor.
///         pub fn new(network: &frp::Network) -> Self {
///             frp::extend! { network
///                 focused <- any(...);
///                 output1 <- any(...);
///                 output2 <- any(...);
///                 output3 <- any(...);
///             }
///             Self {focused,output1,output2,output3}
///         }
///     }
///
///     impl CommandApi for Frp {
///         fn command_api(&self) -> Rc<RefCell<HashMap<String,Command>>> {
///             self.command_map.clone()
///         }
///
///         fn status_api(&self) -> Rc<RefCell<HashMap<String, frp::Sampler<bool>>>> {
///             self.status_map.clone()
///         }
///     }
/// ```
#[macro_export]
macro_rules! define_endpoints {
    (
        $([$($global_opts:tt)*])?
        $(<$($param:ident),*>)?

        $(Input { $([$($input_opts:tt)*])?
            $($(#[doc=$($in_doc:tt)*])*
            $in_field : ident ($($in_field_type : tt)*)),* $(,)?
        })?

        $(Output { $([$($output_opts:tt)*])?
            $($(#[doc=$($out_doc:tt)*])*
            $out_field : ident ($($out_field_type : tt)*)),* $(,)?
        })?
    ) => {
        $crate::define_endpoints! {
            NORMALIZED
            $([$($global_opts)*])?
            $(<$($param),*>)?

            Input { $($([$($input_opts)*])?)?
                /// Focus the element. Focused elements are meant to receive shortcut events.
                focus(),
                /// Defocus the element. Non-focused elements are meant to be inactive and don't
                /// receive shortcut events.
                defocus(),
                /// Wrapper for `focus` and `defocus`.
                set_focus(bool),
                $($($(#[doc=$($in_doc )*])*
                $in_field ($($in_field_type )*)),*)?
            }

            Output { $($([$($output_opts)*])?)?
                /// Focus state checker.
                focused(bool),
                $($($(#[doc=$($out_doc)*])*
                $out_field ($($out_field_type)*)),*)?
            }
        }
    };

    (
        NORMALIZED
        $([$($global_opts:tt)*])?
        $(<$($param:ident),*>)?

        Input { $([$($input_opts:tt)*])?
            $($(#[doc=$($in_doc :tt)*])*
            $in_field : ident ($($in_field_type : tt)*)),* $(,)?
        }

        Output { $([$($output_opts:tt)*])?
            $($(#[doc=$($out_doc:tt)*])*
            $out_field : ident ($($out_field_type : tt)*)),* $(,)?
        }
    ) => {
        use $crate::frp::IntoParam;

        /// Frp network and endpoints.
        #[derive(Debug,Clone,CloneRef)]
        #[allow(missing_docs)]
        pub struct Frp $(<$($param:Debug+'static),*>)? {
            pub network : $crate::frp::Network,
            pub output  : FrpEndpoints $(<$($param),*>)?,
        }

        impl $(<$($param:Debug+'static),*>)? Frp $(<$($param),*>)? {
            /// Create Frp endpoints within and the associated network.
            pub fn new() -> Self {
                let network = $crate::frp::Network::new(file!());
                let output  = Self::extend(&network);
                Self {network,output}
            }

            /// Create Frp endpoints within the provided network.
            pub fn extend(network:&$crate::frp::Network) -> FrpEndpoints $(<$($param),*>)? {
                let input = FrpInputs::new(network);
                FrpEndpoints::new(network,input)
            }
        }

        impl $(<$($param:Debug+'static),*>)? Default for Frp $(<$($param),*>)? {
            fn default() -> Self {
                Self::new()
            }
        }

        impl $(<$($param:Debug+'static),*>)? Deref for Frp $(<$($param),*>)? {
            type Target = FrpEndpoints $(<$($param),*>)?;
            fn deref(&self) -> &Self::Target {
                &self.output
            }
        }

        /// Frp inputs.
        #[derive(Debug,Clone,CloneRef)]
        #[allow(missing_docs)]
        #[allow(unused_parens)]
        // Clippy thinks `_param` is a field we want to add in future, but it is not: it is to
        // suppress "not used generic param" error.
        #[allow(clippy::manual_non_exhaustive)]
        pub struct FrpInputs $(<$($param:Debug+'static),*>)? {
            $( $(#[doc=$($in_doc)*])* pub $in_field : $crate::frp::Any<($($in_field_type)*)>,)*
            _params : ($($(PhantomData<$param>),*)?),
        }

        #[allow(unused_parens)]
        impl $(<$($param:Debug+'static),*>)? FrpInputs $(<$($param),*>)? {
            /// Constructor.
            pub fn new(network:&$crate::frp::Network) -> Self {
                $crate::frp::extend! { $($($global_opts)*)? $($($input_opts)*)? network
                    $($in_field <- any_mut();)*
                }
                let _params = default();
                Self { $($in_field),*, _params }
            }

            $($crate::define_endpoints_emit_alias!{$in_field ($($in_field_type)*)})*
        }

        /// Frp outputs.
        #[derive(Debug,Clone,CloneRef)]
        #[allow(unused_parens)]
        #[allow(missing_docs)]
        // Clippy thinks `_param` is a field we want to add in future, but it is not: it is to
        // suppress "not used generic param" error.
        #[allow(clippy::manual_non_exhaustive)]
        pub struct FrpEndpoints $(<$($param:Debug+'static),*>)? {
            pub input         : FrpInputs $(<$($param),*>)?,
            // TODO[WD]: Consider making it private and exposing only on-demand with special macro
            //           usage syntax.
            pub(crate) source : FrpOutputsSource $(<$($param),*>)?,
            pub status_map    : Rc<RefCell<HashMap<String,$crate::frp::Sampler<bool>>>>,
            pub command_map   : Rc<RefCell<HashMap<String,$crate::application::command::Command>>>,
            $($(#[doc=$($out_doc)*])*
                pub $out_field  : $crate::frp::Sampler<($($out_field_type)*)>,
            )*
            _params : ($($(PhantomData<$param>),*)?),
        }

        impl $(<$($param:Debug+'static),*>)? Deref for FrpEndpoints $(<$($param),*>)? {
            type Target = FrpInputs $(<$($param),*>)?;
            fn deref(&self) -> &Self::Target {
                &self.input
            }
        }

        impl $(<$($param:Debug+'static),*>)? FrpEndpoints $(<$($param),*>)? {
            /// Constructor.
            pub fn new(network:&$crate::frp::Network, input:FrpInputs $(<$($param),*>)?) -> Self {
                use $crate::application::command::*;
                let source = FrpOutputsSource::new(network);
                let mut status_map  : HashMap<String,$crate::frp::Sampler<bool>> = default();
                let mut command_map : HashMap<String,Command> = default();
                $crate::frp::extend! { $($($global_opts)*)? $($($output_opts)*)? network
                    $($out_field <- source.$out_field.sampler();)*
                    focus_events   <- bool(&input.defocus,&input.focus);
                    focused        <- any(&input.set_focus,&focus_events);
                    source.focused <+ focused;
                }
                $($crate::build_status_map!
                    {status_map $out_field ($($out_field_type)*) $out_field })*
                $($crate::build_command_map!
                    {command_map $in_field ($($in_field_type)*) input.$in_field })*
                let status_map  = Rc::new(RefCell::new(status_map));
                let command_map = Rc::new(RefCell::new(command_map));
                let _params     = default();
                Self {source,input,status_map,command_map,$($out_field),*,_params}
            }
        }

        /// Frp output setters.
        #[derive(Debug,Clone,CloneRef)]
        #[allow(unused_parens)]
        // Clippy thinks `_param` is a field we want to add in future, but it is not: it is to
        // suppress "not used generic param" error.
        #[allow(clippy::manual_non_exhaustive)]
        pub(crate) struct FrpOutputsSource $(<$($param:Debug+'static),*>)? {
            $(pub(crate) $out_field : $crate::frp::Any<($($out_field_type)*)>,)*
            _params : ($($(PhantomData<$param>),*)?),
        }

        impl $(<$($param:Debug+'static),*>)? FrpOutputsSource $(<$($param),*>)? {
            /// Constructor.
            pub fn new(network:&$crate::frp::Network) -> Self {
                $crate::frp::extend! { network
                    $($out_field <- any(...);)*
                }
                let _params = default();
                Self {$($out_field),*,_params}
            }
        }

        impl $(<$($param:Debug+'static),*>)?  $crate::application::command::CommandApi for Frp $(<$($param),*>)?  {
            fn command_api(&self)
            -> Rc<RefCell<HashMap<String,$crate::application::command::Command>>> {
                self.command_map.clone()
            }

            fn status_api(&self) -> Rc<RefCell<HashMap<String,$crate::frp::Sampler<bool>>>> {
                self.status_map.clone()
            }
        }

        impl $(<$($param:Debug+'static),*>)?  $crate::application::command::FrpNetworkProvider for Frp $(<$($param),*>)?  {
            fn network(&self) -> &$crate::frp::Network { &self.network }
        }
    };
}

/// Internal helper of `define_endpoints` macro.
#[macro_export]
macro_rules! build_status_map {
    ($map:ident $field:ident (bool) $frp:expr) => {
        $map.insert(stringify!($field).into(),$frp.clone_ref());
    };
    ($($ts:tt)*) => {}
}

/// Internal helper of `define_endpoints` macro.
#[macro_export]
macro_rules! build_command_map {
    ($map:ident $field:ident () $frp:expr) => {
        $map.insert(stringify!($field).into(),Command::new($frp.clone_ref()));
    };
    ($($ts:tt)*) => {}
}

/// Defines a method which is an alias to FRP emit method. Used internally by the `define_endpoints`
/// macro.
#[macro_export]
macro_rules! define_endpoints_emit_alias {
    ($field:ident ()) => {
        #[allow(missing_docs)]
        pub fn $field(&self) {
            self.$field.emit(());
        }
    };

    ($field:ident ($t1:ty,$t2:ty)) => {
        #[allow(missing_docs)]
        pub fn $field
        ( &self
        , t1:impl IntoParam<$t1>
        , t2:impl IntoParam<$t2>
        ) {
            let t1 = t1.into_param();
            let t2 = t2.into_param();
            self.$field.emit((t1,t2));
        }
    };

    ($field:ident ($t1:ty,$t2:ty,$t3:ty)) => {
        #[allow(missing_docs)]
        pub fn $field
        ( &self
        , t1:impl IntoParam<$t1>
        , t2:impl IntoParam<$t2>
        , t3:impl IntoParam<$t3>
        ) {
            let t1 = t1.into_param();
            let t2 = t2.into_param();
            let t3 = t3.into_param();
            self.$field.emit((t1,t2,t3));
        }
    };

    ($field:ident ($t1:ty,$t2:ty,$t3:ty,$t4:ty)) => {
        #[allow(missing_docs)]
        pub fn $field
        ( &self
        , t1:impl IntoParam<$t1>
        , t2:impl IntoParam<$t2>
        , t3:impl IntoParam<$t3>
        , t4:impl IntoParam<$t4>
        ) {
            let t1 = t1.into_param();
            let t2 = t2.into_param();
            let t3 = t3.into_param();
            let t4 = t4.into_param();
            self.$field.emit((t1,t2,t3,t4));
        }
    };

    ($field:ident ($t1:ty,$t2:ty,$t3:ty,$t4:ty,$t5:ty)) => {
        #[allow(missing_docs)]
        pub fn $field
        ( &self
        , t1:impl IntoParam<$t1>
        , t2:impl IntoParam<$t2>
        , t3:impl IntoParam<$t3>
        , t4:impl IntoParam<$t4>
        , t5:impl IntoParam<$t5>
        ) {
            let t1 = t1.into_param();
            let t2 = t2.into_param();
            let t3 = t3.into_param();
            let t4 = t4.into_param();
            let t5 = t5.into_param();
            self.$field.emit((t1,t2,t3,t4,t5));
        }
    };

    ($field:ident $t1:ty) => {
        #[allow(missing_docs)]
        pub fn $field(&self,t1:impl IntoParam<$t1>) {
            self.$field.emit(t1);
        }
    };
}
