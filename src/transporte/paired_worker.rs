//! Opt-in Linux worker: no socket, no persisted session, no captured URL.
use super::{RespostaWeb, read_box_via};
use serde::Deserialize;
use serde_json::json;
use std::collections::BTreeMap;
use std::fs;
use std::io::Cursor;
use std::path::Path;
use std::time::{Duration, Instant};

#[derive(Deserialize)]
struct Job {
    body: String,
    cookie: String,
    mime: String,
    unit: String,
    repetitions: usize,
    delay_ms: u64,
}

#[repr(C)]
struct Timespec {
    seconds: std::ffi::c_long,
    nanos: std::ffi::c_long,
}

unsafe extern "C" {
    fn clock_gettime(clock: std::ffi::c_int, value: *mut Timespec) -> std::ffi::c_int;
}

fn cpu_ns() -> u64 {
    let mut value = Timespec {
        seconds: 0,
        nanos: 0,
    };
    // Linux CLOCK_PROCESS_CPUTIME_ID=2; valid aligned writable timespec.
    assert_eq!(unsafe { clock_gettime(2, &raw mut value) }, 0);
    u64::try_from(value.seconds).unwrap() * 1_000_000_000 + u64::try_from(value.nanos).unwrap()
}

#[test]
#[ignore = "private paired benchmark requires a verified cassette job"]
fn paired_unit_worker() {
    let file = std::env::var("IES_PAIR_JOB").expect("explicit local job required");
    let job: Job = serde_json::from_slice(&fs::read(file).unwrap()).unwrap();
    assert!((5..=100).contains(&job.repetitions));
    assert!(job.delay_ms <= 100);
    let body = fs::read(&job.body).unwrap();
    assert!(body.len() <= crate::MAX_HTML_BYTES);
    let origin = "https://sei.treinamento.unb.br/sei";
    let url = format!(
        "{origin}/controlador.php?acao=procedimento_controlar&infra_hash={}",
        "a".repeat(64)
    );
    let mut samples = Vec::new();
    let mut expected = None;
    for index in 0..job.repetitions + 3 {
        let (mut requests, mut response_bytes, mut wait_ns) = (0, 0, 0);
        let start = Instant::now();
        let cpu = cpu_ns();
        let html = read_box_via(&url, Path::new(&job.cookie), origin, |_, _| {
            requests += 1;
            let wait = Instant::now();
            if job.delay_ms != 0 {
                std::thread::sleep(Duration::from_millis(job.delay_ms));
            }
            wait_ns = wait.elapsed().as_nanos();
            response_bytes += body.len();
            Ok(RespostaWeb {
                status_code: 200,
                mime: job.mime.clone(),
                content_length: Some(body.len()),
                corpo: Cursor::new(&body),
            })
        })
        .unwrap();
        let reading = crate::paginas::juntar(&[html], Some(&job.unit)).unwrap();
        assert!(reading.recebidos.completa(&reading.caixa.recebidos));
        assert!(reading.gerados.completa(&reading.caixa.gerados));
        let serialized = serde_json::to_vec(&crate::saida::json(&reading, "paired-mock")).unwrap();
        let sample = json!({
            "total_ns": start.elapsed().as_nanos(), "cpu_ns": cpu_ns() - cpu,
            "mock_wait_ns": wait_ns, "requests": requests,
            "response_bytes": response_bytes,
        });
        assert_ne!(serialized.len(), 0);
        assert_eq!(requests, 1);
        let mut seen = BTreeMap::new();
        let mut rows = Vec::new();
        for process in reading
            .caixa
            .recebidos
            .processos
            .iter()
            .chain(&reading.caixa.gerados.processos)
        {
            if let Some(previous) = seen.insert(&process.numero, process.id.as_str()) {
                assert_eq!(previous, process.id.as_str());
            } else {
                rows.push((process.id.as_str(), &process.numero));
            }
        }
        let value = json!({"unidade": reading.unidade, "processos": rows});
        if let Some(previous) = &expected {
            assert_eq!(previous, &value);
        }
        expected = Some(value);
        if index >= 3 {
            samples.push(sample);
        }
    }
    let status = fs::read_to_string("/proc/self/status").unwrap();
    let rss: u64 = status
        .lines()
        .find(|line| line.starts_with("VmHWM:"))
        .unwrap()
        .split_whitespace()
        .nth(1)
        .unwrap()
        .parse()
        .unwrap();
    println!(
        "PAIR_JSON={}",
        json!({
            "canonical": expected.unwrap(), "samples": samples, "rss_peak_kib": rss,
        })
    );
}
