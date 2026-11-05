//! Recolhe toda a lupa por transporte Web injetável, sem escrita no SEI.
//! Navegação: Controle → grupo_unidade_listar → Novo → cadastro
//! → unidade_selecionar_envio_processo → GET lupa + POST por página.
use super::{ErroLupa, PaginaDaLupa, ler_pagina, reunir_paginas};
use crate::formulario_web::{
    ConferenciaDaUnidade, EfeitoAvaliado, EnvioDoFormulario, EstadoDeSessao,
    OperacaoDeFormulario, ResultadoDoEnvio, SubmissaoDerivada, TelaServida, derivar_submissao,
};
use crate::indice_local::UnidadeGuardada;
use crate::instalacao::Instalacao;
use crate::pagina_do_controle::codificacao_da_tela;
use crate::transporte::{FalhaTransporte, decodificar_resposta, validar_origem};
use crate::formulario_web::RespostaDoPost;
use std::io::Read;

const CONSULTAR: OperacaoDeFormulario = OperacaoDeFormulario {
    acao: "unidade_selecionar_envio_processo",
    formulario: "frmUnidadeLista",
    envio: EnvioDoFormulario::BotaoDeEnvio("btnPesquisar"),
    campos_ocultos_exigidos: &["hdnInfraPaginaAtual"],
    campos_ocultos_reescritos: &["hdnInfraPaginaAtual"],
    campos_usuario: &[
        "selOrgaoUnidade", "txtSiglaUnidade", "txtDescricaoUnidade",
        "selInfraPaginacaoSuperior", "selInfraPaginacaoInferior",
    ],
    resultado: ResultadoDoEnvio::ConferidoPorLeituraPosterior,
    unidade: ConferenciaDaUnidade::NaTelaDeOrigemDoLink,
    efeito: EfeitoAvaliado::AlteraEstadoDaSessao {
        o_que: EstadoDeSessao::PaginaOuFiltroLembrado,
    },
};

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum FalhaDaLupa {
    Transporte(&'static str),
    Tela(ErroLupa),
    LinkAusente,
    LinkAmbiguo,
    LinkForaDaInstalacao,
    CharsetInesperado,
    FormularioInesperado,
    ExcessoDePaginas,
}

impl FalhaDaLupa {
    pub(crate) fn subcodigo(&self) -> &'static str {
        match self {
            Self::Transporte(_) => "lupa_transporte_recusado",
            Self::Tela(ErroLupa::PaginasDivergentes) => "lista_unidades_incompleta",
            Self::Tela(_) => "pagina_da_lupa_fora_da_forma",
            Self::LinkAusente => "link_da_lupa_ausente",
            Self::LinkAmbiguo => "link_da_lupa_ambiguo",
            Self::LinkForaDaInstalacao => "link_da_lupa_fora_da_instalacao",
            Self::CharsetInesperado => "charset_da_lupa_nao_suportado",
            Self::FormularioInesperado => "formulario_da_lupa_inesperado",
            Self::ExcessoDePaginas => "lupa_excedeu_orcamento_de_paginas",
        }
    }
}

/// GET / POST sempre uma tentativa, sem redirect automático.
pub(crate) trait TransporteLupa {
    type Corpo: Read;
    fn get(&mut self, url: &str, cookie: &str)
        -> Result<RespostaDoPost<Self::Corpo>, FalhaTransporte>;
    fn post(&mut self, submissao: SubmissaoDerivada, cookie: &str)
        -> Result<RespostaDoPost<Self::Corpo>, FalhaTransporte>;
}
pub(crate) struct HttpsLupa;
impl TransporteLupa for HttpsLupa {
    type Corpo = crate::transporte::cliente_https::CorpoHttps;
    fn get(&mut self, url: &str, cookie: &str)
        -> Result<RespostaDoPost<Self::Corpo>, FalhaTransporte> {
        crate::transporte::envio_post::get_com_localizacao(url, cookie)
    }
    fn post(&mut self, submissao: SubmissaoDerivada, cookie: &str)
        -> Result<RespostaDoPost<Self::Corpo>, FalhaTransporte> {
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
    t: &mut T, url: &str, cookie: &str, origem: &str, requisicoes: &mut usize,
) -> Result<(String,String), FalhaDaLupa> {
    validar_origem(url, origem).map_err(|_| FalhaDaLupa::LinkForaDaInstalacao)?;
    *requisicoes += 1;
    let resposta = t.get(url, cookie).map_err(|e| FalhaDaLupa::Transporte(e.subcodigo()))?;
    let mime = resposta.resposta.mime.clone();
    let html = decodificar_resposta(resposta.resposta)
        .map_err(|e| FalhaDaLupa::Transporte(e.subcodigo()))?;
    Ok((html,mime))
}

/// URL publicada num anchor, com href ou assignment literal de location.
fn candidatos(html: &str) -> Result<Vec<String>, FalhaDaLupa> {
    let dom = tl::parse(html, tl::ParserOptions::default())
        .map_err(|_| FalhaDaLupa::Tela(ErroLupa::HtmlInesperado))?;
    let parser = dom.parser();
    let mut urls = Vec::new();
    for no in dom.query_selector("a").into_iter().flatten() {
        let Some(tag) = no.get(parser).and_then(tl::Node::as_tag) else {continue};
        for atributo in ["href", "onclick"] {
            let bruto = crate::html_atributos::valor_do_atributo(tag, atributo)
                .map_err(|_| FalhaDaLupa::LinkAmbiguo)?;
            let Some(bruto) = bruto else {continue};
            let decodificado = crate::formulario_web::decodificar_atributo(&bruto)
                .ok_or(FalhaDaLupa::LinkAmbiguo)?;
            let literal = if atributo == "href" {
                Some(decodificado.as_str())
            } else {
                // Apenas literal entre aspas atribuído a location, sem JS.
                let local = decodificado.split("location").nth(1);
                local.and_then(|s| s.split_once('=')).and_then(|(_,v)| {
                    let v = v.trim();
                    let v = v.strip_prefix('\'').or_else(|| v.strip_prefix('"'))?;
                    let fim = if decodificado.contains("='") { '\'' } else { '"' };
                    v.split(fim).next()
                })
            };
            if let Some(literal) = literal.filter(|x| x.contains("controlador.php?")) {
                urls.push(literal.to_owned());
            }
        }
    }
    Ok(urls)
}

/// Resolve URL publicada, com ação única e hash hexadecimal; nunca gera hash.
fn destino(
    html: &str, url_da_tela: &str, origem: &str, acao: &str,
) -> Result<String, FalhaDaLupa> {
    let candidatos = candidatos(html)?;
    let mut aceitos = std::collections::HashSet::new();
    for relativo in candidatos {
        if relativo.contains(['#','\\']) || relativo.contains("..") {continue}
        let url = if relativo.starts_with("https://") {
            relativo
        } else if relativo.starts_with("controlador.php?") {
            let base = url_da_tela.split('?').next().unwrap_or_default();
            let pasta = base.rsplit_once('/').map(|(p,_)|p)
                .ok_or(FalhaDaLupa::LinkForaDaInstalacao)?;
            format!("{pasta}/{relativo}")
        } else {continue};
        if validar_origem(&url, origem).is_err() {continue}
        let Some((_,consulta)) = url.split_once('?') else {continue};
        let mut acoes = 0;let mut hashes = 0;
        for item in consulta.split('&') {
            let Some((chave,valor)) = item.split_once('=') else {continue};
            if chave == "acao" && valor == acao {acoes+=1}
            if chave == "infra_hash" && valor.len() == 64
                && valor.bytes().all(|b| b.is_ascii_hexdigit()) {hashes+=1}
        }
        if acoes == 1 && hashes == 1 {aceitos.insert(url);}
    }
    match aceitos.len() {
        0 => Err(FalhaDaLupa::LinkAusente),
        1 => Ok(aceitos.into_iter().next().unwrap_or_default()),
        _ => Err(FalhaDaLupa::LinkAmbiguo),
    }
}

fn botao_novo(html: &str, url: &str, origem: &str) -> Result<String,FalhaDaLupa> {
    // O Todos identifica o botão btnNovo pelo id; a URL continua publicada
    // na própria tela e é conferida pelo destino.
    let dom = tl::parse(html, tl::ParserOptions::default())
        .map_err(|_| FalhaDaLupa::LinkAusente)?;
    let parser = dom.parser();
    let mut achados=0;
    for no in dom.query_selector("#btnNovo").into_iter().flatten() {
        if no.get(parser).and_then(tl::Node::as_tag).is_some() {achados+=1}
    }
    if achados!=1 {return Err(FalhaDaLupa::LinkAusente)}
    destino(html,url,origem,"grupo_unidade_cadastrar")
}

fn consulta(
    instalacao: &Instalacao, url: &str, html: &str, mime: &str, pagina: usize,
) -> Result<SubmissaoDerivada,FalhaDaLupa> {
    let codificacao = codificacao_da_tela(mime).ok_or(FalhaDaLupa::CharsetInesperado)?;
    let indice = pagina.to_string();
    let tela = TelaServida {url,html,codificacao};
    // Controles de paginação devem existir como o SEI os publica.
    // A biblioteca de formulário insere os ocultos fixos na ordem correta.
    derivar_submissao(&CONSULTAR,instalacao,&tela,"",&[
        ("selOrgaoUnidade",""),("txtSiglaUnidade",""),
        ("txtDescricaoUnidade",""),("hdnInfraPaginaAtual",&indice),
        ("selInfraPaginacaoSuperior",&indice),("selInfraPaginacaoInferior",&indice),
    ]).map_err(|_|FalhaDaLupa::FormularioInesperado)
}

/// Consulta integral: quatro GETs de navegação + um POST por página;
/// a própria lupa custa 1 GET + N POSTs, sem pagina repetida.
pub(crate) fn ler_integral<T: TransporteLupa>(
    transporte: &mut T, url_controle: &str, origem: &str, cookie: &str,
) -> Result<ListaWeb,FalhaDaLupa> {
    let instalacao = Instalacao::interpretar(origem)
        .ok_or(FalhaDaLupa::LinkForaDaInstalacao)?;
    let mut requisicoes=0;
    let (controle,_) = obter(transporte,url_controle,cookie,origem,&mut requisicoes)?;
    let lista = destino(&controle,url_controle,origem,"grupo_unidade_listar")?;
    let (lista_html,_) = obter(transporte,&lista,cookie,origem,&mut requisicoes)?;
    let novo = botao_novo(&lista_html,&lista,origem)?;
    let (cadastro,_) = obter(transporte,&novo,cookie,origem,&mut requisicoes)?;
    let url_lupa = destino(&cadastro,&novo,origem,"unidade_selecionar_envio_processo")?;
    let (mut html,mut mime) = obter(transporte,&url_lupa,cookie,origem,&mut requisicoes)?;
    let mut url = url_lupa;
    let mut lidas: Vec<PaginaDaLupa> = Vec::new();
    let mut total_paginas = 1;
    for pagina in 0.. {
        if pagina >= total_paginas || pagina >= 10000 {break}
        let post = consulta(&instalacao,&url,&html,&mime,pagina)?;
        url = post.url().to_owned();
        requisicoes+=1;
        let resposta=transporte.post(post,cookie)
            .map_err(|e|FalhaDaLupa::Transporte(e.subcodigo()))?;
        mime = resposta.resposta.mime.clone();
        html=decodificar_resposta(resposta.resposta)
            .map_err(|e|FalhaDaLupa::Transporte(e.subcodigo()))?;
        let lida=ler_pagina(&html).map_err(FalhaDaLupa::Tela)?;
        if lida.pagina!=pagina || (!lidas.is_empty() && lida.paginas!=total_paginas) {
            return Err(FalhaDaLupa::Tela(ErroLupa::PaginasDivergentes))
        }
        total_paginas=lida.paginas;
        lidas.push(lida);
    }
    if lidas.len()!=total_paginas {return Err(FalhaDaLupa::ExcessoDePaginas)}
    let unidades=reunir_paginas(lidas).map_err(FalhaDaLupa::Tela)?;
    Ok(ListaWeb {total:unidades.len(),unidades,requisicoes})
}
