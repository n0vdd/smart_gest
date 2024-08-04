use anyhow::Context;
use tracing::{error, warn};


//TODO this could all be done with reqwest i think
pub async fn checa_voip_down() -> Result<(),anyhow::Error>{
    let client = reqwest::Client::new();
    //Check all the devices
    let res = client.get("http://172.27.27.37:7557/devices?projection=InternetGatewayDevice.Services.VoiceService.1.VoiceProfile.1.Line.1.Enable,InternetGatewayDevice.Services.VoiceService.1.VoiceProfile.1.Line.1.Status,_id")
        .send().await.context("Failed to get routers")?
        .text().await.context("Failed to get routers text")?;
    let routers: serde_json::Value = serde_json::from_str(&res).context("Failed to parse routers JSON")?;
    let routers = routers.as_array().context("Expected an array of routers")?;
    //TODO loop for each router, checking if the line is enabled and if the status is not registered
    //Reboot the router
    for router in routers {
        if let Some(router_id) = router.get("_id").and_then(serde_json::Value::as_str) {
            if let Some(enable) = router.pointer("/InternetGatewayDevice/Services/VoiceService/1/VoiceProfile/1/Line/1/Enable/_value").and_then(serde_json::Value::as_str) {
                if let Some(status) = router.pointer("/InternetGatewayDevice/Services/VoiceService/1/VoiceProfile/1/Line/1/Status/_value").and_then(serde_json::Value::as_str) {
                    //if the voip is enabled bu it is not up
                    if enable == "Enabled" && status != "Up" {
                        warn!("Rebooting router with VoIP line not registered: {} ({})", router_id, status);

                        //reset the router
                        let reset_url = format!("http://172.27.27.37:7557/devices/{}/tasks?timeout=3000&connection_request", router_id);
                        client.post(&reset_url)
                        .json(&serde_json::json!({ "name": "reboot" }))
                        .send().await.map_err(|e| {
                            error!("Failed to reset router: {}", e);
                            anyhow::anyhow!("Failed to reset router: {}", e)
                        })?;
                    }
                }
            }
        }
    }
    Ok(())
}