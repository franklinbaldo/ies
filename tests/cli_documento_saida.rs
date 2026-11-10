//! `--saida` e a prévia de `documento salvar` sem sessão nem rede (#207):
//! a foto `DOC.vN.html` conferida antes da rede e o par vN / v(N+1), offline.
use std::process::Command;

#[path = "support/falha.rs"]
mod falha_cli;
use falha_cli::falha;

fn ies_sem_sessao(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_ies"))
        .args(args)
        .env_remove("IES_CONFIG_FILE")
        .env_remove("IES_URL_CONTROLE_FILE")
        .env_remove("IES_COOKIE_FILE")
        .env_remove("IES_ORIGEM_FILE")
        .output()
        .expect("IES deve iniciar")
}

#[test]
fn documento_saida_confere_a_foto_e_o_diretorio_antes_da_sessao() {
    // #207: com `--versao N` o nome `DOC.vN.html` é conhecido antes da
    // rede; uma foto presente que declara outro documento é recusada.
    let pasta = std::env::temp_dir().join(format!("ies-foto-versao-{}", std::process::id()));
    std::fs::create_dir_all(&pasta).unwrap();
    let outra =
        "<html><head><title>SEI/X - 0099999 - Despacho</title></head><body><p>x</p></body></html>";
    let alvo = pasta.join("0011519.v2.html");
    std::fs::write(&alvo, outra).unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&alvo, std::fs::Permissions::from_mode(0o600)).unwrap();
    }
    let dir = pasta.to_str().unwrap();
    let args = [
        "documento",
        "0011519",
        "--versao",
        "2",
        "--saida",
        dir,
        "--json",
    ];
    let output = ies_sem_sessao(&args);
    falha(&output, "FotoDivergente");
    let ausente = pasta.join("nao-existe");
    let args = [
        "documento",
        "0011519",
        "--saida",
        ausente.to_str().unwrap(),
        "--json",
    ];
    let output = ies_sem_sessao(&args);
    falha(&output, "DiretorioDeSaida");
    std::fs::remove_dir_all(pasta).unwrap();
}

/// Grava `DOC.vN.html` privado (0600) em `pasta`.
fn versao_local(pasta: &std::path::Path, n: u32, corpo: &str) {
    let alvo = pasta.join(format!("0011519.v{n}.html"));
    std::fs::write(&alvo, corpo).unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&alvo, std::fs::Permissions::from_mode(0o600)).unwrap();
    }
}

/// `ies documento salvar` no diretório `pasta`, sem nenhum arquivo de
/// configuração ou sessão: `IES_CONFIG_FILE` aponta para um arquivo que
/// não existe e as demais variáveis somem, como numa máquina limpa.
fn salvar_offline(pasta: &std::path::Path, extra: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_ies"))
        .current_dir(pasta)
        .env("IES_CONFIG_FILE", pasta.join("nao-existe.json"))
        .env_remove("IES_URL_CONTROLE_FILE")
        .env_remove("IES_COOKIE_FILE")
        .env_remove("IES_ORIGEM_FILE")
        .env("IES_INDICE_FILE", pasta.join("ies.sqlite3"))
        .args(["documento", "salvar", "0011519", "--json"])
        .args(extra)
        .output()
        .unwrap()
}

/// O JSON da prévia que saiu com sucesso.
fn previa_offline(pasta: &std::path::Path, extra: &[&str]) -> serde_json::Value {
    let output = salvar_offline(pasta, extra);
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).unwrap()
}

fn pasta_vazia(rotulo: &str) -> std::path::PathBuf {
    let pasta = std::env::temp_dir().join(format!("ies-salvar-{rotulo}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&pasta);
    std::fs::create_dir_all(&pasta).unwrap();
    pasta
}

#[test]
fn salvar_previa_offline_sem_indice_nem_configuracao() {
    // #207: o par `DOC.v2.html` / `DOC.v3.html` basta; nenhum índice nem
    // sessão. A base não é verificada aqui (só a releitura ou o salvar
    // remoto a conferem com o SEI), e a saída diz isso.
    let pasta = pasta_vazia("offline");
    versao_local(&pasta, 2, "<p>a</p>");
    versao_local(&pasta, 3, "<p>b</p>");
    let value = previa_offline(&pasta, &[]);
    let previa = &value["previa"];
    assert_eq!(
        (
            previa["versao_base"].as_u64(),
            previa["versao_editada"].as_u64()
        ),
        (Some(2), Some(3))
    );
    assert_eq!(previa["base_verificada"], false);
    assert_eq!(previa["escrita_remota"], false);
    assert_eq!(value["confirmado"], false);
    assert!(value["assinaturas_remotas"].is_null());
    assert!(
        value["aviso_assinaturas"]
            .as_str()
            .unwrap()
            .contains("pode remover assinaturas")
    );
    assert!(!value.to_string().contains("<p>"));
    assert!(!pasta.join("ies.sqlite3").exists());
    let value = previa_offline(&pasta, &["--listar-copias"]);
    assert_eq!(value["escopo"], "arquivos_locais");
    assert_eq!(value["versao_remota_confirmada"], false);
    let copias = value["copias"].as_array().unwrap();
    assert_eq!(copias.len(), 2);
    for copia in copias {
        let mut chaves: Vec<&str> = copia
            .as_object()
            .unwrap()
            .keys()
            .map(String::as_str)
            .collect();
        chaves.sort_unstable();
        assert_eq!(chaves, ["bytes", "documento", "versao"]);
    }
    std::fs::remove_dir_all(pasta).unwrap();
}

#[test]
fn salvar_previa_offline_recusa_base_ausente_edicao_ausente_e_igual() {
    let pasta = pasta_vazia("recusas");
    falha(&salvar_offline(&pasta, &[]), "Html");
    versao_local(&pasta, 3, "<p>b</p>");
    falha(&salvar_offline(&pasta, &["--versao", "2"]), "Html");
    // Só v3: ela é a base e a edição v4 falta.
    falha(&salvar_offline(&pasta, &[]), "Html");
    versao_local(&pasta, 4, "<p>b</p>");
    falha(&salvar_offline(&pasta, &[]), "EdicaoIgualABase");
    std::fs::remove_dir_all(pasta).unwrap();
}

#[test]
fn bdd_89_01_previa_humana_mostra_antes_e_depois_sem_salvar() {
    // BDD-89-01 preexistente na issue #89: comparar a foto local vN e a
    // edição v(N+1), sem acessar sessão, índice ou SEI.
    let pasta = pasta_vazia("bdd-89-01");
    let base = "<html>\n<p>antigo</p>\n<p>igual</p>\n</html>";
    let editada = "<html>\n<p>novo</p>\n<p>igual</p>\n</html>";
    versao_local(&pasta, 2, base);
    versao_local(&pasta, 3, editada);
    let output = Command::new(env!("CARGO_BIN_EXE_ies"))
        .current_dir(&pasta)
        .env_remove("IES_CONFIG_FILE")
        .env_remove("IES_URL_CONTROLE_FILE")
        .env_remove("IES_COOKIE_FILE")
        .env_remove("IES_ORIGEM_FILE")
        .env("IES_INDICE_FILE", pasta.join("ies.sqlite3"))
        .args(["documento", "salvar", "0011519", "--versao", "2"])
        .output()
        .unwrap();
    let humano = String::from_utf8(output.stdout).unwrap();
    assert_eq!(output.status.code(), Some(0), "{humano}");
    assert!(humano.contains("-<p>antigo</p>"), "{humano}");
    assert!(humano.contains("+<p>novo</p>"), "{humano}");
    assert!(humano.contains("base não verificada"), "{humano}");
    assert!(humano.contains("pode remover assinaturas"), "{humano}");
    assert!(!pasta.join("ies.sqlite3").exists());
    assert_eq!(
        std::fs::read_to_string(pasta.join("0011519.v2.html")).unwrap(),
        base
    );
    assert_eq!(
        std::fs::read_to_string(pasta.join("0011519.v3.html")).unwrap(),
        editada
    );
    std::fs::remove_dir_all(pasta).unwrap();
}

#[test]
fn previa_89_01_exibe_html_com_atributos_comuns_sem_divulgar_link_assinado() {
    // Regressão técnica do BDD-89-01: em HTML normal, a maioria das tags
    // possui class/style; a prévia não pode omitir todas as diferenças.
    let pasta = pasta_vazia("atributos");
    let base = "<p class=\"texto\" style=\"font-weight:bold\">Antes = 1?</p>\n<a href=\"https://sei.test/?infra_hash=segredo\">link</a>";
    let editada = "<p class=\"texto\" style=\"font-weight:bold\">Depois = 2?</p>\n<a href=\"https://sei.test/?infra_hash=outro\">link</a>";
    versao_local(&pasta, 2, base);
    versao_local(&pasta, 3, editada);
    let saida = Command::new(env!("CARGO_BIN_EXE_ies"))
        .current_dir(&pasta)
        .env("IES_CONFIG_FILE", pasta.join("nao-existe.json"))
        .env_remove("IES_URL_CONTROLE_FILE")
        .env_remove("IES_COOKIE_FILE")
        .env_remove("IES_ORIGEM_FILE")
        .env("IES_INDICE_FILE", pasta.join("ies.sqlite3"))
        .args(["documento", "salvar", "0011519", "--versao", "2"])
        .output()
        .unwrap();
    let humano = String::from_utf8(saida.stdout).unwrap();
    assert_eq!(saida.status.code(), Some(0), "{humano}");
    assert!(
        humano.contains("-<p class=\"texto\" style=\"font-weight:bold\">Antes = 1?</p>"),
        "{humano}"
    );
    assert!(
        humano.contains("+<p class=\"texto\" style=\"font-weight:bold\">Depois = 2?</p>"),
        "{humano}"
    );
    assert!(humano.contains("[linha omitida:"), "{humano}");
    assert!(!humano.contains("infra_hash"), "{humano}");
    assert!(!humano.contains("segredo"), "{humano}");
    assert!(!humano.contains("outro"), "{humano}");
    assert!(humano.contains("base não verificada"), "{humano}");
    assert!(!pasta.join("ies.sqlite3").exists());
    assert_eq!(
        std::fs::read_to_string(pasta.join("0011519.v2.html")).unwrap(),
        base
    );
    assert_eq!(
        std::fs::read_to_string(pasta.join("0011519.v3.html")).unwrap(),
        editada
    );
    std::fs::remove_dir_all(pasta).unwrap();
}
