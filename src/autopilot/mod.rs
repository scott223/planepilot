use anyhow::Context;
use tracing::{event, Level};

pub(super) async fn run_autopilot(app_state: std::sync::Arc<std::sync::Mutex<crate::types::AppState>>, tx: tokio::sync::mpsc::Sender<crate::types::Command>) -> anyhow::Result<()> {
    const MILLISECONDS_PER_LOOP: u64 = 200;

    loop {

        {
        let mut state = app_state.lock().expect("cannot get lock on the state");

        let mut are_we_flying: bool = false;
        if state.plane_state.contains_key("last_updated_timestamp") {
            
            if state.plane_state.get("last_updated_timestamp").context("cannot get last_update_timestamp")?.as_i64().context("cannot convert timestamp to i64")? > (chrono::Utc::now().timestamp_millis() - 1000) {
                    are_we_flying = true;


            }
        }

        if are_we_flying {
            if !state.autopilot_state.are_we_flying {
                    state.autopilot_state.are_we_flying = true;
                    event!(Level::INFO, "Recent (< 1 sec) plane data available, setting to flying!");
            }



        } else {
            if state.autopilot_state.are_we_flying {
                state.autopilot_state.set_autopilot_to_standby_and_clean_parameters();

                event!(Level::ERROR, "No recent (< 1 sec) plane data available, disabling autopilot and setting to not flying!");
                
            }
        }

        /* 

        match update_state(&app_state_proxy).await {
            Ok(plane_state) => {
                app_state_proxy.set_plane_state(plane_state).await?;
                event!(Level::TRACE, "Plane state updated");

                if !local_error_state {
                    local_error_state = true;
                    app_state_proxy.set_flying(true).await?;
                    event!(
                        Level::INFO,
                        "Connection to planeconnector achieved and state updated"
                    );
                }
            }
            Err(e) => {
                if local_error_state {
                    local_error_state = false;
                    app_state_proxy.set_flying(false).await?;
                    app_state_proxy.clear_plane_state().await?;
                    app_state_proxy.activate_vertical_standby_mode().await?;
                    app_state_proxy.activate_horizontal_standby_mode().await?;

                    event!(
                        Level::ERROR,
                        "Error when updating state so autopilot set to standby: {:?}",
                        e
                    );
                }
            }
        };

        let auto_pilot_state: types::AutoPilotState =
            app_state_proxy.get_auto_pilot_state().await?;

        // refresh the constants now every cycle, to iterate fast
        app_state_proxy.refresh_autopilot_constants().await?;

        if auto_pilot_state.are_we_flying {
            let plane_state: PlaneStateStruct = app_state_proxy.get_plane_state_as_struct().await?;

            let dt: f64 = MILLISECONDS_PER_LOOP as f64 / 1000.0;

            verticalguidance::execute_vertical_guidance(
                dt,
                &reqwest_client,
                &app_state_proxy,
                &auto_pilot_state,
                &plane_state,
            )
            .await?;

            horizontalguidance::execute_horizontal_guidance(
                dt,
                &reqwest_client,
                &app_state_proxy,
                &auto_pilot_state,
                &plane_state,
            )
            .await?
        }

    }

    */

        }

        let _ = tokio::time::sleep(tokio::time::Duration::from_millis(MILLISECONDS_PER_LOOP)).await;
    }
}
