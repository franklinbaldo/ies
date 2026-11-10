//! #244: contrato público SQLite e CLI real, com sessão sintética.
//! O cookie deliberadamente não existe: uma consulta ao SEI falharia.
//! Caso de Todos `tests/test_destinos_indice_010.py` @323eb01 (pesquisa local).
use rusqlite::{Connection, OpenFlags};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

#[path = "support/falha.rs"]
mod falha_cli;
use falha_cli::falha_por;

const ORIGEM: &str = "https://sei.exemplo/sei";

struct Ambiente {
    pasta: PathBuf,
    banco: PathBuf,
}
impl Ambiente {
    fn novo() -> Self {
        let pasta = std::env::temp_dir().join(format!(
            "ies-244-cli-offline-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir(&pasta).unwrap();
        modo_privado(&pasta, 0o700);
        let url = pasta.join("controle");
        let origem = pasta.join("origem");
        fs::write(
            &url,
            format!(
                "{ORIGEM}/controlador.php?acao=procedimento_controlar&infra_unidade_atual=7&infra_hash={}",
                "a".repeat(64)
            ),
        ).unwrap();
        fs::write(&origem, ORIGEM).unwrap();
        modo_privado(&url, 0o600);
        modo_privado(&origem, 0o600);
        let banco = pasta.join("ies.sqlite3");
        let conexao = Connection::open(&banco).unwrap();
        // Esquema público v3: o agente pode consultar estas duas tabelas
        // diretamente, mas não pode alterar o banco durante a operação.
        conexao
            .execute_batch(
                "CREATE TABLE lista_unidade (
                instalacao TEXT NOT NULL, unidade_ativa TEXT NOT NULL,
                lida_em REAL NOT NULL, total INTEGER NOT NULL,
                PRIMARY KEY (instalacao, unidade_ativa)
            ) STRICT;
            CREATE TABLE unidade (
                instalacao TEXT NOT NULL, unidade_ativa TEXT NOT NULL,
                id TEXT NOT NULL, sigla TEXT NOT NULL,
                descricao TEXT NOT NULL, orgao TEXT NOT NULL,
                lida_em REAL NOT NULL,
                PRIMARY KEY (instalacao, unidade_ativa, id)
            ) STRICT;
            PRAGMA user_version=3;
            INSERT INTO lista_unidade VALUES
                ('https://sei.exemplo/sei', '7', 1234.0, 3);
            INSERT INTO unidade VALUES
                ('https://sei.exemplo/sei', '7', '1', 'CDE/SEAD', 'Administração', 'SEAD', 1234.0),
                ('https://sei.exemplo/sei', '7', '2', 'SÃO/CDE', 'Unidade São', 'SEAD', 1234.0),
                ('https://sei.exemplo/sei', '7', '3', 'OUTRA', 'Outro setor', 'SEAD', 1234.0);",
            )
            .unwrap();
        drop(conexao);
        modo_privado(&banco, 0o600);
        Self { pasta, banco }
    }

    fn executar(&self, args: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_ies"))
            .args(args)
            .env_remove("IES_CONFIG_FILE")
            .env_remove("IES_SEM_INDICE")
            .env("IES_URL_CONTROLE_FILE", self.pasta.join("controle"))
            .env("IES_ORIGEM_FILE", self.pasta.join("origem"))
            // Sem arquivo de cookie: uma tentativa de HTTP não seria possível.
            .env("IES_COOKIE_FILE", self.pasta.join("cookie-ausente"))
            .env("IES_INDICE_FILE", &self.banco)
            .output()
            .unwrap()
    }
}
impl Drop for Ambiente {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.pasta);
    }
}
fn modo_privado(caminho: &Path, modo: u32) {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(caminho, fs::Permissions::from_mode(modo)).unwrap();
    }
    #[cfg(not(unix))]
    let _ = (caminho, modo);
}

#[test]
fn binario_real_lista_e_pesquisa_no_indice_sem_cookie_ou_http() {
    let ambiente = Ambiente::novo();
    let listar = ambiente.executar(&["unidade", "listar", "--json"]);
    assert!(listar.status.success(), "{listar:?}");
    let lista: serde_json::Value = serde_json::from_slice(&listar.stdout).unwrap();
    assert_eq!(lista["origem"], "indice");
    assert_eq!(lista["unidade_ativa"], "7");
    assert_eq!(lista["lida_em"], 1234.0);
    assert_eq!(lista["total"], 3);
    assert_eq!(lista["requisicoes"], 0);
    assert_eq!(lista["persistida_no_indice"], true);
    assert_eq!(lista["unidades"].as_array().unwrap().len(), 3);

    let pesquisar = ambiente.executar(&["unidade", "pesquisar", "sao/cde", "--json"]);
    assert!(pesquisar.status.success(), "{pesquisar:?}");
    let filtrada: serde_json::Value = serde_json::from_slice(&pesquisar.stdout).unwrap();
    assert_eq!(filtrada["origem"], "indice");
    assert_eq!(filtrada["requisicoes"], 0);
    assert_eq!(filtrada["total"], 1);
    assert_eq!(filtrada["unidades"][0]["sigla"], "SÃO/CDE");

    let exata = ambiente.executar(&["unidade", "pesquisar", "CDE/SEAD", "--json"]);
    assert!(exata.status.success(), "{exata:?}");
    let exata: serde_json::Value = serde_json::from_slice(&exata.stdout).unwrap();
    assert_eq!(exata["total"], 1);
    assert_eq!(exata["unidades"][0]["id"], "1");
    assert_eq!(exata["requisicoes"], 0);

    // Falha fechada: --atualizar não pode mostrar o cache antigo como novo.
    // Sem cookie, a coleta é recusada antes da primeira requisição.
    let atualizar = ambiente.executar(&["unidade", "listar", "--atualizar", "--json"]);
    assert_eq!(atualizar.status.code(), Some(1));
    assert_eq!(atualizar.stdout, b"", "o cache antigo não é publicado");
    falha_por(
        &atualizar,
        "LupaSemListaIntegral",
        "ArquivoPrivadoAusenteOuInacessivel",
    );

    // A tabela documentada é consultável por um agente externo em modo
    // somente leitura, e os comandos não mudam o snapshot nem o esquema.
    let banco =
        Connection::open_with_flags(&ambiente.banco, OpenFlags::SQLITE_OPEN_READ_ONLY).unwrap();
    let (linhas, versao): (i64, i64) = (
        banco
            .query_row("SELECT count(*) FROM unidade", [], |r| r.get(0))
            .unwrap(),
        banco
            .pragma_query_value(None, "user_version", |r| r.get(0))
            .unwrap(),
    );
    assert_eq!((linhas, versao), (3, 4));
    for saida in [listar, pesquisar] {
        let texto = format!(
            "{}{}",
            String::from_utf8_lossy(&saida.stdout),
            String::from_utf8_lossy(&saida.stderr)
        );
        assert!(!texto.contains("infra_hash"));
        assert!(!texto.contains("PHPSESSID"));
        assert!(!texto.contains("controlador.php"));
    }
}
