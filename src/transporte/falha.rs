//! Falhas tipadas do transporte Web: um erro por fato
//! (docs/erros-tipados.md). Contrato: contracts/cli.md.

/// Status de uma recusa HTTP e o texto que o SEI enviou junto, já saneado
/// (`corpo_de_recusa`).
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[error("HTTP {status}")]
pub struct StatusHttpRecusado {
    pub status: u16,
    /// `None` em 3xx (corpo não lido) ou quando o corpo não rendeu texto.
    pub mensagem_do_servidor: Option<String>,
}

impl StatusHttpRecusado {
    pub(crate) fn sem_mensagem(status: u16) -> Self {
        Self {
            status,
            mensagem_do_servidor: None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[error("{self:?}")]
pub enum FalhaTransporte {
    ChaveDeConsultaNaoCanonica,
    SeparadorDeConsultaAmbiguo,
    OrigemNaoAutorizada,
    /// URL assinada inválida: falha da validação do transporte.
    UrlInvalida,
    /// Cookie local fora do formato permitido.
    CookieInvalido,
    /// A resposta não declarou o Content-Type HTML esperado.
    TipoDeConteudoNaoHtml,
    /// A consulta AJAX não declarou `application/xml` nem `text/xml`.
    TipoDeConteudoNaoXml,
    ArquivoPrivadoAusenteOuInacessivel,
    #[cfg_attr(not(unix), allow(dead_code))]
    ArquivoPrivadoComPermissoesInseguras,
    CaminhoDeArquivoNaoEhArquivo,
    ArquivoPrivadoLinkSimbolico,
    ArquivoPrivadoVazio,
    ArquivoPrivadoAcimaDoLimite,
    ArquivoPrivadoNaoPodeSerLido,
    /// O marcador `.geracao` mudou (ou não confere com URL e cookie) nas
    /// duas leituras locais da sessão: outro processo gravou no meio.
    SessaoTrocadaDuranteALeitura,
    /// Status HTTP diferente de 200, com o número e o texto do SEI.
    Status(#[source] StatusHttpRecusado),
    /// 403 com a página de bloqueio de segurança da instalação (WAF): a
    /// sessão não venceu; não há relogin nem nova tentativa.
    BloqueioDeSegurancaDaInstalacao(#[source] StatusHttpRecusado),
    /// 302 do GET do Controle cujo `Location` é a entrada do login da
    /// própria instalação: a sessão foi encerrada. Mesmo erro dos
    /// fluxos que veem o 302 para o login (entrada por id, link guardado).
    ControleRedirecionadoParaOLogin(#[source] StatusHttpRecusado),
    /// Corpo que não decodifica no charset declarado (ou UTF-8).
    CorpoNaoDecodificavelNoCharsetDeclarado,
    PaginaDeLoginRecebidaNoControle,
    RespostaExcessiva,
    Rede,
    /// O proxy do ambiente (`HTTPS_PROXY`) é inválido ou recusou o túnel.
    Proxy,
}

impl FalhaTransporte {
    /// A sessão do SEI acabou (decide o relogin único): por `match` no tipo, sem
    /// percorrer a cadeia `source()`.
    pub(crate) fn sessao_encerrada(&self) -> bool {
        match self {
            Self::ControleRedirecionadoParaOLogin(_) | Self::PaginaDeLoginRecebidaNoControle => {
                true
            }
            Self::ChaveDeConsultaNaoCanonica
            | Self::SeparadorDeConsultaAmbiguo
            | Self::OrigemNaoAutorizada
            | Self::UrlInvalida
            | Self::CookieInvalido
            | Self::TipoDeConteudoNaoHtml
            | Self::TipoDeConteudoNaoXml
            | Self::ArquivoPrivadoAusenteOuInacessivel
            | Self::ArquivoPrivadoComPermissoesInseguras
            | Self::CaminhoDeArquivoNaoEhArquivo
            | Self::ArquivoPrivadoLinkSimbolico
            | Self::ArquivoPrivadoVazio
            | Self::ArquivoPrivadoAcimaDoLimite
            | Self::ArquivoPrivadoNaoPodeSerLido
            | Self::SessaoTrocadaDuranteALeitura
            | Self::Status(_)
            | Self::BloqueioDeSegurancaDaInstalacao(_)
            | Self::CorpoNaoDecodificavelNoCharsetDeclarado
            | Self::RespostaExcessiva
            | Self::Rede
            | Self::Proxy => false,
        }
    }
}

impl FalhaTransporte {
    /// Status e mensagem do SEI, só nas recusas por status HTTP.
    pub fn status_http(&self) -> Option<&StatusHttpRecusado> {
        match self {
            Self::Status(recusa)
            | Self::BloqueioDeSegurancaDaInstalacao(recusa)
            | Self::ControleRedirecionadoParaOLogin(recusa) => Some(recusa),
            Self::ChaveDeConsultaNaoCanonica
            | Self::SeparadorDeConsultaAmbiguo
            | Self::OrigemNaoAutorizada
            | Self::UrlInvalida
            | Self::CookieInvalido
            | Self::TipoDeConteudoNaoHtml
            | Self::TipoDeConteudoNaoXml
            | Self::ArquivoPrivadoAusenteOuInacessivel
            | Self::ArquivoPrivadoComPermissoesInseguras
            | Self::CaminhoDeArquivoNaoEhArquivo
            | Self::ArquivoPrivadoLinkSimbolico
            | Self::ArquivoPrivadoVazio
            | Self::ArquivoPrivadoAcimaDoLimite
            | Self::ArquivoPrivadoNaoPodeSerLido
            | Self::SessaoTrocadaDuranteALeitura
            | Self::CorpoNaoDecodificavelNoCharsetDeclarado
            | Self::PaginaDeLoginRecebidaNoControle
            | Self::RespostaExcessiva
            | Self::Rede
            | Self::Proxy => None,
        }
    }
}
