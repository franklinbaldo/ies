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

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FalhaTransporte {
    UrlInvalida,
    ChaveDeConsultaNaoCanonica,
    SeparadorDeConsultaAmbiguo,
    OrigemNaoAutorizada,
    ArquivoPrivadoAusenteOuInacessivel,
    ArquivoPrivadoComPermissoesInseguras,
    CaminhoDeArquivoNaoEhArquivo,
    ArquivoPrivadoLinkSimbolico,
    ArquivoPrivadoVazio,
    ArquivoPrivadoAcimaDoLimite,
    ArquivoPrivadoNaoPodeSerLido,
    CookieInvalido,
    /// Status HTTP 401 ou 403.
    AcessoNegado(StatusHttpRecusado),
    /// Status HTTP 3xx; o corpo não é lido.
    Redirecionamento(StatusHttpRecusado),
    /// Status HTTP 5xx, fora 3xx/401/403 já tipados.
    StatusHttpDeErroDoServidor(StatusHttpRecusado),
    /// Status HTTP 429.
    StatusHttpDeLimiteDeRequisicoes(StatusHttpRecusado),
    /// Qualquer outro status diferente de 200.
    StatusHttpDiferenteDe200(StatusHttpRecusado),
    /// `Content-Type` cujo tipo principal não é `text/html`.
    TipoDeConteudoNaoHtml,
    /// Corpo que não decodifica no charset declarado (ou UTF-8).
    CorpoNaoDecodificavelNoCharsetDeclarado,
    PaginaDeLoginRecebidaNoControle,
    RespostaExcessiva,
    Rede,
    /// Proxy do ambiente (cliente HTTPS único, todos os comandos): `transporte::proxy`.
    Proxy(proxy::FalhaDoProxy),
}

impl FalhaTransporte {
    pub fn subcodigo(&self) -> &'static str {
        match self {
            Self::UrlInvalida => "url_invalida",
            Self::ChaveDeConsultaNaoCanonica => "chave_de_consulta_do_controle_nao_canonica",
            Self::SeparadorDeConsultaAmbiguo => "separador_de_consulta_do_controle_ambiguo",
            Self::OrigemNaoAutorizada => "origem_nao_autorizada",
            Self::ArquivoPrivadoAusenteOuInacessivel => "arquivo_privado_ausente_ou_inacessivel",
            Self::ArquivoPrivadoComPermissoesInseguras => {
                "arquivo_privado_com_permissoes_inseguras"
            }
            Self::CaminhoDeArquivoNaoEhArquivo => "caminho_nao_eh_arquivo",
            Self::ArquivoPrivadoLinkSimbolico => "arquivo_privado_link_simbolico",
            Self::ArquivoPrivadoVazio => "arquivo_privado_vazio",
            Self::ArquivoPrivadoAcimaDoLimite => "arquivo_privado_acima_do_limite",
            Self::ArquivoPrivadoNaoPodeSerLido => "arquivo_privado_nao_pode_ser_lido",
            Self::CookieInvalido => "cookie_invalido",
            Self::AcessoNegado(_) => "sessao_nao_autorizada",
            Self::Redirecionamento(_) => "redirecionamento_recusado",
            Self::StatusHttpDeErroDoServidor(_) => "status_http_de_erro_do_servidor",
            Self::StatusHttpDeLimiteDeRequisicoes(_) => "status_http_de_limite_de_requisicoes",
            Self::StatusHttpDiferenteDe200(_) => "status_http_diferente_de_200",
            Self::TipoDeConteudoNaoHtml => "tipo_de_conteudo_nao_html",
            Self::CorpoNaoDecodificavelNoCharsetDeclarado => {
                "corpo_nao_decodificavel_no_charset_declarado"
            }
            Self::PaginaDeLoginRecebidaNoControle => "pagina_de_login_recebida_no_controle",
            Self::RespostaExcessiva => "resposta_excessiva",
            Self::Rede => "falha_de_rede",
            Self::Proxy(falha) => falha.subcodigo(),
        }
    }
    pub fn mensagem(&self) -> &'static str {
        match self {
            Self::UrlInvalida => "URL assinada da caixa inválida",
            Self::ChaveDeConsultaNaoCanonica => {
                "nome de parâmetro da URL de controle não é canônico"
            }
            Self::SeparadorDeConsultaAmbiguo => {
                "separador de parâmetros da URL de controle não é inequívoco"
            }
            Self::OrigemNaoAutorizada => "destino HTTPS diferente da origem autorizada da sessão",
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
            Self::CookieInvalido => "formato de cookie de sessão inválido",
            Self::AcessoNegado(_) => "SEI negou acesso; a sessão pode estar vencida",
            Self::Redirecionamento(_) => {
                "redirecionamento recusado; a sessão pode exigir novo login"
            }
            Self::StatusHttpDeErroDoServidor(_) => "o SEI respondeu com erro do servidor (5xx)",
            Self::StatusHttpDeLimiteDeRequisicoes(_) => "o SEI limitou as requisições (429)",
            Self::StatusHttpDiferenteDe200(_) => "o SEI respondeu com status HTTP diferente de 200",
            Self::TipoDeConteudoNaoHtml => "a resposta não declara Content-Type text/html",
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
        match self {
            Self::AcessoNegado(recusa)
            | Self::Redirecionamento(recusa)
            | Self::StatusHttpDeErroDoServidor(recusa)
            | Self::StatusHttpDeLimiteDeRequisicoes(recusa)
            | Self::StatusHttpDiferenteDe200(recusa) => Some(recusa),
            _ => None,
        }
    }

    /// Status com que o proxy recusou o `CONNECT`.
    pub fn status_do_proxy(&self) -> Option<u16> {
        match self {
            Self::Proxy(proxy::FalhaDoProxy::RecusouOTunel { status }) => Some(*status),
            _ => None,
        }
    }
}
