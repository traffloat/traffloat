//! Renders a single duct.

use std::f64::consts::FRAC_1_SQRT_2;

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
        match msg {
            Msg::MouseDown(event) => {
                self.props.mouse_down.emit(event);
                false
            }
            Msg::MouseUp(event) => {
                self.props.mouse_up.emit(event);
                false
            }
        }
    }

    fn change(&mut self, props: Props) -> ShouldRender {
        self.props = props;
        true
    }

    fn view(&self) -> Html {
        const DISABLED_RADIUS: f64 = 0.6;
        const DISABLED_WIDTH: f64 = 0.15;

        html! {
            <>
                <circle
                    cx=(self.props.origin + self.props.center.vector().x).to_string()
                    cy=(self.props.origin + self.props.center.vector().y).to_string()
                    r=self.props.radius.to_string()
                    fill=duct_fill(self.props.ty)
                    style=style!("cursor": "pointer")
                    onmousedown=self.link.callback(Msg::MouseDown)
                    onmouseup=self.link.callback(Msg::MouseUp)
                    />
                { for (!self.props.ty.active()).then(|| html! {
                    <>
                        <circle
                            cx=(self.props.origin + self.props.center.vector().x).to_string()
                            cy=(self.props.origin + self.props.center.vector().y).to_string()
                            r=(self.props.radius * (DISABLED_RADIUS - DISABLED_WIDTH / 2.)).to_string()
                            stroke="red"
                            stroke-width=(self.props.radius * DISABLED_WIDTH).to_string()
                            fill="none"
                            style=style!("pointer-events": "none;")
                            />
                        <line
                            x1=(self.props.origin + self.props.center.vector().x + self.props.radius * DISABLED_RADIUS * FRAC_1_SQRT_2).to_string()
                            y1=(self.props.origin + self.props.center.vector().y + self.props.radius * DISABLED_RADIUS * FRAC_1_SQRT_2).to_string()
                            x2=(self.props.origin + self.props.center.vector().x - self.props.radius * DISABLED_RADIUS * FRAC_1_SQRT_2).to_string()
                            y2=(self.props.origin + self.props.center.vector().y - self.props.radius * DISABLED_RADIUS * FRAC_1_SQRT_2).to_string()
                            stroke="red"
                            stroke-width=(self.props.radius * DISABLED_WIDTH).to_string()
                            style=style!("pointer-events": "none;")
                            />
                    </>
                }) }
                { for (self.props.ty.direction() == Some(edge::Direction::FromTo)).then(|| html! {
                    <polygon
                        points="\
                            -4,3 -3,4 0,1 \
                            3,4 4,3 1,0 \
                            4,-3 3,-4 0,-1 \
                            -3,-4 -4,-3 -1,0 \
                        "
                        fill="black"
                        transform=format!(
                            "\
                                translate({} {}) \
                                scale({}) \
                            ",
                            self.props.origin + self.props.center.vector().x,
                            self.props.origin + self.props.center.vector().y,
                            0.2 * FRAC_1_SQRT_2 * self.props.radius,
                        )
                        style=style!("pointer-events": "none;")
                        />
                }) }
                { for (self.props.ty.direction() == Some(edge::Direction::ToFrom)).then(|| html! {
                    <circle
                        cx=(self.props.origin + self.props.center.vector().x).to_string()
                        cy=(self.props.origin + self.props.center.vector().y).to_string()
                        r=(self.props.radius * 0.25).to_string()
                        fill="black"
                        style=style!("pointer-events": "none;")
                        />
                }) }
            </>
        }
    }
}

/// Events for [`Comp`].
pub enum Msg {
    /// When the user presses down the circle.
    MouseDown(MouseEvent),
    /// When the user releases mouse on the circle.
    MouseUp(MouseEvent),
}

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

    /// When the user presses down the circle.
    pub mouse_down: Callback<MouseEvent>,
    /// When the user releases mouse on the circle.
    pub mouse_up: Callback<MouseEvent>,
}

fn duct_fill(ty: edge::DuctType) -> String {
    String::from(match ty {
        edge::DuctType::Rail(..) => "#aa44bb",
        edge::DuctType::Liquid { .. } => "#3322aa",
        edge::DuctType::Electricity(..) => "#ccaa00",
    })
}
