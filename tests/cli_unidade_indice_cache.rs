//! #244: prova de ponta a ponta do snapshot local, com SQLite e sessão
//! sintéticos. O domínio .example nunca recebe requisições.
use rusqlite::{Connection, params};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

#[path = "support/falha.rs"]
mod falha_cli;
use falha_cli::falha;

struct SessaoSintetica {
    pasta: PathBuf,
    url: PathBuf,
    origem: PathBuf,
    cookie: PathBuf,
    indice: PathBuf,
}

impl SessaoSintetica {
    fn nova() -> Self {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("relógio")
            .as_nanos();
        let pasta =
            std::env::temp_dir().join(format!("ies-244-cli-{}-{nanos}", std::process::id()));
        fs::create_dir(&pasta).unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt as _;
            fs::set_permissions(&pasta, fs::Permissions::from_mode(0o700)).unwrap();
        }
        let url = pasta.join("url");
        let origem = pasta.join("origem");
        let cookie = pasta.join("cookie");
        let indice = pasta.join("ies.sqlite3");
        Self::privado(
            &url,
            &format!(
                "https://sei.example/sei/controlador.php?acao=procedimento_controlar&infra_hash={}&infra_unidade_atual=7\n",
                "a".repeat(64)
            ),
        );
        Self::privado(&origem, "https://sei.example/sei\n");
        Self::privado(&cookie, "PHPSESSID=SINTETICO\n");
        let banco = Connection::open(&indice).unwrap();
        banco
            .execute_batch(
                "PRAGMA user_version=3;
             CREATE TABLE lista_unidade (
                instalacao TEXT NOT NULL,
                unidade_ativa TEXT NOT NULL,
                lida_em REAL NOT NULL,
                total INTEGER NOT NULL,
                PRIMARY KEY(instalacao, unidade_ativa)
             ) STRICT;
             CREATE TABLE unidade (
                instalacao TEXT NOT NULL,
                unidade_ativa TEXT NOT NULL,
                id TEXT NOT NULL,
                sigla TEXT NOT NULL,
                descricao TEXT NOT NULL,
                orgao TEXT NOT NULL,
                lida_em REAL NOT NULL,
                PRIMARY KEY(instalacao, unidade_ativa, id)
             ) STRICT;",
            )
            .unwrap();
        banco
            .execute(
                "INSERT INTO lista_unidade VALUES (?1, ?2, ?3, ?4)",
                params!["https://sei.example/sei", "7", 1234.0, 3_i64],
            )
            .unwrap();
        for (id, sigla, descricao) in [
            ("10", "CDE/SEAD", "Coordenação de Ensino"),
            ("11", "CDE", "São Paulo"),
            ("12", "OUTRA", "Descrição"),
        ] {
            banco
                .execute(
                    "INSERT INTO unidade VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                    params![
                        "https://sei.example/sei",
                        "7",
                        id,
                        sigla,
                        descricao,
                        "Órgão",
                        1234.0
                    ],
                )
                .unwrap();
        }
        banco
            .execute(
                "INSERT INTO lista_unidade VALUES (?1, ?2, ?3, ?4)",
                params!["https://sei.example/sei", "8", 1235.0, 0_i64],
            )
            .unwrap();
        drop(banco);
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt as _;
            fs::set_permissions(&indice, fs::Permissions::from_mode(0o600)).unwrap();
        }
        Self {
            pasta,
            url,
            origem,
            cookie,
            indice,
        }
    }

    fn privado(caminho: &Path, texto: &str) {
        fs::write(caminho, texto).unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt as _;
            fs::set_permissions(caminho, fs::Permissions::from_mode(0o600)).unwrap();
        }
    }

    fn executar(&self, argumentos: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_ies"))
            .args(argumentos)
            .env_remove("IES_CONFIG_FILE")
            .env_remove("IES_SEM_INDICE")
            .env("IES_URL_CONTROLE_FILE", &self.url)
            .env("IES_ORIGEM_FILE", &self.origem)
            .env("IES_COOKIE_FILE", &self.cookie)
            .env("IES_INDICE_FILE", &self.indice)
            .output()
            .unwrap()
    }
}

impl Drop for SessaoSintetica {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.pasta);
    }
}

#[test]
fn pesquisa_offline_aceita_acento_ignorado_e_sigla_exata_primeiro() {
    let sessao = SessaoSintetica::nova();
    let saida = sessao.executar(&["unidade", "pesquisar", "cde", "--json"]);
    assert!(
        saida.status.success(),
        "{:?}",
        String::from_utf8_lossy(&saida.stderr)
    );
    let valor: serde_json::Value = serde_json::from_slice(&saida.stdout).unwrap();
    assert_eq!(valor["origem"], "indice");
    assert_eq!(valor["requisicoes"], 0);
    assert_eq!(valor["total"], 2);
    assert_eq!(valor["lida_em"], 1234.0);
    assert_eq!(valor["unidades"][0]["sigla"], "CDE");
    assert_eq!(valor["unidades"][1]["sigla"], "CDE/SEAD");

    let saida = sessao.executar(&["unidade", "pesquisar", "sao", "--json"]);
    assert!(saida.status.success());
    let valor: serde_json::Value = serde_json::from_slice(&saida.stdout).unwrap();
    assert_eq!(valor["total"], 1);
    assert_eq!(valor["unidades"][0]["descricao"], "São Paulo");

    let saida = sessao.executar(&["unidade", "pesquisar", "inexistente", "--json"]);
    assert!(saida.status.success());
    let valor: serde_json::Value = serde_json::from_slice(&saida.stdout).unwrap();
    assert_eq!(valor["total"], 0);
    assert_eq!(valor["unidades"].as_array().unwrap().len(), 0);
}

#[test]
fn listagem_distingue_snapshot_valido_e_unidade_sem_lista() {
    let sessao = SessaoSintetica::nova();
    let saida = sessao.executar(&["unidade", "listar", "--json"]);
    assert!(
        saida.status.success(),
        "{:?}",
        String::from_utf8_lossy(&saida.stderr)
    );
    let valor: serde_json::Value = serde_json::from_slice(&saida.stdout).unwrap();
    assert_eq!(valor["total"], 3);
    assert_eq!(valor["requisicoes"], 0);

    // Trocar o ID somente nos arquivos sintéticos não autoriza o uso do
    // snapshot antigo, nem infere a unidade pela sigla do argumento.
    SessaoSintetica::privado(
        &sessao.url,
        &format!(
            "https://sei.example/sei/controlador.php?acao=procedimento_controlar&infra_hash={}&infra_unidade_atual=8\n",
            "a".repeat(64)
        ),
    );
    let saida = sessao.executar(&["unidade", "listar", "--json"]);
    assert!(saida.status.success());
    let valor: serde_json::Value = serde_json::from_slice(&saida.stdout).unwrap();
    assert_eq!(valor["total"], 0);
    assert_eq!(valor["unidade_ativa"], "8");
}

#[test]
fn opcao_de_unidade_sem_tela_web_recusa_em_vez_de_ignorar() {
    let sessao = SessaoSintetica::nova();
    let saida = sessao.executar(&["--unidade", "PPI", "unidade", "pesquisar", "cde", "--json"]);
    falha(&saida, "UnidadePedidaNaoVerificavelOffline");
}
