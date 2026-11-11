//! Leitura compartilhada dos arquivos locais de sessão do SEI.
//! Não faz login nem rede; a validade remota continua sendo conferida
//! pela operação que efetivamente consulta o Controle.
use super::{FalhaTransporte, arquivo_privado, validar_cookie, validar_origem};
use std::fmt;
use std::path::Path;

/// Cookie de um retrato da sessão, já lido do arquivo privado (`0600`,
/// sem link, sintaxe validada) ou a falha da leitura. A falha só aparece
/// onde a requisição pede o cookie, como antes da #222: as recusas offline
/// que vêm antes dela continuam vencendo. Nunca sai no `Debug`.
#[derive(Clone, PartialEq, Eq)]
pub(crate) struct CookieDaSessao(Result<String, FalhaTransporte>);

impl CookieDaSessao {
    /// O chamador deve ter autorizado a URL de destino antes desta função.
    pub(crate) fn ler(cookie_file: &Path) -> Self {
        Self(arquivo_privado(cookie_file, 8_192).and_then(|texto| {
            let cookie = texto.trim().to_owned();
            validar_cookie(&cookie)?;
            Ok(cookie)
        }))
    }

    /// Cookie sintético dos testes, validado como o lido do arquivo.
    #[cfg(test)]
    pub(crate) fn sintetico(cookie: &str) -> Self {
        validar_cookie(cookie).unwrap();
        Self(Ok(cookie.to_owned()))
    }

    pub(crate) fn valor(&self) -> Result<&str, FalhaTransporte> {
        self.0.as_deref().map_err(Clone::clone)
    }
}

impl fmt::Debug for CookieDaSessao {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("CookieDaSessao(<redigido>)")
    }
}

/// Retrato coerente dos arquivos da sessão (#222): URL assinada do
/// Controle, origem autorizada e cookie de uma mesma geração, lidos uma
/// vez para o comando inteiro. Nunca expor URL nem cookie em `Debug`.
pub(crate) struct ReferenciasDaSessao {
    url_do_controle: String,
    origem: String,
    cookie: CookieDaSessao,
}

impl ReferenciasDaSessao {
    /// Marcador `<url>.geracao`, URL, origem (conferida antes de abrir o
    /// cookie), cookie e o marcador de novo. Incoerente (o marcador mudou
    /// ou não confere com o par): relê os arquivos locais uma vez;
    /// incoerente de novo: `SessaoTrocadaDuranteALeitura`. Sem marcador
    /// (sessão preparada fora do `ies login`), a leitura de antes.
    pub(crate) fn ler(
        url_file: &Path,
        origem_file: &Path,
        cookie_file: &Path,
    ) -> Result<Self, FalhaTransporte> {
        Self::ler_intercalado(url_file, origem_file, cookie_file, || {})
    }

    /// [`Self::ler`] com `entre` chamado depois da URL e antes do cookie,
    /// o ponto em que um login concorrente pode gravar (testes).
    pub(crate) fn ler_intercalado(
        url_file: &Path,
        origem_file: &Path,
        cookie_file: &Path,
        mut entre: impl FnMut(),
    ) -> Result<Self, FalhaTransporte> {
        for _ in 0..2 {
            if let Some(lidas) = Self::ler_uma_vez(url_file, origem_file, cookie_file, &mut entre)?
            {
                return Ok(lidas);
            }
        }
        Err(FalhaTransporte::SessaoTrocadaDuranteALeitura)
    }

    /// `None`: incoerente.
    fn ler_uma_vez(
        url_file: &Path,
        origem_file: &Path,
        cookie_file: &Path,
        entre: &mut impl FnMut(),
    ) -> Result<Option<Self>, FalhaTransporte> {
        let antes = marcador(url_file)?;
        let url = arquivo_privado(url_file, 8_192)?.trim().to_owned();
        let origem = arquivo_privado(origem_file, 512)?.trim().to_owned();
        entre();
        // A origem é conferida ANTES de abrir o cookie; a recusa fica
        // guardada no lugar do cookie, para a primeira requisição.
        let cookie = match validar_origem(&url, &origem) {
            Ok(()) => CookieDaSessao::ler(cookie_file),
            Err(falha) => CookieDaSessao(Err(falha)),
        };
        let depois = marcador(url_file)?;
        let coerente = match (antes, depois) {
            (None, None) => true,
            // Sem cookie legível nenhuma requisição sai; basta o marcador
            // estável.
            (Some(antes), Some(depois)) => {
                antes == depois
                    && cookie.valor().map_or(true, |valor| {
                        crate::sessao_login::marcador_confere(&depois, &url, valor)
                    })
            }
            _ => false,
        };
        Ok(coerente.then_some(Self {
            url_do_controle: url,
            origem,
            cookie,
        }))
    }

    pub(crate) fn url_do_controle(&self) -> &str {
        &self.url_do_controle
    }

    pub(crate) fn origem(&self) -> &str {
        &self.origem
    }

    pub(crate) fn cookie(&self) -> &CookieDaSessao {
        &self.cookie
    }

    /// A URL do Controle é da origem autorizada (o que guarda o cookie).
    pub(crate) fn conferir_controle(&self) -> Result<(), FalhaTransporte> {
        validar_origem(&self.url_do_controle, &self.origem)
    }
}

/// Texto do marcador `<url-controle>.geracao` (formato em
/// `sessao_login::gravacao`); `None` se não existe.
fn marcador(url_file: &Path) -> Result<Option<String>, FalhaTransporte> {
    let mut caminho = url_file.as_os_str().to_owned();
    caminho.push(".geracao");
    match arquivo_privado(Path::new(&caminho), 4_096) {
        Ok(texto) => Ok(Some(texto)),
        Err(FalhaTransporte::ArquivoPrivadoAusenteOuInacessivel(_)) => Ok(None),
        Err(falha) => Err(falha),
    }
}

impl fmt::Debug for ReferenciasDaSessao {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ReferenciasDaSessao")
            .field("url_do_controle", &"<redigido>")
            .field("origem", &"<redigido>")
            .field("cookie", &self.cookie)
            .finish()
    }
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
        let sessao =
            ReferenciasDaSessao::ler(&arquivos.url, &arquivos.origem, &arquivos.cookie).unwrap();
        assert_eq!(sessao.url_do_controle(), URL);
        assert_eq!(sessao.origem(), ORIGEM);
        assert_eq!(sessao.cookie().valor(), Ok("PHPSESSID=SINTETICO"));
        assert!(!format!("{sessao:?}").contains("infra_hash"));
        assert!(!format!("{sessao:?}").contains("sei.exemplo.gov.br"));
        assert!(!format!("{sessao:?}").contains("SINTETICO"));
    }

    #[test]
    fn origem_diferente_eh_recusada_antes_de_abrir_cookie_ausente() {
        let arquivos = ArquivosTeste::novos("https://outro.exemplo.gov.br/sei", None);
        let sessao =
            ReferenciasDaSessao::ler(&arquivos.url, &arquivos.origem, &arquivos.cookie).unwrap();
        assert_eq!(
            sessao.conferir_controle(),
            Err(FalhaTransporte::OrigemNaoAutorizada)
        );
        // A recusa fica no lugar do cookie (ausente, nunca aberto).
        assert_eq!(
            sessao.cookie().valor(),
            Err(FalhaTransporte::OrigemNaoAutorizada)
        );
    }

    #[test]
    fn cookie_invalido_eh_recusado_sem_rede() {
        let arquivos = ArquivosTeste::novos(ORIGEM, Some("PHPSESSID=SINTETICO\rX"));
        let sessao =
            ReferenciasDaSessao::ler(&arquivos.url, &arquivos.origem, &arquivos.cookie).unwrap();
        assert_eq!(
            sessao.cookie().valor(),
            Err(FalhaTransporte::CookieInvalido)
        );
    }
}
