//! Falhas tipadas do transporte Web: uma variante por fato, subcódigos
//! estáveis (docs/erros-tipados.md). Contrato: contracts/cli.md.
use super::proxy;

/// Status de uma recusa HTTP e o texto que o SEI enviou junto, já saneado
/// (`corpo_de_recusa`). É apresentação: o subcódigo da variante classifica.
#[derive(Clone, Debug, PartialEq, Eq)]
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

#[derive(Clone, Debug, PartialEq, Eq, crate::erro_publico::Causa)]
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
    /// Status HTTP 401 ou 403.
    #[causa(http)]
    AcessoNegado(StatusHttpRecusado),
    /// 403 com a página de bloqueio de segurança da instalação (WAF): a
    /// sessão não venceu; não há relogin nem nova tentativa.
    #[causa(http)]
    BloqueioDeSegurancaDaInstalacao(StatusHttpRecusado),
    /// Status HTTP 3xx; o corpo não é lido.
    #[causa(http)]
    Redirecionamento(StatusHttpRecusado),
    /// 302 do GET do Controle cujo `Location` é a entrada do login da
    /// própria instalação: a sessão foi encerrada. Mesmo subcódigo dos
    /// fluxos que veem o 302 para o login (entrada por id, link guardado).
    #[causa(http, sessao_encerrada)]
    ControleRedirecionadoParaOLogin(StatusHttpRecusado),
    /// Status HTTP 5xx, fora 3xx/401/403 já tipados.
    #[causa(http)]
    StatusHttpDeErroDoServidor(StatusHttpRecusado),
    /// Status HTTP 429.
    #[causa(http)]
    StatusHttpDeLimiteDeRequisicoes(StatusHttpRecusado),
    /// Qualquer outro status diferente de 200.
    #[causa(http)]
    StatusHttpDiferenteDe200(StatusHttpRecusado),
    /// Corpo que não decodifica no charset declarado (ou UTF-8).
    CorpoNaoDecodificavelNoCharsetDeclarado,
    #[causa(sessao_encerrada)]
    PaginaDeLoginRecebidaNoControle,
    RespostaExcessiva,
    Rede,
    /// Proxy do ambiente (cliente HTTPS único, todos os comandos): `transporte::proxy`.
    #[causa(interna)]
    Proxy(proxy::FalhaDoProxy),
}

impl FalhaTransporte {
    pub fn mensagem(&self) -> &'static str {
        match self {
            Self::ChaveDeConsultaNaoCanonica => {
                "nome de parâmetro da URL de controle não é canônico"
            }
            Self::SeparadorDeConsultaAmbiguo => {
                "separador de parâmetros da URL de controle não é inequívoco"
            }
            Self::OrigemNaoAutorizada => "destino HTTPS diferente da origem autorizada da sessão",
            Self::UrlInvalida => "URL assinada da caixa inválida",
            Self::CookieInvalido => "formato de cookie de sessão inválido",
            Self::TipoDeConteudoNaoHtml => "a resposta não declara Content-Type text/html",
            Self::TipoDeConteudoNaoXml => "a consulta não declara Content-Type XML",
            Self::ArquivoPrivadoAusenteOuInacessivel => {
                "arquivo privado não encontrado ou sem acesso"
            }
            Self::ArquivoPrivadoComPermissoesInseguras => {
                "arquivo privado permite acesso por outros usuários"
            }
            Self::CaminhoDeArquivoNaoEhArquivo => {
                "caminho informado não aponta para arquivo regular"
            }
            Self::ArquivoPrivadoLinkSimbolico => "arquivo privado não pode ser link simbólico",
            Self::ArquivoPrivadoVazio => "arquivo privado vazio",
            Self::ArquivoPrivadoAcimaDoLimite => "arquivo privado supera o limite permitido",
            Self::ArquivoPrivadoNaoPodeSerLido => "não foi possível ler o arquivo privado",
            Self::SessaoTrocadaDuranteALeitura => {
                "a sessão local foi regravada durante a leitura, duas vezes; nada foi pedido ao SEI"
            }
            Self::AcessoNegado(_) => "SEI negou acesso; a sessão pode estar vencida",
            Self::BloqueioDeSegurancaDaInstalacao(_) => {
                "a política de segurança da instalação bloqueou a requisição; espace as execuções"
            }
            Self::Redirecionamento(_) => {
                "redirecionamento recusado; a sessão pode exigir novo login"
            }
            Self::ControleRedirecionadoParaOLogin(_) => {
                "o SEI redirecionou o Controle para a tela de login; rode ies login"
            }
            Self::StatusHttpDeErroDoServidor(_) => "o SEI respondeu com erro do servidor (5xx)",
            Self::StatusHttpDeLimiteDeRequisicoes(_) => "o SEI limitou as requisições (429)",
            Self::StatusHttpDiferenteDe200(_) => "o SEI respondeu com status HTTP diferente de 200",
            Self::CorpoNaoDecodificavelNoCharsetDeclarado => {
                "o corpo da resposta não decodifica no charset declarado"
            }
            Self::PaginaDeLoginRecebidaNoControle => {
                "o SEI devolveu formulário de login em vez da caixa"
            }
            Self::RespostaExcessiva => "resposta HTML supera o limite permitido",
            Self::Rede => "falha de transporte HTTPS; nenhum pedido foi repetido",
            Self::Proxy(falha) => falha.mensagem(),
        }
    }
}

impl FalhaTransporte {
    /// Status e mensagem do SEI, só nas recusas por status HTTP.
    pub fn status_http(&self) -> Option<&StatusHttpRecusado> {
        crate::erro_publico::Causa::status_http(self)
    }

    /// Status com que o proxy recusou o `CONNECT`.
    pub fn status_do_proxy(&self) -> Option<u16> {
        match self {
            Self::Proxy(proxy::FalhaDoProxy::RecusouOTunel { status }) => Some(*status),
            _ => None,
        }
    }
}
