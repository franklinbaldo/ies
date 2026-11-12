//! Leitura compartilhada dos arquivos locais de sessão do SEI.
//! Não faz login nem rede; a validade remota continua sendo conferida
//! pela operação que efetivamente consulta o Controle.
use super::{FalhaTransporte, cookie_no_formato, validar_origem};
use crate::arquivo_privado::{self, ErroArquivoPrivado};
use std::fmt;
use std::path::Path;

/// Cookie de um retrato da sessão: lido do arquivo privado (`0600`, sem
/// link) e no formato do cabeçalho, ou o fato que o impediu. A falha só
/// aparece onde a requisição pede o cookie, como antes da #222: as recusas
/// offline que vêm antes dela continuam vencendo. Nunca sai no `Debug`.
#[derive(Clone, PartialEq, Eq)]
pub(crate) enum CookieDaSessao {
    Lido(CookieLido),
    /// O arquivo do cookie não foi lido.
    Ilegivel(ErroArquivoPrivado),
    /// O cookie lido está fora do formato do cabeçalho `Cookie`.
    ForaDoFormato,
    /// A URL do Controle não é da origem autorizada: o cookie nem foi
    /// aberto (a recusa de `validar_origem`).
    NaoAberto(FalhaTransporte),
}

/// O cookie lido e conferido: quem o tem não relê o arquivo nem falha.
/// Só o retrato o cria. Nunca sai no `Debug`.
#[derive(Clone, PartialEq, Eq)]
pub(crate) struct CookieLido(String);

impl std::ops::Deref for CookieLido {
    type Target = str;
    fn deref(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for CookieLido {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("CookieLido(<redigido>)")
    }
}

impl CookieLido {
    /// Cookie sintético dos testes, no formato do lido do arquivo.
    #[cfg(test)]
    pub(crate) fn sintetico(cookie: &str) -> Self {
        assert!(cookie_no_formato(cookie));
        Self(cookie.to_owned())
    }
}

impl CookieDaSessao {
    /// O chamador deve ter autorizado a URL de destino antes desta função.
    pub(crate) fn ler(cookie_file: &Path) -> Self {
        match arquivo_privado::ler(cookie_file, 8_192) {
            Ok(texto) if cookie_no_formato(texto.trim()) => {
                Self::Lido(CookieLido(texto.trim().to_owned()))
            }
            Ok(_) => Self::ForaDoFormato,
            Err(erro) => Self::Ilegivel(erro),
        }
    }

    /// Cookie sintético dos testes, validado como o lido do arquivo.
    #[cfg(test)]
    pub(crate) fn sintetico(cookie: &str) -> Self {
        Self::Lido(CookieLido::sintetico(cookie))
    }

    /// O cookie para a requisição; a falha, como o transporte a nomeia.
    pub(crate) fn lido(&self) -> Result<CookieLido, FalhaTransporte> {
        self.valor().map(|cookie| CookieLido(cookie.to_owned()))
    }

    pub(crate) fn valor(&self) -> Result<&str, FalhaTransporte> {
        match self {
            Self::Lido(cookie) => Ok(cookie),
            Self::Ilegivel(erro) => Err(FalhaTransporte::CookieDaSessaoIlegivel(*erro)),
            Self::ForaDoFormato => Err(FalhaTransporte::CookieInvalido),
            Self::NaoAberto(recusa) => Err(recusa.clone()),
        }
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
        let url = arquivo_privado::ler(url_file, 8_192)
            .map_err(FalhaTransporte::UrlDoControleIlegivel)?
            .trim()
            .to_owned();
        let origem = arquivo_privado::ler(origem_file, 512)
            .map_err(FalhaTransporte::OrigemIlegivel)?
            .trim()
            .to_owned();
        entre();
        // A origem é conferida ANTES de abrir o cookie; a recusa fica
        // guardada no lugar do cookie, para a primeira requisição.
        let cookie = match validar_origem(&url, &origem) {
            Ok(()) => CookieDaSessao::ler(cookie_file),
            Err(falha) => CookieDaSessao::NaoAberto(falha),
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
    match arquivo_privado::ler(Path::new(&caminho), 4_096) {
        Ok(texto) => Ok(Some(texto)),
        Err(ErroArquivoPrivado::Ausente) => Ok(None),
        Err(falha) => Err(FalhaTransporte::MarcadorDaSessaoIlegivel(falha)),
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
