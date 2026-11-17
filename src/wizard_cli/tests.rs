//! Assistente sem rede nem cofre da pessoa: entrada em memória, pasta
//! pessoal temporária e o cofre em memória dos testes ([`crate::cofre`]).
use super::*;
use std::io::Cursor;
use std::sync::atomic::{AtomicU32, Ordering};

type Esperado = fn(&ErroWizard) -> bool;

fn pasta_pessoal() -> PathBuf {
    static N: AtomicU32 = AtomicU32::new(0);
    let pasta = std::env::temp_dir().join(format!(
        "ies-wizard-{}-{}",
        std::process::id(),
        N.fetch_add(1, Ordering::Relaxed)
    ));
    std::fs::create_dir_all(&pasta).unwrap();
    pasta
}

fn rodar(
    pasta: &Path,
    respostas: &str,
    senha: &str,
    perfil: Option<&str>,
) -> Result<PerfilGravado, ErroWizard> {
    let senha = senha.to_owned();
    configurar(
        &mut Cursor::new(respostas.as_bytes().to_vec()),
        &mut Vec::new(),
        move || Ok(senha),
        pasta,
        perfil,
    )
}

fn conteudo(caminho: &Path) -> String {
    crate::arquivo_privado::ler(caminho, 4096).unwrap_or_default()
}

#[test]
fn outra_instalacao_grava_perfil_privado_sem_senha_e_senha_exata_no_cofre() {
    let pasta = pasta_pessoal();
    let gravado = rodar(
        &pasta,
        "5\nhttps://sei.wizard-outra.test/sei\nPGE\nfulano\n\n",
        " senha falsa ",
        None,
    )
    .unwrap();
    assert_eq!(gravado.perfil, "sei.wizard-outra.test");
    assert_eq!(gravado.servico, "ies:sei.wizard-outra.test");
    let perfil = pasta.join(".ies/sei.wizard-outra.test");
    assert_eq!(
        conteudo(&perfil.join("origem")),
        "https://sei.wizard-outra.test/sei\n"
    );
    assert_eq!(
        conteudo(&gravado.credenciais),
        "usuario=fulano\norgao=PGE\n"
    );
    for nome in [
        "origem",
        "cookie",
        "url-controle",
        "credenciais",
        "ies-config.json",
    ] {
        let info = std::fs::symlink_metadata(perfil.join(nome)).unwrap();
        assert!(info.is_file(), "{nome}");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(info.permissions().mode() & 0o777, 0o600, "{nome}");
        }
    }
    let instalacao = Instalacao::interpretar("https://sei.wizard-outra.test/sei").unwrap();
    let chave = ChaveDoCofre::nova(&instalacao, "fulano");
    assert_eq!(
        cofre::obter(&chave).unwrap().as_deref(),
        Some(" senha falsa ")
    );
    // O login lê o mesmo perfil: usuário e órgão do arquivo, senha do cofre.
    let lidas = crate::sessao_login::Credenciais::ler(&gravado.credenciais, &instalacao).unwrap();
    assert_eq!(
        (lidas.usuario(), lidas.senha(), lidas.orgao()),
        ("fulano", " senha falsa ", "PGE")
    );
    let _ = std::fs::remove_dir_all(&pasta);
}

#[test]
fn perfil_explicito_vale_tambem_para_a_predefinicao() {
    let pasta = pasta_pessoal();
    let gravado = rodar(&pasta, "1\n", "nao-usada", Some("temporario")).unwrap();
    assert_eq!(gravado.perfil, "temporario");
    assert!(pasta.join(".ies/temporario/ies-config.json").is_file());
    assert!(!pasta.join(".ies/unb").exists());
    assert_eq!(
        conteudo(&pasta.join(".ies/temporario/credenciais")),
        "usuario=aluno1\norgao=UnB\n"
    );
    let _ = std::fs::remove_dir_all(&pasta);
}

#[test]
fn perfil_inseguro_e_recusado_antes_de_qualquer_gravacao() {
    for nome in [
        "../fora",
        "/tmp/fora",
        "a/b",
        "a\\b",
        ".",
        "..",
        ".oculto",
        "",
    ] {
        assert!(nome_do_perfil(nome).is_err(), "{nome:?}");
    }
    let pasta = pasta_pessoal();
    let erro = rodar(
        &pasta,
        "5\nhttps://sei.wizard-inseguro.test/sei\nPGE\nfulano\n../../fora\n",
        "senha-falsa",
        None,
    )
    .unwrap_err();
    assert!(matches!(erro, ErroWizard::PerfilInvalido));
    assert!(!pasta.join(".ies").exists());
    let instalacao = Instalacao::interpretar("https://sei.wizard-inseguro.test/sei").unwrap();
    assert_eq!(
        cofre::obter(&ChaveDoCofre::nova(&instalacao, "fulano")),
        Ok(None)
    );
    let _ = std::fs::remove_dir_all(&pasta);
}

#[test]
fn perfil_existente_nao_e_sobrescrito_nem_toca_o_cofre() {
    let pasta = pasta_pessoal();
    std::fs::create_dir_all(pasta.join(".ies/existente")).unwrap();
    let erro = rodar(
        &pasta,
        "5\nhttps://sei.wizard-existente.test/sei\nPGE\nfulano\n",
        "senha-falsa",
        Some("existente"),
    )
    .unwrap_err();
    assert!(matches!(erro, ErroWizard::PerfilJaExiste));
    assert_eq!(
        std::fs::read_dir(pasta.join(".ies/existente"))
            .unwrap()
            .count(),
        0
    );
    let instalacao = Instalacao::interpretar("https://sei.wizard-existente.test/sei").unwrap();
    assert_eq!(
        cofre::obter(&ChaveDoCofre::nova(&instalacao, "fulano")),
        Ok(None)
    );
    let _ = std::fs::remove_dir_all(&pasta);
}

#[test]
fn cofre_indisponivel_nao_grava_nada_no_disco() {
    let pasta = pasta_pessoal();
    let instalacao = Instalacao::interpretar("https://sei.wizard-sem-cofre.test/sei").unwrap();
    cofre::falhar_na_proxima(
        &ChaveDoCofre::nova(&instalacao, "fulano"),
        keyring_core::Error::NoStorageAccess("sem sessão".into()),
    );
    let erro = rodar(
        &pasta,
        "5\nhttps://sei.wizard-sem-cofre.test/sei\nPGE\nfulano\n\n",
        "senha-falsa",
        None,
    )
    .unwrap_err();
    assert!(matches!(
        erro,
        ErroWizard::Cofre(ErroCofre::AcessoNegado { .. })
    ));
    assert!(!pasta.join(".ies").exists());
    let _ = std::fs::remove_dir_all(&pasta);
}

#[test]
fn respostas_invalidas_param_sem_gravar() {
    let pasta = pasta_pessoal();
    let casos: [(&str, Esperado); 6] = [
        ("9\n", |e| matches!(e, ErroWizard::OpcaoInvalida)),
        ("x\n", |e| matches!(e, ErroWizard::OpcaoInvalida)),
        ("", |e| matches!(e, ErroWizard::EntradaEncerrada)),
        ("5\nhttp://sei.exemplo.test/sei\n", |e| {
            matches!(e, ErroWizard::OrigemNaoReconhecida)
        }),
        ("5\nhttps://sei.exemplo.test/sei\n\n", |e| {
            matches!(e, ErroWizard::OrgaoVazio)
        }),
        ("5\nhttps://sei.exemplo.test/sei\nPGE\n\n", |e| {
            matches!(e, ErroWizard::UsuarioVazio)
        }),
    ];
    for (respostas, esperado) in casos {
        let erro = rodar(&pasta, respostas, "senha-falsa", None).unwrap_err();
        assert!(esperado(&erro), "{respostas:?}: {erro:?}");
    }
    let senha_vazia = configurar(
        &mut Cursor::new(b"5\nhttps://sei.exemplo.test/sei\nPGE\nfulano\n\n".to_vec()),
        &mut Vec::new(),
        || Err(ErroCredenciais::SenhaVazia),
        &pasta,
        None,
    )
    .unwrap_err();
    assert!(matches!(
        senha_vazia,
        ErroWizard::Senha(ErroCredenciais::SenhaVazia)
    ));
    assert!(!pasta.join(".ies").exists());
    let _ = std::fs::remove_dir_all(&pasta);
}

/// As linhas POSIX da saída, coladas num `sh` descartável, exportam os
/// caminhos exatos e não executam nada do nome (espaço, `$(...)`, aspas,
/// crase). Sem cofre, sem rede: só o texto que a pessoa copia.
#[cfg(unix)]
#[test]
fn instrucoes_posix_exportam_o_caminho_exato_sem_executar_o_nome() {
    let pasta = pasta_pessoal();
    let marcador = pasta.join("executado");
    let perfil = format!("p q$(touch {})'`x`\"", marcador.display());
    let config = format!("{}/.ies/{perfil}/ies-config.json", pasta.display());
    let credenciais = format!("{}/.ies/{perfil}/credenciais", pasta.display());
    let texto = instrucoes_de_ativacao(&config, &credenciais);
    let exports: Vec<&str> = texto
        .lines()
        .map(str::trim)
        .filter(|linha| linha.starts_with("export "))
        .collect();
    assert_eq!(exports.len(), 2, "{texto}");
    let script = format!(
        "{}\nprintf '%s\\n%s\\n' \"$IES_CONFIG_FILE\" \"$IES_CREDENCIAIS_FILE\"\n\
         sh -c 'test -n \"$IES_CONFIG_FILE\"' || echo NAO_EXPORTADO",
        exports.join("\n")
    );
    let saida = std::process::Command::new("sh")
        .arg("-c")
        .arg(&script)
        .env_clear()
        .output()
        .unwrap();
    assert!(saida.status.success());
    assert_eq!(
        String::from_utf8(saida.stdout).unwrap(),
        format!("{config}\n{credenciais}\n")
    );
    assert!(!marcador.exists());
    assert!(texto.contains("$env:IES_CONFIG_FILE = '"));
    let _ = std::fs::remove_dir_all(&pasta);
}
