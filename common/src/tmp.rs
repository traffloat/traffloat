pub mod graph {
    //! Basic node and edge management
    use std::collections::BTreeMap;
    use std::num::NonZeroUsize;
    use derive_new::new;
    use legion::Entity;
    use crate::SetupEcs;
    /// Identifies a node
    pub struct NodeId {
        inner: u32,
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::fmt::Debug for NodeId {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match *self {
                NodeId { inner: ref __self_0_0 } => {
                    let debug_trait_builder =
                        &mut ::core::fmt::Formatter::debug_struct(f,
                                                                  "NodeId");
                    let _ =
                        ::core::fmt::DebugStruct::field(debug_trait_builder,
                                                        "inner",
                                                        &&(*__self_0_0));
                    ::core::fmt::DebugStruct::finish(debug_trait_builder)
                }
            }
        }
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::clone::Clone for NodeId {
        #[inline]
        fn clone(&self) -> NodeId {
            { let _: ::core::clone::AssertParamIsClone<u32>; *self }
        }
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::marker::Copy for NodeId { }
    impl ::core::marker::StructuralPartialEq for NodeId { }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::cmp::PartialEq for NodeId {
        #[inline]
        fn eq(&self, other: &NodeId) -> bool {
            match *other {
                NodeId { inner: ref __self_1_0 } =>
                match *self {
                    NodeId { inner: ref __self_0_0 } =>
                    (*__self_0_0) == (*__self_1_0),
                },
            }
        }
        #[inline]
        fn ne(&self, other: &NodeId) -> bool {
            match *other {
                NodeId { inner: ref __self_1_0 } =>
                match *self {
                    NodeId { inner: ref __self_0_0 } =>
                    (*__self_0_0) != (*__self_1_0),
                },
            }
        }
    }
    impl ::core::marker::StructuralEq for NodeId { }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::cmp::Eq for NodeId {
        #[inline]
        #[doc(hidden)]
        fn assert_receiver_is_total_eq(&self) -> () {
            { let _: ::core::cmp::AssertParamIsEq<u32>; }
        }
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::cmp::PartialOrd for NodeId {
        #[inline]
        fn partial_cmp(&self, other: &NodeId)
         -> ::core::option::Option<::core::cmp::Ordering> {
            match *other {
                NodeId { inner: ref __self_1_0 } =>
                match *self {
                    NodeId { inner: ref __self_0_0 } =>
                    match ::core::cmp::PartialOrd::partial_cmp(&(*__self_0_0),
                                                               &(*__self_1_0))
                        {
                        ::core::option::Option::Some(::core::cmp::Ordering::Equal)
                        =>
                        ::core::option::Option::Some(::core::cmp::Ordering::Equal),
                        cmp => cmp,
                    },
                },
            }
        }
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::cmp::Ord for NodeId {
        #[inline]
        fn cmp(&self, other: &NodeId) -> ::core::cmp::Ordering {
            match *other {
                NodeId { inner: ref __self_1_0 } =>
                match *self {
                    NodeId { inner: ref __self_0_0 } =>
                    match ::core::cmp::Ord::cmp(&(*__self_0_0),
                                                &(*__self_1_0)) {
                        ::core::cmp::Ordering::Equal =>
                        ::core::cmp::Ordering::Equal,
                        cmp => cmp,
                    },
                },
            }
        }
    }
    impl crate::proto::ProtoType for NodeId {
        const CHECKSUM: u128 =
            {
                let mut output = 762111264484740707u128;
                output =
                    output.wrapping_mul(crate::proto::PROTO_TYPE_CKSUM_PRIME);
                output = output.wrapping_add(8533281656851243364u128);
                output =
                    output.wrapping_mul(crate::proto::PROTO_TYPE_CKSUM_PRIME);
                output =
                    output.wrapping_add(<u32 as
                                            crate::proto::ProtoType>::CHECKSUM);
                output =
                    output.wrapping_mul(crate::proto::PROTO_TYPE_CKSUM_PRIME);
                output
            };
    }
    impl crate::proto::BinWrite for NodeId {
        fn write(&self, buf: &mut Vec<u8>) {
            crate::proto::BinWrite::write(&self.inner, &mut *buf);
        }
    }
    impl crate::proto::BinRead for NodeId {
        fn read(buf: &mut &[u8]) -> Result<Self, crate::proto::Error> {
            Ok(Self{inner: crate::proto::BinRead::read(&mut *buf)?,})
        }
    }
    /// Identifies an edge
    pub struct EdgeId {
        /// The "source" node
        #[getset(get_copy = "pub")]
        from: NodeId,
        /// The "dest" node
        #[getset(get_copy = "pub")]
        to: NodeId,
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::fmt::Debug for EdgeId {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match *self {
                EdgeId { from: ref __self_0_0, to: ref __self_0_1 } => {
                    let debug_trait_builder =
                        &mut ::core::fmt::Formatter::debug_struct(f,
                                                                  "EdgeId");
                    let _ =
                        ::core::fmt::DebugStruct::field(debug_trait_builder,
                                                        "from",
                                                        &&(*__self_0_0));
                    let _ =
                        ::core::fmt::DebugStruct::field(debug_trait_builder,
                                                        "to",
                                                        &&(*__self_0_1));
                    ::core::fmt::DebugStruct::finish(debug_trait_builder)
                }
            }
        }
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::clone::Clone for EdgeId {
        #[inline]
        fn clone(&self) -> EdgeId {
            {
                let _: ::core::clone::AssertParamIsClone<NodeId>;
                let _: ::core::clone::AssertParamIsClone<NodeId>;
                *self
            }
        }
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::marker::Copy for EdgeId { }
    impl ::core::marker::StructuralPartialEq for EdgeId { }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::cmp::PartialEq for EdgeId {
        #[inline]
        fn eq(&self, other: &EdgeId) -> bool {
            match *other {
                EdgeId { from: ref __self_1_0, to: ref __self_1_1 } =>
                match *self {
                    EdgeId { from: ref __self_0_0, to: ref __self_0_1 } =>
                    (*__self_0_0) == (*__self_1_0) &&
                        (*__self_0_1) == (*__self_1_1),
                },
            }
        }
        #[inline]
        fn ne(&self, other: &EdgeId) -> bool {
            match *other {
                EdgeId { from: ref __self_1_0, to: ref __self_1_1 } =>
                match *self {
                    EdgeId { from: ref __self_0_0, to: ref __self_0_1 } =>
                    (*__self_0_0) != (*__self_1_0) ||
                        (*__self_0_1) != (*__self_1_1),
                },
            }
        }
    }
    impl ::core::marker::StructuralEq for EdgeId { }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::cmp::Eq for EdgeId {
        #[inline]
        #[doc(hidden)]
        fn assert_receiver_is_total_eq(&self) -> () {
            {
                let _: ::core::cmp::AssertParamIsEq<NodeId>;
                let _: ::core::cmp::AssertParamIsEq<NodeId>;
            }
        }
    }
    impl crate::proto::ProtoType for EdgeId {
        const CHECKSUM: u128 =
            {
                let mut output = 1571569740518573802u128;
                output =
                    output.wrapping_mul(crate::proto::PROTO_TYPE_CKSUM_PRIME);
                output = output.wrapping_add(3299847730264110667u128);
                output =
                    output.wrapping_mul(crate::proto::PROTO_TYPE_CKSUM_PRIME);
                output =
                    output.wrapping_add(<NodeId as
                                            crate::proto::ProtoType>::CHECKSUM);
                output =
                    output.wrapping_mul(crate::proto::PROTO_TYPE_CKSUM_PRIME);
                output = output.wrapping_add(3957028272457675022u128);
                output =
                    output.wrapping_mul(crate::proto::PROTO_TYPE_CKSUM_PRIME);
                output =
                    output.wrapping_add(<NodeId as
                                            crate::proto::ProtoType>::CHECKSUM);
                output =
                    output.wrapping_mul(crate::proto::PROTO_TYPE_CKSUM_PRIME);
                output
            };
    }
    impl crate::proto::BinWrite for EdgeId {
        fn write(&self, buf: &mut Vec<u8>) {
            crate::proto::BinWrite::write(&self.from, &mut *buf);
            crate::proto::BinWrite::write(&self.to, &mut *buf);
        }
    }
    impl crate::proto::BinRead for EdgeId {
        fn read(buf: &mut &[u8]) -> Result<Self, crate::proto::Error> {
            Ok(Self{from: crate::proto::BinRead::read(&mut *buf)?,
                    to: crate::proto::BinRead::read(&mut *buf)?,})
        }
    }
    impl EdgeId {
        #[doc = "Constructs a new `EdgeId`."]
        pub fn new(from: NodeId, to: NodeId) -> Self {
            EdgeId{from: from, to: to,}
        }
    }
    impl EdgeId {
        #[doc = " The \"source\" node"]
        #[inline(always)]
        pub fn from(&self) -> NodeId { self.from }
        #[doc = " The \"dest\" node"]
        #[inline(always)]
        pub fn to(&self) -> NodeId { self.to }
    }
    /// Indicates that a node is added
    pub struct NodeAddEvent {
        /// The added node
        #[getset(get_copy = "pub")]
        node: NodeId,
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::fmt::Debug for NodeAddEvent {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match *self {
                NodeAddEvent { node: ref __self_0_0 } => {
                    let debug_trait_builder =
                        &mut ::core::fmt::Formatter::debug_struct(f,
                                                                  "NodeAddEvent");
                    let _ =
                        ::core::fmt::DebugStruct::field(debug_trait_builder,
                                                        "node",
                                                        &&(*__self_0_0));
                    ::core::fmt::DebugStruct::finish(debug_trait_builder)
                }
            }
        }
    }
    impl NodeAddEvent {
        #[doc = "Constructs a new `NodeAddEvent`."]
        pub fn new(node: NodeId) -> Self { NodeAddEvent{node: node,} }
    }
    impl NodeAddEvent {
        #[doc = " The added node"]
        #[inline(always)]
        pub fn node(&self) -> NodeId { self.node }
    }
    /// Indicates that a node is flagged for removal
    pub struct NodeRemoveEvent {
        /// The added node
        #[getset(get_copy = "pub")]
        node: NodeId,
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::fmt::Debug for NodeRemoveEvent {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match *self {
                NodeRemoveEvent { node: ref __self_0_0 } => {
                    let debug_trait_builder =
                        &mut ::core::fmt::Formatter::debug_struct(f,
                                                                  "NodeRemoveEvent");
                    let _ =
                        ::core::fmt::DebugStruct::field(debug_trait_builder,
                                                        "node",
                                                        &&(*__self_0_0));
                    ::core::fmt::DebugStruct::finish(debug_trait_builder)
                }
            }
        }
    }
    impl NodeRemoveEvent {
        #[doc = "Constructs a new `NodeRemoveEvent`."]
        pub fn new(node: NodeId) -> Self { NodeRemoveEvent{node: node,} }
    }
    impl NodeRemoveEvent {
        #[doc = " The added node"]
        #[inline(always)]
        pub fn node(&self) -> NodeId { self.node }
    }
    /// Indicates that nodes have been removed
    pub struct PostNodeRemoveEvent {
        /// Number of nodes removed
        #[getset(get_copy = "pub")]
        count: NonZeroUsize,
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::fmt::Debug for PostNodeRemoveEvent {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match *self {
                PostNodeRemoveEvent { count: ref __self_0_0 } => {
                    let debug_trait_builder =
                        &mut ::core::fmt::Formatter::debug_struct(f,
                                                                  "PostNodeRemoveEvent");
                    let _ =
                        ::core::fmt::DebugStruct::field(debug_trait_builder,
                                                        "count",
                                                        &&(*__self_0_0));
                    ::core::fmt::DebugStruct::finish(debug_trait_builder)
                }
            }
        }
    }
    impl PostNodeRemoveEvent {
        #[doc = "Constructs a new `PostNodeRemoveEvent`."]
        pub fn new(count: NonZeroUsize) -> Self {
            PostNodeRemoveEvent{count: count,}
        }
    }
    impl PostNodeRemoveEvent {
        #[doc = " Number of nodes removed"]
        #[inline(always)]
        pub fn count(&self) -> NonZeroUsize { self.count }
    }
    /// Indicates that an edge is added
    pub struct EdgeAddEvent {
        /// The added node
        #[getset(get_copy = "pub")]
        edge: EdgeId,
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::fmt::Debug for EdgeAddEvent {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match *self {
                EdgeAddEvent { edge: ref __self_0_0 } => {
                    let debug_trait_builder =
                        &mut ::core::fmt::Formatter::debug_struct(f,
                                                                  "EdgeAddEvent");
                    let _ =
                        ::core::fmt::DebugStruct::field(debug_trait_builder,
                                                        "edge",
                                                        &&(*__self_0_0));
                    ::core::fmt::DebugStruct::finish(debug_trait_builder)
                }
            }
        }
    }
    impl EdgeAddEvent {
        #[doc = "Constructs a new `EdgeAddEvent`."]
        pub fn new(edge: EdgeId) -> Self { EdgeAddEvent{edge: edge,} }
    }
    impl EdgeAddEvent {
        #[doc = " The added node"]
        #[inline(always)]
        pub fn edge(&self) -> EdgeId { self.edge }
    }
    /// Indicates that an edge is flagged for removal
    pub struct EdgeRemoveEvent {
        /// The added node
        #[getset(get_copy = "pub")]
        edge: EdgeId,
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::fmt::Debug for EdgeRemoveEvent {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match *self {
                EdgeRemoveEvent { edge: ref __self_0_0 } => {
                    let debug_trait_builder =
                        &mut ::core::fmt::Formatter::debug_struct(f,
                                                                  "EdgeRemoveEvent");
                    let _ =
                        ::core::fmt::DebugStruct::field(debug_trait_builder,
                                                        "edge",
                                                        &&(*__self_0_0));
                    ::core::fmt::DebugStruct::finish(debug_trait_builder)
                }
            }
        }
    }
    impl EdgeRemoveEvent {
        #[doc = "Constructs a new `EdgeRemoveEvent`."]
        pub fn new(edge: EdgeId) -> Self { EdgeRemoveEvent{edge: edge,} }
    }
    impl EdgeRemoveEvent {
        #[doc = " The added node"]
        #[inline(always)]
        pub fn edge(&self) -> EdgeId { self.edge }
    }
    /// Tracks the nodes and edges in the world
    pub struct Graph {
        node_index: BTreeMap<NodeId, Entity>,
        node_deletion_queue: Vec<NodeId>,
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::default::Default for Graph {
        #[inline]
        fn default() -> Graph {
            Graph{node_index: ::core::default::Default::default(),
                  node_deletion_queue: ::core::default::Default::default(),}
        }
    }
    impl Graph {
        /// Retrieves the entity ID for the given node
        pub fn get_node(&self, id: NodeId) -> Option<Entity> {
            self.node_index.get(&id).copied()
        }
    }
    fn delete_nodes_system(mut state_0: ::shrev::ReaderId<NodeRemoveEvent>)
     -> impl ::legion::systems::Runnable {
        use legion::IntoQuery;
        let generic_names = "";
        ::legion::systems::SystemBuilder::new({
                                                  let res =
                                                      ::alloc::fmt::format(::core::fmt::Arguments::new_v1(&["",
                                                                                                            ""],
                                                                                                          &match (&"delete_nodes",
                                                                                                                  &generic_names)
                                                                                                               {
                                                                                                               (arg0,
                                                                                                                arg1)
                                                                                                               =>
                                                                                                               [::core::fmt::ArgumentV1::new(arg0,
                                                                                                                                             ::core::fmt::Display::fmt),
                                                                                                                ::core::fmt::ArgumentV1::new(arg1,
                                                                                                                                             ::core::fmt::Display::fmt)],
                                                                                                           }));
                                                  res
                                              }).read_resource::<::shrev::EventChannel<NodeRemoveEvent>>().read_resource::<::codegen::Perf>().write_resource::<Graph>().write_resource::<shrev::EventChannel<PostNodeRemoveEvent>>().build(move
                                                                                                                                                                                                                                               |cmd,
                                                                                                                                                                                                                                                world,
                                                                                                                                                                                                                                                resources,
                                                                                                                                                                                                                                                query|
                                                                                                                                                                                                                                               {
                                                                                                                                                                                                                                                   delete_nodes::<>(cmd,
                                                                                                                                                                                                                                                                    &mut *resources.2,
                                                                                                                                                                                                                                                                    &mut state_0,
                                                                                                                                                                                                                                                                    &*resources.0,
                                                                                                                                                                                                                                                                    &mut *resources.3,
                                                                                                                                                                                                                                                                    &*resources.1);
                                                                                                                                                                                                                                               })
    }
    #[allow(dead_code)]
    fn delete_nodes(cmd_buf: &mut legion::systems::CommandBuffer,
                    graph: &mut Graph,
                    __reader_id_for_node_removals:
                        &mut ::shrev::ReaderId<NodeRemoveEvent>,
                    __channel_for_node_removals:
                        &::shrev::EventChannel<NodeRemoveEvent>,
                    post_node_remove_pub:
                        &mut shrev::EventChannel<PostNodeRemoveEvent>,
                    __traffloat_codegen_perf: &::codegen::Perf) {
        fn imp(cmd_buf: &mut legion::systems::CommandBuffer,
               graph: &mut Graph,
               __reader_id_for_node_removals:
                   &mut ::shrev::ReaderId<NodeRemoveEvent>,
               __channel_for_node_removals:
                   &::shrev::EventChannel<NodeRemoveEvent>,
               post_node_remove_pub:
                   &mut shrev::EventChannel<PostNodeRemoveEvent>,
               __traffloat_codegen_perf: &::codegen::Perf) {
            let node_removals =
                __channel_for_node_removals.read(__reader_id_for_node_removals);
            {
                for &node in &graph.node_deletion_queue {
                    let entity =
                        graph.node_index.remove(&node).expect("Removing nonexistent node entity");
                    cmd_buf.remove(entity);
                }
                let count = graph.node_deletion_queue.len();
                graph.node_deletion_queue.clear();
                if let Some(count) = NonZeroUsize::new(count) {
                    post_node_remove_pub.single_write(PostNodeRemoveEvent{count,});
                }
                for removal in node_removals {
                    graph.node_deletion_queue.push(removal.node);
                }
            }
        }
        let __traffloat_codegen_perf_start = ::codegen::hrtime();
        imp(cmd_buf, graph, __reader_id_for_node_removals,
            __channel_for_node_removals, post_node_remove_pub,
            __traffloat_codegen_perf);
        let __traffloat_codegen_perf_end = ::codegen::hrtime();
        __traffloat_codegen_perf.push("traffloat::graph::delete_nodes",
                                      __traffloat_codegen_perf_end -
                                          __traffloat_codegen_perf_start)
    }
    fn delete_nodes_setup(mut setup: ::codegen::SetupEcs)
     -> ::codegen::SetupEcs {
        setup.resource(<Graph>::default());
        let __reader_id_for_node_removals =
            setup.subscribe::<NodeRemoveEvent>();
        setup.resource(<shrev::EventChannel<PostNodeRemoveEvent>>::default());
        setup.system(delete_nodes_system(__reader_id_for_node_removals))
    }
    /// Initializes ECS
    pub fn setup_ecs(setup: SetupEcs) -> SetupEcs {
        setup.uses(delete_nodes_setup)
    }
}
pub mod sun {
    //! Calculates the sunlight level of each building
    use std::collections::{btree_map::Entry, BTreeMap};
    use std::f64::consts::PI;
    use smallvec::SmallVec;
    use crate::config;
    use crate::graph::*;
    use crate::shape::Shape;
    use crate::space::{Position, Vector};
    use crate::time;
    use crate::util::Finite;
    use crate::SetupEcs;
    /// The position of the sun
    pub struct Sun {
        /// Orientation of the sun, in radians from +x towards +y
        #[getset(get_copy = "pub")]
        yaw: f64,
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::default::Default for Sun {
        #[inline]
        fn default() -> Sun { Sun{yaw: ::core::default::Default::default(),} }
    }
    impl Sun {
        #[doc = " Orientation of the sun, in radians from +x towards +y"]
        #[inline(always)]
        pub fn yaw(&self) -> f64 { self.yaw }
    }
    impl Sun {
        /// Direction vector from any opaque object to the sun.
        pub fn direction(&self) -> Vector {
            Vector::new(self.yaw().cos(), self.yaw().sin(), 0.)
        }
    }
    fn move_sun_system() -> impl ::legion::systems::Runnable {
        use legion::IntoQuery;
        let generic_names = "";
        ::legion::systems::SystemBuilder::new({
                                                  let res =
                                                      ::alloc::fmt::format(::core::fmt::Arguments::new_v1(&["",
                                                                                                            ""],
                                                                                                          &match (&"move_sun",
                                                                                                                  &generic_names)
                                                                                                               {
                                                                                                               (arg0,
                                                                                                                arg1)
                                                                                                               =>
                                                                                                               [::core::fmt::ArgumentV1::new(arg0,
                                                                                                                                             ::core::fmt::Display::fmt),
                                                                                                                ::core::fmt::ArgumentV1::new(arg1,
                                                                                                                                             ::core::fmt::Display::fmt)],
                                                                                                           }));
                                                  res
                                              }).read_resource::<time::Clock>().read_resource::<config::Scalar>().read_resource::<::codegen::Perf>().write_resource::<Sun>().build(move
                                                                                                                                                                                       |cmd,
                                                                                                                                                                                        world,
                                                                                                                                                                                        resources,
                                                                                                                                                                                        query|
                                                                                                                                                                                       {
                                                                                                                                                                                           move_sun::<>(&mut *resources.3,
                                                                                                                                                                                                        &*resources.0,
                                                                                                                                                                                                        &*resources.1,
                                                                                                                                                                                                        &*resources.2);
                                                                                                                                                                                       })
    }
    #[allow(dead_code)]
    fn move_sun(sun: &mut Sun, clock: &time::Clock, config: &config::Scalar,
                __traffloat_codegen_perf: &::codegen::Perf) {
        fn imp(sun: &mut Sun, clock: &time::Clock, config: &config::Scalar,
               __traffloat_codegen_perf: &::codegen::Perf) {
            { sun.yaw += config.sun_speed * clock.delta; sun.yaw %= PI * 2.; }
        }
        let __traffloat_codegen_perf_start = ::codegen::hrtime();
        imp(sun, clock, config, __traffloat_codegen_perf);
        let __traffloat_codegen_perf_end = ::codegen::hrtime();
        __traffloat_codegen_perf.push("traffloat::sun::move_sun",
                                      __traffloat_codegen_perf_end -
                                          __traffloat_codegen_perf_start)
    }
    fn move_sun_setup(mut setup: ::codegen::SetupEcs) -> ::codegen::SetupEcs {
        setup.resource(<Sun>::default());
        setup.resource(<time::Clock>::default());
        setup.resource(<config::Scalar>::default());
        setup.system(move_sun_system())
    }
    /// Number of partitions to compute shadow casting for
    pub const MONTH_COUNT: usize = 12;
    /// A component storing the lighting data for a node.
    pub struct LightStats {
        /// The brightness values in each month.
        ///
        /// The brightness value is the area receiving sunlight.
        #[getset(get = "pub")]
        brightness: [f64; MONTH_COUNT],
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::fmt::Debug for LightStats {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match *self {
                LightStats { brightness: ref __self_0_0 } => {
                    let debug_trait_builder =
                        &mut ::core::fmt::Formatter::debug_struct(f,
                                                                  "LightStats");
                    let _ =
                        ::core::fmt::DebugStruct::field(debug_trait_builder,
                                                        "brightness",
                                                        &&(*__self_0_0));
                    ::core::fmt::DebugStruct::finish(debug_trait_builder)
                }
            }
        }
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::core::default::Default for LightStats {
        #[inline]
        fn default() -> LightStats {
            LightStats{brightness: ::core::default::Default::default(),}
        }
    }
    impl LightStats {
        #[doc = " The brightness values in each month."]
        #[doc = ""]
        #[doc = " The brightness value is the area receiving sunlight."]
        #[inline(always)]
        pub fn brightness(&self) -> &[f64; MONTH_COUNT] { &self.brightness }
    }
    fn shadow_cast_system(mut state_0: bool,
                          mut state_1: ::shrev::ReaderId<NodeAddEvent>,
                          mut state_2: ::shrev::ReaderId<PostNodeRemoveEvent>)
     -> impl ::legion::systems::Runnable {
        use legion::IntoQuery;
        let generic_names = "";
        ::legion::systems::SystemBuilder::new({
                                                  let res =
                                                      ::alloc::fmt::format(::core::fmt::Arguments::new_v1(&["",
                                                                                                            ""],
                                                                                                          &match (&"shadow_cast",
                                                                                                                  &generic_names)
                                                                                                               {
                                                                                                               (arg0,
                                                                                                                arg1)
                                                                                                               =>
                                                                                                               [::core::fmt::ArgumentV1::new(arg0,
                                                                                                                                             ::core::fmt::Display::fmt),
                                                                                                                ::core::fmt::ArgumentV1::new(arg1,
                                                                                                                                             ::core::fmt::Display::fmt)],
                                                                                                           }));
                                                  res
                                              }).read_component::<Position>().read_component::<Shape>().write_component::<LightStats>().read_resource::<::shrev::EventChannel<NodeAddEvent>>().read_resource::<::shrev::EventChannel<PostNodeRemoveEvent>>().read_resource::<::codegen::Perf>().build(move
                                                                                                                                                                                                                                                                                                          |cmd,
                                                                                                                                                                                                                                                                                                           world,
                                                                                                                                                                                                                                                                                                           resources,
                                                                                                                                                                                                                                                                                                           query|
                                                                                                                                                                                                                                                                                                          {
                                                                                                                                                                                                                                                                                                              shadow_cast::<>(world,
                                                                                                                                                                                                                                                                                                                              &mut state_0,
                                                                                                                                                                                                                                                                                                                              &mut state_1,
                                                                                                                                                                                                                                                                                                                              &*resources.0,
                                                                                                                                                                                                                                                                                                                              &mut state_2,
                                                                                                                                                                                                                                                                                                                              &*resources.1,
                                                                                                                                                                                                                                                                                                                              &*resources.2);
                                                                                                                                                                                                                                                                                                          })
    }
    #[allow(dead_code)]
    fn shadow_cast(world: &mut legion::world::SubWorld, first: &mut bool,
                   __reader_id_for_node_additions:
                       &mut ::shrev::ReaderId<NodeAddEvent>,
                   __channel_for_node_additions:
                       &::shrev::EventChannel<NodeAddEvent>,
                   __reader_id_for_node_post_removals:
                       &mut ::shrev::ReaderId<PostNodeRemoveEvent>,
                   __channel_for_node_post_removals:
                       &::shrev::EventChannel<PostNodeRemoveEvent>,
                   __traffloat_codegen_perf: &::codegen::Perf) {
        fn imp(world: &mut legion::world::SubWorld, first: &mut bool,
               __reader_id_for_node_additions:
                   &mut ::shrev::ReaderId<NodeAddEvent>,
               __channel_for_node_additions:
                   &::shrev::EventChannel<NodeAddEvent>,
               __reader_id_for_node_post_removals:
                   &mut ::shrev::ReaderId<PostNodeRemoveEvent>,
               __channel_for_node_post_removals:
                   &::shrev::EventChannel<PostNodeRemoveEvent>,
               __traffloat_codegen_perf: &::codegen::Perf) {
            let node_additions =
                __channel_for_node_additions.read(__reader_id_for_node_additions);
            let node_post_removals =
                __channel_for_node_post_removals.read(__reader_id_for_node_post_removals);
            {
                let has_change =
                    node_additions.count() > 0 &&
                        node_post_removals.count() > 0;
                if !has_change && !*first { return; }
                *first = false;

                #[allow(clippy :: indexing_slicing)]
                for month in 0..MONTH_COUNT {
                    use legion::IntoQuery;
                    let mut query =
                        <(&mut LightStats, &Position, &Shape)>::query();
                    struct Marker<'t> {
                        light: &'t mut f64,
                        min: [Finite; 2],
                        max: [Finite; 2],
                        priority: Finite,
                    }
                    let mut markers = Vec::new();
                    for (stats, &position, shape) in query.iter_mut(world) {
                        #[allow(clippy :: cast_precision_loss)]
                        let yaw =
                            {
                                PI * 2. / (MONTH_COUNT as f64) *
                                    (month as f64)
                            };
                        let rot =
                            nalgebra::Rotation3::from_axis_angle(&Vector::z_axis(),
                                                                 -yaw).matrix().to_homogeneous();
                        let trans = shape.transform(position);
                        let (min, max) = shape.unit().bb_under(rot * trans);
                        let priority = Finite::new(max.x);
                        let light =
                            stats.brightness.get_mut(month).expect("month < MONTH_COUNT");
                        *light = 0.;
                        let marker =
                            Marker{light,
                                   min:
                                       [Finite::new(min.y),
                                        Finite::new(min.z)],
                                   max:
                                       [Finite::new(max.y),
                                        Finite::new(max.z)],
                                   priority,};
                        markers.push(marker);
                    }
                    let cuts: SmallVec<[Vec<Finite>; 2]> =
                        (0_usize..2).map(|axis|
                                             {
                                                 let mut vec: Vec<_> =
                                                     markers.iter().map(|marker|
                                                                            marker.min[axis]).chain(markers.iter().map(|marker|
                                                                                                                           marker.max[axis])).collect();
                                                 vec.sort_unstable();
                                                 vec
                                             }).collect();
                    let mut grids = BTreeMap::<(usize, usize), usize>::new();
                    for (marker_index, marker) in markers.iter().enumerate() {
                        let min_grid_index: SmallVec<[usize; 2]> =
                            (0_usize..2).map(|axis|
                                                 {
                                                     cuts[axis].binary_search(&marker.min[axis]).expect("Cut was inserted to Vec")
                                                 }).collect();
                        let max_grid_index: SmallVec<[usize; 2]> =
                            (0_usize..2).map(|axis|
                                                 {
                                                     cuts[axis].binary_search(&marker.max[axis]).expect("Cut was inserted to Vec")
                                                 }).collect();
                        for i in min_grid_index[0]..max_grid_index[0] {
                            for j in min_grid_index[1]..max_grid_index[1] {
                                let key = (i, j);
                                match grids.entry(key) {
                                    Entry::Vacant(entry) => {
                                        entry.insert(marker_index);
                                    }
                                    Entry::Occupied(mut entry) => {
                                        if markers[*entry.get()].priority <
                                               marker.priority {
                                            entry.insert(marker_index);
                                        }
                                    }
                                }
                            }
                        }
                    }
                    {
                        let lvl = ::log::Level::Debug;
                        if lvl <= ::log::STATIC_MAX_LEVEL &&
                               lvl <= ::log::max_level() {
                            ::log::__private_api_log(::core::fmt::Arguments::new_v1(&["Split objects into ",
                                                                                      " grids"],
                                                                                    &match (&grids.len(),)
                                                                                         {
                                                                                         (arg0,)
                                                                                         =>
                                                                                         [::core::fmt::ArgumentV1::new(arg0,
                                                                                                                       ::core::fmt::Display::fmt)],
                                                                                     }),
                                                     lvl,
                                                     &("traffloat::sun",
                                                       "traffloat::sun",
                                                       "common/src/sun.rs",
                                                       162u32));
                        }
                    };
                    for ((i, j), marker_index) in grids {
                        let len0 =
                            cuts[0][i + 1].value() - cuts[0][i].value();
                        let len1 =
                            cuts[1][j + 1].value() - cuts[1][j].value();
                        let area = len0 * len1;
                        let light = &mut *markers[marker_index].light;
                        *light += area;
                    }
                }
            }
        }
        let __traffloat_codegen_perf_start = ::codegen::hrtime();
        imp(world, first, __reader_id_for_node_additions,
            __channel_for_node_additions, __reader_id_for_node_post_removals,
            __channel_for_node_post_removals, __traffloat_codegen_perf);
        let __traffloat_codegen_perf_end = ::codegen::hrtime();
        __traffloat_codegen_perf.push("traffloat::sun::shadow_cast",
                                      __traffloat_codegen_perf_end -
                                          __traffloat_codegen_perf_start)
    }
    fn shadow_cast_setup(mut setup: ::codegen::SetupEcs)
     -> ::codegen::SetupEcs {
        let __reader_id_for_node_additions =
            setup.subscribe::<NodeAddEvent>();
        let __reader_id_for_node_post_removals =
            setup.subscribe::<PostNodeRemoveEvent>();
        setup.system(shadow_cast_system(true, __reader_id_for_node_additions,
                                        __reader_id_for_node_post_removals))
    }
    /// Initializes ECS
    pub fn setup_ecs(setup: SetupEcs) -> SetupEcs {
        setup.uses(move_sun_setup).uses(shadow_cast_setup)
    }
}
