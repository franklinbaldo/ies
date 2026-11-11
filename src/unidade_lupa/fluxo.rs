//! Recolhe toda a lupa por transporte Web injetável, sem escrita no SEI.
//! Navegação: Controle → `grupo_unidade_listar` → Novo → cadastro
//! → `unidade_selecionar_envio_processo` → GET lupa + POST por página.
use super::{ErroLupa, PaginaDaLupa, ler_pagina, reunir_paginas};
use crate::acao_do_sei::AcaoDoSei;
use crate::consulta_do_controlador::ConsultaDoControlador;
use crate::formulario_web::{RespostaDoPost, SubmissaoDerivada};
use crate::indice_local::UnidadeGuardada;
use crate::instalacao::Instalacao;
use crate::transporte::{FalhaTransporte, decodificar_resposta, validar_origem};
use std::io::Read;

mod consulta;
use consulta::consulta;

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
#[error("{self:?}")]
pub(crate) enum FalhaDaLupa {
    /// O tokenizador de atributos recusou a tag.
    Atributo(#[source] crate::html_atributos::ErroAtributoHtml),
    Transporte(#[source] FalhaTransporte),
    Tela(ErroLupa),
    LinkAusente,
    LinkAmbiguo,
    LinkForaDaInstalacao,
    CharsetInesperado,
    FormularioInesperado,
    /// O parser de HTML recusou a página recebida.
    HtmlNaoInterpretado(#[source] tl::ParseError),
    /// O formulário da lupa não foi localizado na página.
    FormularioAusente(#[source] crate::formulario_web::ErroSubmissaoFormulario),
    /// A submissão do formulário da lupa não pôde ser derivada.
    SubmissaoNaoDerivada(#[source] crate::formulario_web::ErroSubmissaoFormulario),
    ExcessoDePaginas,
}

/// GET / POST sempre uma tentativa, sem redirect automático.
pub(crate) trait TransporteLupa {
    type Corpo: Read;
    fn get(
        &mut self,
        url: &str,
        cookie: &str,
    ) -> Result<RespostaDoPost<Self::Corpo>, FalhaTransporte>;
    fn post(
        &mut self,
        submissao: SubmissaoDerivada,
        cookie: &str,
    ) -> Result<RespostaDoPost<Self::Corpo>, FalhaTransporte>;
}
pub(crate) struct HttpsLupa;
impl TransporteLupa for HttpsLupa {
    type Corpo = crate::transporte::cliente_https::CorpoHttps;
    fn get(
        &mut self,
        url: &str,
        cookie: &str,
    ) -> Result<RespostaDoPost<Self::Corpo>, FalhaTransporte> {
        crate::transporte::envio_post::get_com_localizacao(url, cookie)
    }
    fn post(
        &mut self,
        submissao: SubmissaoDerivada,
        cookie: &str,
    ) -> Result<RespostaDoPost<Self::Corpo>, FalhaTransporte> {
        crate::transporte::envio_post::enviar_post(submissao, cookie)
    }
}

/// A contagem inclui tentativas que falharam. Nunca contém URL/cookie.
pub(crate) struct ListaWeb {
    pub(crate) unidades: Vec<UnidadeGuardada>,
    pub(crate) total: usize,
    pub(crate) requisicoes: usize,
}

fn obter<T: TransporteLupa>(
    t: &mut T,
    url: &str,
    cookie: &str,
    origem: &str,
    requisicoes: &mut usize,
) -> Result<(String, String), FalhaDaLupa> {
    validar_url_de_leitura(url, origem)?;
    *requisicoes += 1;
    let resposta = t.get(url, cookie).map_err(FalhaDaLupa::Transporte)?;
    let mime = resposta.resposta.mime.clone();
    let html = decodificar_resposta(resposta.resposta).map_err(FalhaDaLupa::Transporte)?;
    Ok((html, mime))
}

/// Confere parâmetros da URL servida sem aceitar aliases ou duplicatas.
/// A validação estrita do Controle não serve para as outras ações da lupa.
fn url_publicada_autorizada(url: &str, origem: &str, acao: AcaoDoSei) -> bool {
    let Some(partes) = crate::url_observada::UrlHttps::interpretar(url) else {
        return false;
    };
    let Some(instalacao) = Instalacao::interpretar(origem) else {
        return false;
    };
    if !instalacao.autoriza_controlador(partes.host, partes.caminho) {
        return false;
    }
    let Some(query) = partes.consulta else {
        return false;
    };
    let Ok(query) = ConsultaDoControlador::ler(query) else {
        return false;
    };
    query.unico("acao") == Ok(acao.como_na_url())
        && query.valores("acao_ajax").next().is_none()
        && query
            .unico("infra_hash")
            .is_ok_and(|hash| hash.len() == 64 && hash.bytes().all(|b| b.is_ascii_hexdigit()))
}

fn validar_url_de_leitura(url: &str, origem: &str) -> Result<(), FalhaDaLupa> {
    if validar_origem(url, origem).is_ok()
        || [
            AcaoDoSei::GrupoUnidadeListar,
            AcaoDoSei::GrupoUnidadeCadastrar,
            AcaoDoSei::UnidadeSelecionarEnvioProcesso,
        ]
        .into_iter()
        .any(|acao| url_publicada_autorizada(url, origem, acao))
    {
        Ok(())
    } else {
        Err(FalhaDaLupa::LinkForaDaInstalacao)
    }
}

/// Extrai apenas URLs literais publicadas pelo controle fornecido.
/// Não interpreta nem executa código JavaScript.
fn candidatos_do_controle(tag: &tl::HTMLTag<'_>) -> Result<Vec<String>, FalhaDaLupa> {
    let mut urls = Vec::new();
    for atributo in ["href", "onclick"] {
        let bruto = crate::html_atributos::valor_do_atributo(tag, atributo)
            .map_err(FalhaDaLupa::Atributo)?;
        let Some(bruto) = bruto else { continue };
        let decodificado =
            crate::formulario_web::decodificar_atributo(&bruto).ok_or(FalhaDaLupa::LinkAmbiguo)?;
        let literal = if atributo == "href" {
            Some(decodificado.as_str())
        } else {
            // Apenas literal entre aspas atribuído a location, sem JS.
            let local = decodificado.split("location").nth(1);
            local.and_then(|s| s.split_once('=')).and_then(|(_, v)| {
                let v = v.strip_prefix('\'').or_else(|| v.strip_prefix('"'))?;
                let fim = if decodificado.contains("='") {
                    '\''
                } else {
                    '"'
                };
                v.split(fim).next()
            })
        };
        if let Some(literal) = literal.filter(|x| x.contains("controlador.php?")) {
            urls.push(literal.to_owned());
        }
    }
    Ok(urls)
}

/// Extrai somente o conteúdo integral de literais simples entre aspas do
/// script servido. Não interpreta escapes, variáveis nem executa JavaScript.
/// A URL ainda precisa passar por validação de origem, ação e `infra_hash`.
fn literais_do_script(script: &str) -> Vec<String> {
    let mut urls = Vec::new();
    let mut caracteres = script.char_indices().peekable();
    while let Some((_, aspas)) = caracteres.next() {
        if aspas != '\'' && aspas != '"' {
            continue;
        }
        let inicio = caracteres.peek().map_or(script.len(), |(i, _)| *i);
        let mut escapado = false;
        let mut fim = None;
        for (posicao, c) in caracteres.by_ref() {
            if escapado {
                escapado = false;
            } else if c == '\\' {
                escapado = true;
            } else if c == aspas {
                fim = Some(posicao);
                break;
            }
        }
        if let Some(fim) = fim {
            let Some(literal) = script.get(inicio..fim) else {
                continue;
            };
            if !literal.contains('\\') && literal.contains("controlador.php?") {
                urls.push(literal.to_owned());
            }
        }
    }
    urls
}

/// URL publicada em controle da tela; na lupa GO, também em literal
/// estático de script da tela de cadastro, sem executar JavaScript.
pub(super) fn candidatos(html: &str, acao: AcaoDoSei) -> Result<Vec<String>, FalhaDaLupa> {
    let dom = tl::parse(html, tl::ParserOptions::default())
        .map_err(|e| FalhaDaLupa::Tela(ErroLupa::HtmlNaoInterpretado(e)))?;
    let parser = dom.parser();
    let mut urls = Vec::new();
    for seletor in ["a", "button", "input"] {
        for no in dom.query_selector(seletor).into_iter().flatten() {
            let Some(tag) = no.get(parser).and_then(tl::Node::as_tag) else {
                continue;
            };
            urls.extend(candidatos_do_controle(tag)?);
        }
    }
    if acao == AcaoDoSei::UnidadeSelecionarEnvioProcesso {
        for no in dom.query_selector("script").into_iter().flatten() {
            let Some(tag) = no.get(parser).and_then(tl::Node::as_tag) else {
                continue;
            };
            let chave_da_acao = format!("acao={}", acao.como_na_url());
            urls.extend(
                literais_do_script(tag.raw().as_utf8_str().as_ref())
                    .into_iter()
                    .filter(|url| url.contains(&chave_da_acao)),
            );
        }
    }
    Ok(urls)
}

/// Resolve URL publicada, com ação única e hash hexadecimal; nunca gera hash.
fn destino_de_candidatos(
    candidatos: Vec<String>,
    url_da_tela: &str,
    origem: &str,
    acao: AcaoDoSei,
) -> Result<String, FalhaDaLupa> {
    let mut aceitos = std::collections::HashSet::new();
    for relativo in candidatos {
        if relativo.contains(['#', '\\']) || relativo.contains("..") {
            continue;
        }
        let url = if relativo.starts_with("https://") {
            relativo
        } else if relativo.starts_with("controlador.php?") {
            let base = url_da_tela.split('?').next().unwrap_or_default();
            let pasta = base
                .rsplit_once('/')
                .map(|(p, _)| p)
                .ok_or(FalhaDaLupa::LinkForaDaInstalacao)?;
            format!("{pasta}/{relativo}")
        } else {
            continue;
        };
        if url_publicada_autorizada(&url, origem, acao) {
            aceitos.insert(url);
        }
    }
    match aceitos.len() {
        0 => Err(FalhaDaLupa::LinkAusente),
        1 => Ok(aceitos.into_iter().next().unwrap_or_default()),
        _ => Err(FalhaDaLupa::LinkAmbiguo),
    }
}

pub(super) fn destino(
    html: &str,
    url_da_tela: &str,
    origem: &str,
    acao: AcaoDoSei,
) -> Result<String, FalhaDaLupa> {
    destino_de_candidatos(candidatos(html, acao)?, url_da_tela, origem, acao)
}

fn botao_novo(html: &str, url: &str, origem: &str) -> Result<String, FalhaDaLupa> {
    let dom =
        tl::parse(html, tl::ParserOptions::default()).map_err(FalhaDaLupa::HtmlNaoInterpretado)?;
    let parser = dom.parser();
    let achados =
        crate::html_atributos::nos_com_id(&dom, "btnNovo").map_err(FalhaDaLupa::Atributo)?;
    if achados.is_empty() {
        return Err(FalhaDaLupa::LinkAusente);
    }
    // Na UnB a cassette publica dois btnNovo com o MESMO destino assinado.
    // Aceitar duplicidade de id apenas se CADA botão publicar sozinho
    // exatamente a mesma URL autorizada. Botão sem alvo ou alvos distintos
    // continuam ambíguos, antes de qualquer outro GET.
    let duplicados = achados.len() > 1;
    let mut destino_unico: Option<String> = None;
    for no in achados {
        let tag = no
            .get(parser)
            .and_then(tl::Node::as_tag)
            .ok_or(FalhaDaLupa::LinkAmbiguo)?;
        let candidato = match destino_de_candidatos(
            candidatos_do_controle(tag)?,
            url,
            origem,
            AcaoDoSei::GrupoUnidadeCadastrar,
        ) {
            Ok(candidato) => candidato,
            Err(_) if duplicados => return Err(FalhaDaLupa::LinkAmbiguo),
            Err(falha) => return Err(falha),
        };
        if let Some(ref anterior) = destino_unico {
            if anterior != &candidato {
                return Err(FalhaDaLupa::LinkAmbiguo);
            }
        } else {
            destino_unico = Some(candidato);
        }
    }
    destino_unico.ok_or(FalhaDaLupa::LinkAusente)
}

/// Consulta integral: quatro GETs de navegação + um POST por página;
/// a própria lupa custa 1 GET + N POSTs, sem pagina repetida.
#[cfg(test)]
pub(crate) fn ler_integral<T: TransporteLupa>(
    transporte: &mut T,
    url_controle: &str,
    origem: &str,
    cookie: &str,
) -> Result<ListaWeb, FalhaDaLupa> {
    let mut requisicoes = 0;
    ler_integral_com_contador(transporte, url_controle, origem, cookie, &mut requisicoes)
}

/// Conserva a contagem das tentativas mesmo quando uma resposta e recusada.
/// O contador e de metadados: nunca inclui URLs assinadas ou cookies.
pub(crate) fn ler_integral_com_contador<T: TransporteLupa>(
    transporte: &mut T,
    url_controle: &str,
    origem: &str,
    cookie: &str,
    requisicoes: &mut usize,
) -> Result<ListaWeb, FalhaDaLupa> {
    let instalacao = Instalacao::interpretar(origem).ok_or(FalhaDaLupa::LinkForaDaInstalacao)?;
    let (controle, _) = obter(transporte, url_controle, cookie, origem, requisicoes)?;
    let lista = destino(
        &controle,
        url_controle,
        origem,
        AcaoDoSei::GrupoUnidadeListar,
    )?;
    let (lista_html, _) = obter(transporte, &lista, cookie, origem, requisicoes)?;
    let novo = botao_novo(&lista_html, &lista, origem)?;
    let (cadastro, _) = obter(transporte, &novo, cookie, origem, requisicoes)?;
    let url_lupa = destino(
        &cadastro,
        &novo,
        origem,
        AcaoDoSei::UnidadeSelecionarEnvioProcesso,
    )?;
    let (mut html, mut mime) = obter(transporte, &url_lupa, cookie, origem, requisicoes)?;
    let mut url = url_lupa;
    let mut lidas: Vec<PaginaDaLupa> = Vec::new();
    let mut total_paginas = 1;
    for pagina in 0.. {
        if pagina >= total_paginas || pagina >= 10000 {
            break;
        }
        let post = consulta(&instalacao, &url, &html, &mime, pagina)?;
        post.url().clone_into(&mut url);
        *requisicoes += 1;
        let resposta = transporte
            .post(post, cookie)
            .map_err(FalhaDaLupa::Transporte)?;
        mime.clone_from(&resposta.resposta.mime);
        html = decodificar_resposta(resposta.resposta).map_err(FalhaDaLupa::Transporte)?;
        super::conferir_filtros_de_lista_integral(&html).map_err(FalhaDaLupa::Tela)?;
        let lida = ler_pagina(&html).map_err(FalhaDaLupa::Tela)?;
        if lida.pagina != pagina || (!lidas.is_empty() && lida.paginas != total_paginas) {
            return Err(FalhaDaLupa::Tela(ErroLupa::PaginasDivergentes));
        }
        total_paginas = lida.paginas;
        lidas.push(lida);
    }
    if lidas.len() != total_paginas {
        return Err(FalhaDaLupa::ExcessoDePaginas);
    }
    let unidades = reunir_paginas(lidas).map_err(FalhaDaLupa::Tela)?;
    Ok(ListaWeb {
        total: unidades.len(),
        unidades,
        requisicoes: *requisicoes,
    })
}

#[cfg(test)]
#[path = "fluxo_regressoes.rs"]
mod regressoes;

#[cfg(test)]
#[path = "fluxo_testes.rs"]
mod tests;
