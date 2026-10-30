use base64::{Engine as _, engine::general_purpose::STANDARD};
use sha1::{Digest as _, Sha1};
use sha2::Sha256;
use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;

const HTML: &str = r#"<html><body>
<a id="lnkInfraUnidade">TESTE</a>
<table id="tblProcessosRecebidos"><caption>2 registros:</caption>
<tr><th>Número</th></tr>
<tr id="P42"><td><a class="processoVisualizado" href="controlador.php?acao=procedimento_trabalhar&amp;infra_hash=SEGREDO">23106.000001/2026-01</a></td></tr>
<tr id="P43"><td><a class="processoNaoVisualizado" href="controlador.php?acao=procedimento_trabalhar">23106.000002/2026-02</a></td></tr>
</table>
<table id="tblProcessosGerados"><caption>0 registros:</caption></table>
</body></html>"#;

fn preparar_fixture_com_html(html: &str) -> (PathBuf, PathBuf, PathBuf) {
    let folder = std::env::temp_dir().join(format!(
        "ies-replay-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir_all(&folder).unwrap();
    let zip = folder.join(".cache/ies/todos-sei-cassettes-unb.zip");
    fs::create_dir_all(zip.parent().unwrap()).unwrap();
    let member = "unidade-processos-unb.json";
    let cassette = serde_json::json!({
        "complete": true, "schema_version": 1,
        "sei_root": "https://sei.treinamento.unb.br/",
        "interactions": [{
            "request":{"method":"GET","operation":"procedimento_controlar"},
            "response":{
                "status_code":200,
                "encoding":"utf-8",
                "headers":{"content-type":"text/html; charset=utf-8"},
                "body_base64": STANDARD.encode(html.as_bytes())
            }
        }]
    });
    let json_bytes = cassette.to_string();
    let mut writer = zip::ZipWriter::new(File::create(&zip).unwrap());
    let options = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);
    writer.start_file(member, options).unwrap();
    writer.write_all(json_bytes.as_bytes()).unwrap();
    writer.finish().unwrap();

    let card = folder.join("cassettes/unb/replay--synthetic.md");
    fs::create_dir_all(card.parent().unwrap()).unwrap();
    fs::write(
        &card,
        format!(
            "---\ntype: Cassette\ninstallation: unb\nsource_zip: https://archive.org/download/todos-sei-cassettes-2026/todos-sei-cassettes-unb.zip\narchive_member: {member}\narchive_sha1: {:x}\ncassette_sha256_lf: {:x}\n---\n\n# Replay\n",
            Sha1::digest(fs::read(&zip).unwrap()),
            Sha256::digest(json_bytes.as_bytes()),
        ),
    )
    .unwrap();
    (folder, card, zip)
}

fn preparar_fixture() -> (PathBuf, PathBuf, PathBuf) {
    preparar_fixture_com_html(HTML)
}

#[test]
fn replay_valido_lê_duas_tabelas_sem_url_assinada() {
    let (folder, _card, _zip) = preparar_fixture();
    let output = Command::new(env!("CARGO_BIN_EXE_ies"))
        .args([
            "replay",
            "replay",
            "--acervo",
            folder.to_str().unwrap(),
            "--json",
        ])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    let result: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(result["recebidos"].get("completa").is_none());
    assert!(result["gerados"].get("completa").is_none());
    assert_eq!(result["recebidos"]["total_declarado"], 2);
    assert_eq!(result["gerados"]["total_declarado"], 0);
    assert_eq!(result["recebidos"]["processos"][0]["id"], "42");
    assert_eq!(result["recebidos"]["processos"][1]["visualizado"], false);
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(!stdout.contains("SEGREDO"));
    assert!(!stdout.contains("infra_hash"));
    fs::remove_dir_all(folder).unwrap();
}

#[test]
fn hash_adulterado_é_recusado() {
    let (folder, card, _zip) = preparar_fixture();
    let text = fs::read_to_string(&card)
        .unwrap()
        .replace("archive_sha1: ", "archive_sha1: 0000");
    fs::write(&card, text).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_ies"))
        .args([
            "replay",
            "replay",
            "--acervo",
            folder.to_str().unwrap(),
            "--json",
        ])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
    let result: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(result["codigo"], "fonte_invalida");
    fs::remove_dir_all(folder).unwrap();
}

#[test]
fn comando_sem_configuracao_informa_arquivo_de_controle_ausente() {
    let output = Command::new(env!("CARGO_BIN_EXE_ies"))
        .args(["unidade", "listar-processos", "--json"])
        .env_remove("IES_URL_CONTROLE_FILE")
        .env_remove("IES_COOKIE_FILE")
        .env_remove("IES_ORIGEM_FILE")
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
    let result: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(result["subcodigo"], "caminho_url_controle_nao_informado");
}

fn preparar_fixture_paginas() -> PathBuf {
    use std::time::{SystemTime, UNIX_EPOCH};
    let folder = std::env::temp_dir().join(format!(
        "ies-paginas-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir_all(&folder).unwrap();
    let zip = folder.join(".cache/ies/todos-sei-cassettes-unb.zip");
    fs::create_dir_all(zip.parent().unwrap()).unwrap();
    let member = "paginas-unb.json";
    let build = |inicio: u32, fim: u32| {
        let lines = (inicio..=fim).fold(String::new(), |mut linhas, id| {
            use std::fmt::Write as _;
            write!(
                linhas,
                "<tr id=\"P{id}\"><td><a class=\"processoVisualizado\">NUP-{id}</a></td></tr>"
            )
            .unwrap();
            linhas
        });
        format!(
            "<html><a id=\"lnkInfraUnidade\">TESTE</a>\
             <input id=\"hdnTipoVisualizacao\" value=\"R\">\
             <input id=\"hdnRecebidosHashCriterios\" value=\"recebidos\">\
             <input id=\"hdnGeradosHashCriterios\" value=\"gerados\">\
             <table id=\"tblProcessosRecebidos\"><caption>4 registros - {inicio} a {fim}):</caption>{lines}</table>\
             <table id=\"tblProcessosGerados\"><caption>0 registros:</caption></table></html>"
        )
    };
    let interactions = [build(1,2), build(3,4)].iter().enumerate().map(|(index, html)| {
        serde_json::json!({
            "request": {"method": if index == 0 { "GET" } else { "POST" }, "operation": "procedimento_controlar"},
            "response": {
                "status_code":200,"encoding":"utf-8",
                "headers":{"content-type":"text/html"},
                "body_base64":STANDARD.encode(html.as_bytes())
            }
        })
    }).collect::<Vec<_>>();
    let bytes = serde_json::json!({
        "complete":true,"schema_version":1,
        "sei_root":"https://sei.treinamento.unb.br/",
        "interactions": interactions
    })
    .to_string();
    let mut writer = zip::ZipWriter::new(File::create(&zip).unwrap());
    writer
        .start_file(
            member,
            zip::write::SimpleFileOptions::default()
                .compression_method(zip::CompressionMethod::Deflated),
        )
        .unwrap();
    writer.write_all(bytes.as_bytes()).unwrap();
    writer.finish().unwrap();
    let card = folder.join("cassettes/unb/paginas--synthetic.md");
    fs::create_dir_all(card.parent().unwrap()).unwrap();
    fs::write(&card,format!(
        "---\ntype: Cassette\ninstallation: unb\n\
         source_zip: https://archive.org/download/todos-sei-cassettes-2026/todos-sei-cassettes-unb.zip\n\
         archive_member: {member}\narchive_sha1: {:x}\ncassette_sha256_lf: {:x}\n---\n",
        Sha1::digest(fs::read(&zip).unwrap()),
        Sha256::digest(bytes.as_bytes())
    )).unwrap();
    folder
}

#[test]
fn replay_de_multiplos_passos_consolida_faixas_sem_post() {
    let folder = preparar_fixture_paginas();
    let output = Command::new(env!("CARGO_BIN_EXE_ies"))
        .args([
            "replay",
            "paginas",
            "--acervo",
            folder.to_str().unwrap(),
            "--passo",
            "0",
            "--passo",
            "1",
            "--json",
        ])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(value["recebidos"].get("completa").is_none());
    assert!(value["gerados"].get("completa").is_none());
    assert_eq!(value["recebidos"]["processos"].as_array().unwrap().len(), 4);
    assert_eq!(value["recebidos"]["processos"][3]["id"], "4");
    let no_selection = Command::new(env!("CARGO_BIN_EXE_ies"))
        .args([
            "replay",
            "paginas",
            "--acervo",
            folder.to_str().unwrap(),
            "--json",
        ])
        .output()
        .unwrap();
    assert_eq!(no_selection.status.code(), Some(0));
    let partial: serde_json::Value = serde_json::from_slice(&no_selection.stdout).unwrap();
    assert!(partial["recebidos"].get("completa").is_none());
    assert!(partial["gerados"].get("completa").is_none());
    let repeat = Command::new(env!("CARGO_BIN_EXE_ies"))
        .args([
            "replay",
            "paginas",
            "--acervo",
            folder.to_str().unwrap(),
            "--passo",
            "0",
            "--passo",
            "0",
            "--json",
        ])
        .output()
        .unwrap();
    assert_eq!(repeat.status.code(), Some(1));
    fs::remove_dir_all(folder).unwrap();
}

#[test]
fn replay_todas_paginas_consolida_get_e_post_sem_enviar_requisicoes() {
    let folder = preparar_fixture_paginas();
    let output = Command::new(env!("CARGO_BIN_EXE_ies"))
        .args([
            "replay",
            "paginas",
            "--acervo",
            folder.to_str().unwrap(),
            "--todas-paginas",
            "--json",
        ])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    let resultado: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(
        resultado["recebidos"]["processos"]
            .as_array()
            .unwrap()
            .len(),
        4
    );
    assert_eq!(resultado["recebidos"]["processos"][3]["id"], "4");
    fs::remove_dir_all(folder).unwrap();
}

#[test]
fn contrato_json_v1_e_deterministico() {
    let (pasta, _card, _arquivo) = preparar_fixture();
    let resposta = Command::new(env!("CARGO_BIN_EXE_ies"))
        .args([
            "replay",
            "replay",
            "--acervo",
            pasta.to_str().unwrap(),
            "--unidade",
            "TESTE",
            "--json",
        ])
        .output()
        .unwrap();
    assert_eq!(resposta.status.code(), Some(0), "{resposta:?}");
    assert_eq!(
        std::str::from_utf8(&resposta.stdout).unwrap(),
        include_str!("fixtures/caixa-v2.json"),
    );
    let valor: serde_json::Value = serde_json::from_slice(&resposta.stdout).unwrap();
    assert!(valor.get("completude").is_none());
    assert!(valor["recebidos"].get("linhas_lidas").is_none());
    assert!(valor["recebidos"].get("paginas_lidas").is_none());
    assert!(!String::from_utf8_lossy(&resposta.stdout).contains("infra_hash"));
    fs::remove_dir_all(pasta).unwrap();
}

#[test]
fn parser_repassa_subcodigo_sem_vazar_html() {
    let html = "<table id='tblProcessosRecebidos'><caption>1 registros:</caption>        <tr id='P8'><td><a class='outraClasse' href='?infra_hash=SEGREDO'>123</a></td></tr></table>        <table id='tblProcessosGerados'><caption>0 registros:</caption></table>";
    let (pasta, _card, _zip) = preparar_fixture_com_html(html);
    let resposta = Command::new(env!("CARGO_BIN_EXE_ies"))
        .args([
            "replay",
            "replay",
            "--acervo",
            pasta.to_str().unwrap(),
            "--json",
        ])
        .output()
        .unwrap();
    assert_eq!(resposta.status.code(), Some(1));
    let data: serde_json::Value = serde_json::from_slice(&resposta.stdout).unwrap();
    assert_eq!(data["codigo"], "tela_inesperada");
    assert_eq!(
        data["subcodigo"],
        "link_do_processo_nao_identificado_na_linha"
    );
    assert_eq!(data["id"], "8");
    let stdout = String::from_utf8(resposta.stdout).unwrap();
    assert!(!stdout.contains("SEGREDO"));
    assert!(!stdout.contains("infra_hash"));
    fs::remove_dir_all(pasta).unwrap();
}

#[test]
fn unidade_solicitada_precisa_ser_confirmada_pelo_sei() {
    let (pasta, _card, _zip) = preparar_fixture();
    let resposta = Command::new(env!("CARGO_BIN_EXE_ies"))
        .args([
            "replay",
            "replay",
            "--acervo",
            pasta.to_str().unwrap(),
            "--unidade",
            "OUTRA",
            "--json",
        ])
        .output()
        .unwrap();
    assert_eq!(resposta.status.code(), Some(1));
    let dado: serde_json::Value = serde_json::from_slice(&resposta.stdout).unwrap();
    assert_eq!(dado["subcodigo"], "unidade_inesperada");
    fs::remove_dir_all(pasta).unwrap();
}

#[test]
fn origem_http_e_obrigatoria_antes_de_qualquer_acesso() {
    let resposta = Command::new(env!("CARGO_BIN_EXE_ies"))
        .args(["unidade", "listar-processos", "--json"])
        .env("IES_URL_CONTROLE_FILE", "qualquer-arquivo-de-url")
        .env("IES_COOKIE_FILE", "qualquer-arquivo-de-cookie")
        .env_remove("IES_ORIGEM_FILE")
        .output()
        .unwrap();
    assert_eq!(resposta.status.code(), Some(1));
    let dados: serde_json::Value = serde_json::from_slice(&resposta.stdout).unwrap();
    assert_eq!(dados["subcodigo"], "caminho_origem_nao_informado");
}

#[test]
fn url_adulterada_falha_antes_de_abrir_cookies_ou_enviar_http() {
    let dir = std::env::temp_dir().join(format!(
        "ies-origem-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir_all(&dir).unwrap();
    let url = dir.join("url");
    let origem = dir.join("origem");
    fs::write(
        &url,
        format!(
            "https://sei.adulterado.tld/sei/controlador.php?acao=procedimento_controlar&infra_hash={}",
            "a".repeat(64)
        ),
    ).unwrap();
    fs::write(&origem, "https://sei.treinamento.unb.br").unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        for p in [&url, &origem] {
            fs::set_permissions(p, fs::Permissions::from_mode(0o600)).unwrap();
        }
    }
    let resposta = Command::new(env!("CARGO_BIN_EXE_ies"))
        .args(["unidade", "listar-processos", "--json"])
        .env("IES_URL_CONTROLE_FILE", url.to_str().unwrap())
        .env("IES_ORIGEM_FILE", origem.to_str().unwrap())
        .env("IES_COOKIE_FILE", dir.join("cookie-inexistente"))
        .output()
        .unwrap();
    assert_eq!(resposta.status.code(), Some(1));
    let json: serde_json::Value = serde_json::from_slice(&resposta.stdout).unwrap();
    assert_eq!(json["codigo"], "fonte_invalida");
    assert_eq!(json["subcodigo"], "origem_nao_autorizada");
    assert!(!String::from_utf8_lossy(&resposta.stdout).contains("infra_hash"));
    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn replay_busca_processo_pelo_nup_opaco_sem_consultar_rede() {
    let (pasta, _card, _zip) = preparar_fixture();
    let resposta = Command::new(env!("CARGO_BIN_EXE_ies"))
        .args([
            "replay",
            "replay",
            "--acervo",
            pasta.to_str().unwrap(),
            "--numero-processo",
            "23106.000001/2026-01",
            "--json",
        ])
        .output()
        .unwrap();
    assert_eq!(resposta.status.code(), Some(0), "{resposta:?}");
    let json: serde_json::Value = serde_json::from_slice(&resposta.stdout).unwrap();
    assert_eq!(json["numero"], "23106.000001/2026-01");
    assert_eq!(json["escopo"], "resumo_cassette_caixa");
    assert_eq!(json["instalacao"], "unb");
    assert_eq!(json["ocorrencias"].as_array().unwrap().len(), 1);
    assert_eq!(json["ocorrencias"][0]["categoria"], "recebidos");
    assert!(!String::from_utf8_lossy(&resposta.stdout).contains("SEGREDO"));
    fs::remove_dir_all(pasta).unwrap();
}

#[test]
fn replay_busca_nup_ausente_na_caixa_inteira_nao_afirma_inexistencia() {
    let (pasta, _card, _zip) = preparar_fixture();
    let resposta = Command::new(env!("CARGO_BIN_EXE_ies"))
        .args([
            "replay",
            "replay",
            "--acervo",
            pasta.to_str().unwrap(),
            "--numero-processo",
            "000123",
            "--json",
        ])
        .output()
        .unwrap();
    assert_eq!(resposta.status.code(), Some(1));
    let json: serde_json::Value = serde_json::from_slice(&resposta.stdout).unwrap();
    assert_eq!(json["subcodigo"], "numero_nao_observado_na_caixa");
    fs::remove_dir_all(pasta).unwrap();
}

#[test]
fn replay_busca_nup_recusa_cobertura_incompleta_e_aceita_paginas_explicitas() {
    let pasta = preparar_fixture_paginas();
    let consulta = |extra: &[&str]| {
        Command::new(env!("CARGO_BIN_EXE_ies"))
            .args([
                "replay",
                "paginas",
                "--acervo",
                pasta.to_str().unwrap(),
                "--numero-processo",
                "NUP-3",
                "--json",
            ])
            .args(extra)
            .output()
            .unwrap()
    };
    let parcial = consulta(&[]);
    assert_eq!(parcial.status.code(), Some(1));
    let erro: serde_json::Value = serde_json::from_slice(&parcial.stdout).unwrap();
    assert_eq!(erro["subcodigo"], "paginas_do_controle_nao_obtidas");
    let integral = consulta(&["--passo", "0", "--passo", "1"]);
    assert_eq!(integral.status.code(), Some(0), "{integral:?}");
    let resultado: serde_json::Value = serde_json::from_slice(&integral.stdout).unwrap();
    assert_eq!(resultado["ocorrencias"][0]["processo"]["numero"], "NUP-3");
    fs::remove_dir_all(pasta).unwrap();
}

#[test]
fn replay_busca_nup_com_caractere_de_controle_retorna_erro_tipado() {
    let (pasta, _card, _zip) = preparar_fixture();
    let resposta = Command::new(env!("CARGO_BIN_EXE_ies"))
        .args([
            "replay",
            "replay",
            "--acervo",
            pasta.to_str().unwrap(),
            "--numero-processo",
            "00123\nSEGREDO",
            "--json",
        ])
        .output()
        .unwrap();
    assert_eq!(resposta.status.code(), Some(2));
    let json: serde_json::Value = serde_json::from_slice(&resposta.stdout).unwrap();
    assert_eq!(json["subcodigo"], "numero_processo_controle");
    assert!(!String::from_utf8_lossy(&resposta.stdout).contains("SEGREDO"));
    fs::remove_dir_all(pasta).unwrap();
}
