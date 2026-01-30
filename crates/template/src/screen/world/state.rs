use crate::prelude::*;

pub fn plugin(app: &mut App) {
    app.add_loading_state(
        LoadingState::new(ScreenDataRef::<WorldScreen>::Loading)
            .continue_to_state(ScreenDataRef::<WorldScreen>::Ready)
            .load_collection::<PlayerAssets>(),
    );
}
