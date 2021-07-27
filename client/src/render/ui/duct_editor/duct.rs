//! Renders a single duct.

use super::*;

/// Displays an editor for ducts in an edge.
pub struct Comp {
    props: Props,
    link: ComponentLink<Self>,
}

impl Component for Comp {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Props, link: ComponentLink<Self>) -> Self {
        Self { props, link }
    }

    fn update(&mut self, msg: Msg) -> ShouldRender {
        match msg {}
    }

    fn change(&mut self, props: Props) -> ShouldRender {
        self.props = props;
        true
    }

    fn view(&self) -> Html {
        html! {
            <circle
                cx=(self.props.origin + self.props.center.vector().x).to_string()
                cy=(self.props.origin + self.props.center.vector().y).to_string()
                r=self.props.radius.to_string()
                fill=duct_fill(self.props.ty)
                style="cursor: pointer;"
                />
        }
    }
}

/// Events for [`Comp`].
pub enum Msg {}

/// Yew properties for [`Comp`].
#[derive(Clone, Properties)]
pub struct Props {
    /// The origin offset of the editor.
    pub origin: f64,
    /// The radius of the duct.
    pub radius: f64,
    /// The radius of the duct.
    pub center: edge::CrossSectionPosition,
    /// The type of the duct.
    pub ty: edge::DuctType,
    /// The index of the duct in `super::Comp::state`.
    ///
    /// This is the *new* index, not the old index as in [`super::Circle::original_index`].
    pub index: usize,
}

fn duct_fill(ty: edge::DuctType) -> String {
    String::from(match ty {
        edge::DuctType::Rail(Some(edge::Direction::FromTo)) => "#900090",
        edge::DuctType::Rail(Some(edge::Direction::ToFrom)) => "#a000a0",
        edge::DuctType::Rail(None) => "#700070",
        edge::DuctType::Liquid(Some(edge::Direction::FromTo)) => "#000090",
        edge::DuctType::Liquid(Some(edge::Direction::ToFrom)) => "#0000a0",
        edge::DuctType::Liquid(None) => "#000070",
        edge::DuctType::Electricity(true) => "#909000",
        edge::DuctType::Electricity(false) => "#a0a000",
    })
}
