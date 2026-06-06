use egui_dock::{DockState, NodeIndex, SurfaceIndex, TabIndex};

use crate::dock::TabState;

pub type TabPath = (SurfaceIndex, NodeIndex, TabIndex);

pub trait TabPlacement: Sized {
    fn place<F: FnOnce() -> TabState>(
        self,
        state: &mut DockState<TabState>,
        tab: F,
    ) -> Result<TabPath, F>;

    fn only_if(self, condition: bool) -> impl TabPlacement {
        struct OnlyIf<P>(P, bool);

        impl<P: TabPlacement> TabPlacement for OnlyIf<P> {
            fn place<F: FnOnce() -> TabState>(
                self,
                state: &mut DockState<TabState>,
                tab: F,
            ) -> Result<TabPath, F> {
                if self.1 { self.0.place(state, tab) } else { Err(tab) }
            }
        }

        OnlyIf(self, condition)
    }

    fn or<P: TabPlacement>(self, other: P) -> impl TabPlacement {
        struct Or<A, B>(A, B);

        impl<A: TabPlacement, B: TabPlacement> TabPlacement for Or<A, B> {
            fn place<F: FnOnce() -> TabState>(
                self,
                state: &mut DockState<TabState>,
                tab: F,
            ) -> Result<TabPath, F> {
                self.0.place(state, tab).or_else(|tab| self.1.place(state, tab))
            }
        }

        Or(self, other)
    }

    fn or_always<P: AlwaysTabPlacement>(self, other: P) -> impl AlwaysTabPlacement {
        struct Or<A, B>(A, B);

        impl<A: TabPlacement, B: AlwaysTabPlacement> AlwaysTabPlacement for Or<A, B> {
            fn always_place<F: FnOnce() -> TabState>(
                self,
                state: &mut DockState<TabState>,
                tab: F,
            ) -> TabPath {
                self.0.place(state, tab).unwrap_or_else(|tab| self.1.always_place(state, tab))
            }
        }

        Or(self, other)
    }
}

pub trait AlwaysTabPlacement: TabPlacement {
    fn always_place<F: FnOnce() -> TabState>(
        self,
        state: &mut DockState<TabState>,
        tab: F,
    ) -> TabPath;
}

impl<T: AlwaysTabPlacement> TabPlacement for T {
    fn place<F: FnOnce() -> TabState>(
        self,
        state: &mut DockState<TabState>,
        tab: F,
    ) -> Result<TabPath, F> {
        Ok(self.always_place(state, tab))
    }
}

pub struct ReplaceTab<R: Fn(&TabState) -> bool>(pub R);

impl<R> TabPlacement for ReplaceTab<R>
where
    R: Fn(&TabState) -> bool,
{
    fn place<F: FnOnce() -> TabState>(
        self,
        state: &mut DockState<TabState>,
        make_tab: F,
    ) -> Result<TabPath, F> {
        for (si, surface) in state.iter_surfaces_mut().enumerate() {
            let si = SurfaceIndex(si);
            for (ni, node) in surface.iter_nodes_mut().enumerate() {
                let ni = NodeIndex(ni);
                if let Some(leaf) = node.get_leaf_mut() {
                    for (ti, tab) in leaf.tabs.iter_mut().enumerate() {
                        let ti = TabIndex(ti);

                        if (self.0)(tab) {
                            *tab = make_tab();
                            return Ok((si, ni, ti));
                        }
                    }
                }
            }
        }

        Err(make_tab)
    }
}

pub struct AfterTab<R: Fn(&TabState) -> bool>(pub R);

impl<R> TabPlacement for AfterTab<R>
where
    R: Fn(&TabState) -> bool,
{
    fn place<F: FnOnce() -> TabState>(
        self,
        state: &mut DockState<TabState>,
        tab_fn: F,
    ) -> Result<TabPath, F> {
        for (si, surface) in state.iter_surfaces_mut().enumerate() {
            let si = SurfaceIndex(si);
            for (ni, node) in surface.iter_nodes_mut().enumerate() {
                let ni = NodeIndex(ni);
                if let Some(leaf) = node.get_leaf_mut() {
                    for (ti, tab) in leaf.tabs.iter().enumerate() {
                        if (self.0)(tab) {
                            leaf.tabs.insert(ti + 1, tab_fn());
                            return Ok((si, ni, TabIndex(ti + 1)));
                        }
                    }
                }
            }
        }

        Err(tab_fn)
    }
}

pub struct Split {
    pub split: egui_dock::Split,
    pub ratio: f32,
}

impl AlwaysTabPlacement for Split {
    fn always_place<F: FnOnce() -> TabState>(
        self,
        state: &mut DockState<TabState>,
        tab: F,
    ) -> TabPath {
        let [_, new_node] = state.split(
            (SurfaceIndex::main(), NodeIndex::root()),
            self.split,
            self.ratio,
            egui_dock::Node::leaf(tab()),
        );
        (SurfaceIndex::main(), new_node, TabIndex(0))
    }
}

impl Split {
    pub fn at<R: Fn(&TabState) -> bool>(self, leaf_fn: R) -> SplitLeaf<R> {
        SplitLeaf { split: self.split, ratio: self.ratio, leaf_fn }
    }
}

pub struct SplitLeaf<R: Fn(&TabState) -> bool> {
    pub split:   egui_dock::Split,
    pub ratio:   f32,
    pub leaf_fn: R,
}

impl<R: Fn(&TabState) -> bool> TabPlacement for SplitLeaf<R> {
    fn place<F: FnOnce() -> TabState>(
        self,
        state: &mut DockState<TabState>,
        new_tab: F,
    ) -> Result<TabPath, F> {
        let mut path = None;
        for (si, surface) in state.iter_surfaces_mut().enumerate() {
            let si = SurfaceIndex(si);
            for (ni, node) in surface.iter_nodes_mut().enumerate() {
                let ni = NodeIndex(ni);
                if let Some(leaf) = node.get_leaf_mut() {
                    for (ti, tab) in leaf.tabs.iter().enumerate() {
                        let ti = TabIndex(ti);
                        if (self.leaf_fn)(tab) {
                            path = Some((si, ni));
                            break;
                        }
                    }
                }
            }
        }
        let Some(path) = path else { return Err(new_tab) };
        let [_, new_node] =
            state.split(path, self.split, self.ratio, egui_dock::Node::leaf(new_tab()));
        Ok((path.0, new_node, TabIndex(0)))
    }
}

pub struct NewWindow;

impl AlwaysTabPlacement for NewWindow {
    fn always_place<F: FnOnce() -> TabState>(
        self,
        state: &mut DockState<TabState>,
        tab: F,
    ) -> TabPath {
        let window = state.add_window([tab()].into());
        (window, NodeIndex::root(), TabIndex(0))
    }
}
