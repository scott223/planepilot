use itertools::Itertools;
use tracing::event;

// share the state with external data store on a set time interval
pub(super) async fn share_with_external(
    app_state: std::sync::Arc<std::sync::Mutex<crate::types::AppState>>,
) -> anyhow::Result<()> {
    loop {
        {
            let state = app_state
                .lock()
                .expect("cannot get lock on app state")
                .clone();

            if !state.plane_state.is_empty() {
                match send_state(state).await {
                    Ok(()) => {}
                    Err(e) => {
                        event!(
                            tracing::Level::ERROR,
                            "Error while sending state to external data provider: {:}",
                            e
                        );
                    }
                }
            }
        }

        event!(
            tracing::Level::TRACE,
            "Plane state shared with external data provider"
        );

        let _ = tokio::time::sleep(tokio::time::Duration::from_millis(2000)).await;
    }
}

async fn send_state(app_state: crate::types::AppState) -> anyhow::Result<()> {
    let timestamp = chrono::Utc::now().timestamp_nanos_opt().unwrap();

    let mut line: String = "plane_state ".to_owned();

    line.push_str(
        &app_state
            .plane_state
            .iter()
            .filter(|(_k, v)| v.is_number())
            .map(|(k, v)| format!("{}={}", k, v.as_f64().unwrap()))
            .join(","),
    );

    line.push_str(" ");
    line.push_str(&timestamp.to_string());

    //line.push_str(" ");
    //line.push_str(&timestamp.to_string());

    // dbg!(line.clone());

    // let params = [("bucket", "Planepilot")];
    let client = reqwest::Client::new();
    let res = client
    .post("https://eu-central-1-1.aws.cloud2.influxdata.com/api/v2/write/?bucket=Planepilot")
    .header(reqwest::header::AUTHORIZATION, "Token fNYYFLGey5QUItHsf98hMZqJiB9f_FjzJZlPqih3UfD1QlXRy2AA4MU4p3UnxCBWXx90_FMKvHWoEmMALiQ_ew==")
    .body(line.to_string())
    .send()
    .await;

    match res {
        Ok(_r) => {
            //println!("ok {:?}", r.text().await);
        }
        Err(e) => {
            println! {"e: {}", e}
        }
    }

    let mut line: String = "autopilot_state ".to_owned();

    //using JSON to flatten the struct into a string, and then convert
    // TODO make this more idiomatic :)
    let s = serde_json::to_string(&app_state.autopilot_state).unwrap();
    let map: std::collections::HashMap<String, serde_json::Value> =
        serde_json::from_str(&s).unwrap();

    line.push_str(
        &map.iter()
            .filter(|(_k, v)| v.is_number())
            .map(|(k, v)| format!("{}={}", k, v.as_f64().unwrap()))
            .join(","),
    );

    let client = reqwest::Client::new();
    let res = client
    .post("https://eu-central-1-1.aws.cloud2.influxdata.com/api/v2/write/?bucket=Planepilot")
    .header(reqwest::header::AUTHORIZATION, "Token fNYYFLGey5QUItHsf98hMZqJiB9f_FjzJZlPqih3UfD1QlXRy2AA4MU4p3UnxCBWXx90_FMKvHWoEmMALiQ_ew==")
    .body(line.to_string())
    .send()
    .await;

    match res {
        Ok(_r) => {
            //println!("ok {:?}", r.text().await);
        }
        Err(e) => {
            println! {"e: {}", e}
        }
    }

    // line += &timestamp.timestamp().to_string();
    //dbg!(line);
    Ok(())
}
