use egui_dock::{DockState, NodeIndex, NodePath, SurfaceIndex, TabIndex, TabPath};

use crate::dock::TabState;

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
        match state.iter_all_tabs_mut().find(|(_, tab)| (self.0)(tab)) {
            Some((path, tab)) => {
                *tab = make_tab();
                Ok(path)
            }
            None => Err(make_tab),
        }
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
        for (path, leaf) in state.iter_leaves_mut() {
            if let Some(index) = leaf.tabs.iter().position(|tab| (self.0)(tab)) {
                leaf.tabs.insert(index + 1, tab_fn());
                return Ok(TabPath::from((path, TabIndex(index + 1))));
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
        let [_, new_node] =
            state.split(NodePath::MAIN_ROOT, self.split, self.ratio, egui_dock::Node::leaf(tab()));
        TabPath::new(SurfaceIndex::main(), new_node, TabIndex(0))
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
        let Some(path) = state
            .iter_all_tabs()
            .find(|(_, tab)| (self.leaf_fn)(tab))
            .map(|(path, _)| path.node_path())
        else {
            return Err(new_tab);
        };
        let [_, new_node] =
            state.split(path, self.split, self.ratio, egui_dock::Node::leaf(new_tab()));
        Ok(TabPath::new(path.surface, new_node, TabIndex(0)))
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
        TabPath::new(window, NodeIndex::root(), TabIndex(0))
    }
}
