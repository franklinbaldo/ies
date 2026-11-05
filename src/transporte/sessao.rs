//! Leitura compartilhada dos arquivos locais de sessão do SEI.
//! Não faz login nem rede; a validade remota continua sendo conferida
//! pela operação que efetivamente consulta o Controle.
use super::{FalhaTransporte, arquivo_privado, validar_cookie, validar_origem};
use std::fmt;
use std::path::Path;

/// URL assinada do Controle e origem autorizada, lidas uma única vez.
/// Nunca expor URL assinada em `Debug`.
pub(crate) struct ReferenciasDaSessao {
    url_do_controle: String,
    origem: String,
    cookie: String,
}

impl ReferenciasDaSessao {
    /// Preserva a ordem histórica: primeiro URL, depois origem;
    /// não abre o arquivo de cookie nesta etapa.
    pub(crate) fn ler(url_file: &Path, origem_file: &Path) -> Result<Self, FalhaTransporte> {
        let url = arquivo_privado(url_file, 8_192)?;
        let origem = arquivo_privado(origem_file, 512)?;
        Ok(Self {
            url_do_controle: url.trim().to_owned(),
            origem: origem.trim().to_owned(),
            cookie: String::new(),
        })
    }

    /// Shim do contrato da #222 (RED): `entre` roda depois da URL e antes
    /// do cookie, o ponto em que um login concorrente pode gravar.
    pub(crate) fn ler_intercalado(
        url_file: &Path,
        origem_file: &Path,
        cookie_file: &Path,
        mut entre: impl FnMut(),
    ) -> Result<Self, FalhaTransporte> {
        let mut lidas = Self::ler(url_file, origem_file)?;
        entre();
        lidas.cookie = lidas.cookie_validado(cookie_file)?;
        Ok(lidas)
    }

    pub(crate) fn cookie(&self) -> &str {
        &self.cookie
    }

    pub(crate) fn url_do_controle(&self) -> &str {
        &self.url_do_controle
    }

    pub(crate) fn origem(&self) -> &str {
        &self.origem
    }

    /// A origem precisa ser conferida ANTES de abrir o cookie.
    pub(crate) fn conferir_controle(&self) -> Result<(), FalhaTransporte> {
        validar_origem(&self.url_do_controle, &self.origem)
    }

    pub(crate) fn cookie_validado(&self, cookie_file: &Path) -> Result<String, FalhaTransporte> {
        self.conferir_controle()?;
        cookie_privado_validado(cookie_file)
    }
}

impl fmt::Debug for ReferenciasDaSessao {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ReferenciasDaSessao")
            .field("url_do_controle", &"<redigido>")
            .field("origem", &"<redigido>")
            .finish()
    }
}

/// Lê um cookie privado 0600, sem symlink, e valida sua sintaxe.
/// O chamador deve ter autorizado a URL de destino antes desta função.
pub(crate) fn cookie_privado_validado(cookie_file: &Path) -> Result<String, FalhaTransporte> {
    let texto = arquivo_privado(cookie_file, 8_192)?;
    let cookie = texto.trim().to_owned();
    validar_cookie(&cookie)?;
    Ok(cookie)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    const ORIGEM: &str = "https://sei.exemplo.gov.br/sei";
    const URL: &str = "https://sei.exemplo.gov.br/sei/controlador.php?acao=procedimento_controlar&infra_hash=aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

    struct ArquivosTeste {
        pasta: PathBuf,
        url: PathBuf,
        origem: PathBuf,
        cookie: PathBuf,
    }

    impl ArquivosTeste {
        fn novos(origem: &str, cookie: Option<&str>) -> Self {
            let pasta = std::env::temp_dir().join(format!(
                "ies-ref-sessao-{}-{}",
                std::process::id(),
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_nanos()
            ));
            fs::create_dir(&pasta).unwrap();
            let url = pasta.join("url");
            let origem_file = pasta.join("origem");
            let cookie_file = pasta.join("cookie");
            gravar(&url, &format!("{URL}\n"));
            gravar(&origem_file, &format!("{origem}\n"));
            if let Some(texto) = cookie {
                gravar(&cookie_file, texto);
            }
            Self {
                pasta,
                url,
                origem: origem_file,
                cookie: cookie_file,
            }
        }
    }

    impl Drop for ArquivosTeste {
        fn drop(&mut self) {
            fs::remove_dir_all(&self.pasta).unwrap();
        }
    }

    fn gravar(caminho: &Path, conteudo: &str) {
        fs::write(caminho, conteudo).unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(caminho, fs::Permissions::from_mode(0o600)).unwrap();
        }
    }

    #[test]
    fn leitura_preserva_dados_sem_expor_assinatura_em_debug() {
        let arquivos = ArquivosTeste::novos(ORIGEM, Some("PHPSESSID=SINTETICO\n"));
        let sessao = ReferenciasDaSessao::ler(&arquivos.url, &arquivos.origem).unwrap();
        assert_eq!(sessao.url_do_controle(), URL);
        assert_eq!(sessao.origem(), ORIGEM);
        assert_eq!(
            sessao.cookie_validado(&arquivos.cookie).unwrap(),
            "PHPSESSID=SINTETICO"
        );
        assert!(!format!("{sessao:?}").contains("infra_hash"));
        assert!(!format!("{sessao:?}").contains("sei.exemplo.gov.br"));
    }

    #[test]
    fn origem_diferente_eh_recusada_antes_de_abrir_cookie_ausente() {
        let arquivos = ArquivosTeste::novos("https://outro.exemplo.gov.br/sei", None);
        let sessao = ReferenciasDaSessao::ler(&arquivos.url, &arquivos.origem).unwrap();
        assert_eq!(
            sessao.cookie_validado(&arquivos.cookie),
            Err(FalhaTransporte::OrigemNaoAutorizada)
        );
    }

    #[test]
    fn cookie_invalido_eh_recusado_sem_rede() {
        let arquivos = ArquivosTeste::novos(ORIGEM, Some("PHPSESSID=SINTETICO\rX"));
        let sessao = ReferenciasDaSessao::ler(&arquivos.url, &arquivos.origem).unwrap();
        assert_eq!(
            sessao.cookie_validado(&arquivos.cookie),
            Err(FalhaTransporte::CookieInvalido)
        );
    }
}
