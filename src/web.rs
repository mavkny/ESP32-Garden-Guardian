use crate::config;
use crate::config::Reading;
use crate::control::ControlCmd;
use anyhow::Result;
use embedded_svc::http::Method;
use embedded_svc::io::Write;
use esp_idf_svc::http::server::{Configuration, EspHttpServer};
use std::sync::{mpsc::Sender, Arc, Mutex};

pub fn start_web(state: Arc<Mutex<Reading>>, ctrl_tx: Sender<ControlCmd>) -> Result<EspHttpServer<'static>> {
    let cfg = Configuration {
        stack_size: 12 * 1024,
        ..Default::default()
    };
    let mut server = EspHttpServer::new(&cfg)?;

    // --- GET / : UI ---
    {
        let _ctrl = ctrl_tx.clone();
        let _state = state.clone();
        server.fn_handler("/", Method::Get, move |req| -> anyhow::Result<()> {
            let headers = [("Content-Type", "text/html; charset=utf-8")];
            let mut resp = req.into_response(200, Some("OK"), &headers)?;
            resp.write_all(INDEX_HTML.as_bytes())?;
            Ok(())
        })?;
    }

    // --- GET /api : JSON status ---
    {
        let state = state.clone();
        server.fn_handler("/api", Method::Get, move |req| -> anyhow::Result<()> {
            let s = state.lock().unwrap().clone();
            let uptime_us = unsafe { esp_idf_sys::esp_timer_get_time() as u64 };
            let json = format!(
                "{{\"moisture\":{},\"temp_c\":{},\"pump_on\":{},\"last_pump_us\":{},\"uptime_us\":{}}}",
                s.moisture,
                s.temp_c,
                s.pump_on,
                match s.last_pump_us { Some(v) => v.to_string(), None => "null".into() },
                uptime_us
            );
            let headers = [("Content-Type", "application/json")];
            let mut resp = req.into_response(200, Some("OK"), &headers)?;
            resp.write_all(json.as_bytes())?;
            Ok(())
        })?;
    }

    {
        let ctrl = ctrl_tx.clone();
        server.fn_handler("/pump", Method::Post, move |req| -> anyhow::Result<()> {
            let _ = ctrl.send(ControlCmd::ManualPump(config::MANUAL_PUMP_SECS));
            let mut resp = req.into_response(204, Some("No Content"), &[])?;
            resp.write_all(&[])?; // leerer Body
            Ok(())
        })?;
    }

    Ok(server)
}

const INDEX_HTML: &str = r#"<!doctype html>
<html lang="de">
<meta charset="utf-8" />
<meta name="viewport" content="width=device-width,initial-scale=1" />
<title>ESP32 – Soil Control</title>
<style>
  body { font: 16px/1.4 system-ui, sans-serif; margin: 0; padding: 16px; background:#0b1020; color:#eaeef5; }
  h1 { font-weight: 600; margin: 0 0 12px; }
  .grid { display:grid; gap:12px; grid-template-columns: repeat(auto-fit, minmax(220px,1fr)); }
  .card { background:#111831; border-radius:14px; padding:14px; box-shadow: 0 1px 0 #0008 inset, 0 1px 20px #0006; }
  .kpi { font-size:28px; font-weight:700; margin-top:6px; }
  button { font-size:16px; padding:10px 14px; border-radius:10px; border:0; background:#2d5bff; color:#fff; cursor:pointer; }
  button:active { transform: translateY(1px); }
  .row { display:flex; gap:10px; align-items:center; flex-wrap:wrap; }
</style>
<h1>ESP32-KnePlant</h1>

<div class="grid">
  <div class="card">
    <div>Feuchtigkeit</div>
    <div id="moist" class="kpi">–</div>
  </div>

  <div class="card">
    <div>Temperatur</div>
    <div id="temp" class="kpi">–</div>
  </div>

  <div class="card">
    <div>Letzte Pumpenaktivierung</div>
    <div id="last" class="kpi">niemals</div>
    <div class="row" style="margin-top:10px">
      <button id="btnPump">Pumpe 5 s</button>
      <span id="pumpState" style="opacity:.8"></span>
    </div>
  </div>
</div>

<script>
async function fetchApi() {
  const r = await fetch('/api');
  return await r.json();
}
function fmtAgoSec(sec) {
  if (sec < 0.5) return 'gerade eben';
  const s = Math.floor(sec % 60);
  const m = Math.floor(sec / 60) % 60;
  const h = Math.floor(sec / 3600);
  if (h > 0) return `vor ${h}h ${m}m`;
  if (m > 0) return `vor ${m}m ${s}s`;
  return `vor ${s}s`;
}
async function refresh() {
  try {
    const d = await fetchApi();
    document.getElementById('moist').textContent = d.moisture;
    document.getElementById('temp').textContent = isFinite(d.temp_c) ? d.temp_c.toFixed(2) + ' °C' : '–';
    document.getElementById('pumpState').textContent = d.pump_on ? 'läuft…' : 'aus';

    let txt = 'niemals';
    if (d.last_pump_us !== null && typeof d.last_pump_us === 'number') {
      const deltaSec = (d.uptime_us - d.last_pump_us) / 1e6;
      txt = fmtAgoSec(deltaSec);
    }
    document.getElementById('last').textContent = txt;
  } catch (e) {
    console.log(e);
  }
}
document.getElementById('btnPump').addEventListener('click', async () => {
  try {
    await fetch('/pump', { method: 'POST' });
    setTimeout(refresh, 300);
  } catch (e) { console.log(e); }
});
refresh();
setInterval(refresh, 3000);
</script>
</html>
"#;

