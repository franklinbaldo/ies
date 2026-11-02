//! Submissão derivada de formulário HTML servido pelo SEI, sem rede.
//! Contrato: docs/arquitetura/0002-submeter-formulario-observado.md.
//! `action` aqui é o atributo do `<form>`, não um despachante genérico.
mod codificacao;
mod leitura;

use crate::html_atributos::ErroAtributoHtml;
use crate::instalacao::Instalacao;
use crate::transporte::{
    FalhaTransporte, RespostaWeb, arquivo_privado, decodificar_resposta, validar_cookie,
};
use codificacao::codificar;
pub(crate) use codificacao::decodificar_atributo;
pub(crate) use leitura::localizar_formulario;
use leitura::{Controle, UrlHttps, conferir_metodo_e_codificacao, ler_controles, resolver_action};
use std::fmt;
use std::io::Read;
use std::path::Path;

/// Estado de sessão que uma operação pode alterar, mesmo quando consulta.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum EstadoDeSessao {
    PaginaOuFiltroLembrado,
    UnidadeAtual,
    PreferenciaDeVisualizacao,
}

/// Avaliação obrigatória por operação, sem valor padrão.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum EfeitoAvaliado {
    ConsultaSemEfeitoObservado { evidencia: &'static str },
    AlteraEstadoDaSessao { o_que: EstadoDeSessao },
    Desconhecido,
}

/// Declaração `const` feita pela própria operação; não há catálogo central.
#[derive(Debug)]
pub(crate) struct OperacaoDeFormulario {
    pub(crate) acao: &'static str,
    pub(crate) formulario: &'static str,
    pub(crate) botao_de_envio: &'static str,
    pub(crate) campos_ocultos_exigidos: &'static [&'static str],
    pub(crate) campos_usuario: &'static [&'static str],
    pub(crate) marcador_de_resultado: &'static str,
    pub(crate) efeito: EfeitoAvaliado,
}

/// Charset declarado pela resposta que serviu a tela.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CodificacaoDaTela {
    Iso88591,
    Utf8,
}

/// HTML e URL exata da mesma resposta, na mesma geração de sessão.
pub(crate) struct TelaServida<'a> {
    pub(crate) url: &'a str,
    pub(crate) html: &'a str,
    pub(crate) codificacao: CodificacaoDaTela,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum ErroSubmissaoFormulario {
    EfeitoDaOperacaoNaoAvaliado,
    IdentificacaoDaUnidadeAusenteNaTela,
    IdentificacoesDaUnidadeRepetidasNaTela,
    UnidadeDaTelaDiferenteDaSolicitada,
    FormularioAusenteNaTela,
    FormulariosComMesmoIdentificadorNaTela,
    MetodoDoFormularioDiferenteDePost,
    EnctypeDoFormularioNaoUrlencoded,
    AcceptCharsetPublicadoNoFormulario,
    AcaoDoFormularioAusente,
    AcaoDoFormularioNaoEhUrlHttps,
    AcaoDoFormularioForaDaInstalacao,
    AcaoDoFormularioDiferenteDaOperacao,
    AcaoDoFormularioSemInfraHashUnico,
    EntidadeHtmlDesconhecidaNoAtributo {
        campo: String,
    },
    CampoOcultoAusenteNoFormulario {
        campo: String,
    },
    CampoOcultoRepetidoNoFormulario {
        campo: String,
    },
    HdnAcaoPublicadoSemRegraDaOperacao,
    CampoForaDaListaDaOperacao {
        campo: String,
    },
    CampoNaoPublicadoPeloFormulario {
        campo: String,
    },
    CampoInformadoMaisDeUmaVez {
        campo: String,
    },
    ValorNaoRepresentavelNaCodificacao {
        campo: String,
    },
    BotaoDeEnvioAusenteNoFormulario,
    BotoesDeEnvioRepetidosNoFormulario,
    TelaDeLoginRecebidaAposEnvio,
    MarcadorDeResultadoAusenteNaResposta,
    /// Tag de abertura servida que o tokenizador de atributos recusou.
    AtributoHtml(ErroAtributoHtml),
    Transporte(FalhaTransporte),
}

impl ErroSubmissaoFormulario {
    pub(crate) fn subcodigo(&self) -> &'static str {
        match self {
            Self::EfeitoDaOperacaoNaoAvaliado => "efeito_da_operacao_nao_avaliado",
            Self::IdentificacaoDaUnidadeAusenteNaTela => "identificacao_da_unidade_ausente_na_tela",
            Self::IdentificacoesDaUnidadeRepetidasNaTela => {
                "identificacoes_da_unidade_repetidas_na_tela"
            }
            Self::UnidadeDaTelaDiferenteDaSolicitada => "unidade_da_tela_diferente_da_solicitada",
            Self::FormularioAusenteNaTela => "formulario_ausente_na_tela",
            Self::FormulariosComMesmoIdentificadorNaTela => {
                "formularios_com_mesmo_identificador_na_tela"
            }
            Self::MetodoDoFormularioDiferenteDePost => "metodo_do_formulario_diferente_de_post",
            Self::EnctypeDoFormularioNaoUrlencoded => "enctype_do_formulario_nao_urlencoded",
            Self::AcceptCharsetPublicadoNoFormulario => "accept_charset_publicado_no_formulario",
            Self::AcaoDoFormularioAusente => "acao_do_formulario_ausente",
            Self::AcaoDoFormularioNaoEhUrlHttps => "acao_do_formulario_nao_eh_url_https",
            Self::AcaoDoFormularioForaDaInstalacao => "acao_do_formulario_fora_da_instalacao",
            Self::AcaoDoFormularioDiferenteDaOperacao => "acao_do_formulario_diferente_da_operacao",
            Self::AcaoDoFormularioSemInfraHashUnico => "acao_do_formulario_sem_infra_hash_unico",
            Self::EntidadeHtmlDesconhecidaNoAtributo { .. } => {
                "entidade_html_desconhecida_no_atributo"
            }
            Self::CampoOcultoAusenteNoFormulario { .. } => "campo_oculto_ausente_no_formulario",
            Self::CampoOcultoRepetidoNoFormulario { .. } => "campo_oculto_repetido_no_formulario",
            Self::HdnAcaoPublicadoSemRegraDaOperacao => "hdn_acao_publicado_sem_regra_da_operacao",
            Self::CampoForaDaListaDaOperacao { .. } => "campo_fora_da_lista_da_operacao",
            Self::CampoNaoPublicadoPeloFormulario { .. } => "campo_nao_publicado_pelo_formulario",
            Self::CampoInformadoMaisDeUmaVez { .. } => "campo_informado_mais_de_uma_vez",
            Self::ValorNaoRepresentavelNaCodificacao { .. } => {
                "valor_nao_representavel_na_codificacao"
            }
            Self::BotaoDeEnvioAusenteNoFormulario => "botao_de_envio_ausente_no_formulario",
            Self::BotoesDeEnvioRepetidosNoFormulario => "botoes_de_envio_repetidos_no_formulario",
            Self::TelaDeLoginRecebidaAposEnvio => "tela_de_login_recebida_apos_envio",
            Self::MarcadorDeResultadoAusenteNaResposta => {
                "marcador_de_resultado_ausente_na_resposta"
            }
            Self::AtributoHtml(falha) => falha.subcodigo(),
            Self::Transporte(falha) => falha.subcodigo(),
        }
    }
}

/// POST derivado; não é `Clone`, e o envio o consome (sem retry).
pub(crate) struct SubmissaoDerivada {
    url: String,
    host: String,
    caminho: String,
    acao: &'static str,
    referer: String,
    nomes: Vec<String>,
    corpo: Vec<u8>,
    efeito: EfeitoAvaliado,
    marcador_de_resultado: &'static str,
}

impl SubmissaoDerivada {
    pub(crate) const METODO: &'static str = "POST";
    pub(crate) const TIPO_DE_CONTEUDO: &'static str = "application/x-www-form-urlencoded";
    pub(crate) fn url(&self) -> &str {
        &self.url
    }
    pub(crate) fn origem(&self) -> String {
        format!("https://{}", self.host)
    }
    pub(crate) fn referer(&self) -> &str {
        &self.referer
    }
    pub(crate) fn nomes_dos_campos(&self) -> &[String] {
        &self.nomes
    }
    pub(crate) fn corpo(&self) -> &[u8] {
        &self.corpo
    }
    pub(crate) fn efeito(&self) -> EfeitoAvaliado {
        self.efeito
    }
}

/// Só nomes e destino sem consulta: `infra_hash`, `Referer` e valores
/// nunca aparecem em logs ou mensagens de teste.
impl fmt::Debug for SubmissaoDerivada {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SubmissaoDerivada")
            .field("metodo", &Self::METODO)
            .field(
                "destino",
                &format_args!("https://{}/{}", self.host, self.caminho),
            )
            .field("acao", &self.acao)
            .field("campos", &self.nomes)
            .field("valores", &"<redigido>")
            .field("referer", &"<redigido>")
            .field("efeito", &self.efeito)
            .finish_non_exhaustive()
    }
}

fn conferir_unidade(dom: &tl::VDom<'_>, solicitada: &str) -> Result<(), ErroSubmissaoFormulario> {
    use ErroSubmissaoFormulario as E;
    // `unidade_html` só falha por identificações repetidas fora do par móvel/desktop.
    let publicada = crate::controle_contexto::unidade_html(dom)
        .map_err(|_| E::IdentificacoesDaUnidadeRepetidasNaTela)?
        .ok_or(E::IdentificacaoDaUnidadeAusenteNaTela)?;
    let solicitada = solicitada.split_whitespace().collect::<Vec<_>>().join(" ");
    if publicada != solicitada {
        return Err(E::UnidadeDaTelaDiferenteDaSolicitada);
    }
    Ok(())
}

fn ordenar_campos<'c>(
    controles: &'c [Controle],
    operacao: &OperacaoDeFormulario,
    campos: &'c [(&'c str, &'c str)],
) -> Result<Vec<(&'c str, &'c str)>, ErroSubmissaoFormulario> {
    use ErroSubmissaoFormulario as E;
    for (i, (nome, _)) in campos.iter().enumerate() {
        let campo = (*nome).to_owned();
        if !operacao.campos_usuario.contains(nome) {
            return Err(E::CampoForaDaListaDaOperacao { campo });
        }
        let publicados: Vec<_> = controles
            .iter()
            .filter(
                |c| matches!(c, Controle::Usuario(n) | Controle::SelecaoMultipla(n) if n == nome),
            )
            .collect();
        if publicados.is_empty() {
            return Err(E::CampoNaoPublicadoPeloFormulario { campo });
        }
        // Só um `<select multiple>` (e nenhum outro controle de mesmo nome)
        // admite o nome repetido; os valores vêm da operação.
        let multipla = publicados
            .iter()
            .all(|c| matches!(c, Controle::SelecaoMultipla(_)));
        if !multipla && campos[..i].iter().any(|(anterior, _)| anterior == nome) {
            return Err(E::CampoInformadoMaisDeUmaVez { campo });
        }
    }
    // Ordem do documento; um rádio com vários controles é emitido uma vez,
    // e os pares de um `select multiple` saem juntos na posição dele.
    let mut emitidos: Vec<&str> = Vec::new();
    let mut ordenados = Vec::new();
    for controle in controles {
        match controle {
            Controle::Oculto { nome, valor } | Controle::Botao { nome, valor } => {
                ordenados.push((nome.as_str(), valor.as_str()));
            }
            Controle::Usuario(nome) | Controle::SelecaoMultipla(nome)
                if !emitidos.contains(&nome.as_str()) =>
            {
                emitidos.push(nome);
                ordenados.extend(campos.iter().filter(|(n, _)| n == nome).copied());
            }
            Controle::Usuario(_) | Controle::SelecaoMultipla(_) => {}
        }
    }
    Ok(ordenados)
}

/// Deriva a submissão do formulário servido, sem rede.
pub(crate) fn derivar_submissao(
    operacao: &OperacaoDeFormulario,
    instalacao: &Instalacao,
    tela: &TelaServida<'_>,
    unidade_solicitada: &str,
    campos: &[(&str, &str)],
) -> Result<SubmissaoDerivada, ErroSubmissaoFormulario> {
    use ErroSubmissaoFormulario as E;
    if operacao.efeito == EfeitoAvaliado::Desconhecido {
        return Err(E::EfeitoDaOperacaoNaoAvaliado);
    }
    if tela.html.len() > crate::MAX_HTML_BYTES {
        return Err(E::Transporte(FalhaTransporte::RespostaExcessiva));
    }
    let servida =
        UrlHttps::interpretar(tela.url).ok_or(E::Transporte(FalhaTransporte::UrlInvalida))?;
    if !instalacao.autoriza_controlador(servida.host, servida.caminho) {
        return Err(E::Transporte(FalhaTransporte::OrigemNaoAutorizada));
    }
    // tl só recusa entradas acima de u32::MAX bytes, já barradas acima.
    let dom = tl::parse(tela.html, tl::ParserOptions::default())
        .map_err(|_| E::Transporte(FalhaTransporte::RespostaExcessiva))?;
    conferir_unidade(&dom, unidade_solicitada)?;
    let form = localizar_formulario(&dom, operacao.formulario)?;
    conferir_metodo_e_codificacao(form)?;
    let url = resolver_action(form, &servida, instalacao, operacao)?;
    let controles = ler_controles(form, dom.parser(), operacao)?;
    let ordenados = ordenar_campos(&controles, operacao, campos)?;
    let mut corpo = String::new();
    for (i, (nome, valor)) in ordenados.iter().enumerate() {
        if i > 0 {
            corpo.push('&');
        }
        let nao_representavel = || E::ValorNaoRepresentavelNaCodificacao {
            campo: (*nome).to_owned(),
        };
        codificar(nome, tela.codificacao, &mut corpo).ok_or_else(nao_representavel)?;
        corpo.push('=');
        codificar(valor, tela.codificacao, &mut corpo).ok_or_else(nao_representavel)?;
    }
    Ok(SubmissaoDerivada {
        url,
        host: servida.host.to_owned(),
        caminho: servida.caminho.to_owned(),
        acao: operacao.acao,
        referer: tela.url.to_owned(),
        nomes: ordenados.iter().map(|(n, _)| (*n).to_owned()).collect(),
        corpo: corpo.into_bytes(),
        efeito: operacao.efeito,
        marcador_de_resultado: operacao.marcador_de_resultado,
    })
}

/// Resposta do POST: a mesma `RespostaWeb` do GET mais o cabeçalho
/// `location`, que só o envio de formulário lê. O GET do Controle não muda.
pub(crate) struct RespostaDoPost<R: Read> {
    pub(crate) resposta: RespostaWeb<R>,
    pub(crate) localizacao: Option<String>,
}

/// 302 recebido e **não seguido**; o corpo não é lido. `Location` e URL do
/// POST são assinados: sem `Clone`, e o `Debug` os redige.
pub(crate) struct Redirecionamento302 {
    localizacao: Option<String>,
    url_do_envio: String,
}

impl Redirecionamento302 {
    pub(crate) fn localizacao(&self) -> Option<&str> {
        self.localizacao.as_deref()
    }
    pub(crate) fn url_do_envio(&self) -> &str {
        &self.url_do_envio
    }
    #[cfg(test)]
    pub(crate) fn sintetico(localizacao: Option<&str>, url_do_envio: &str) -> Self {
        Self {
            localizacao: localizacao.map(str::to_owned),
            url_do_envio: url_do_envio.to_owned(),
        }
    }
}

impl fmt::Debug for Redirecionamento302 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Redirecionamento302")
            .field(
                "localizacao",
                &self.localizacao.as_ref().map(|_| "<redigido>"),
            )
            .field("url_do_envio", &"<redigido>")
            .finish()
    }
}

/// HTML com o marcador da operação, ou 302 entregue à operação como dado.
#[derive(Debug)]
pub(crate) enum RespostaDoEnvio {
    Html(String),
    Redirecionamento(Redirecionamento302),
}

/// Envio injetável: `FnOnce` recebe (e consome) a submissão e o cookie,
/// uma única vez. O cookie é lido do arquivo privado só depois da derivação.
/// Nenhum 3xx é seguido; só o 302 volta como dado, para a operação interpretar.
pub(crate) fn enviar_submissao_via<R: Read, F>(
    submissao: SubmissaoDerivada,
    cookie_file: &Path,
    enviar: F,
) -> Result<RespostaDoEnvio, ErroSubmissaoFormulario>
where
    F: FnOnce(SubmissaoDerivada, &str) -> Result<RespostaDoPost<R>, FalhaTransporte>,
{
    use ErroSubmissaoFormulario as E;
    let texto = arquivo_privado(cookie_file, 8_192).map_err(E::Transporte)?;
    let cookie = texto.trim();
    validar_cookie(cookie).map_err(E::Transporte)?;
    let marcador_de_resultado = submissao.marcador_de_resultado;
    let url_do_envio = submissao.url.clone();
    let RespostaDoPost {
        resposta,
        localizacao,
    } = enviar(submissao, cookie).map_err(E::Transporte)?;
    if resposta.status_code == 302 {
        // Corpo descartado sem leitura; a operação decide o que o 302 significa.
        return Ok(RespostaDoEnvio::Redirecionamento(Redirecionamento302 {
            localizacao,
            url_do_envio,
        }));
    }
    let html = decodificar_resposta(resposta).map_err(|falha| match falha {
        FalhaTransporte::PaginaDeLoginRecebidaNoControle => E::TelaDeLoginRecebidaAposEnvio,
        outra => E::Transporte(outra),
    })?;
    let marcado = tl::parse(&html, tl::ParserOptions::default())
        .map_err(|_| E::Transporte(FalhaTransporte::RespostaExcessiva))?
        .query_selector(marcador_de_resultado)
        .is_some_and(|mut achados| achados.next().is_some());
    if !marcado {
        return Err(E::MarcadorDeResultadoAusenteNaResposta);
    }
    Ok(RespostaDoEnvio::Html(html))
}

#[cfg(test)]
#[path = "formulario_web/tests.rs"]
pub(crate) mod tests;

#[cfg(test)]
#[path = "formulario_web/tests_envio.rs"]
mod tests_envio;

#[cfg(test)]
#[path = "formulario_web/tests_selecao.rs"]
mod tests_selecao;

#[cfg(test)]
#[path = "formulario_web/tests_atributos.rs"]
mod tests_atributos;
