//! Cenários sintéticos da #212; não há dados de sessão ou acesso à rede.
use super::{FalhaConfiguracaoLocal, ler_configuracao_local};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

static PROXIMO: AtomicUsize = AtomicUsize::new(0);

struct Pasta(PathBuf);

impl Pasta {
    fn nova() -> Self {
        let n = PROXIMO.fetch_add(1, Ordering::Relaxed);
        let path =
            std::env::temp_dir().join(format!("ies-config-sintetica-{}-{n}", std::process::id()));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
    fn arquivo(&self) -> PathBuf {
        self.0.join("config.json")
    }
}

impl Drop for Pasta {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn gravar(path: &Path, conteudo: &str) {
    fs::write(path, conteudo).unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt as _;
        fs::set_permissions(path, fs::Permissions::from_mode(0o600)).unwrap();
    }
}

fn valido() -> &'static str {
    r#"{"url_controle":"/tmp/ies-ficticio-url","cookie":"/tmp/ies-ficticio-cookie","origem":"/tmp/ies-ficticio-origem"}"#
}

#[test]
fn configuracao_valida_devolve_so_os_caminhos() {
    let pasta = Pasta::nova();
    gravar(&pasta.arquivo(), valido());
    let arquivos = ler_configuracao_local(&pasta.arquivo()).unwrap();
    assert_eq!(arquivos.cookie, PathBuf::from("/tmp/ies-ficticio-cookie"));
}

#[test]
fn ausente_tem_causa_distinta() {
    let pasta = Pasta::nova();
    let causa = ler_configuracao_local(&pasta.arquivo()).err().unwrap();
    assert_eq!(causa, FalhaConfiguracaoLocal::Ausente);
}

#[test]
fn diretorio_e_excesso_sao_diagnosticos_internos_distintos() {
    let pasta = Pasta::nova();
    assert_eq!(
        ler_configuracao_local(&pasta.0).err(),
        Some(FalhaConfiguracaoLocal::NaoRegular)
    );
    gravar(&pasta.arquivo(), &"x".repeat(8_193));
    assert_eq!(
        ler_configuracao_local(&pasta.arquivo()).err(),
        Some(FalhaConfiguracaoLocal::AcimaDoLimite)
    );
}

#[cfg(unix)]
#[test]
fn ancestral_regular_preserva_causa_io_sem_alterar_bytes() {
    let pasta = Pasta::nova();
    let ancestral = pasta.0.join("ancestral-regular");
    gravar(&ancestral, "sentinela-sintetica");
    let destino = ancestral.join("config.json");
    assert_eq!(
        ler_configuracao_local(&destino).err(),
        Some(FalhaConfiguracaoLocal::ErroDeIoNaLeitura(
            std::io::ErrorKind::NotADirectory
        ))
    );
    assert_eq!(fs::read_to_string(ancestral).unwrap(), "sentinela-sintetica");
}

#[test]
fn json_invalido_nao_e_campo_invalido() {
    let pasta = Pasta::nova();
    gravar(&pasta.arquivo(), "{");
    assert!(matches!(
        ler_configuracao_local(&pasta.arquivo()).err(),
        Some(FalhaConfiguracaoLocal::JsonInvalido(_))
    ));
    gravar(&pasta.arquivo(), r#"{"url_controle":7}"#);
    assert_eq!(
        ler_configuracao_local(&pasta.arquivo()).err(),
        Some(FalhaConfiguracaoLocal::CampoInvalido)
    );
}

#[test]
fn caminhos_relativos_ou_vazios_nao_sao_aceitos() {
    let pasta = Pasta::nova();
    for bruto in [
        r#"{"url_controle":"relativo","cookie":"/tmp/teste","origem":"/tmp/teste"}"#,
        r#"{"url_controle":"/tmp/teste","cookie":"","origem":"/tmp/teste"}"#,
        r"[]",
    ] {
        gravar(&pasta.arquivo(), bruto);
        assert_eq!(
            ler_configuracao_local(&pasta.arquivo()).err(),
            Some(FalhaConfiguracaoLocal::CampoInvalido)
        );
    }
}

#[cfg(unix)]
#[test]
fn permissao_aberta_eh_recusada_antes_da_leitura() {
    use std::os::unix::fs::PermissionsExt as _;
    let pasta = Pasta::nova();
    gravar(&pasta.arquivo(), valido());
    fs::set_permissions(pasta.arquivo(), fs::Permissions::from_mode(0o644)).unwrap();
    assert_eq!(
        ler_configuracao_local(&pasta.arquivo()).err(),
        Some(FalhaConfiguracaoLocal::PermissoesAbertas)
    );
}

#[cfg(unix)]
#[test]
fn link_simbolico_nao_e_seguido() {
    use std::os::unix::fs::symlink;
    let pasta = Pasta::nova();
    let alvo = pasta.0.join("original.json");
    gravar(&alvo, valido());
    symlink(&alvo, pasta.arquivo()).unwrap();
    assert_eq!(
        ler_configuracao_local(&pasta.arquivo()).err(),
        Some(FalhaConfiguracaoLocal::LinkSimbolico)
    );
}

/// #211: a configuração nasce pela gravação privada única.
#[cfg(unix)]
#[test]
fn salvar_configuracao_aceita_dir_link_recusa_dir_aberto_e_nao_sobrescreve() {
    use super::{ArquivosDeAcesso, salvar_arquivos_de_acesso};
    use std::os::unix::fs::PermissionsExt as _;
    let pasta = Pasta::nova();
    let acesso = |nome: &str| {
        let caminho = pasta.0.join(nome);
        gravar(&caminho, "x");
        caminho
    };
    let arquivos = ArquivosDeAcesso {
        url_controle: acesso("url"),
        cookie: acesso("cookie"),
        origem: acesso("origem"),
    };
    let real = pasta.0.join("real");
    fs::create_dir(&real).unwrap();
    std::os::unix::fs::symlink(&real, pasta.0.join("atalho")).unwrap();
    // Diretório por link: aceito, o arquivo vai para o diretório real.
    assert_eq!(
        salvar_arquivos_de_acesso(&pasta.0.join("atalho/config.json"), &arquivos),
        Ok(())
    );
    assert!(real.join("config.json").is_file());
    // Diretório do IES gravável pelo grupo ou por outros sem sticky:
    // recusado sem criar nada; com sticky, aceito.
    for bits in [0o770, 0o707, 0o777] {
        fs::set_permissions(&real, fs::Permissions::from_mode(bits)).unwrap();
        assert_eq!(
            salvar_arquivos_de_acesso(&pasta.0.join("atalho/outra.json"), &arquivos),
            Err(super::ErroGravacaoDaConfiguracao::ConfiguracaoJaExisteOuSemPermissao)
        );
    }
    assert_eq!(fs::read_dir(&real).unwrap().count(), 1);
    fs::set_permissions(&real, fs::Permissions::from_mode(0o1777)).unwrap();
    assert_eq!(
        salvar_arquivos_de_acesso(&pasta.0.join("atalho/sticky.json"), &arquivos),
        Ok(())
    );
    fs::set_permissions(&real, fs::Permissions::from_mode(0o700)).unwrap();
    let destino = pasta.arquivo();
    assert_eq!(salvar_arquivos_de_acesso(&destino, &arquivos), Ok(()));
    assert_eq!(
        salvar_arquivos_de_acesso(&destino, &arquivos),
        Err(super::ErroGravacaoDaConfiguracao::ConfiguracaoJaExisteOuSemPermissao)
    );
    assert_eq!(
        fs::metadata(&destino).unwrap().permissions().mode() & 0o777,
        0o600
    );
}

/// Qual arquivo privado falhou é campo do erro (o papel, nunca o caminho).
#[test]
fn arquivo_privado_que_falha_e_identificado_pelo_papel() {
    use super::{
        ArquivoPrivado, ArquivosDeAcesso, ErroArquivoPrivado, verificar_arquivos_privados,
    };
    let pasta = Pasta::nova();
    let privado = |nome: &str| {
        let caminho = pasta.0.join(nome);
        fs::write(&caminho, "DADO FICTICIO").unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&caminho, fs::Permissions::from_mode(0o600)).unwrap();
        }
        caminho
    };
    let completos = ArquivosDeAcesso {
        url_controle: privado("url"),
        cookie: privado("cookie"),
        origem: privado("origem"),
    };
    assert_eq!(verificar_arquivos_privados(&completos), Ok(()));
    let sem_cookie = ArquivosDeAcesso {
        cookie: pasta.0.join("ausente"),
        ..completos.clone()
    };
    assert_eq!(
        verificar_arquivos_privados(&sem_cookie),
        Err(ErroArquivoPrivado::Ausente(ArquivoPrivado::Cookie))
    );
    let relativa = ArquivosDeAcesso {
        url_controle: PathBuf::from("url"),
        ..completos.clone()
    };
    assert_eq!(
        verificar_arquivos_privados(&relativa),
        Err(ErroArquivoPrivado::CaminhoNaoAbsoluto(
            ArquivoPrivado::UrlControle
        ))
    );
    fs::write(&completos.origem, "").unwrap();
    assert_eq!(
        verificar_arquivos_privados(&completos),
        Err(ErroArquivoPrivado::Vazio(ArquivoPrivado::Origem))
    );
}
