//! Renders an icon in an atlas.

use yew::prelude::*;

use safety::Safety;

/// Displays an editor for ducts in an edge.
pub struct Comp {
    props: Props,
    _link: ComponentLink<Self>,
}

impl Component for Comp {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Props, link: ComponentLink<Self>) -> Self {
        Self { props, _link: link }
    }

    fn update(&mut self, msg: Msg) -> ShouldRender {
        match msg {}
    }

    fn change(&mut self, props: Props) -> ShouldRender {
        if self.props == props {
            return false;
        }
        self.props = props;
        true
    }

    fn view(&self) -> Html {
        html! {
            <span
                style=format!(
                    "
                        background-image: url('{url}');
                        background-repeat: no-repeat;
                        display: inline-block;
                        background-size: {atlas_x}px {atlas_y}px;
                        width: {size_x}px;
                        height: {size_y}px;
                        background-position: {pos_x}px {pos_y}px;
                        font-size: 0;
                        vertical-align: text-bottom;
                    ",
                    url=self.props.atlas_path,
                    atlas_x=self.props.scaled_atlas_x(),
                    atlas_y=self.props.scaled_atlas_y(),
                    size_x=self.props.scaled_size_x(),
                    size_y=self.props.scaled_size_y(),
                    pos_x=-self.props.scaled_pos_x().homosign(),
                    pos_y=-self.props.scaled_pos_y().homosign(),
                )
                title=self.props.text.clone()
            >
                { &self.props.text }
            </span>
        }
    }
}

/// Events for [`Comp`].
pub enum Msg {}

/// Yew properties for [`Comp`].
#[derive(Clone, Properties, PartialEq)]
pub struct Props {
    /// The path to the atlas.
    pub atlas_path: String,
    /// Number of horizontal pixels in the atlas image.
    pub atlas_width: u32,
    /// Number of vertical pixels in the atlas image.
    pub atlas_height: u32,
    /// Number of horizontal pixels from the atlas origin to the sprite origin.
    pub x0: u32,
    /// Number of vertical pixels from the atlas origin to the sprite origin.
    pub y0: u32,
    /// Number of horizontal pixels from the atlas origin to the sprite end.
    pub x1: u32,
    /// Number of vertical pixels from the atlas origin to the sprite end.
    pub y1: u32,
    /// Rendered width.
    pub out_width: u32,
    /// Rendered height.
    pub out_height: u32,
    /// Copyable text.
    pub text: String,
}

impl Props {
    fn unscaled_x(&self) -> u32 {
        self.x1 - self.x0
    }
    fn unscaled_y(&self) -> u32 {
        self.y1 - self.y0
    }

    fn scaled_atlas_x(&self) -> u32 {
        self.atlas_width * self.out_width / self.unscaled_x()
    }
    fn scaled_atlas_y(&self) -> u32 {
        self.atlas_height * self.out_width / self.unscaled_y()
    }
    fn scaled_pos_x(&self) -> u32 {
        self.x0 * self.out_width / self.unscaled_x()
    }
    fn scaled_pos_y(&self) -> u32 {
        self.y0 * self.out_height / self.unscaled_y()
    }
    fn scaled_size_x(&self) -> u32 {
        self.out_width
    }
    fn scaled_size_y(&self) -> u32 {
        self.out_height
    }
}
